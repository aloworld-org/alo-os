//! **Both chords put the keyboard in the alo Bar, and leave what was typed.**

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_shortcuts::{Action, Chord, Key, Modifier, Modifiers, Shortcuts};

use crate::Server;
use crate::composer_keys::ComposerKey;

/// A session on this machine's seat, with a private runtime directory.
fn a_session(what: &str) -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind(runtime.path(), what).expect("a display");
    (runtime, server)
}

/// The chord the release ships for the agent, read out of the shipped list.
///
/// **Read rather than typed**, so this test cannot drift from the binding: a
/// literal `Super+A` here would keep passing after somebody changed the
/// default, and what this proves is that the chord a person presses works.
fn the_agents_chord(shortcuts: &Shortcuts) -> Chord {
    shortcuts
        .chord_for(Action::TheAgent)
        .expect("the release ships a chord for the agent")
}

#[test]
fn the_chord_gives_the_composer_the_keyboard() {
    let (_runtime, mut server) = a_session("the-agents-chord");
    let shortcuts = Shortcuts::shipped();
    assert!(
        !server.composer().has_the_keyboard(),
        "a fresh session already had the keyboard in the composer, so this proves nothing"
    );

    assert!(
        server.a_chord_opens_the_bar(&shortcuts, the_agents_chord(&shortcuts)),
        "the agent's own chord was not taken by the agent's road"
    );
    assert!(
        server.composer().has_the_keyboard(),
        "the chord was taken and the composer still does not have the keyboard"
    );
}

/// **The next letter typed goes into the field**, which is the whole point: the
/// chord is useless if the keyboard arrives somewhere a person cannot type.
#[test]
fn what_is_typed_after_the_chord_reaches_the_field() {
    let (_runtime, mut server) = a_session("typing-after-the-chord");
    let shortcuts = Shortcuts::shipped();
    server.a_chord_opens_the_bar(&shortcuts, the_agents_chord(&shortcuts));

    server.a_key_reached_the_composer(ComposerKey::Letter('h'));
    server.a_key_reached_the_composer(ComposerKey::Letter('i'));
    assert_eq!(
        server.composer().typed(),
        "hi",
        "the keyboard was said to be in the field and what was typed did not arrive there"
    );
}

/// **What was typed is kept**, so somebody who clicked away and pressed the
/// chord to come back is still writing one request.
#[test]
fn the_chord_does_not_empty_the_field() {
    let (_runtime, mut server) = a_session("keeping-what-was-typed");
    let shortcuts = Shortcuts::shipped();
    let chord = the_agents_chord(&shortcuts);
    server.a_chord_opens_the_bar(&shortcuts, chord);
    server.a_key_reached_the_composer(ComposerKey::Letter('x'));
    assert_eq!(server.composer().typed(), "x");

    // The keyboard goes somewhere else, as it would if they clicked a window.
    server.a_key_reached_the_composer(ComposerKey::Clear);
    assert!(!server.composer().has_the_keyboard());

    server.a_chord_opens_the_bar(&shortcuts, chord);
    assert!(server.composer().has_the_keyboard());
}

/// **Nothing is asked for.** A person who pressed the chord has asked to start
/// asking; a task window opened here would be a request they never made.
#[test]
fn the_chord_opens_no_task_and_asks_for_nothing() {
    let (_runtime, mut server) = a_session("no-task-from-the-chord");
    let shortcuts = Shortcuts::shipped();
    server.a_chord_opens_the_bar(&shortcuts, the_agents_chord(&shortcuts));

    assert_eq!(
        server.tasks().how_many(),
        0,
        "the chord opened a task window with nothing in it"
    );
    assert!(
        server.take_what_was_asked_of_alo().is_empty(),
        "the chord asked alo for something the person never typed"
    );
}

#[test]
fn another_chord_is_left_for_the_layers_below() {
    let (_runtime, mut server) = a_session("another-chord-than-the-agents");
    let shortcuts = Shortcuts::shipped();
    let settings = shortcuts
        .chord_for(Action::Settings)
        .expect("the release ships a chord for Settings");
    assert_ne!(
        settings,
        the_agents_chord(&shortcuts),
        "Settings and the agent share a chord, so this test proves nothing"
    );

    assert!(
        !server.a_chord_opens_the_bar(&shortcuts, settings),
        "Settings' chord was taken by the agent's road"
    );
    assert!(
        !server.composer().has_the_keyboard(),
        "another action's chord put the keyboard in the composer"
    );
}

#[test]
fn an_unbound_chord_is_left_for_the_layers_below() {
    let (_runtime, mut server) = a_session("unbound-chord-than-the-agents");
    let shortcuts = Shortcuts::shipped();
    let unbound = Chord::checked(Modifiers::just(Modifier::Super).and(Modifier::Ctrl), Key::J)
        .expect("Super+Ctrl+J is a chord a person could bind");
    assert!(
        shortcuts.action_for(unbound).is_none(),
        "the release binds Super+Ctrl+J after all, so this test proves nothing"
    );

    assert!(!server.a_chord_opens_the_bar(&shortcuts, unbound));
    assert!(!server.composer().has_the_keyboard());
}

/// `Super`+`Space`, as this release ships it.
fn the_launchers_chord(shortcuts: &Shortcuts) -> Chord {
    shortcuts
        .chord_for(Action::Launcher)
        .expect("the release ships a chord for the launcher")
}

/// **`Super`+`Space` opens the Bar too**, which is the owner's ruling of
/// 2026-10-20 and the reason `Action::Launcher` stopped being the one declared
/// action that reached nothing.
///
/// # Why this is a test of its own and not a case in the one above
///
/// The two chords share an arm **on purpose**, and a test that pressed only
/// the agent's would pass with `Action::Launcher` deleted from the `matches!`.
/// This is the half that fails if somebody narrows it back.
#[test]
fn the_launchers_chord_gives_the_bar_the_keyboard() {
    let (_runtime, mut server) = a_session("the-launchers-chord");
    let shortcuts = Shortcuts::shipped();
    let chord = the_launchers_chord(&shortcuts);

    // **A control before the press.** A Bar that already had the keyboard
    // would make any chord look like it worked, including one that did
    // nothing at all.
    assert!(
        !server.composer().has_the_keyboard(),
        "a session nobody has pressed anything on already has the keyboard in the Bar"
    );

    assert!(
        server.a_chord_opens_the_bar(&shortcuts, chord),
        "Super+Space was not taken, so it falls through to the layers below and a person \
         pressing a bound chord gets the behaviour of an unbound one"
    );
    assert!(
        server.composer().has_the_keyboard(),
        "Super+Space was taken and the Bar did not get the keyboard, so the next thing typed \
         goes to whatever window is in front"
    );
}

/// **It is the chord the release ships, not one this test chose.**
///
/// A test that bound `Action::Launcher` to a chord of its own would pass on a
/// release that shipped no binding for it at all — which is the state every
/// other declared-and-dead action is in before somebody notices.
#[test]
fn the_launchers_chord_is_the_one_the_release_ships() {
    let shipped = the_launchers_chord(&Shortcuts::shipped());
    let expected = Chord::checked(Modifiers::just(Modifier::Super), Key::Space)
        .expect("Super and a key is a chord");
    assert_eq!(
        shipped, expected,
        "the launcher's shipped chord moved, so a person following the documentation presses \
         something else — and the two chords this file is about are no longer the two it names"
    );
}

/// **What was typed is kept**, which is this file's rule for the agent's chord
/// and is the same rule here: somebody who started typing, went to read
/// something and pressed the chord to come back is writing one thing.
#[test]
fn the_launchers_chord_keeps_what_was_typed() {
    let (_runtime, mut server) = a_session("the-launchers-chord-keeps");
    let shortcuts = Shortcuts::shipped();
    let chord = the_launchers_chord(&shortcuts);
    server.a_chord_opens_the_bar(&shortcuts, chord);
    for letter in "find".chars() {
        server.a_key_reached_the_composer(ComposerKey::Letter(letter));
    }
    assert_eq!(
        server.composer().typed(),
        "find",
        "the premise: the field holds what was typed"
    );

    // **The keyboard goes somewhere else, as it would if they pressed a
    // window.** Not `ComposerKey::Clear`, which is Escape and is specified to
    // *take back everything typed and leave the field* — a test that used it
    // here would be measuring Escape and would pass whatever this chord did.
    server.the_composer.it_gave_the_keyboard_up();
    assert!(
        !server.composer().has_the_keyboard(),
        "the premise: the keyboard left the field"
    );
    assert_eq!(
        server.composer().typed(),
        "find",
        "the premise: leaving the field keeps what was typed"
    );

    assert!(server.a_chord_opens_the_bar(&shortcuts, chord));
    assert_eq!(
        server.composer().typed(),
        "find",
        "the chord emptied the field, so a person who looked something up mid-sentence lost it"
    );
}
