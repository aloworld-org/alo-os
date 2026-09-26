//! What this machine's readings say, and what they say when it cannot answer.
//!
//! These run on whatever machine they are run on, so they assert what is true
//! of **every** machine rather than what is true of this one: that the clock
//! moves when the clock moves, that an absence is an absence, and that nothing
//! invents a number. A test asserting this laptop's battery is at sixty-four
//! per cent would fail tomorrow and prove nothing today.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;

/// The way a time is written here.
fn region() -> Regionally {
    Regionally::reading("en")
        .and_then(|reading| reading.in_region("GB"))
        .unwrap()
}

/// A moment, named rather than taken from a clock.
fn at(hour: u8, minute: u8) -> At {
    At {
        hour,
        minute,
        moment: std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_760_000_000),
    }
}

/// **The clock advances as the machine's clock does.**
///
/// The acceptance's own test: move the time and find the text moved with it.
/// A status area on a laptop showing a clock that never moves does not look
/// unfinished, it looks broken.
#[test]
fn the_clock_moves_when_the_machine_s_clock_moves() {
    let (earlier, _) = taken(&at(9, 41), &region()).unwrap();
    let (later, _) = taken(&at(9, 42), &region()).unwrap();
    let (much_later, _) = taken(&at(17, 5), &region()).unwrap();

    assert_ne!(
        earlier.clock(),
        later.clock(),
        "a minute passed and the clock did not move"
    );
    assert_ne!(earlier.clock(), much_later.clock());
    assert!(
        !earlier.clock().is_empty(),
        "the clock is empty, which is not a time"
    );
}

/// **The clock is the region's own way of writing a time**, not one assembled
/// here.
#[test]
fn the_clock_is_written_the_way_the_region_writes_one() {
    let (items, _) = taken(&at(9, 41), &region()).unwrap();

    assert_eq!(
        items.clock(),
        region().time(9, 41).unwrap(),
        "the clock is not what alo-formats wrote"
    );
}

/// **Taking the readings never fails because a device is missing.**
///
/// A machine with no battery, no media server and no network manager still has
/// a clock and still shows a status area. Whatever could not be read is absent
/// and the reason is beside it; nothing is invented and nothing is refused.
#[test]
fn a_machine_that_can_answer_nothing_still_has_a_status_area() {
    let (items, missing) = taken(&at(9, 41), &region()).unwrap();

    assert!(!items.clock().is_empty());
    // Whatever this machine answered, every absence has a reason and every
    // reading has none: a reading that came back with a reason beside it would
    // mean the two disagree about what happened.
    assert_eq!(items.battery().is_none(), missing.battery.is_some());
    assert_eq!(items.volume().is_none(), missing.volume.is_some());
}

/// **Every reason is said in one line, and silence means every reading came.**
#[test]
fn what_was_not_read_says_which_and_why() {
    let nothing_missing = WhatWasNotRead::default();
    assert_eq!(nothing_missing.said(), "every reading was taken");

    let missing = WhatWasNotRead {
        battery: Some("this machine has no battery".to_owned()),
        network: None,
        volume: Some("the media server: nobody answered".to_owned()),
    };
    let said = missing.said();
    assert!(
        said.contains("battery: this machine has no battery"),
        "{said}"
    );
    assert!(said.contains("volume: the media server"), "{said}");
    assert!(
        !said.contains("network"),
        "a reading that was taken was reported as missing: {said}"
    );
}

/// **A time nobody writes is a refusal, not an empty clock.**
#[test]
fn a_time_that_cannot_be_written_is_refused() {
    let refused = taken(&at(99, 99), &region());

    assert!(
        refused.is_err(),
        "a time outside a day was written rather than refused"
    );
}
