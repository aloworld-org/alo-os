//! **A terminal that closes must not stop the machine, and before this it
//! could.**
//!
//! `alacritty_terminal`'s `Drop for Pty` sends `SIGHUP` and then calls
//! `self.child.wait()` with **no deadline**. `SIGHUP` is a signal a process may
//! ignore, and a process that ignores it never dies, so that wait never
//! returns.
//!
//! A terminal is dropped when a person closes its window, on the thread that
//! draws every window on the machine. So the failure is not a slow close: it is
//! **the whole desktop stopping**, for ever, because of what a shell chose to do
//! with one signal.
//!
//! # How it was found, and why that matters more than the bug
//!
//! As a test in `alo-shell` hanging for **ten minutes** in CI and being killed
//! by the harness — the last test of 10395, which is the position at which a
//! hang is most easily read as *CI being slow*.
//!
//! `CLAUDE.md`'s question is *what would have to be true for this to pass while
//! the product is broken*, and a hang answers worse than a failure does: it
//! produces no assertion, no diff and no name of anything that was wrong. **A
//! test that hangs is not a test that fails.** So this file does not test the
//! fix by observing a shell close nicely — every shell does that. It tests it
//! with a process that **ignores `SIGHUP` on purpose**, which is the only shape
//! that distinguishes the fix from its absence.
//!
//! # Why a bound and not a mock
//!
//! Nothing here can be faked: the fault is in a destructor of a rented type
//! running against a real child on a real pseudoterminal. So the test spawns
//! one, drops it, and asserts the drop **returned**. A clock is the only
//! instrument that can see this, and the bound is generous for exactly that
//! reason — it is not measuring how fast the close is, only that it ends.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::time::{Duration, Instant};

use alo_terminal::{AShellOnAPseudoterminal, HowBig};

/// Generous, and deliberately so.
///
/// The fix's own bound is forty polls of five milliseconds — two hundred
/// milliseconds — before it insists, and `SIGKILL` then takes the kernel a
/// moment. This is that with room for a loaded machine, and it is still four
/// hundred times shorter than the ten minutes the fault cost. **It is not a
/// performance assertion**: anything finite would have caught this.
const LONG_ENOUGH_TO_BE_SURE_IT_ENDED: Duration = Duration::from_secs(20);

/// A small grid; nothing here looks at the screen.
fn a_grid() -> HowBig {
    HowBig::of(40, 10, 50).expect("a small grid")
}

/// **A shell that ignores `SIGHUP` is still closed.**
///
/// `trap '' HUP` is the shape a real one arrives in: a process started from
/// something that was itself ignoring `SIGHUP` inherits that disposition, which
/// is how a CI runner's shell ends up unkillable by the signal a terminal
/// sends.
///
/// Before the `Drop` in `a_shell_on_a_pseudoterminal.rs` this did not finish.
/// Not slowly — **not at all**, until a harness killed the process.
#[test]
fn a_shell_that_ignores_the_hangup_is_still_closed() {
    let shell =
        AShellOnAPseudoterminal::running(a_grid(), "/bin/sh", &["-c", "trap '' HUP; sleep 600"])
            .expect("a shell on a pseudoterminal");

    // Let it reach the `trap` before it is signalled. Without this the test
    // could pass by signalling a process that had not yet installed the
    // handler, which would make it a test of nothing.
    std::thread::sleep(Duration::from_millis(300));

    let started = Instant::now();
    drop(shell);
    let took = started.elapsed();

    assert!(
        took < LONG_ENOUGH_TO_BE_SURE_IT_ENDED,
        "closing a terminal whose shell ignores SIGHUP took {took:?}, which means it did not end"
    );
}

/// **And one that honours it is closed the ordinary way**, which is the control.
///
/// Without this the test above would pass on a `Drop` that sent `SIGKILL`
/// immediately and never gave any shell its exit hooks. What separates the two
/// is that this one is **much** faster: it goes on the first or second poll
/// rather than after the full bound.
#[test]
fn a_shell_that_honours_the_hangup_goes_at_once() {
    let shell = AShellOnAPseudoterminal::running(a_grid(), "/bin/sh", &["-c", "sleep 600"])
        .expect("a shell on a pseudoterminal");
    std::thread::sleep(Duration::from_millis(300));

    let started = Instant::now();
    drop(shell);
    let took = started.elapsed();

    assert!(
        took < Duration::from_secs(2),
        "a shell that honours SIGHUP took {took:?} to close, so the gentle road is not being tried \
         first"
    );
}

/// **A shell that has already gone is closed without complaint.**
///
/// The case a destructor meets most often and the one where an error path would
/// show: the child is a zombie, or has been reaped, and every signal and wait
/// answers an error. All of those mean *the shell is not running*, which is what
/// was being asked for.
#[test]
fn a_shell_that_has_already_finished_closes_cleanly() {
    let mut shell = AShellOnAPseudoterminal::running(a_grid(), "/bin/sh", &["-c", "exit 0"])
        .expect("a shell on a pseudoterminal");

    // Read until it reports that it has gone, so the drop below meets a child
    // that has genuinely exited rather than one that merely might have.
    let mut screen = alo_terminal::TerminalOutput::of(a_grid());
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut finished = false;
    while Instant::now() < deadline {
        match shell.what_it_wrote(&mut screen) {
            Ok(alo_terminal::WhatArrived::ItFinished) => {
                finished = true;
                break;
            }
            Ok(_) => std::thread::sleep(Duration::from_millis(5)),
            Err(why) => panic!("the shell could not be read: {why}"),
        }
    }
    assert!(finished, "the shell never reported that it had finished");

    let started = Instant::now();
    drop(shell);
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "closing a shell that had already gone waited anyway"
    );
}
