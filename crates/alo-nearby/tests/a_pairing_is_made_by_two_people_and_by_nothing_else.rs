//! *Pairing: mutual, deliberate, enumerated, revocable in one action, and
//! expiring — grants, across a machine boundary.*
//!
//! [ADR 0003] is the decision this holds: **being on the same network is not
//! authority.** So the tests here are mostly about *not* pairing, and each
//! thing that confers nothing is refused under its own name — a test called
//! `being_on_the_same_network_confers_nothing` is a sentence somebody can read
//! in a failure log and know what was promised.
//!
//! | The acceptance | The test |
//! |---|---|
//! | both machines' people confirm, and one side alone pairs nothing | [`a_pairing_needs_two_people_and_one_of_them_is_not_enough`] |
//! | it expires, and the duration is stated where it is made | [`a_pairing_ends_and_how_long_it_lasts_is_part_of_what_was_agreed`] |
//! | revocable in one action, taking effect immediately | [`revoking_takes_effect_in_the_middle_of_being_used`] |
//! | enumerated, in words `alo-saying` collects | [`what_a_pairing_permits_is_a_list_a_person_can_read`] |
//! | being on the same network confers nothing | [`being_on_the_same_network_confers_nothing`] |
//! | sharing a wireless password confers nothing | [`sharing_the_wireless_password_confers_nothing`] |
//! | having paired before confers nothing | [`having_paired_before_confers_nothing`] |
//!
//! [ADR 0003]: https://github.com/aloworld-org/alo-os/blob/main/docs/decisions/0003-the-network-is-not-authority.md

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, SystemTime};

use alo_nearby::{
    Deliberating, EVERYTHING_A_PAIRING_MAY_PERMIT, Found, MachineId, MayAskIts, NotPaired,
    Pairings, Proposal, Side, nearby_words,
};
use alo_strings::Strings;

/// The machine a person is sitting at.
fn this_machine() -> MachineId {
    MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
}

/// The machine down the corridor, with the GPU in it.
fn down_the_corridor() -> MachineId {
    MachineId::read("aaaabbbbccccddddeeeeffff00001111").unwrap()
}

/// A moment to reason from, so nothing here depends on when it is run.
fn a_moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// What a person would ask the machine down the corridor for.
fn asking_for_its_models() -> Proposal {
    Proposal::checked(
        this_machine(),
        down_the_corridor(),
        &[MayAskIts::Models],
        Duration::from_secs(86_400),
    )
    .unwrap()
}

/// A pairing, made the only way one can be.
fn paired_for_a_day() -> Pairings {
    let mut pairings = Pairings::none();
    pairings.keep(
        Deliberating::of(asking_for_its_models())
            .agreed_at(Side::TheOneAsking)
            .agreed_at(Side::TheOneAsked)
            .agreed(a_moment())
            .unwrap(),
    );
    pairings
}

/// **Mutual and deliberate.** Two people, each at their own machine — and one
/// of them alone leaves nothing paired, whichever one it is.
#[test]
fn a_pairing_needs_two_people_and_one_of_them_is_not_enough() {
    let both = Deliberating::of(asking_for_its_models())
        .agreed_at(Side::TheOneAsking)
        .agreed_at(Side::TheOneAsked)
        .agreed(a_moment());
    assert!(both.is_ok());

    for alone in [Side::TheOneAsking, Side::TheOneAsked] {
        assert_eq!(
            Deliberating::of(asking_for_its_models())
                .agreed_at(alone)
                .agreed(a_moment())
                .unwrap_err(),
            NotPaired::OnlyOneSideAgreed,
            "{alone:?} paired a machine on its own"
        );
    }
}

/// **It expires**, and how long it lasts is part of what was agreed rather than
/// a number somebody has to go and look up afterwards.
#[test]
fn a_pairing_ends_and_how_long_it_lasts_is_part_of_what_was_agreed() {
    assert_eq!(
        asking_for_its_models().lasting(),
        Duration::from_secs(86_400),
        "how long the pairing would last was not part of what was proposed"
    );

    let pairings = paired_for_a_day();
    assert!(pairings.permits(&down_the_corridor(), MayAskIts::Models, a_moment()));
    assert!(
        !pairings.permits(
            &down_the_corridor(),
            MayAskIts::Models,
            a_moment() + Duration::from_secs(86_400)
        ),
        "a pairing that was agreed for a day was still permitting things a day later"
    );

    // And there is no way to ask for one that does not end.
    assert_eq!(
        Proposal::checked(
            this_machine(),
            down_the_corridor(),
            &[MayAskIts::Models],
            Duration::ZERO
        )
        .unwrap_err(),
        NotPaired::NoTime
    );
}

/// **Revocable in one action, taking effect immediately** — measured in the
/// middle of being used rather than between two uses, because *immediately* is
/// exactly the word a cache would quietly make untrue.
#[test]
fn revoking_takes_effect_in_the_middle_of_being_used() {
    let mut pairings = paired_for_a_day();
    let asked_before = pairings.permits(&down_the_corridor(), MayAskIts::Models, a_moment());

    // One action, from the person's own machine, with nothing owed to the other.
    let there_was_one = pairings.revoke(&down_the_corridor());

    let asked_after = pairings.permits(&down_the_corridor(), MayAskIts::Models, a_moment());
    assert!(
        asked_before,
        "the pairing was not permitting anything first"
    );
    assert!(there_was_one);
    assert!(
        !asked_after,
        "the very next question after revoking was still permitted"
    );
    assert!(
        pairings.every().is_empty(),
        "the revoked pairing is still in the list a person reads"
    );
}

/// **Enumerated**, and enumerated in a language somebody reads: every arm has a
/// sentence, and the sentences are in the vocabulary `alo-saying` collects.
#[test]
fn what_a_pairing_permits_is_a_list_a_person_can_read() {
    let strings = Strings::of(nearby_words().unwrap());
    for may in EVERYTHING_A_PAIRING_MAY_PERMIT {
        let said = strings.say(&may.word().key(), &alo_strings::Filling::nothing());
        assert!(
            !said.text().is_empty(),
            "{may:?} has no sentence in this crate's own vocabulary"
        );
    }

    // And a pairing permits what is on its list, and not the other arm.
    let pairings = paired_for_a_day();
    assert!(pairings.permits(&down_the_corridor(), MayAskIts::Models, a_moment()));
    assert!(
        !pairings.permits(&down_the_corridor(), MayAskIts::Workspace, a_moment()),
        "a pairing for one thing permitted another"
    );
}

/// **Being on the same network confers nothing.**
///
/// The machine was found, it is right there, and it is permitted nothing — and
/// there is no function anywhere that turns the one into the other. A `Found`
/// is a fact written down.
#[test]
fn being_on_the_same_network_confers_nothing() {
    let found = Found::seen(down_the_corridor(), 7_610);
    let pairings = Pairings::none();

    for may in EVERYTHING_A_PAIRING_MAY_PERMIT {
        assert!(
            !pairings.permits(&found.machine, may, a_moment()),
            "a machine that was merely found was permitted {may:?}"
        );
    }
}

/// **Sharing the wireless password confers nothing**, which is held by there
/// being nothing in this crate that could know about one.
///
/// The test reads the crate's own shipped source for every word by which a
/// network could vouch for a machine. The failure being guarded against is not
/// a stranger: it is the reasonable-sounding change — *machines on the office
/// network are already trusted, so skip the second confirmation* — that ADR
/// 0003 names as the whole vulnerability.
#[test]
fn sharing_the_wireless_password_confers_nothing() {
    for (file, code) in this_crates_shipped_code() {
        for vouching in [
            "ssid",
            "Ssid",
            "SSID",
            "wifi",
            "WiFi",
            "Wifi",
            "wireless",
            "subnet",
            "Subnet",
            "trusted",
            "Trusted",
            "certificate",
            "Certificate",
        ] {
            assert!(
                !code.contains(vouching),
                "{} knows about `{vouching}`, and being on a network is not authority",
                file.display()
            );
        }
    }
}

/// **Having paired before confers nothing.**
///
/// The pairing ended. The machine is the same machine, the person is the same
/// person, and what they agreed last month is not an agreement now: the second
/// pairing needs both people again, exactly like the first.
#[test]
fn having_paired_before_confers_nothing() {
    let pairings = paired_for_a_day();
    let a_month_later = a_moment() + Duration::from_secs(60 * 60 * 24 * 30);

    assert!(
        !pairings.permits(&down_the_corridor(), MayAskIts::Models, a_month_later),
        "a pairing that had ended was still permitting things"
    );

    // And pairing again is the whole deliberation again, not a shortcut.
    assert_eq!(
        Deliberating::of(asking_for_its_models())
            .agreed_at(Side::TheOneAsking)
            .agreed(a_month_later)
            .unwrap_err(),
        NotPaired::OnlyOneSideAgreed,
        "having paired before let one machine pair again on its own"
    );
}

/// A machine cannot pair with itself, and what it is told is a statement of
/// fact rather than a refusal — because that is what it is.
#[test]
fn a_person_who_picks_their_own_machine_is_told_what_it_is() {
    let refused = Proposal::checked(
        this_machine(),
        this_machine(),
        &[MayAskIts::Models],
        Duration::from_secs(60),
    )
    .unwrap_err();
    assert_eq!(refused, NotPaired::WithItself);

    let strings = Strings::of(nearby_words().unwrap());
    assert_eq!(
        refused.said(&strings).text(),
        "This is the machine you are using."
    );
}

/// Every refusal a person can be given has a sentence, which is why
/// [`NotPaired`] has no `Display`: the only road to words is the one that takes
/// the strings the reader reads.
#[test]
fn every_refusal_reaches_the_person_in_their_own_language() {
    let strings = Strings::of(nearby_words().unwrap());
    for refused in [
        NotPaired::OnlyOneSideAgreed,
        NotPaired::WithItself,
        NotPaired::NothingAsked,
        NotPaired::NoTime,
        NotPaired::TooLong,
        NotPaired::NoEnd,
    ] {
        let said = refused.said(&strings);
        assert!(!said.text().is_empty(), "{refused:?} has nothing to say");
    }
}

/// Every `.rs` file in this crate's `src`, with its tests cut off.
///
/// Comments say what this crate does not do, and a test may name a thing in
/// order to prove the crate does not know about it, so only the code that ships
/// is read.
fn this_crates_shipped_code() -> Vec<(std::path::PathBuf, String)> {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut read = Vec::new();
    for file in std::fs::read_dir(&source).unwrap() {
        let file = file.unwrap().path();
        if file.extension().is_none_or(|of| of != "rs") {
            continue;
        }
        let written = std::fs::read_to_string(&file).unwrap();
        let ships = written
            .split_once("#[cfg(test)]")
            .map_or(written.as_str(), |(before, _)| before)
            .to_owned();
        let code: String = ships
            .lines()
            .filter(|line| {
                let line = line.trim_start();
                !line.starts_with("//") && !line.starts_with('*')
            })
            .collect::<Vec<_>>()
            .join("\n");
        read.push((file, code));
    }
    assert!(read.len() > 4, "only {} files were read", read.len());
    read
}
