//! **Super+A puts the keyboard in the composer, and leaves what was typed.**

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
        server.a_chord_brings_up_the_agent(&shortcuts, the_agents_chord(&shortcuts)),
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
    server.a_chord_brings_up_the_agent(&shortcuts, the_agents_chord(&shortcuts));

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
    server.a_chord_brings_up_the_agent(&shortcuts, chord);
    server.a_key_reached_the_composer(ComposerKey::Letter('x'));
    assert_eq!(server.composer().typed(), "x");

    // The keyboard goes somewhere else, as it would if they clicked a window.
    server.a_key_reached_the_composer(ComposerKey::Clear);
    assert!(!server.composer().has_the_keyboard());

    server.a_chord_brings_up_the_agent(&shortcuts, chord);
    assert!(server.composer().has_the_keyboard());
}

/// **Nothing is asked for.** A person who pressed the chord has asked to start
/// asking; a task window opened here would be a request they never made.
#[test]
fn the_chord_opens_no_task_and_asks_for_nothing() {
    let (_runtime, mut server) = a_session("no-task-from-the-chord");
    let shortcuts = Shortcuts::shipped();
    server.a_chord_brings_up_the_agent(&shortcuts, the_agents_chord(&shortcuts));

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
        !server.a_chord_brings_up_the_agent(&shortcuts, settings),
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

    assert!(!server.a_chord_brings_up_the_agent(&shortcuts, unbound));
    assert!(!server.composer().has_the_keyboard());
}
