//! **What a boxed command says reaches the person, carried by a process
//! outside the box.**
//!
//! A box refuses every place nobody granted, and the kernel asks on every read
//! and every write, not only on every open (`alo-bounding-kernel`'s
//! `file_permission`, since 2026-09-12). **The person's terminal is one of
//! those places.** So a command that inherited the terminal as its output
//! could not write to it: on a booted machine on 2026-10-31 `try id` printed
//! nothing at all, and its record says it returned 101.
//!
//! 101 was not `id`. It was `alo-enter-the-box`, inside the box, failing to
//! start `id` and then failing to write why: `eprintln!` panics when the write
//! is refused, and the panic's own message is refused the same way, so the
//! process ended with Rust's panic code and no word to anybody. Measured inside
//! the image the same day, once, by hand, with a stderr that refuses writes:
//! the shipped helper exited 101 and printed nothing.
//!
//! So the box's three standard streams are **pipes**, which a box may use
//! because a pipe holds no contents of its own (`decide_use` says so at
//! length), and this process â€” outside the box â€” copies them to and from the
//! person's terminal. Nothing the box could not reach becomes reachable: it
//! writes into a pipe, and what reads the pipe is the program the person ran.
//!
//! # What it costs, said rather than discovered
//!
//! A boxed command sees pipes rather than a terminal, so it prints as it does
//! into `| cat`: no colours a program chooses only for a terminal, no columns
//! from `ls`, no prompt that asks a terminal for its size. A command that
//! leaves something running in the background holding its output keeps `try`
//! waiting until that ends, as `command | cat` would.

use std::io::{Read, Write};
use std::process::{Child, ExitStatus};

/// Where a boxed command's output goes, and where its input comes from.
pub enum Carrying<'a> {
    /// The person's own: this process's standard streams.
    ToThePerson,
    /// Somewhere a caller holds, with no input at all. For tests, which must
    /// see what was said.
    To {
        /// What the command writes to its standard output.
        out: &'a mut (dyn Write + Send),
        /// What it writes to its standard error.
        err: &'a mut (dyn Write + Send),
    },
}

/// Carry what `child` says until it ends, and say how it ended.
///
/// `child` must have been started with all three streams piped; a stream that
/// was not is simply not carried.
///
/// # Errors
/// What waiting for the child answered. A failure to copy is not an error of
/// the run: the command ran, and a terminal that went away mid-sentence does
/// not change how it ended.
pub fn carried_until_it_ends(
    child: &mut Child,
    carrying: Carrying<'_>,
) -> std::io::Result<ExitStatus> {
    let said = child.stdout.take();
    let complained = child.stderr.take();
    let input = child.stdin.take();

    match carrying {
        Carrying::ToThePerson => {
            if let Some(mut into_it) = input {
                // **Not waited for.** A terminal never ends, so a thread
                // copying from it is still reading when the command is done,
                // and it ends with this process.
                std::thread::spawn(move || {
                    let mut from = std::io::stdin();
                    copied(&mut from, &mut into_it);
                });
            }
            let mut out = std::io::stdout();
            let mut err = std::io::stderr();
            both_carried(child, said, complained, &mut out, &mut err)
        }
        Carrying::To { out, err } => {
            // No input, which is what a test wants: dropping the pipe is an
            // end of input the command reads at once.
            drop(input);
            both_carried(child, said, complained, out, err)
        }
    }
}

/// Copy both output streams while the child runs, then wait for it.
fn both_carried(
    child: &mut Child,
    said: Option<std::process::ChildStdout>,
    complained: Option<std::process::ChildStderr>,
    out: &mut (dyn Write + Send),
    err: &mut (dyn Write + Send),
) -> std::io::Result<ExitStatus> {
    std::thread::scope(|scope| {
        if let Some(mut from) = said {
            scope.spawn(move || copied(&mut from, out));
        }
        if let Some(mut from) = complained {
            scope.spawn(move || copied(&mut from, err));
        }
        child.wait()
    })
}

/// Copy until the end, and flush.
///
/// **Errors are let go of on purpose**: the only ones are a destination that
/// went away â€” a closed terminal â€” or a source that broke, and in both cases
/// the command's own end is what the caller reports.
fn copied(from: &mut dyn Read, to: &mut (dyn Write + Send)) {
    if std::io::copy(from, to).is_ok() {
        to.flush().ok();
    }
}
