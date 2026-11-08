//! The plain sentence for the refused start on a composefs machine, and for
//! nothing else.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::path::PathBuf;

use alo_running::NotRun;
use alo_running::the_helpers_word::StartRefused;
use alo_strings::{Strings, Vocabulary};

use super::{Told, WhatItSays};

/// What the helper said about the refusal on a booted machine on 2026-11-08.
const ACCOUNT: &str =
    "/usr/bin/ls did not start inside /sys/fs/cgroup/alo-try: Permission denied (os error 13)";

/// The words `alo-try` speaks.
fn strings() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// The start the box denied after it was joined.
fn denied() -> NotRun {
    NotRun::TheBoxRefusedTheCommand {
        program: PathBuf::from("/usr/bin/ls"),
        refused: StartRefused::PermissionDenied,
        account: ACCOUNT.to_owned(),
    }
}

/// **The refused start on a composefs machine says so plainly**, naming the
/// program and where to go instead.
#[test]
fn a_denied_start_on_composefs_says_so_plainly() {
    let not_run = denied();
    let says = WhatItSays::of(&not_run, true);
    assert_eq!(
        says,
        WhatItSays::NotOnThisComputerYet {
            program: PathBuf::from("/usr/bin/ls")
        }
    );
    assert_eq!(
        says.said(&not_run, &strings()),
        "/usr/bin/ls was not run. The sealed box can't run programs on this computer yet. \
         Choose 'ask each time' in Settings → Running code to run commands after you approve \
         them."
    );
}

/// **Every other refusal keeps its own sentence**: the same start off
/// composefs, a start refused for another reason, and a box that could not be
/// built at all.
#[test]
fn every_other_refusal_keeps_its_own_sentence() {
    let strings = strings();
    let otherwise = NotRun::TheBoxRefusedTheCommand {
        program: PathBuf::from("/usr/bin/ls"),
        refused: StartRefused::Otherwise,
        account: ACCOUNT.to_owned(),
    };
    let unbuilt = NotRun::NothingToReach;
    for (not_run, composefs) in [(denied(), false), (otherwise, true), (unbuilt, true)] {
        let says = WhatItSays::of(&not_run, composefs);
        assert_eq!(says, WhatItSays::ItsOwn, "{not_run}");
        let said = says.said(&not_run, &strings);
        assert!(
            !said.contains("can't run programs on this computer yet"),
            "{said}"
        );
    }
}

/// **On a composefs machine the plain sentence is the only line the person
/// sees**, and the helper's account and `alo-try`'s own go to the journal —
/// the walk of 2026-11-08 found both on the terminal around it.
#[test]
fn the_plain_sentence_stands_alone_and_the_account_goes_to_the_journal() {
    let not_run = denied();
    let told = WhatItSays::of(&not_run, true).told(&not_run, &strings());
    let [only] = told.on_the_terminal.as_slice() else {
        panic!("more than the sentence: {:?}", told.on_the_terminal);
    };
    assert!(only.starts_with("/usr/bin/ls was not run."), "{only}");
    assert!(!only.contains("Permission denied"), "{only}");
    assert!(!only.contains("alo-try:"), "{only}");
    let account = told.for_the_journal.unwrap();
    assert!(
        account.starts_with("alo-try: the box was joined"),
        "{account}"
    );
    assert!(
        account.contains(ACCOUNT),
        "the helper's account was lost: {account}"
    );
}

/// **The refusal path: every other refusal keeps both of its lines on the
/// terminal**, the person's sentence and then the administrator's, and
/// nothing goes to the journal instead — a fault in the machine is said where
/// it is sure to be seen.
#[test]
fn every_other_refusal_keeps_its_two_lines() {
    let strings = strings();
    for (not_run, composefs) in [(denied(), false), (NotRun::NothingToReach, true)] {
        let told = WhatItSays::of(&not_run, composefs).told(&not_run, &strings);
        assert_eq!(
            told,
            Told {
                on_the_terminal: vec![
                    WhatItSays::ItsOwn.said(&not_run, &strings),
                    format!("alo-try: {not_run}"),
                ],
                for_the_journal: None,
            }
        );
    }
}
