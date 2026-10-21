//! That a key means what it should, that the ring cannot leave the cards, and
//! that a person can reach every layout and come back.

#![expect(
    clippy::indexing_slicing,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use super::{TheAsking, WhatThePersonDid, WhichScreen, what_the_person_did};

/// One key event, as the kernel lays out `struct input_event`: two 64-bit
/// times, then type, code and value.
///
/// Built here rather than taken from a fixture so that these tests exercise the
/// same 24 bytes `/dev/input/event*` delivers.
fn an_event(code: u16, value: i32) -> Vec<u8> {
    const EV_KEY: u16 = 1;
    let mut event = vec![0_u8; 24];
    event[16..18].copy_from_slice(&EV_KEY.to_ne_bytes());
    event[18..20].copy_from_slice(&code.to_ne_bytes());
    event[20..24].copy_from_slice(&value.to_ne_bytes());
    event
}

/// evdev codes, from `linux/input-event-codes.h`.
const UP: u16 = 103;
const DOWN: u16 = 108;
const ENTER: u16 = 28;
const ESCAPE: u16 = 1;
const TAB: u16 = 15;
const A: u16 = 30;

#[test]
fn each_key_means_what_a_person_expects() {
    for (code, meant) in [
        (UP, WhatThePersonDid::WentUp),
        (DOWN, WhatThePersonDid::WentDown),
        (ENTER, WhatThePersonDid::Chose),
        (ESCAPE, WhatThePersonDid::WentBack),
        (TAB, WhatThePersonDid::NothingThisScreenActsOn),
    ] {
        assert_eq!(
            what_the_person_did(&an_event(code, 1)),
            Some(meant),
            "evdev {code} did not mean {meant:?}"
        );
    }
}

#[test]
fn a_release_is_not_a_press_and_a_repeat_is() {
    // Acting on the release as well would move the ring twice for one
    // keystroke, which a person reads as the list being out of control.
    assert_eq!(what_the_person_did(&an_event(DOWN, 0)), None);
    // Holding an arrow down to travel a list is how a person uses one, so an
    // autorepeat has to count.
    assert_eq!(
        what_the_person_did(&an_event(DOWN, 2)),
        Some(WhatThePersonDid::WentDown)
    );
}

#[test]
fn a_key_this_does_not_understand_is_nothing_rather_than_something_wrong() {
    assert_eq!(what_the_person_did(&an_event(A, 1)), None);
    assert_eq!(what_the_person_did(&[]), None);
    assert_eq!(what_the_person_did(&[0_u8; 8]), None);
}

#[test]
fn the_asking_begins_on_the_keyboard_question() {
    // Because typing presupposes the layout, and the two things this
    // environment asks a person to type both fail silently on the wrong one.
    let asking = TheAsking::begins();
    assert_eq!(asking.on(), WhichScreen::TheKeyboard);
    assert_eq!(asking.card(), 0);
    assert_eq!(asking.settled(), None);
}

#[test]
fn the_ring_cannot_leave_the_cards_at_either_end() {
    let mut asking = TheAsking::begins();
    asking.there_are_this_many_cards(2);

    // Up from the first card stays on it, and says nothing changed — so the
    // screen is not redrawn for a key that did nothing.
    assert!(!asking.and_then(WhatThePersonDid::WentUp, None));
    assert_eq!(asking.card(), 0);

    assert!(asking.and_then(WhatThePersonDid::WentDown, None));
    assert_eq!(asking.card(), 1);

    // And down from the last stays on it.
    assert!(!asking.and_then(WhatThePersonDid::WentDown, None));
    assert_eq!(asking.card(), 1);
}

#[test]
fn something_else_opens_the_list_and_escape_comes_back() {
    let mut asking = TheAsking::begins();

    // Down to *Something else*, then take it.
    assert!(asking.and_then(WhatThePersonDid::WentDown, None));
    assert!(asking.and_then(WhatThePersonDid::Chose, None));
    assert_eq!(asking.on(), WhichScreen::EveryLayout);
    assert_eq!(asking.card(), 0, "the list did not open at its top");
    assert_eq!(
        asking.settled(),
        None,
        "opening the list settled a layout, which nobody chose"
    );

    // And back.
    assert!(asking.and_then(WhatThePersonDid::WentBack, None));
    assert_eq!(asking.on(), WhichScreen::TheKeyboard);
    assert_eq!(asking.card(), 0);
}

#[test]
fn escape_on_the_first_screen_does_nothing_rather_than_leaving_a_blank_display() {
    // There is nowhere back to from the first screen, and a person left
    // looking at nothing has no way on.
    let mut asking = TheAsking::begins();
    assert!(!asking.and_then(WhatThePersonDid::WentBack, None));
    assert_eq!(asking.on(), WhichScreen::TheKeyboard);
}

#[test]
fn taking_the_first_card_settles_the_layout_the_environment_guessed() {
    let mut asking = TheAsking::begins();
    assert!(asking.and_then(WhatThePersonDid::Chose, Some("be")));
    assert_eq!(asking.settled(), Some("be"));
}

#[test]
fn taking_a_layout_from_the_list_settles_that_one() {
    let mut asking = TheAsking::begins();
    asking.and_then(WhatThePersonDid::WentDown, None);
    asking.and_then(WhatThePersonDid::Chose, None);
    assert_eq!(asking.on(), WhichScreen::EveryLayout);

    asking.there_are_this_many_cards(8);
    for _ in 0..3 {
        asking.and_then(WhatThePersonDid::WentDown, None);
    }
    assert_eq!(asking.card(), 3);
    assert!(asking.and_then(WhatThePersonDid::Chose, Some("es")));
    assert_eq!(asking.settled(), Some("es"));
}

#[test]
fn choosing_with_nothing_under_the_ring_settles_nothing() {
    // The empty-list case, which is real on the network screen and would be a
    // bug on this one. Either way a screen must not settle on a name nobody
    // can see.
    let mut asking = TheAsking::begins();
    assert!(!asking.and_then(WhatThePersonDid::Chose, None));
    assert_eq!(asking.settled(), None);
}

#[test]
fn a_shorter_list_pulls_the_ring_back_onto_it() {
    // The list shows as many cards as the display has room for, so the count
    // can shrink under the ring — on a smaller screen, or when a scan finds
    // fewer networks than it did a moment ago. The ring must not be left
    // pointing past the end.
    let mut asking = TheAsking::begins();
    asking.there_are_this_many_cards(8);
    for _ in 0..7 {
        asking.and_then(WhatThePersonDid::WentDown, None);
    }
    assert_eq!(asking.card(), 7);

    asking.there_are_this_many_cards(3);
    assert_eq!(
        asking.card(),
        2,
        "the ring was left past the end of the list"
    );
}

#[test]
fn a_list_with_nothing_in_it_leaves_the_ring_somewhere_sane() {
    let mut asking = TheAsking::begins();
    asking.there_are_this_many_cards(0);
    assert_eq!(asking.card(), 0);
    // And moving about in it does not run away.
    assert!(!asking.and_then(WhatThePersonDid::WentDown, None));
    assert!(!asking.and_then(WhatThePersonDid::WentUp, None));
    assert_eq!(asking.card(), 0);
}

#[test]
fn a_whole_walk_through_the_asking_ends_with_a_layout() {
    // The journey a person actually makes when the guess is wrong: see the
    // guessed layout, decide it is not theirs, open the list, travel to
    // theirs, take it.
    let mut asking = TheAsking::begins();
    let the_eight = ["be", "fr", "de", "es", "it", "nl", "gb", "us"];

    asking.and_then(WhatThePersonDid::WentDown, None);
    asking.and_then(WhatThePersonDid::Chose, None);
    asking.there_are_this_many_cards(the_eight.len());

    while asking.card() < 5 {
        let moved = asking.and_then(WhatThePersonDid::WentDown, None);
        assert!(moved, "the ring stopped before it reached the layout");
    }
    let under_the_ring = the_eight.get(asking.card()).copied();
    asking.and_then(WhatThePersonDid::Chose, under_the_ring);

    assert_eq!(asking.settled(), Some("nl"));
}
