//! The plan's other acceptance for pairing: **a device is paired with only when
//! a person chose it from what was found, and only when they said yes to what it
//! asked — including devices that ask to be paired.**
//!
//! Three roads are closed here, and each of them is how a machine somewhere else
//! ends up paired with something nobody meant to pair with:
//!
//! - an address that was not in the list a person was looking at;
//! - a device in the list, with the radio off;
//! - a device in the list, with the person saying no.
//!
//! And the fourth, which is not a road at all: **there is no shape of pairing
//! this crate completes on its own.** A device that asks for nothing is still
//! asked about.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_bluetooth::testing::{AService, a_device};
use alo_bluetooth::{Address, Asked, Devices, Kind, NotChosen, NotDone, pair_with};

fn a_headset() -> alo_bluetooth::Found {
    a_device("AA:BB:CC:DD:EE:01", "A headset", Kind::Audio, false)
}

/// **An address nobody was shown is not a device anybody chose.**
#[test]
fn an_address_that_was_not_in_the_list_is_not_paired_with() {
    let service = AService::with(vec![a_headset()]);
    let found = service.now().expect("the service answered");
    let elsewhere = Address::reported("11:22:33:44:55:66").expect("an address");

    let why = pair_with(&service, &found, &elsewhere).expect_err("it was not in the list");
    assert!(matches!(
        why,
        NotDone::NotChosen(NotChosen::NotInWhatWasFound { .. })
    ));
    assert!(
        service.told().is_empty(),
        "the service was told to pair anyway"
    );
}

/// **With the radio off, nothing pairs** — including a device that was in the
/// list a moment ago, which is what off means.
#[test]
fn with_the_radio_off_nothing_pairs() {
    let service = AService::with(vec![a_headset()]).with_the_radio_off();
    let found = service.now().expect("the service answered");

    let why = pair_with(&service, &found, a_headset().address()).expect_err("the radio is off");
    assert!(matches!(why, NotDone::NotChosen(NotChosen::TheRadioIsOff)));
    assert!(service.told().is_empty());
}

/// **A person who says no is a pairing that did not happen**, and the machine
/// says so rather than leaving a device half-paired.
#[test]
fn a_person_saying_no_is_a_pairing_that_did_not_happen() {
    let service = AService::with(vec![a_headset()]).where_the_person_says_no();
    let found = service.now().expect("the service answered");

    let why = pair_with(&service, &found, a_headset().address()).expect_err("they said no");
    assert!(matches!(why, NotDone::NobodySaidYes));
    assert!(service.told().is_empty());
}

/// **Every shape of pairing waits for a person**, including a device that has
/// nothing to show and asks for nothing.
#[test]
fn a_device_that_asks_for_nothing_is_still_asked_about() {
    for asked in [
        Asked::Nothing,
        Asked::TypeThisOnIt(123_456),
        Asked::TheSameOnBoth(123_456),
        Asked::ItsOwnCode,
    ] {
        let service = AService::with(vec![a_headset()])
            .asking(asked.clone())
            .where_the_person_says_no();
        let found = service.now().expect("the service answered");
        assert!(
            matches!(
                pair_with(&service, &found, a_headset().address()),
                Err(NotDone::NobodySaidYes)
            ),
            "{asked:?} paired without anybody saying yes"
        );
        assert_eq!(service.will_ask(), &asked);
    }
}
