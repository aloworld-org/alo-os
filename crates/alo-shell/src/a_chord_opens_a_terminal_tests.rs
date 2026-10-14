//! **Ctrl+Alt+T opens alo's own terminal, and asks for nothing else.**
//!
//! The refusal paths are tested as carefully as the one that works, which for
//! this file means three of them: a chord that is not this action, a machine
//! with no installed terminal, and the question of whether the chord is still
//! taken when there is nothing to open.
//!
//! # What these two tests used to assert, and why they changed
//!
//! They asserted that the chord **asked the bus to start the installed
//! terminal** — one ask per press, two presses two terminals. That was the
//! behaviour, and it was the behaviour because alo had no terminal of its own:
//! this file's header records that it exists because *nothing could open a
//! terminal at all*.
//!
//! Now it opens alo's own, so the old assertions are assertions about the
//! stopgap. They are rewritten rather than deleted, and each says what it now
//! holds — including that **nothing is asked of the bus**, which is the half a
//! deletion would have lost.
//!
//! # The fallback is not tested here, and that is said rather than hidden
//!
//! When alo's own terminal cannot be opened, the installed one is asked for
//! instead. **No test here reaches that branch**, because it needs
//! `AShellOnAPseudoterminal::the_persons_own` to fail and this test cannot
//! stop a machine from opening a pseudoterminal. It is reachable on a real
//! machine — a container with no `/dev/pts`, a process out of file
//! descriptors — and `open_alos_own_terminal` says on the error stream why,
//! which is the half that is checkable by reading.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_shortcuts::{Action, Chord, Key, Modifier, Modifiers, Shortcuts};

use crate::Server;

/// A session on this machine's seat, with a private runtime directory.
fn a_session(what: &str) -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind(runtime.path(), what).expect("a display");
    (runtime, server)
}

/// The chord the release ships for a terminal, read out of the shipped list
/// rather than typed here.
///
/// **Read rather than written**, so this test cannot drift from the binding: a
/// copy of `Ctrl+Alt+T` here would keep passing after somebody changed the
/// default, and the thing it is meant to prove is that the chord a person
/// presses is the one that works.
fn the_terminals_chord(shortcuts: &Shortcuts) -> Chord {
    shortcuts
        .chord_for(Action::Terminal)
        .expect("the release ships a chord for the terminal")
}

/// A terminal, named the way `shipped.toml` names one.
fn a_terminal() -> alo_dock::AppId {
    alo_dock::AppId::named("app.devsuite.Ptyxis").expect("a named application")
}

#[test]
fn the_chord_opens_alos_own_terminal_and_asks_the_bus_for_nothing() {
    let (_runtime, mut server) = a_session("a-terminal-chord");
    let shortcuts = Shortcuts::shipped();
    // **An installed terminal is present throughout**, which is what makes
    // this test mean something: the assertion is that alo's own is opened
    // *although* there is one to fall back to.
    server.its_terminal_is(a_terminal());

    assert_eq!(server.how_many_are_waiting_to_start(), 0);
    assert!(server.alos_own_terminal().is_none(), "nothing is open yet");
    assert!(
        server.a_chord_opens_a_terminal(&shortcuts, the_terminals_chord(&shortcuts)),
        "the terminal's own chord was not taken by the terminal's road"
    );
    assert!(
        server.alos_own_terminal().is_some(),
        "the chord was taken and alo's own terminal was not opened"
    );
    assert_eq!(
        server.how_many_are_waiting_to_start(),
        0,
        "the installed terminal was asked for as well, so the chord opens two terminals"
    );
}

#[test]
fn two_presses_open_one_terminal() {
    let (_runtime, mut server) = a_session("two-terminals");
    let shortcuts = Shortcuts::shipped();
    let chord = the_terminals_chord(&shortcuts);
    server.its_terminal_is(a_terminal());

    assert!(server.a_chord_opens_a_terminal(&shortcuts, chord));
    let first = std::ptr::from_ref(server.alos_own_terminal().expect("it opened"));
    assert!(server.a_chord_opens_a_terminal(&shortcuts, chord));
    let second = std::ptr::from_ref(server.alos_own_terminal().expect("it is still open"));

    assert_eq!(
        first, second,
        "a second press replaced the terminal, so it started another shell"
    );
    assert_eq!(
        server.how_many_are_waiting_to_start(),
        0,
        "a second press asked the bus for a terminal as well"
    );
}

#[test]
fn another_chord_is_left_for_the_layers_below() {
    let (_runtime, mut server) = a_session("another-chord");
    let shortcuts = Shortcuts::shipped();
    server.its_terminal_is(a_terminal());
    // Settings', which the layer below this one answers.
    let settings = shortcuts
        .chord_for(Action::Settings)
        .expect("the release ships a chord for Settings");
    assert_ne!(
        settings,
        the_terminals_chord(&shortcuts),
        "Settings and the terminal share a chord, so this test proves nothing"
    );

    assert!(
        !server.a_chord_opens_a_terminal(&shortcuts, settings),
        "Settings' chord was taken by the terminal's road"
    );
    assert_eq!(
        server.how_many_are_waiting_to_start(),
        0,
        "another action's chord asked for a terminal"
    );
}

#[test]
fn an_unbound_chord_is_left_for_the_layers_below() {
    let (_runtime, mut server) = a_session("unbound-chord");
    let shortcuts = Shortcuts::shipped();
    server.its_terminal_is(a_terminal());
    // Nothing the release ships is bound to this.
    let unbound = Chord::checked(Modifiers::just(Modifier::Super).and(Modifier::Ctrl), Key::Q)
        .expect("Super+Ctrl+Q is a chord a person could bind");
    assert!(
        shortcuts.action_for(unbound).is_none(),
        "the release binds Super+Ctrl+Q after all, so this test proves nothing"
    );

    assert!(!server.a_chord_opens_a_terminal(&shortcuts, unbound));
    assert_eq!(server.how_many_are_waiting_to_start(), 0);
}

/// **A machine with no terminal still takes the chord**, which is the deliberate
/// half of the answer and the one worth a test of its own.
///
/// Falling through would hand `Action::Terminal` to the canvas road, which has
/// no arm for it — so a person pressing a chord that *is* bound would get the
/// behaviour of one that is not.
#[test]
fn a_machine_with_no_terminal_takes_the_chord_and_asks_for_nothing() {
    let (_runtime, mut server) = a_session("no-terminal");
    let shortcuts = Shortcuts::shipped();
    // Nobody has told this session what its terminal is, which is the state
    // every session starts in and the state a machine without one stays in.

    assert!(
        server.a_chord_opens_a_terminal(&shortcuts, the_terminals_chord(&shortcuts)),
        "the chord fell through, so it would reach a road with no arm for it"
    );
    assert_eq!(
        server.how_many_are_waiting_to_start(),
        0,
        "a session with no terminal asked for one anyway"
    );
}
