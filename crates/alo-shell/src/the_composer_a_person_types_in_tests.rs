//! That the composer collects what a person types, hands it over once, and
//! collects nothing when it does not have the keyboard.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a directory or socket that cannot be made is the failure \
              being reported"
)]

use unicode_segmentation::UnicodeSegmentation;

use super::{THE_MOST_THAT_CAN_BE_TYPED, TheComposer, WhatTheComposerDid};
use crate::composer_keys::ComposerKey;

/// A composer with the keyboard, ready to be typed in.
fn ready() -> TheComposer {
    let mut composer = TheComposer::empty();
    composer.it_took_the_keyboard();
    composer
}

/// Type a whole phrase into one.
fn type_in(composer: &mut TheComposer, words: &str) {
    for letter in words.chars() {
        composer.pressed(ComposerKey::Letter(letter));
    }
}

/// **A fresh composer holds nothing and has no keyboard.**
#[test]
fn a_fresh_composer_is_empty_and_unfocused() {
    let composer = TheComposer::empty();

    assert!(composer.typed().is_empty());
    assert!(composer.is_empty());
    assert!(!composer.has_the_keyboard());
}

/// **A field without the keyboard collects nothing.**
///
/// The refusal that matters: a Dock field that took keystrokes aimed at a
/// window would be a text box reading a person's typing from across the screen.
#[test]
fn a_field_without_the_keyboard_collects_nothing() {
    let mut composer = TheComposer::empty();

    for key in [
        ComposerKey::Letter('h'),
        ComposerKey::Erase,
        ComposerKey::Ask,
        ComposerKey::NewLine,
        ComposerKey::Clear,
    ] {
        assert_eq!(
            composer.pressed(key),
            WhatTheComposerDid::Nothing,
            "{key:?} reached a field that does not have the keyboard"
        );
    }
    assert!(composer.typed().is_empty());
}

/// **Typing collects the letters, in order.**
#[test]
fn typing_collects_what_was_typed() {
    let mut composer = ready();

    type_in(&mut composer, "tidy my invoices");

    assert_eq!(composer.typed(), "tidy my invoices");
    assert!(!composer.is_empty());
}

/// **Enter hands the request over and leaves the field empty.**
///
/// A request still in the field after being sent is a request a person sends
/// twice.
#[test]
fn enter_hands_the_request_over_and_empties_the_field() {
    let mut composer = ready();
    type_in(&mut composer, "tidy my invoices");

    let what = composer.pressed(ComposerKey::Ask);

    assert_eq!(
        what,
        WhatTheComposerDid::TheyAsked("tidy my invoices".to_owned())
    );
    assert!(composer.typed().is_empty(), "the field is empty afterwards");
    assert_eq!(
        composer.pressed(ComposerKey::Ask),
        WhatTheComposerDid::Nothing,
        "and a second Enter asks for nothing"
    );
}

/// **Enter on an empty field asks for nothing.**
#[test]
fn enter_on_an_empty_field_asks_nothing() {
    let mut composer = ready();

    assert_eq!(
        composer.pressed(ComposerKey::Ask),
        WhatTheComposerDid::Nothing
    );
}

/// **Whitespace alone asks for nothing**, so a stray space and Enter does not
/// open a window a person has to close.
#[test]
fn whitespace_alone_asks_nothing() {
    let mut composer = ready();

    type_in(&mut composer, "   ");

    assert!(composer.is_empty(), "spaces are not a request");
    assert_eq!(
        composer.pressed(ComposerKey::Ask),
        WhatTheComposerDid::Nothing
    );
    assert_eq!(
        composer.typed(),
        "   ",
        "and what they typed is still there, not silently thrown away"
    );
}

/// **Shift+Enter puts a line in the field and asks for nothing.**
#[test]
fn a_new_line_goes_in_the_field_and_asks_nothing() {
    let mut composer = ready();
    type_in(&mut composer, "first");

    let what = composer.pressed(ComposerKey::NewLine);
    type_in(&mut composer, "second");

    assert_eq!(what, WhatTheComposerDid::ItChanged);
    assert_eq!(composer.typed(), "first\nsecond");
}

/// **One backspace takes back one thing a person sees**, not one code point.
#[test]
fn erasing_takes_back_a_whole_grapheme() {
    let mut composer = ready();
    // A family emoji is several code points and one grapheme; "é" here is e
    // plus a combining accent, which is two code points and one grapheme.
    type_in(&mut composer, "cafe\u{301}");
    assert_eq!(composer.typed().chars().count(), 5, "five code points");

    composer.pressed(ComposerKey::Erase);

    assert_eq!(
        composer.typed(),
        "caf",
        "the letter and its accent went together"
    );
}

/// **Erasing an empty field does nothing** rather than reporting a change
/// nobody can see.
#[test]
fn erasing_an_empty_field_changes_nothing() {
    let mut composer = ready();

    assert_eq!(
        composer.pressed(ComposerKey::Erase),
        WhatTheComposerDid::Nothing
    );
}

/// **Escape takes back everything and gives the keyboard up.**
#[test]
fn escape_clears_the_field_and_leaves_it() {
    let mut composer = ready();
    type_in(&mut composer, "never mind");

    let what = composer.pressed(ComposerKey::Clear);

    assert_eq!(what, WhatTheComposerDid::ItWasLeft);
    assert!(composer.typed().is_empty());
    assert!(!composer.has_the_keyboard());
}

/// **Going away and coming back keeps what was typed.** Somebody who clicks off
/// to read a document is still writing the same request.
#[test]
fn leaving_and_returning_keeps_what_was_typed() {
    let mut composer = ready();
    type_in(&mut composer, "half a thought");

    composer.it_gave_the_keyboard_up();
    assert_eq!(
        composer.pressed(ComposerKey::Letter('x')),
        WhatTheComposerDid::Nothing
    );
    composer.it_took_the_keyboard();
    type_in(&mut composer, " finished");

    assert_eq!(composer.typed(), "half a thought finished");
}

/// **A held key cannot grow the field without end**, and the bound is counted
/// in graphemes so it is the same number of characters in every language.
#[test]
fn the_field_is_bounded_and_the_bound_is_graphemes() {
    let mut composer = ready();
    for _ in 0..THE_MOST_THAT_CAN_BE_TYPED {
        assert_eq!(
            composer.pressed(ComposerKey::Letter('a')),
            WhatTheComposerDid::ItChanged
        );
    }

    assert_eq!(
        composer.pressed(ComposerKey::Letter('a')),
        WhatTheComposerDid::Nothing,
        "the bound holds"
    );
    assert_eq!(
        composer.typed().graphemes(true).count(),
        THE_MOST_THAT_CAN_BE_TYPED
    );

    // And it is still a field: erasing makes room again.
    composer.pressed(ComposerKey::Erase);
    assert_eq!(
        composer.pressed(ComposerKey::Letter('b')),
        WhatTheComposerDid::ItChanged
    );
}

/// Tests that the field and the request recorder are actually joined, through a
/// real `Server` rather than through the field alone.
mod the_join {
    use std::os::unix::fs::PermissionsExt as _;

    use super::super::WhatTheComposerDid;
    use crate::Server;
    use crate::composer_keys::{ComposerKey, Composing};

    /// A server on a socket of its own.
    fn server(named: &str) -> (tempfile::TempDir, Server) {
        let directory = tempfile::tempdir().expect("a private directory");
        std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
            .expect("a private runtime directory");
        let bound = Server::bind(directory.path(), named).expect("a bound socket");
        (directory, bound)
    }

    /// Type a phrase through the `Server`'s own road, not the field's.
    fn type_through_the_server(server: &mut Server, words: &str) {
        for letter in words.chars() {
            server.a_key_reached_the_composer(ComposerKey::Letter(letter));
        }
    }

    /// **A person types in the Dock and presses Enter, and a request comes out
    /// the other end.**
    ///
    /// The test the two halves existed without for a day: the field had no road
    /// out and `a_person_asked_alo` had no caller, and every test of each half
    /// passed.
    #[test]
    fn typing_in_the_dock_and_pressing_enter_records_a_request() {
        let (_directory, mut server) = server("composer-join");
        server.the_composer.it_took_the_keyboard();

        type_through_the_server(&mut server, "tidy my invoices folder");
        let what = server.a_key_reached_the_composer(ComposerKey::Ask);

        assert_eq!(
            what,
            WhatTheComposerDid::TheyAsked("tidy my invoices folder".to_owned())
        );
        assert_eq!(
            server.take_what_was_asked_of_alo(),
            vec!["tidy my invoices folder".to_owned()],
            "the request reached the Server, which is what the binary drains"
        );
        assert!(
            server.composer().typed().is_empty(),
            "and the field is empty, so it cannot be sent twice"
        );
    }

    /// **Typing without pressing Enter records nothing.**
    ///
    /// *Typing alone must not start a model request* — the owner's first
    /// requirement for this surface.
    #[test]
    fn typing_alone_records_nothing() {
        let (_directory, mut server) = server("composer-typing-only");
        server.the_composer.it_took_the_keyboard();

        type_through_the_server(&mut server, "tidy my invoices folder");

        assert_eq!(
            server.composer().typed(),
            "tidy my invoices folder",
            "it is all in the field"
        );
        assert!(
            server.take_what_was_asked_of_alo().is_empty(),
            "and none of it has been asked for"
        );
    }

    /// **A press on the field is what gives it the keyboard.**
    ///
    /// Before the press, the Dock's field collects nothing — so keys aimed at a
    /// window never land in it.
    #[test]
    fn nothing_is_collected_until_the_field_is_pressed() {
        let (_directory, mut server) = server("composer-unfocused");

        type_through_the_server(&mut server, "secret");
        let what = server.a_key_reached_the_composer(ComposerKey::Ask);

        assert_eq!(what, WhatTheComposerDid::Nothing);
        assert!(server.composer().typed().is_empty());
        assert!(server.take_what_was_asked_of_alo().is_empty());
    }

    /// **Enter while an input method is composing records nothing**, through the
    /// whole road rather than at the key enum alone.
    ///
    /// This is the one that would otherwise be proven only where it is cheapest
    /// to prove. `ComposerKey::of` refusing to produce `Ask` is necessary and
    /// not sufficient: what matters is that no request reaches the `Server`.
    #[test]
    fn enter_while_composing_records_no_request() {
        let (_directory, mut server) = server("composer-ime");
        server.the_composer.it_took_the_keyboard();
        type_through_the_server(&mut server, "\u{3053}\u{3093}");

        let key = ComposerKey::of(
            smithay::input::keyboard::Keysym::Return,
            &smithay::input::keyboard::ModifiersState::default(),
            Composing::InProgress,
        );
        let what = server.a_key_reached_the_composer(key);

        assert_eq!(what, WhatTheComposerDid::Nothing);
        assert!(
            server.take_what_was_asked_of_alo().is_empty(),
            "a candidate being accepted must not fire a request at the agent"
        );
        assert_eq!(
            server.composer().typed(),
            "\u{3053}\u{3093}",
            "and what was being composed is still there"
        );
    }

    /// **One Enter is one request**, however many frames pass before the binary
    /// drains it.
    #[test]
    fn one_enter_is_one_request() {
        let (_directory, mut server) = server("composer-once");
        server.the_composer.it_took_the_keyboard();
        type_through_the_server(&mut server, "one thing");
        server.a_key_reached_the_composer(ComposerKey::Ask);

        assert_eq!(server.take_what_was_asked_of_alo().len(), 1);
        assert!(
            server.take_what_was_asked_of_alo().is_empty(),
            "draining twice does not open a second window"
        );
    }
}
