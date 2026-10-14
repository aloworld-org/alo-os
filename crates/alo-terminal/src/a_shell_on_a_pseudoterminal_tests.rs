//! **A real child process, a real pseudoterminal, and what the screen then
//! says.**
//!
//! These start processes. That is the point — the file they test is the one
//! place in this crate where something outside it exists, and a test of it
//! that mocked the child would be a test of the mock.
//!
//! # Why every test has a deadline
//!
//! A child writes when it is scheduled to, not when a test would like. So each
//! of these reads in a loop until what it is waiting for arrives or a deadline
//! passes — and **the deadline failing is itself an assertion**, with a message
//! saying what the screen held instead. A test that looped forever would hang
//! a gate, which is the worst way to report a failure.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, Instant};

use super::{AShellOnAPseudoterminal, WhatArrived};
use crate::how_big_the_grid_is::HowBig;
use crate::terminal_output_becomes_a_grid::TerminalOutput;

/// Long enough for a process to start and print on a loaded machine, short
/// enough that a failure is reported rather than waited for.
const PATIENCE: Duration = Duration::from_secs(10);

/// A screen a shell can write onto.
fn a_screen() -> TerminalOutput {
    TerminalOutput::of(HowBig::of(80, 24, 100).expect("eighty by twenty-four is a terminal"))
}

/// Read until the screen says something, or the deadline passes.
///
/// Answers whether the shell finished, so a caller can tell *it ended having
/// said this* from *it is still going and has said this so far*.
fn read_until(
    shell: &mut AShellOnAPseudoterminal,
    screen: &mut TerminalOutput,
    enough: impl Fn(&TerminalOutput) -> bool,
) -> bool {
    let deadline = Instant::now() + PATIENCE;
    let mut finished = false;
    while Instant::now() < deadline {
        match shell.what_it_wrote(screen).expect("the shell answers") {
            WhatArrived::Bytes(_) => {
                if enough(screen) {
                    return finished;
                }
            }
            WhatArrived::NothingYet => std::thread::sleep(Duration::from_millis(5)),
            WhatArrived::ItFinished => {
                finished = true;
                // **Read on after it finished.** Bytes written just before the
                // child exited are still in the kernel's queue, and stopping
                // here would lose the last line of every command that ends
                // quickly — which is most of them.
                if enough(screen) {
                    return true;
                }
                break;
            }
        }
    }
    finished
}

/// Whether any line of the screen contains this.
fn screen_says(screen: &TerminalOutput, what: &str) -> bool {
    screen.lines().iter().any(|line| line.contains(what))
}

#[test]
fn a_program_on_a_pseudoterminal_writes_onto_the_screen() {
    let mut shell = AShellOnAPseudoterminal::running(
        HowBig::of(80, 24, 100).expect("a terminal"),
        "/bin/sh",
        &["-c", "echo hello from a real shell"],
    )
    .expect("this machine can start /bin/sh on a pseudoterminal");

    let mut screen = a_screen();
    read_until(&mut shell, &mut screen, |screen| {
        screen_says(screen, "hello from a real shell")
    });

    assert!(
        screen_says(&screen, "hello from a real shell"),
        "the screen never showed what the shell printed. It held: {:?}",
        screen.lines()
    );
}

/// **A shell that has gone says so, rather than looking like one that is
/// quiet.** This is the distinction a `Result<usize>` would have lost, and a
/// caller that got it wrong would close a terminal the moment a program paused.
#[test]
fn a_shell_that_has_exited_reports_that_it_finished() {
    let mut shell = AShellOnAPseudoterminal::running(
        HowBig::of(80, 24, 100).expect("a terminal"),
        "/bin/sh",
        &["-c", "exit 0"],
    )
    .expect("this machine can start /bin/sh");

    let mut screen = a_screen();
    let deadline = Instant::now() + PATIENCE;
    let mut finished = false;
    while Instant::now() < deadline && !finished {
        match shell.what_it_wrote(&mut screen).expect("the shell answers") {
            WhatArrived::ItFinished => finished = true,
            WhatArrived::NothingYet => std::thread::sleep(Duration::from_millis(5)),
            WhatArrived::Bytes(_) => {}
        }
    }
    assert!(
        finished,
        "a shell told to exit never reported that it had, within {PATIENCE:?}"
    );

    // **And it keeps saying so**, so a caller that reads once more does not
    // see it come back to life.
    assert_eq!(
        shell.what_it_wrote(&mut screen).expect("it still answers"),
        WhatArrived::ItFinished
    );
}

/// **A shell that is alive and quiet is not a shell that has gone.** The
/// control for the test above: if `NothingYet` and `ItFinished` were ever
/// confused, this would be the one that caught it.
#[test]
fn a_shell_that_is_only_quiet_does_not_report_that_it_finished() {
    let mut shell = AShellOnAPseudoterminal::running(
        HowBig::of(80, 24, 100).expect("a terminal"),
        "/bin/sh",
        &["-c", "sleep 30"],
    )
    .expect("this machine can start /bin/sh");

    let mut screen = a_screen();
    // Give it long enough to have started and said nothing.
    std::thread::sleep(Duration::from_millis(200));
    for _ in 0..5 {
        let what = shell.what_it_wrote(&mut screen).expect("the shell answers");
        assert_ne!(
            what,
            WhatArrived::ItFinished,
            "a sleeping shell was reported as finished, so a terminal would close under somebody"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// **What a person types reaches the shell, and what it answers comes back.**
/// The round trip, which is the whole point of the file.
#[test]
fn what_a_person_types_reaches_the_shell() {
    let mut shell = AShellOnAPseudoterminal::running(
        HowBig::of(80, 24, 100).expect("a terminal"),
        "/bin/sh",
        // No prompt and no echo, so the screen holds the answer and not the
        // question — this test is about the round trip, not about how a shell
        // decorates it.
        &["-s"],
    )
    .expect("this machine can start /bin/sh");

    let mut screen = a_screen();
    shell
        .a_person_typed(b"echo typed-and-answered\n")
        .expect("the shell takes what was typed");

    read_until(&mut shell, &mut screen, |screen| {
        screen_says(screen, "typed-and-answered")
    });

    assert!(
        screen_says(&screen, "typed-and-answered"),
        "what was typed never came back. The screen held: {:?}",
        screen.lines()
    );
}

/// **The shell is told how big its terminal is**, which is how a program knows
/// where to wrap. Asked of the shell itself rather than of our own type,
/// because what matters is what the **child** sees — this crate agreeing with
/// itself would prove nothing.
#[test]
fn the_shell_is_told_the_size_of_its_terminal() {
    let mut shell = AShellOnAPseudoterminal::running(
        HowBig::of(80, 24, 100).expect("a terminal"),
        "/bin/sh",
        &["-c", "stty size"],
    )
    .expect("this machine can start /bin/sh");

    let mut screen = a_screen();
    read_until(&mut shell, &mut screen, |screen| {
        screen_says(screen, "24 80")
    });

    assert!(
        screen_says(&screen, "24 80"),
        "the child was not told it had 24 lines and 80 columns. `stty size` said: {:?}",
        screen.lines()
    );
}
