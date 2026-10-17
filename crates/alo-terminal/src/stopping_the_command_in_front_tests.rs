//! That a real command on a real pseudoterminal really stops, and that the two
//! refusals refuse.
//!
//! # Why `sleep` and not something that prints
//!
//! A command that prints could be *believed* to have stopped because the screen
//! went quiet, which is the measurement standing in for the thing. `sleep 60`
//! has one observable property — it is there, or it is not — and the terminal
//! reports when nothing is in front of it any more. So what is measured is the
//! command going away rather than the output stopping.

use std::time::{Duration, Instant};

use super::NotStopped;
use crate::a_shell_on_a_pseudoterminal::AShellOnAPseudoterminal;
use crate::how_big_the_grid_is::HowBig;
use crate::terminal_output_becomes_a_grid::TerminalOutput;

/// Long enough for a process to start or go on a loaded machine, short enough
/// that a failure is reported rather than waited for.
const PATIENCE: Duration = Duration::from_secs(10);

/// A shell on a pseudoterminal, with a screen for it to write onto.
fn a_shell() -> (AShellOnAPseudoterminal, TerminalOutput) {
    let shell = AShellOnAPseudoterminal::running(
        HowBig::of(80, 24, 100).expect("a terminal"),
        "/bin/sh",
        &["-s"],
    )
    .expect("a shell starts on a pseudoterminal");
    let screen = TerminalOutput::of(HowBig::of(80, 24, 100).expect("a terminal"));
    (shell, screen)
}

/// Wait until the terminal has something other than the shell in front, or give
/// up loudly.
fn until_a_command_is_in_front(shell: &mut AShellOnAPseudoterminal, screen: &mut TerminalOutput) {
    let began = Instant::now();
    while began.elapsed() < PATIENCE {
        drop(shell.what_it_wrote(screen));
        if shell
            .the_command_in_front()
            .is_some_and(|front| front != shell.the_shell_itself())
        {
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("no command reached the front of the terminal, so nothing below is being measured");
}

/// **A command that is running really stops.**
///
/// `sleep 60` is started, confirmed to be in front, interrupted, and then the
/// terminal is asked again until the shell is back in front — which is the
/// machine saying the command is gone rather than the screen merely going
/// quiet.
#[test]
fn a_command_in_front_is_stopped() {
    let (mut shell, mut screen) = a_shell();
    shell
        .a_person_typed(b"sleep 60\n")
        .expect("what a person types reaches the shell");
    until_a_command_is_in_front(&mut shell, &mut screen);

    let stopped = shell
        .stop_the_command_in_front()
        .expect("a running command can be stopped");
    assert_ne!(
        stopped,
        shell.the_shell_itself(),
        "the shell was interrupted rather than the command in front of it"
    );

    let began = Instant::now();
    let mut gone = false;
    while began.elapsed() < PATIENCE {
        drop(shell.what_it_wrote(&mut screen));
        if shell.the_command_in_front() == Some(shell.the_shell_itself()) {
            gone = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    assert!(
        gone,
        "the command was interrupted and is still in front of the terminal"
    );
}

/// **And then there is nothing waiting**, which is the two halves of this
/// promise agreeing with each other.
///
/// A person asks why a command is waiting, stops it, and asks again. If the
/// second answer were still an answer, one of the two readings would be wrong
/// about the same moment.
#[test]
fn after_stopping_there_is_nothing_waiting() {
    let (mut shell, mut screen) = a_shell();
    shell
        .a_person_typed(b"sleep 60\n")
        .expect("what a person types reaches the shell");
    until_a_command_is_in_front(&mut shell, &mut screen);

    assert!(
        shell.why_the_command_in_front_is_waiting().is_some(),
        "a running command is not reported as waiting for anything, so this test is not \
         measuring what it says"
    );
    shell
        .stop_the_command_in_front()
        .expect("a running command can be stopped");

    let began = Instant::now();
    let mut quiet = false;
    while began.elapsed() < PATIENCE {
        drop(shell.what_it_wrote(&mut screen));
        if shell.why_the_command_in_front_is_waiting().is_none() {
            quiet = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    assert!(
        quiet,
        "the command was stopped and the terminal still reports something waiting"
    );
}

/// **A person at a prompt is refused**, by name.
///
/// Interrupting a shell at a prompt does nothing useful on a good day and ends
/// the session on a bad one, so it is a refusal rather than a signal nobody
/// needed.
#[test]
fn stopping_when_nothing_is_running_is_refused() {
    let (shell, _) = a_shell();

    // The shell is in front from the moment it starts: nothing has run yet.
    let why = shell
        .stop_the_command_in_front()
        .expect_err("a person at a prompt has nothing to stop");
    assert!(
        matches!(why, NotStopped::NothingIsRunning),
        "the wrong refusal: {why}"
    );
    // And it says so in words somebody can read.
    assert!(
        format!("{why}").contains("nothing is running"),
        "the refusal does not say what happened: {why}"
    );
}

/// **The three refusals each say something different**, so a caller showing one
/// is not showing the same sentence three times.
#[test]
fn each_refusal_says_its_own_thing() {
    let said = [
        format!("{}", NotStopped::NothingInFront),
        format!("{}", NotStopped::NothingIsRunning),
        format!(
            "{}",
            NotStopped::TheMachineRefused {
                group: 1234,
                why: rustix::io::Errno::PERM,
            }
        ),
    ];
    assert_ne!(said[0], said[1]);
    assert_ne!(said[1], said[2]);
    assert_ne!(said[0], said[2]);
    // And the one about the machine names the group, which is the fact a person
    // looking at a journal needs.
    assert!(said[2].contains("1234"), "{}", said[2]);
}
