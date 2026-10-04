//! The one fact this road depends on that lives in another crate.
//!
//! The road itself is walked in `tests/a_chord_reaches_its_action`, which
//! presses evdev codes against a real display and asks what the canvas did.
//! Nothing is repeated here: a second test of the road that did not press a key
//! would be exactly the kind of green that task 19 was written about.

use super::*;
use alo_shortcuts::{Action, Key, Modifier, Modifiers};

/// **The shipped chord for *Show all* is one this road can make from a key.**
///
/// `the_chord_a_key_makes` reads the **unshifted** symbol, which is what a
/// person's key cap says. That makes a digit reachable on every layout and
/// makes `⊞`+`+` reachable only where `+` is a key of its own — so a shipped
/// default moved onto a shifted symbol would quietly stop arriving on an
/// American keyboard.
///
/// Asserted against `alo-shortcuts` rather than written out, so a default that
/// moves moves this with it rather than leaving it asserting a key nobody has.
#[test]
fn show_all_ships_with_a_chord_an_unshifted_key_can_make() {
    let shipped = Shortcuts::shipped();
    let made = Chord::checked(Modifiers::just(Modifier::Super), Key::Digit0)
        .expect("Super and a digit is a chord");
    assert_eq!(
        shipped.chord_for(Action::ShowAllOnTheCanvas),
        Some(made),
        "the shipped Show all chord is no longer ⊞+0"
    );
    assert_eq!(
        shipped.action_for(made),
        Some(Action::ShowAllOnTheCanvas),
        "the chord this road makes does not reach the action"
    );
}
