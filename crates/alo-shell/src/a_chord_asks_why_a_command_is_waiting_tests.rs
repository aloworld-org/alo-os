//! The chord really reaches the answer, through the road a keypress takes.
//!
//! Not by calling `TheTerminal::asked_why_the_command_in_front_is_waiting`
//! directly: the whole fault this change fixes was a built answer with no road
//! to it, so a test that called the answer would be testing the half that was
//! never broken. Each test below presses a chord.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err or None is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_shortcuts::{Action, Chord, Shortcuts};

use crate::the_terminal_a_person_opened::A_SENSIBLE_FIRST_GRID;

/// A server on a private socket of its own, as the other terminal tests build
/// one.
fn a_machine(named: &str) -> (tempfile::TempDir, crate::Server) {
    let directory = tempfile::tempdir().expect("a temporary directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = crate::Server::bind(directory.path(), named).expect("a bound socket");
    (directory, server)
}

/// The chord a fresh machine has for this, read from the shipped defaults rather
/// than written here — so rebinding it in `defaults.rs` does not quietly make
/// these tests about a key nobody presses.
fn the_chord() -> (Shortcuts, Chord) {
    let shortcuts = Shortcuts::shipped();
    let chord = shortcuts
        .chord_for(Action::WhyIsItWaiting)
        .expect("a fresh machine binds a chord to asking why a command is waiting");
    (shortcuts, chord)
}

/// **The chord reaches the answer**, which nothing did before this change.
///
/// A shell is running with nothing in front of it, so the honest answer is that
/// there is nothing waiting — and what matters here is that the chord was
/// *carried* rather than falling through to whatever is behind it.
#[test]
fn the_chord_is_carried_when_a_terminal_is_open() {
    let (_directory, mut server) = a_machine("alo-asks-why-carried");
    assert!(server.open_alos_own_terminal(A_SENSIBLE_FIRST_GRID));
    let (shortcuts, chord) = the_chord();

    assert!(
        server.a_chord_asks_why_a_command_is_waiting(&shortcuts, chord),
        "the chord bound to asking why a command is waiting was not carried"
    );
}

/// **And it is carried with no terminal open**, rather than handed on.
///
/// A person who pressed it deliberately should not have it come to mean
/// something else because nothing was open. The answer is *nothing changed*, and
/// the chord is still consumed.
#[test]
fn the_chord_is_carried_even_with_no_terminal() {
    let (_directory, mut server) = a_machine("alo-asks-why-no-terminal");
    assert!(server.alos_own_terminal().is_none());
    let (shortcuts, chord) = the_chord();

    assert!(
        server.a_chord_asks_why_a_command_is_waiting(&shortcuts, chord),
        "the chord was handed on because nothing was open"
    );
    assert!(
        !server.alos_terminal_was_asked_why(),
        "a machine with no terminal reported that something changed"
    );
}

/// **Another chord is not this one**, which is the control.
///
/// Without it, a dispatch that answered `true` for everything would pass the two
/// tests above and swallow every other shortcut on the machine.
#[test]
fn a_different_chord_is_not_this_one() {
    let (_directory, mut server) = a_machine("alo-asks-why-control");
    assert!(server.open_alos_own_terminal(A_SENSIBLE_FIRST_GRID));
    let shortcuts = Shortcuts::shipped();
    let somebody_elses = shortcuts
        .chord_for(Action::Terminal)
        .expect("a fresh machine binds a chord to opening a terminal");

    assert!(
        !server.a_chord_asks_why_a_command_is_waiting(&shortcuts, somebody_elses),
        "the chord that opens a terminal was taken for the one that asks why"
    );
}

/// **A shell at a prompt has nothing waiting, and alo says nothing.**
///
/// The case every person meets first, and the one a careless answer gets wrong:
/// asking about the shell itself answers *waiting for the keyboard* every time,
/// which is why `why_the_command_in_front_is_waiting` refuses the shell. This
/// holds that the refusal reaches the transcript as *nothing said* rather than as
/// a line a person would read and act on.
#[test]
fn a_prompt_has_nothing_waiting_so_alo_says_nothing() {
    let (_directory, mut server) = a_machine("alo-asks-why-prompt");
    assert!(server.open_alos_own_terminal(A_SENSIBLE_FIRST_GRID));
    let (shortcuts, chord) = the_chord();

    assert!(server.a_chord_asks_why_a_command_is_waiting(&shortcuts, chord));

    let terminal = server.alos_own_terminal().expect("it is open");
    assert_eq!(
        terminal.what_alo_said().lines(),
        0,
        "alo answered about a shell sitting at a prompt: {:?}",
        terminal.what_alo_said().remark()
    );
}

/// **A command that really waits gets a real answer, through the chord.**
///
/// The test this change exists for. Everything above proves the road is
/// connected; this proves what arrives at the end of it is the answer, about a
/// process that is genuinely waiting, on this machine's own kernel.
///
/// `cat` with no argument is the case `docs/features.md` names in its own
/// measurement: it reads its standard input, which is the terminal, so a person
/// is the only thing that can release it — **waiting for the keyboard**.
///
/// # Why it polls
///
/// A shell takes a moment to start the command, and the kernel takes a moment to
/// park it. Polling for the answer rather than sleeping a fixed time is
/// `alo-terminal`'s own rule for these: *a fixed sleep is either slow or flaky,
/// and on a loaded builder it is both.* A reader that was genuinely broken never
/// reaches the answer and the test fails on the deadline rather than passing by
/// luck.
#[test]
fn a_command_that_waits_for_the_keyboard_is_answered_through_the_chord() {
    let (_directory, mut server) = a_machine("alo-asks-why-really");
    assert!(server.open_alos_own_terminal(A_SENSIBLE_FIRST_GRID));
    let (shortcuts, chord) = the_chord();

    // The person types a command that waits for them.
    assert!(server.alos_own_terminal().is_some());
    server
        .alos_own_terminal_mut_for_tests()
        .expect("it is open")
        .a_person_typed(b"cat\n");

    let wanted = alo_terminal::why_a_command_is_waiting::WhyItIsWaiting::ForTheKeyboard
        .the_word()
        .key();
    let until = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        // Drain whatever the shell wrote, as a frame would, so the terminal's
        // idea of what is in front is current.
        let _wrote = server.the_terminal_was_written_to();
        assert!(server.a_chord_asks_why_a_command_is_waiting(&shortcuts, chord));
        let said = server
            .alos_own_terminal()
            .expect("it is open")
            .what_alo_said()
            .remark()
            .and_then(|remark| match remark {
                alo_terminal::AloSRemark::InOurWords(key) => Some(key.clone()),
                alo_terminal::AloSRemark::AnExplanation(_) => None,
            });
        if said.as_ref() == Some(&wanted) {
            break;
        }
        assert!(
            std::time::Instant::now() < until,
            "a command reading the terminal was never answered 'waiting for the keyboard'; \
             alo said {said:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}
