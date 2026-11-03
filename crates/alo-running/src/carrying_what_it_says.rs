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
                // copying from it is still waiting when the command is done,
                // and it ends with this process.
                std::thread::spawn(move || offered_as_it_arrives(&mut into_it));
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

/// Carry the person's input into the command, **reading only what has
/// arrived**.
///
/// # Why it waits to be told, rather than reading
///
/// This used to read this process's input straight away, whether or not the
/// command ever wanted any. A process that reads its terminal from a
/// **background** process group is stopped by the terminal (`SIGTTIN`), all of
/// it, so `try` was stopped before its command could end. That is every `try`
/// run by `timeout`, which gives what it runs a process group of its own, and
/// every `try … &` at a prompt. Measured on a booted machine on 2026-11-02
/// (main `8263ddcb`, once): every boxed run under `timeout 60` ended 124 with no
/// output, `/usr/bin/true` included.
///
/// So it asks first whether anything is there to read, which a terminal answers
/// without stopping anybody, and reads only when something is. A command run
/// in the background now ends when it ends, as any other program run so would;
/// if the person types while it runs, the read is what the terminal stops, as
/// it would stop `cat &`.
///
/// # When it ends
///
/// At the end of the input — the command is then told its input is over — or
/// when the command will take no more. Neither is waited for by the caller.
fn offered_as_it_arrives(into_it: &mut std::process::ChildStdin) {
    use rustix::event::{PollFd, PollFlags, poll};
    use rustix::io::{Errno, read};
    use std::os::fd::AsFd;

    let from = std::io::stdin();
    let from = from.as_fd();
    let mut arrived = [0_u8; 8192];
    loop {
        let mut asking = [PollFd::new(&from, PollFlags::IN)];
        match poll(&mut asking, None) {
            // Readable, or hung up, or broken: in every case a read now does
            // not wait, and a hang-up or a fault reads as the end.
            Ok(_) => {}
            // Asked again rather than read: a read now could be the one the
            // terminal stops.
            Err(Errno::INTR) => continue,
            Err(_) => return,
        }
        let length = match read(from, &mut arrived) {
            Ok(0) => return,
            Ok(length) => length,
            Err(Errno::INTR | Errno::AGAIN) => continue,
            Err(_) => return,
        };
        let Some(these) = arrived.get(..length) else {
            return;
        };
        if into_it.write_all(these).is_err() || into_it.flush().is_err() {
            return;
        }
    }
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
