//! **Nothing routed a key to the alo Bar, and every test of the Bar passed.**
//!
//! Measured on a booted machine on 2026-10-23: `Super`+`A` fired, the composer
//! took the keyboard, twenty-nine characters were typed, and the field stayed
//! empty. The shell's own counter confirmed all of them arrived and that it
//! *could not make sense of 0 events* — so the keys were read, understood, and
//! handed to nobody.
//!
//! `Server::a_key_reached_the_composer` had **eighteen mentions across the
//! workspace and every one was in a test**, and
//! `Server::a_key_reached_a_task_window` had eight with the same split. The
//! layers were finished. The entrance was not built.
//!
//! # Why this test reads the source
//!
//! `crate::a_key_reaches_the_bar`'s own unit tests press a key through
//! `the_key_the_bar_took`, which is the road inward from libinput — and they
//! would **all still pass** with nothing in `direct_desktop` routing to it,
//! which is exactly the state the machine was in. A test of the layer cannot
//! see whether anything above calls the layer.
//!
//! So this asserts the call site, the way
//! `a_request_becomes_a_window::the_frame_loop_turns_a_request_into_a_task_and_draws_it`
//! does for the road a request takes, and for the same reason. It carries a
//! positive control so that a read returning nothing fails loudly rather than
//! passing clean.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

/// A file of this crate's source, by name.
fn read(named: &str) -> String {
    let at = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join(named);
    std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// A file's production part: everything before its unit tests.
fn production_part_of(file: &str) -> &str {
    file.split_once("#[cfg(test)]")
        .map_or(file, |(before, _)| before)
}

/// **The frame loop hands a key to the Bar when the Bar is taking keys.**
#[test]
fn the_input_lane_routes_a_key_to_the_bar() {
    let desk = read("direct_desktop.rs");
    let shipped = production_part_of(&desk);

    // The positive control, in the same scope as the claims below it.
    assert!(
        shipped.contains("server.settings_is_taking_the_keys()"),
        "direct_desktop.rs's production part no longer holds the key-routing chain, so nothing \
         below is measuring what it claims"
    );

    assert!(
        shipped.contains("server.the_bar_is_taking_the_keys()"),
        "nothing in production asks whether the alo Bar is taking keys, so a person who presses \
         Super+A can type and every character goes nowhere — which is the machine measured on \
         2026-10-23"
    );
    assert!(
        shipped.contains("a_key_reaches_the_bar::a_key_reaches_the_bar("),
        "the question is asked and the answer reaches no entrance, so the Bar still cannot be \
         typed into"
    );
}

/// **The entrance really carries a key to the composer.**
///
/// The arm above could exist and call a function that does nothing. This holds
/// the other end: the entrance names the handler that puts a letter in the
/// field.
#[test]
fn the_entrance_hands_the_key_to_the_composer() {
    let entrance = read("a_key_reaches_the_bar.rs");
    let shipped = production_part_of(&entrance);

    assert!(
        shipped.contains("fn the_key_the_bar_took"),
        "a_key_reaches_the_bar.rs no longer holds the road inward, so nothing below is measuring \
         what it claims"
    );
    assert!(
        shipped.contains("server.a_key_reached_the_composer("),
        "the Bar's entrance never hands the key to the composer, so the road ends one step short"
    );
    assert!(
        shipped.contains("server.the_bars_key("),
        "the entrance never reads the key off the seat, so there is nothing to hand on"
    );
}

/// **The reply box in a task window is still unreachable, and that is said
/// here rather than left to be rediscovered.**
///
/// `Server::a_key_reached_a_task_window` has the same shape as the Bar's
/// handler had: finished, tested, and called by nothing in production —
/// measured 2026-10-23, eight mentions and all eight in tests. Asking the Bar
/// is the road this change opens; **replying to an answer is not yet open.**
///
/// This test passes today and is written to fail the day somebody wires it,
/// so that the note above is removed in the same change rather than left
/// claiming something untrue.
#[test]
fn replying_in_a_task_window_is_not_wired_yet_and_this_says_so() {
    let desk = read("direct_desktop.rs");
    let shipped = production_part_of(&desk);
    assert!(
        !shipped.contains("a_key_reached_a_task_window("),
        "a task window's reply box is now routed to — good. Delete this test and the note in \
         a_key_reaches_the_bar.rs that calls it unreachable, because both are now false."
    );
}
