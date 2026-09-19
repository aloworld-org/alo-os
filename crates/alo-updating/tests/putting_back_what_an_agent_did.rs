//! ★ *Undo what the agent did*, from the record — the half of task 4's
//! acceptance that needs an entry a machine really wrote.
//!
//! `crates/alo-keeping-up/tests/undo_what_the_agent_did.rs` holds the table
//! itself against every verb alo OS ships. This holds the seam: that a real
//! entry, made the way the machine makes one, is read onto that table and
//! answered — and that an undo leaves a record which stays a complete account
//! of the afternoon rather than one with a hole where a change used to be.
//!
//! | Criterion | Test |
//! |---|---|
//! | for a verb the record says ran, whether it can be undone | [`a_change_the_record_says_ran_is_answered_and_so_is_a_read`] |
//! | and it says plainly when it cannot | [`what_cannot_be_put_back_says_why_in_words_the_machine_collects`] |
//! | undoing is itself recorded, with the entry it undid named | [`an_undo_is_written_down_naming_what_it_undid`] |
//! | an undo that did not happen is recorded too | [`an_undo_that_did_not_happen_is_recorded_as_a_refusal`] |
//! | the record stays complete after the original is gone | [`what_an_undo_undid_is_a_copy_and_outlives_the_original`] |
//! | an undo is the person's, with no agent behind it | [`an_undo_names_no_agent_and_is_no_execution_of_a_verb`] |
//! | no inverse is invented | [`the_seam_puts_nothing_back`] |
//!
//! # The entries are real entries
//!
//! The verbs come from `alo-declared` and the words from `alo-saying`, so what
//! is written down here is what this machine would write down: the real
//! `move_file`, the real grants, one approval redeemed once, and the sentence
//! the person would have read. A fixture verb of this test's own would let the
//! table agree with a machine that does not exist.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use alo_capability::{
    Approvals, Authorised, Call, Given, Grant, Grantee, Grants, Proposal, Reach, Verbs,
};
use alo_declared::every_verb_this_machine_ships;
use alo_keeping_up::{NotUndoable, WhatWasDone};
use alo_record::{Entry, Happened};
use alo_saying::everything_this_machine_can_say;
use alo_strings::Strings;
use alo_updating::{putting_back, what_was_done};

/// A fixed moment, so everything about time here is arithmetic.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long a grant and an approval last here.
fn a_day() -> Duration {
    Duration::from_secs(60 * 60 * 24)
}

/// The agent that moves files about.
fn files() -> Grantee {
    Grantee::named("@files")
}

/// Every verb this machine ships, which is where the calls below come from.
fn the_verbs() -> Verbs {
    every_verb_this_machine_ships().expect("the verbs this machine ships")
}

/// The machine's whole vocabulary, in English.
fn the_machine() -> Strings {
    Strings::of(everything_this_machine_can_say().unwrap())
}

/// Grants to `@files` over these folders, made at noon and lasting a day.
fn granting(reaches: &[&str]) -> Grants {
    let mut grants = Grants::default();
    for reach in reaches {
        grants.grant(
            Grant::checked(
                "@files",
                Reach::Folder(PathBuf::from(reach)),
                noon(),
                a_day(),
            )
            .unwrap(),
        );
    }
    grants
}

/// March's invoice moved into the archive, as the real `move_file` takes it.
fn moving_march() -> Call {
    the_verbs()
        .call(
            "move_file",
            &[
                ("file", Given::text("/home/anna/Invoices/march.pdf")),
                ("into", Given::text("/home/anna/Archive")),
            ],
        )
        .expect("the machine's own move_file takes a file and a folder")
}

/// What is in the invoices folder, as the real `list_folder` takes it.
fn listing_invoices() -> Call {
    the_verbs()
        .call(
            "list_folder",
            &[("folder", Given::text("/home/anna/Invoices"))],
        )
        .expect("the machine's own list_folder takes a folder")
}

/// The invoice really moved: proposed, approved once, redeemed and run.
fn a_file_an_agent_moved() -> Entry {
    let grants = granting(&["/home/anna/Invoices", "/home/anna/Archive"]);
    let mut approvals = Approvals::default();
    let id = approvals
        .propose(Proposal::checked(&moving_march(), &files(), &grants, noon(), a_day()).unwrap());
    let approved = approvals.approve(id, noon()).unwrap();
    let running = approved.redeem(&grants, noon()).unwrap();
    Entry::ran(&running, &the_machine())
}

/// A read that ran, which nobody was asked about.
fn a_folder_an_agent_read() -> Entry {
    let grants = granting(&["/home/anna/Invoices"]);
    let read = Authorised::read(&listing_invoices(), &files(), &grants, noon()).unwrap();
    Entry::ran(&read, &the_machine())
}

/// **A change the record says ran is read onto the table, and so is a read.**
///
/// The first acceptance, asked of an entry a machine really wrote rather than
/// of a verb's name in isolation: what was done is read off the entry, and the
/// answer on this machine is the honest *not yet here* for the change and
/// *nothing changed* for the read.
#[test]
fn a_change_the_record_says_ran_is_answered_and_so_is_a_read() {
    let moved = a_file_an_agent_moved();
    assert_eq!(
        what_was_done(&moved),
        WhatWasDone::ChangedFiles(alo_keeping_up::Change::Moved)
    );
    assert_eq!(
        putting_back(&moved).unwrap_err(),
        NotUndoable::NothingKeepsWhatWasThere,
        "a machine that keeps nothing has to say so rather than offer an undo"
    );

    let read = a_folder_an_agent_read();
    assert_eq!(what_was_done(&read), WhatWasDone::OnlyLooked);
    assert_eq!(putting_back(&read).unwrap_err(), NotUndoable::ItOnlyLooked);
}

/// **What cannot be put back says why, in words the machine collects.**
///
/// The half of ★ that matters most is the half that says no, and a refusal
/// without a reason is a system a person stops trusting. Asked of the whole
/// machine's vocabulary, because a sentence `alo-saying` does not collect is a
/// key in guillemets on a screen.
#[test]
fn what_cannot_be_put_back_says_why_in_words_the_machine_collects() {
    let strings = the_machine();
    for entry in [a_file_an_agent_moved(), a_folder_an_agent_read()] {
        let why = putting_back(&entry).unwrap_err();
        let said = why.said(&strings);
        assert!(!said.is_a_bug(), "{why:?}: {said}");
        assert!(!said.text().is_empty(), "{why:?}");
    }

    // And the sentence for a machine that keeps nothing names the machine
    // rather than the change: it is not the person's fault and not the
    // agent's, and the words say what is true.
    let said = NotUndoable::NothingKeepsWhatWasThere.said(&strings);
    assert!(said.text().contains("does not keep"), "{said}");
}

/// **An undo is written down, naming what it undid and when that happened.**
///
/// ADR 0045 point 4. The record remains a complete account of the afternoon:
/// a change and then the person putting it back are two entries, and the
/// second says which change it was in the words the person approved for it.
#[test]
fn an_undo_is_written_down_naming_what_it_undid() {
    let moved = a_file_an_agent_moved();
    let undo = Entry::undone(&moved, noon() + a_day()).expect("an undo of a call that was made");

    assert_eq!(undo.at(), noon() + a_day());
    let (undid, what) = undo.undid().expect("an undo says what it undid");
    assert_eq!(undid, moved.at());
    assert!(what.verb().is("move_file"));
    assert_eq!(
        Some(what.sentence()),
        moved.what().map(alo_record::What::sentence),
        "the undo and the record worded one change differently"
    );

    // It is not an execution, so a question about what ran never answers with
    // an undo, and it is not a refusal either: this one happened.
    assert_eq!(undo.what(), None);
    assert!(!undo.happened().ran());
    assert!(!undo.happened().was_stopped());
    assert!(!undo.happened().caused_egress());

    // An entry nothing was ever called for has nothing to put back, and the
    // honest shape is no entry at all rather than one with an empty account of
    // what it reversed — including an undo of an undo.
    assert!(Entry::undone(&Entry::paired("the studio", noon()), noon()).is_none());
    assert!(Entry::undone(&undo, noon()).is_none());
    assert!(Entry::undo_failed(&undo, "it could not be", noon()).is_none());
}

/// **An undo that did not happen is recorded too, as a refusal.**
///
/// A record that held only the undos that worked would answer *what became of
/// my afternoon* with the half that went well. The sentence kept is the one
/// the person was shown, so the record and the screen are one account.
#[test]
fn an_undo_that_did_not_happen_is_recorded_as_a_refusal() {
    let moved = a_file_an_agent_moved();
    let why = NotUndoable::ChangedSinceItWasDone
        .said(&the_machine())
        .into_text();
    let failed = Entry::undo_failed(&moved, &why, noon() + a_day()).expect("an undo that failed");

    assert!(failed.happened().was_stopped());
    assert!(
        failed
            .happened()
            .why_stopped()
            .is_some_and(|shown| shown.is(&why)),
        "the record kept a different sentence from the one the person read"
    );
    let (undid, what) = failed
        .undid()
        .expect("a failed undo still says what it was for");
    assert_eq!(undid, moved.at());
    assert!(what.verb().is("move_file"));

    // And the one that happened is not a refusal, so a review looking for what
    // did not happen finds exactly the ones that did not.
    assert!(
        !Entry::undone(&moved, noon())
            .expect("an undo")
            .happened()
            .was_stopped()
    );
}

/// **What an undo undid is a copy, and it outlives the original.**
///
/// The record is appended to and shortened, so a position in the file is not a
/// name that lasts: an undo that pointed at one would become a line about
/// nothing the day the change it reversed was pruned. Read back on its own,
/// with nothing else in the file, the undo still says what it put back.
#[test]
fn what_an_undo_undid_is_a_copy_and_outlives_the_original() {
    let moved = a_file_an_agent_moved();
    let undo = Entry::undone(&moved, noon() + a_day()).expect("an undo");

    let written = serde_json::to_string(&undo).unwrap();
    let alone: Entry = serde_json::from_str(&written).unwrap();
    let (undid, what) = alone.undid().expect("the copy survived being read back");
    assert_eq!(undid, moved.at());
    assert!(what.verb().is("move_file"));
    assert!(
        what.sentence()
            .as_str()
            .contains("/home/anna/Invoices/march.pdf"),
        "{}",
        what.sentence().as_str()
    );

    // The file says `undone`, and the account it carries is the change's own.
    assert!(written.contains("\"undone\""), "{written}");
    assert!(written.contains("\"undid\""), "{written}");
}

/// **An undo is the person's: it names no agent, and there is no field for
/// one.**
///
/// ADR 0045 point 2. A name in that position would be an authority the record
/// invented, and it would sit in a review's *who did what* column beside
/// agents that really were granted something.
#[test]
fn an_undo_names_no_agent_and_is_no_execution_of_a_verb() {
    let undo = Entry::undone(&a_file_an_agent_moved(), noon()).expect("an undo");
    assert_eq!(undo.agent(), None);
    assert!(matches!(undo.happened(), Happened::Undone { .. }));

    let written = serde_json::to_string(&undo).unwrap();
    assert!(!written.contains("agent"), "{written}");

    // And an undo already done is not itself an agent's to undo.
    assert_eq!(what_was_done(&undo), WhatWasDone::NotAnAgents);
    assert_eq!(putting_back(&undo).unwrap_err(), NotUndoable::NotAnAgents);
}

/// **The seam puts nothing back, and invents no inverse.**
///
/// *What can be undone is undone through the mechanism that made it undoable,
/// and never by a second implementation guessing at inverses* — the plan's
/// constraint, held against the one file where the record and the decision are
/// put beside each other, which is where such an implementation would go.
#[test]
fn the_seam_puts_nothing_back() {
    let seam = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("putting_back.rs");
    let text = std::fs::read_to_string(&seam).unwrap();
    for forbidden in [
        "fs::rename",
        "fs::copy",
        "fs::remove",
        "fs::write",
        "fs::create",
        "Command",
        "reflink",
        "TheBase",
    ] {
        assert!(
            !text.contains(forbidden),
            "{} names {forbidden}",
            seam.display()
        );
    }
}
