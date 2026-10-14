//! **A real shell, a real pseudoterminal, and what the grid then says.**
//!
//! These start processes, which is the point: the thing under test is the one
//! place in this crate where a child process exists, and a test that mocked it
//! would be a test of the mock.
//!
//! # Why every test has a deadline
//!
//! A child writes when it is scheduled to, not when a test would like. So each
//! of these pumps in a loop until what it is waiting for arrives or a deadline
//! passes — and **the deadline failing is itself an assertion**, with a message
//! saying what the grid held instead. A test that looped forever would hang a
//! gate, which is the worst way to report a failure.

use std::time::{Duration, Instant};

use alo_terminal::what_a_person_typed::{ANamedKey, Held, WhatWasTyped};

use super::{A_SENSIBLE_FIRST_GRID, TheTerminal};

/// Long enough for a shell to start and print on a loaded machine, short
/// enough that a failure is reported rather than waited for.
const PATIENCE: Duration = Duration::from_secs(10);

/// Pump until the grid says something, or the deadline passes.
fn pump_until(terminal: &mut TheTerminal, enough: impl Fn(&TheTerminal) -> bool) -> bool {
    let deadline = Instant::now() + PATIENCE;
    while Instant::now() < deadline {
        terminal.what_the_shell_wrote();
        if enough(terminal) {
            return true;
        }
        if terminal.has_finished() {
            // **One more pump after it finished.** Bytes written just before
            // the shell exited are still in the kernel's queue, and stopping
            // here would lose the last line of every command that ends
            // quickly — which is most of them.
            terminal.what_the_shell_wrote();
            return enough(terminal);
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    enough(terminal)
}

/// Whether any line of the grid holds this.
fn grid_says(terminal: &TheTerminal, what: &str) -> bool {
    terminal
        .screen()
        .lines()
        .iter()
        .any(|line| line.contains(what))
}

/// Type a string through the real conversion, character by character.
///
/// **Nothing is assembled by hand.** Every byte comes from
/// `alo_terminal::what_a_person_typed`, so what reaches the shell is what a
/// person pressing those keys would send.
fn type_it(terminal: &mut TheTerminal, text: &str) {
    for character in text.chars() {
        terminal.a_person_typed_a_key(WhatWasTyped::Character(character), Held::default());
    }
}

/// **A terminal opens, runs the person's shell, and answers what was typed.**
///
/// The whole point of the file, end to end: the grid is where a person would
/// read the answer, and this reads it there rather than inspecting bytes.
#[test]
fn a_terminal_runs_a_shell_and_answers_what_was_typed() {
    let mut terminal =
        TheTerminal::opened(A_SENSIBLE_FIRST_GRID).expect("this machine can open a terminal");

    type_it(&mut terminal, "echo through-the-whole-chain");
    terminal.a_person_typed_a_key(WhatWasTyped::Named(ANamedKey::Enter), Held::default());

    let arrived = pump_until(&mut terminal, |terminal| {
        grid_says(terminal, "through-the-whole-chain")
    });
    assert!(
        arrived,
        "the grid never showed what the shell answered. It held: {:?}",
        terminal.screen().lines()
    );
}

/// **`Enter` is what makes a shell act**, which is the claim the carriage
/// return rests on.
///
/// The same command typed **without** `Enter` must produce no answer. Without
/// this, the test above would pass on a terminal that sent a newline, a
/// carriage return, or both — and the one that is right is the one a shell's
/// own line discipline expects.
#[test]
fn without_enter_the_shell_does_not_act() {
    let mut terminal =
        TheTerminal::opened(A_SENSIBLE_FIRST_GRID).expect("this machine can open a terminal");

    type_it(&mut terminal, "echo not-submitted-yet");

    // Give it as long as the successful test gets, and expect nothing.
    let deadline = Instant::now() + Duration::from_secs(1);
    while Instant::now() < deadline {
        terminal.what_the_shell_wrote();
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(
        !grid_says(&terminal, "not-submitted-yet\n"),
        "the shell answered a command that was never submitted"
    );
}

/// **`Ctrl`+`C` reaches the shell as an interrupt**, asserted by what it does
/// rather than by the byte it sends.
///
/// A shell running `sleep` and then told to interrupt goes back to a prompt and
/// answers the next command. If `Ctrl`+`C` arrived as the letter `c`, the sleep
/// would still be running and the second command would never answer.
#[test]
fn control_c_interrupts_what_is_running() {
    let mut terminal =
        TheTerminal::opened(A_SENSIBLE_FIRST_GRID).expect("this machine can open a terminal");

    type_it(&mut terminal, "sleep 30");
    terminal.a_person_typed_a_key(WhatWasTyped::Named(ANamedKey::Enter), Held::default());
    // Let it actually start sleeping.
    std::thread::sleep(Duration::from_millis(300));
    terminal.what_the_shell_wrote();

    terminal.a_person_typed_a_key(
        WhatWasTyped::Character('c'),
        Held {
            control: true,
            alt: false,
        },
    );

    type_it(&mut terminal, "echo after-the-interrupt");
    terminal.a_person_typed_a_key(WhatWasTyped::Named(ANamedKey::Enter), Held::default());

    let arrived = pump_until(&mut terminal, |terminal| {
        grid_says(terminal, "after-the-interrupt")
    });
    assert!(
        arrived,
        "the shell never answered after an interrupt, so Ctrl+C did not interrupt the sleep. \
         The grid held: {:?}",
        terminal.screen().lines()
    );
}

/// **A shell that has gone says so, and stops taking what is typed.**
///
/// The refusal path. A terminal that kept accepting keys after its shell ended
/// would be a window a person types into that cannot answer, with nothing
/// saying why.
#[test]
fn a_terminal_whose_shell_has_gone_takes_nothing() {
    let mut terminal =
        TheTerminal::opened(A_SENSIBLE_FIRST_GRID).expect("this machine can open a terminal");

    type_it(&mut terminal, "exit");
    terminal.a_person_typed_a_key(WhatWasTyped::Named(ANamedKey::Enter), Held::default());

    let deadline = Instant::now() + PATIENCE;
    while Instant::now() < deadline && !terminal.has_finished() {
        terminal.what_the_shell_wrote();
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(
        terminal.has_finished(),
        "a shell told to exit never reported that it had, within {PATIENCE:?}"
    );

    assert!(
        !terminal.a_person_typed_a_key(WhatWasTyped::Character('x'), Held::default()),
        "a shell that has gone accepted what was typed"
    );
    assert!(
        !terminal.what_the_shell_wrote(),
        "a shell that has gone reported that it had written something"
    );
}

/// **A key that produces no bytes sends nothing**, rather than sending an empty
/// write.
#[test]
fn a_key_with_nothing_to_send_is_not_sent() {
    let mut terminal =
        TheTerminal::opened(A_SENSIBLE_FIRST_GRID).expect("this machine can open a terminal");
    assert!(
        !terminal.a_person_typed_a_key(
            WhatWasTyped::Named(ANamedKey::Function(99)),
            Held::default()
        ),
        "a key the terminal crate sends nothing for was reported as sent"
    );
}

/// **The starting grid is the size the shell is told**, asked of the shell
/// rather than of our own type.
///
/// `stty size` runs inside the child, so this is what the **child** sees. This
/// crate agreeing with itself would prove nothing — and the figure is the
/// starting one, which the drawing will replace.
#[test]
fn the_shell_is_told_the_starting_grid() {
    let mut terminal =
        TheTerminal::opened(A_SENSIBLE_FIRST_GRID).expect("this machine can open a terminal");

    type_it(&mut terminal, "stty size");
    terminal.a_person_typed_a_key(WhatWasTyped::Named(ANamedKey::Enter), Held::default());

    let arrived = pump_until(&mut terminal, |terminal| grid_says(terminal, "24 80"));
    assert!(
        arrived,
        "the child was not told it had 24 lines and 80 columns. The grid held: {:?}",
        terminal.screen().lines()
    );
}

/// **The chord opens alo's own terminal, and a second press does not open a
/// second shell.**
///
/// Through the `Server`, which is the road a keypress really takes, rather than
/// by calling `TheTerminal::opened` twice.
#[test]
fn the_server_opens_one_terminal_however_often_it_is_asked() {
    use std::os::unix::fs::PermissionsExt as _;
    let directory = tempfile::tempdir().expect("a temporary directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server =
        crate::Server::bind(directory.path(), "alo-terminal-opens-once").expect("a bound socket");

    assert!(
        server.alos_own_terminal().is_none(),
        "a fresh machine has no terminal open"
    );
    assert!(
        !server.the_terminal_is_taking_the_keys(),
        "a machine with no terminal is taking the keys"
    );

    assert!(server.open_alos_own_terminal(A_SENSIBLE_FIRST_GRID));
    assert!(server.alos_own_terminal().is_some());
    assert!(
        server.the_terminal_is_taking_the_keys(),
        "an open terminal with a running shell should be taking the keys"
    );

    // **A second ask is the same terminal.** Compared by the pointer, because
    // two `TheTerminal` values would both be "some" and only their identity
    // says whether a second shell was started.
    let first = std::ptr::from_ref(server.alos_own_terminal().expect("it is open"));
    assert!(server.open_alos_own_terminal(A_SENSIBLE_FIRST_GRID));
    let second = std::ptr::from_ref(server.alos_own_terminal().expect("it is still open"));
    assert_eq!(
        first, second,
        "asking twice replaced the terminal, so a second shell was started"
    );
}
