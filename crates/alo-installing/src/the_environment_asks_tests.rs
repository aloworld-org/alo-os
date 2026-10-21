//! That a key means what a person expects, and that a release is not a press.
//!
//! The walk these feed is `the_asking_walks`, and its own tests are there.
//! What is tested here is only the translation, which is the part that touches
//! the 24 bytes the kernel delivers.

#![expect(
    clippy::indexing_slicing,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use super::what_the_person_did;
use crate::the_asking_walks::WhatThePersonDid;

/// One key event, as the kernel lays out `struct input_event`: two 64-bit
/// times, then type, code and value.
///
/// Built here rather than taken from a fixture, so that these tests exercise
/// the same 24 bytes `/dev/input/event*` delivers.
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
const BACKSPACE: u16 = 14;
const TAB: u16 = 15;
const A: u16 = 30;

#[test]
fn each_key_means_what_a_person_expects() {
    for (code, meant) in [
        (UP, WhatThePersonDid::WentUp),
        (DOWN, WhatThePersonDid::WentDown),
        (ENTER, WhatThePersonDid::Chose),
        (ESCAPE, WhatThePersonDid::WentBack),
        (BACKSPACE, WhatThePersonDid::RubbedOut),
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
fn a_letter_is_not_navigation_and_is_not_read_here() {
    // **A letter depends on the layout and this function has none.** It comes
    // back as nothing, and the loop asks `WhatAPersonTyped` for it instead --
    // which is the whole reason the keyboard question comes first.
    assert_eq!(what_the_person_did(&an_event(A, 1)), None);
}

#[test]
fn a_short_read_is_nothing_rather_than_something_wrong() {
    assert_eq!(what_the_person_did(&[]), None);
    assert_eq!(what_the_person_did(&[0_u8; 8]), None);
    assert_eq!(what_the_person_did(&[0_u8; 23]), None);
}
