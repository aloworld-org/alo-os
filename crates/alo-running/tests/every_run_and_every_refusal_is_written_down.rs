//! Every refusal leaves a record, on a machine with no kernel boundary at all.
//!
//! `CLAUDE.md` names this among the capability guarantees that are **tests, not
//! prose**: *every execution and every refusal leaves a record*. ADR 0064 says
//! the same about runs at every one of its three levels.
//!
//! # Why this runs anywhere, and why that took a change to arrange
//!
//! A run needs a `Boundary`, and a `Boundary` cannot be made without a kernel
//! holding a BPF LSM — which CI does not have. So a test of this guarantee
//! written against `WhereCommandsRun::ran` could only run after a merge, on the
//! one machine that can, and never before one.
//!
//! The refusal road needs **somewhere to write and nothing else**. It is a free
//! function for that reason, and this file is why: the half of the guarantee
//! that can be checked on every build is checked on every build.
//!
//! **What this does not cover** is the other half — that a *run* is written
//! down, and that a box that will not close takes the refusal road. Those need
//! a real boundary and live in
//! `the_kernel_refuses_what_a_run_was_not_given.rs`, which needs root and a
//! started BPF LSM. Said here so that green on this file is not read as the
//! whole guarantee.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{path::PathBuf, time::SystemTime};

use alo_capability::{Grantee, HowCodeRuns};
use alo_record::{Happened, Record};
use alo_running::{AskedToRun, NotRun, SomewhereToWrite, WhatHappened};
use alo_strings::Strings;

/// The words this crate's refusals are said in.
fn in_english() -> Strings {
    let mut vocabulary = alo_strings::Vocabulary::empty();
    alo_capability::declare_into(&mut vocabulary).expect("the capability words declare");
    Strings::of(vocabulary)
}

/// A moment, fixed so nothing here depends on a clock.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_760_000_000)
}

/// What a caller asks for, with one field spoiled per test.
fn asked() -> AskedToRun {
    AskedToRun {
        chose: HowCodeRuns::InASealedBox,
        program: PathBuf::from("/bin/sh"),
        arguments: vec!["-c".into(), "true".into()],
        may_reach: vec![PathBuf::from("/bin")],
    }
}

/// **Every refusal a request can be met with leaves exactly one entry**, and it
/// reads as a refusal with a sentence a person can read.
///
/// The four cases are the ones reachable without a kernel: they are refused
/// before a box is ever built, which is exactly why they used to leave no
/// record at all.
#[test]
fn every_refusal_of_a_request_leaves_one_record() {
    let cases: Vec<(&str, AskedToRun)> = vec![
        (
            "a relative program",
            AskedToRun {
                program: PathBuf::from("sh"),
                ..asked()
            },
        ),
        (
            "a program that is not there",
            AskedToRun {
                program: PathBuf::from("/nowhere/at/all/alo-no-such-program"),
                ..asked()
            },
        ),
        (
            "nowhere to reach",
            AskedToRun {
                may_reach: vec![],
                ..asked()
            },
        ),
        (
            "a level the person did not choose",
            AskedToRun {
                chose: HowCodeRuns::AtFullTrust,
                ..asked()
            },
        ),
    ];

    for (what, asked) in cases {
        let mut record = Record::default();
        let why = alo_running::checked_and_written_down(
            &mut record,
            &Grantee::named("@terminal"),
            &asked,
            &in_english(),
            noon(),
        )
        .expect_err("a spoiled request is refused");

        let kept: Vec<_> = record.everything().collect();
        assert_eq!(
            kept.len(),
            1,
            "{what}: a refusal left {} entries rather than one",
            kept.len()
        );

        let entry = kept.first().expect("a refusal left one entry");
        assert!(
            entry.happened().was_stopped(),
            "{what}: the entry does not read as a refusal"
        );
        assert!(
            !entry.happened().ran(),
            "{what}: a refused command was counted as a verb execution"
        );
        assert!(
            entry.agent().is_some_and(|agent| agent.is("@terminal")),
            "{what}: the entry names nobody's authority"
        );

        let said = entry
            .happened()
            .why_stopped()
            .expect("a refusal carries the sentence the person was shown");
        assert!(
            said.as_str().starts_with("nothing ran"),
            "{what}: the sentence is not one a person was shown: {said:?}"
        );
        assert!(
            !said.as_str().contains('.'),
            "{what}: the sentence is a key rather than words, so the words were never \
             declared: {said:?}"
        );

        assert!(
            matches!(entry.happened(), Happened::ACommandWasNotRun { .. }),
            "{what}: the entry is not the kind a refused command makes"
        );
        // No arguments are kept for something that never ran.
        let written = serde_json::to_string(entry).expect("an entry is written down");
        assert!(
            !written.contains("arguments"),
            "{what}: a request that never ran kept its arguments: {written}"
        );

        // And the error tells a caller which refusal it was.
        assert!(
            !matches!(why, NotRun::ItWasNotWrittenDown(_)),
            "{what}: the record failed rather than the request"
        );
    }
}

/// **Four refusals, four sentences, and each one is a sentence.**
///
/// The control for the test above: a mapping that answered the same word for
/// everything would satisfy every assertion there, because each case is
/// examined on its own.
#[test]
fn the_sentences_a_person_reads_are_not_all_one_sentence() {
    let strings = in_english();
    let mut said: Vec<String> = Vec::new();

    for spoiled in [
        AskedToRun {
            program: PathBuf::from("sh"),
            ..asked()
        },
        AskedToRun {
            chose: HowCodeRuns::AtFullTrust,
            ..asked()
        },
    ] {
        let mut record = Record::default();
        drop(alo_running::checked_and_written_down(
            &mut record,
            &Grantee::named("@terminal"),
            &spoiled,
            &strings,
            noon(),
        ));
        let entry = record
            .everything()
            .next()
            .expect("a refusal left an entry")
            .clone();
        said.push(
            entry
                .happened()
                .why_stopped()
                .expect("a refusal carries a sentence")
                .as_str()
                .to_owned(),
        );
    }

    let (first, second) = (
        said.first().expect("two sentences were collected"),
        said.last().expect("two sentences were collected"),
    );
    assert_eq!(said.len(), 2);
    assert_ne!(
        first, second,
        "an unusable request and the person's own choice were told as the same thing"
    );
}

/// **A record that will not write turns a refusal into a refusal about the
/// record**, because *nothing ran and nothing was written* is the worse news and
/// the one a person's record cannot tell them itself.
#[test]
fn a_record_that_will_not_write_is_itself_the_refusal() {
    /// Somewhere to write that never writes.
    struct AFullDisk;

    impl SomewhereToWrite for AFullDisk {
        fn write_down(&mut self, _entry: alo_record::Entry) -> Result<(), alo_keeping::NotKept> {
            Err(alo_keeping::NotKept::NotAddedTo {
                path: "/var/lib/alo/record".to_owned(),
                why: "no space left on device".to_owned(),
            })
        }
    }

    let spoiled = AskedToRun {
        program: PathBuf::from("sh"),
        ..asked()
    };
    let why = alo_running::checked_and_written_down(
        &mut AFullDisk,
        &Grantee::named("@terminal"),
        &spoiled,
        &in_english(),
        noon(),
    )
    .expect_err("a refusal that could not be written is still a refusal");

    assert!(
        matches!(why, NotRun::ItWasNotWrittenDown(_)),
        "the original refusal was reported and the unwritten record was lost: {why}"
    );
}

/// **A run's account carries what the record needs**, which is what makes the
/// other half of this guarantee keepable where a kernel exists.
#[test]
fn what_a_run_reports_is_what_the_record_asks_for() {
    let what = alo_running::ARun::checked(
        HowCodeRuns::InASealedBox,
        PathBuf::from("/bin/sh"),
        vec!["-c".into(), "true".into()],
        vec![PathBuf::from("/bin"), PathBuf::from("/usr/lib")],
    )
    .expect("a well-formed run");

    let happened: WhatHappened = what.as_it_ended(nothing_ran_well());
    assert_eq!(happened.program, PathBuf::from("/bin/sh"));
    assert_eq!(happened.arguments.len(), 2);
    assert_eq!(happened.was_allowed.len(), 2);
    assert!(
        happened.the_kernel_was_holding_it,
        "this crate runs nothing it cannot bound"
    );
}

/// An exit status to hand the account, made the only way a test can make one.
fn nothing_ran_well() -> std::process::ExitStatus {
    std::process::Command::new("/bin/sh")
        .args(["-c", "exit 3"])
        .status()
        .expect("a shell runs")
}
