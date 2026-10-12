//! What can be held about the card lookup without a seat.
//!
//! **The decision, not the hardware.** `primary_gpu` preferring the `boot_vga`
//! device over `simpledrm` is `smithay`'s and is verified by booting a machine;
//! `docs/booting.md` says a virtual machine cannot show it. What is decided
//! here is **when the seat is asked at all**, which is exactly where the fault
//! lived — `alo-desktop` never asked — and which is testable because it is
//! about an argument rather than about a card.
//!
//! No test sets an environment variable, and none can: this crate forbids
//! `unsafe_code` and `std::env::set_var` is `unsafe`. That is why
//! `super::which_card` takes the override instead of reading it.

use super::which_card;

/// **A card a person named is used as given**, whatever a seat would have said.
#[test]
fn a_card_the_unit_file_names_is_taken() {
    assert_eq!(
        which_card(Some("/dev/dri/card7")).as_deref(),
        Ok(std::path::Path::new("/dev/dri/card7")),
        "a named card was not used as given"
    );
}

/// **An empty or blank name is not a card.**
///
/// `ALO_DISPLAY=` with nothing after it is a unit file that meant to say
/// something and did not. Both spellings of nothing are checked, because
/// `is_empty` alone passes a line whose value is a space — and a trailing space
/// in a unit file is invisible in every editor anybody reads one in.
///
/// Asserted as *it did not return that blank path*, not as *it succeeded*: this
/// host has no login seat, so the other branch is an error here too, and the
/// test must name the thing that separates the two rather than one they share.
#[test]
fn a_blank_name_asks_the_seat_rather_than_naming_nothing() {
    for blank in ["", " ", "   ", "\t"] {
        let answer = which_card(Some(blank));
        assert_ne!(
            answer.as_deref(),
            Ok(std::path::Path::new(blank)),
            "a blank ALO_DISPLAY of {blank:?} was taken as the name of a card"
        );
        if let Err(why) = answer {
            assert!(
                why.contains("seat0"),
                "the refusal does not name the seat it asked: {why}"
            );
        }
    }
}

/// **Nothing set reaches the seat.**
///
/// The whole of the change: `alo-desktop` refused to start here because
/// `ALO_DISPLAY` was unset, which forced its unit file to name a card and the
/// card it named is wrong on most UEFI machines.
///
/// On a host with a seat this answers a path under `/dev/dri`; on one without,
/// a sentence naming seat0. Both are accepted because the test must pass on a
/// developer's laptop and on a certified machine, and **the thing being
/// asserted is that it asked** — which either answer demonstrates and the old
/// behaviour, an error about an unset variable, does not.
#[test]
fn nothing_named_reaches_the_seat() {
    match which_card(None) {
        Ok(card) => assert!(
            card.starts_with("/dev/dri/"),
            "a seat answered with something that is not a card: {card:?}"
        ),
        Err(why) => assert!(
            why.contains("seat0"),
            "it refused without naming the seat it asked: {why}"
        ),
    }
}
