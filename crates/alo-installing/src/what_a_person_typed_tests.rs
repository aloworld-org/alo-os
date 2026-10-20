//! **Typing, with the modifier mistakes that produce a silently wrong
//! password.**
//!
//! Each test here is a sequence somebody could perform at a keyboard. The ones
//! worth having are the three that fail *quietly*: a shift whose release is
//! never seen, a caps lock treated as a shift, and a layout served as US
//! because nobody recognised its name.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::{Typed, WhatAPersonTyped, modifier};

/// `KEY_Q` — the physical key that gives `q` on QWERTY and `a` on AZERTY.
const PHYSICAL_Q: u16 = 16;
/// `KEY_A` — the other half of that swap.
const PHYSICAL_A: u16 = 30;
/// `KEY_1` — the leftmost of the number row.
const PHYSICAL_1: u16 = 2;
/// Backspace.
const BACKSPACE: u16 = 14;
/// `KEY_ESC`, which gives no character anywhere.
const ESCAPE: u16 = 1;

/// One key event, in the 24 bytes the kernel writes.
///
/// Built here rather than taken from a fixture because the field offsets are
/// the thing being relied on: a kernel event is `(sec, usec, type, code,
/// value)` and the type, code and value sit at 16, 18 and 20.
fn an_event(code: u16, value: i32) -> [u8; 24] {
    let mut event = [0u8; 24];
    event[16..18].copy_from_slice(&1u16.to_ne_bytes()); // EV_KEY
    event[18..20].copy_from_slice(&code.to_ne_bytes());
    event[20..24].copy_from_slice(&value.to_ne_bytes());
    event
}

/// Press and release, which is what a person does to type one character.
fn tap(typing: &mut WhatAPersonTyped, code: u16) {
    typing.what_arrived(&an_event(code, 1));
    typing.what_arrived(&an_event(code, 0));
}

/// **A shift that is released stops being held** — and this is the test that
/// matters most in this file.
///
/// `what_was_pressed` discards releases, and a typing road that copied it would
/// keep shift down for the rest of the install. Every letter after the first
/// capital would be a capital, on a password field that shows dots, and the
/// only report would be *the password was refused*.
#[test]
fn a_shift_that_is_released_stops_being_held() {
    let mut typing = WhatAPersonTyped::on("us").expect("us is offered");
    typing.what_arrived(&an_event(modifier::LEFT_SHIFT, 1));
    tap(&mut typing, PHYSICAL_A);
    typing.what_arrived(&an_event(modifier::LEFT_SHIFT, 0));
    tap(&mut typing, PHYSICAL_A);
    assert_eq!(typing.so_far(), "Aa");
}

/// **Releasing one shift while the other is held keeps shift down.**
///
/// A single flag gets this wrong, and holding both shifts is not unusual.
#[test]
fn letting_go_of_one_shift_while_the_other_is_held_keeps_it() {
    let mut typing = WhatAPersonTyped::on("us").expect("us is offered");
    typing.what_arrived(&an_event(modifier::LEFT_SHIFT, 1));
    typing.what_arrived(&an_event(modifier::RIGHT_SHIFT, 1));
    typing.what_arrived(&an_event(modifier::LEFT_SHIFT, 0));
    tap(&mut typing, PHYSICAL_A);
    typing.what_arrived(&an_event(modifier::RIGHT_SHIFT, 0));
    tap(&mut typing, PHYSICAL_A);
    assert_eq!(typing.so_far(), "Aa");
}

/// **Caps lock latches on the press, turns over on the next press, and does
/// not flicker when held.**
///
/// Three properties in one sequence, because they are one mechanism: a latch
/// that turned over on the release as well would be off again immediately, and
/// one that turned over on an autorepeat would flicker while held.
#[test]
fn caps_lock_latches_rather_than_being_held() {
    let mut typing = WhatAPersonTyped::on("us").expect("us is offered");
    tap(&mut typing, modifier::CAPS_LOCK);
    tap(&mut typing, PHYSICAL_A);
    assert_eq!(typing.so_far(), "A", "caps lock did not latch on");

    // held down: one press and three autorepeats, which must not turn it over
    typing.what_arrived(&an_event(modifier::CAPS_LOCK, 1));
    for _ in 0..3 {
        typing.what_arrived(&an_event(modifier::CAPS_LOCK, 2));
    }
    typing.what_arrived(&an_event(modifier::CAPS_LOCK, 0));
    tap(&mut typing, PHYSICAL_A);
    assert_eq!(
        typing.so_far(),
        "Aa",
        "holding caps lock did not turn it over once"
    );
}

/// **AZERTY is typed as AZERTY**, not as US with different stickers.
#[test]
fn the_french_board_gives_what_is_printed_on_it() {
    let mut typing = WhatAPersonTyped::on("fr").expect("fr is offered");
    tap(&mut typing, PHYSICAL_Q);
    tap(&mut typing, PHYSICAL_A);
    assert_eq!(typing.so_far(), "aq");

    let mut us = WhatAPersonTyped::on("us").expect("us is offered");
    tap(&mut us, PHYSICAL_Q);
    tap(&mut us, PHYSICAL_A);
    assert_eq!(us.so_far(), "qa");
}

/// **A digit on AZERTY needs shift, and typing one without it is not a digit.**
///
/// The failure a person cannot see: a password with a `1` in it becomes one
/// with an `&`, and the router refuses it.
#[test]
fn a_digit_on_azerty_needs_shift() {
    let mut without = WhatAPersonTyped::on("fr").expect("fr is offered");
    tap(&mut without, PHYSICAL_1);
    assert_eq!(without.so_far(), "&");

    let mut with = WhatAPersonTyped::on("fr").expect("fr is offered");
    with.what_arrived(&an_event(modifier::LEFT_SHIFT, 1));
    tap(&mut with, PHYSICAL_1);
    assert_eq!(with.so_far(), "1");
}

/// **AltGr reaches the third level**, which is where a Belgian board keeps `@`.
///
/// An email address in a network name, or an `@` in a password, is unreachable
/// without this — and AltGr is the one modifier a US keyboard never needs.
#[test]
fn altgr_reaches_the_third_level() {
    let mut typing = WhatAPersonTyped::on("be").expect("be is offered");
    typing.what_arrived(&an_event(modifier::ALT_GR, 1));
    tap(&mut typing, PHYSICAL_Q);
    typing.what_arrived(&an_event(modifier::ALT_GR, 0));
    tap(&mut typing, PHYSICAL_Q);
    assert_eq!(typing.so_far(), "@a");
}

/// **Holding a key repeats the character**, because that is how a person uses
/// one.
#[test]
fn an_autorepeat_types_again() {
    let mut typing = WhatAPersonTyped::on("us").expect("us is offered");
    typing.what_arrived(&an_event(PHYSICAL_A, 1));
    typing.what_arrived(&an_event(PHYSICAL_A, 2));
    typing.what_arrived(&an_event(PHYSICAL_A, 2));
    typing.what_arrived(&an_event(PHYSICAL_A, 0));
    assert_eq!(typing.so_far(), "aaa");
}

/// **A release types nothing**, so one keystroke is one character.
#[test]
fn a_release_types_nothing() {
    let mut typing = WhatAPersonTyped::on("us").expect("us is offered");
    assert_eq!(
        typing.what_arrived(&an_event(PHYSICAL_A, 1)),
        Typed::Added('a')
    );
    assert_eq!(
        typing.what_arrived(&an_event(PHYSICAL_A, 0)),
        Typed::Nothing
    );
    assert_eq!(typing.so_far(), "a");
}

/// **Backspace takes one back, and takes nothing back from nothing.**
#[test]
fn backspace_takes_one_back_and_refuses_to_go_past_the_start() {
    let mut typing = WhatAPersonTyped::on("us").expect("us is offered");
    assert_eq!(typing.what_arrived(&an_event(BACKSPACE, 1)), Typed::Nothing);
    tap(&mut typing, PHYSICAL_A);
    tap(&mut typing, PHYSICAL_Q);
    assert_eq!(typing.so_far(), "aq");
    assert_eq!(typing.what_arrived(&an_event(BACKSPACE, 1)), Typed::Erased);
    assert_eq!(typing.so_far(), "a");
}

/// **Backspace takes back a whole character and not a byte.**
///
/// `§` is two bytes in UTF-8, so a road that moved a byte at a time would leave
/// half of one behind and the text would no longer be text. `pop` is correct
/// and this is the test that keeps it correct.
#[test]
fn backspace_takes_back_a_character_and_not_a_byte() {
    let mut typing = WhatAPersonTyped::on("be").expect("be is offered");
    typing.what_arrived(&an_event(modifier::ALT_GR, 1));
    typing.what_arrived(&an_event(modifier::LEFT_SHIFT, 1));
    tap(&mut typing, 17);
    typing.what_arrived(&an_event(modifier::ALT_GR, 0));
    typing.what_arrived(&an_event(modifier::LEFT_SHIFT, 0));
    assert_eq!(typing.so_far(), "§");
    assert!(typing.so_far().len() > typing.so_far().chars().count());

    assert_eq!(typing.what_arrived(&an_event(BACKSPACE, 1)), Typed::Erased);
    assert_eq!(typing.so_far(), "");
}

/// **A key that gives no character changes nothing**, rather than adding a
/// replacement or a blank.
#[test]
fn a_key_that_gives_nothing_adds_nothing() {
    let mut typing = WhatAPersonTyped::on("us").expect("us is offered");
    tap(&mut typing, PHYSICAL_A);
    assert_eq!(typing.what_arrived(&an_event(ESCAPE, 1)), Typed::Nothing);
    assert_eq!(typing.so_far(), "a");
}

/// **Something that is not a key event changes nothing**, and a short read is
/// not a panic.
#[test]
fn a_short_read_or_another_kind_of_event_changes_nothing() {
    let mut typing = WhatAPersonTyped::on("us").expect("us is offered");
    assert_eq!(typing.what_arrived(&[]), Typed::Nothing);
    assert_eq!(typing.what_arrived(&[0u8; 12]), Typed::Nothing);
    // a real event of another kind: EV_REL rather than EV_KEY
    let mut moved = an_event(PHYSICAL_A, 1);
    moved[16..18].copy_from_slice(&2u16.to_ne_bytes());
    assert_eq!(typing.what_arrived(&moved), Typed::Nothing);
    assert_eq!(typing.so_far(), "");
}

/// **A layout nobody offers is refused rather than served as US.**
///
/// The silent-US fallback is the failure this whole road exists to remove, so
/// it is refused at construction: there is no object to type into, rather than
/// an object that types the wrong thing.
#[test]
fn a_layout_that_is_not_offered_is_refused_at_the_start() {
    assert!(WhatAPersonTyped::on("xx").is_none());
    assert!(WhatAPersonTyped::on("").is_none());
    assert!(
        WhatAPersonTyped::on("US").is_none(),
        "the names are lower case"
    );
    // the control: the ones the screen offers are all accepted
    for offered in alo_keyboards::EVERY_LAYOUT {
        assert_eq!(
            WhatAPersonTyped::on(offered).map(|it| it.layout().to_owned()),
            Some(offered.to_owned())
        );
    }
}
