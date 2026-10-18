//! **The two sharpest tests here are about the empty answer**, not the full
//! one: an empty list means *the agent did nothing to this file*, and a person
//! must not be able to read it as *nothing did*.

use super::*;
use crate::test_calls::{
    archiving_march, files, granting_both, hour, listing_invoices, noon, proposing,
};
use crate::testing::in_english;
use alo_capability::{Approvals, Authorised, Effect, Given};

/// An afternoon in which one file was read and another was moved.
///
/// `/home/anna/Invoices` is listed at noon; `/home/anna/Invoices/march.pdf` is
/// moved into `/home/anna/Archive` an hour later.
fn an_afternoon() -> Record {
    let mut record = Record::default();
    let grants = granting_both();

    let read = Authorised::read(&listing_invoices(), &files(), &grants, noon()).unwrap();
    record.keep(Entry::ran(&read, &in_english()));

    let mut approvals = Approvals::default();
    let id = approvals.propose(proposing(&archiving_march(), &grants));
    let approved = approvals.approve(id, noon() + hour()).unwrap();
    let running = approved.redeem(&grants, noon() + hour()).unwrap();
    record.keep(Entry::ran(&running, &in_english()));

    record
}

/// **The file that was moved comes back, with the sentence and the effect.**
#[test]
fn the_file_that_was_moved_comes_back_as_a_change() {
    let found = what_touched(&an_afternoon(), "/home/anna/Invoices/march.pdf");

    assert_eq!(found.file(), "/home/anna/Invoices/march.pdf");
    assert_eq!(found.found().len(), 1, "{:?}", found.found());
    let one = &found.found()[0];
    assert_eq!(one.effect(), Effect::Change);
    assert_eq!(one.at(), noon() + hour());
    assert!(
        one.said().as_str().contains("march.pdf"),
        "the sentence the person was shown names the file: {}",
        one.said().as_str()
    );
}

/// **The folder that was listed comes back as a read, not a change.**
///
/// The distinction a person wants when they ask what happened to something:
/// a `list_folder` named it and altered nothing.
#[test]
fn the_folder_that_was_listed_comes_back_as_a_read() {
    let found = what_touched(&an_afternoon(), "/home/anna/Invoices");

    assert_eq!(found.found().len(), 1, "{:?}", found.found());
    assert_eq!(found.found()[0].effect(), Effect::Read);
}

/// **A file nothing named answers with a sentence, not an empty list.**
///
/// This is `docs/features.md`' own *where nothing connects them it says so*.
/// An empty list drawn as an empty list reads as *nothing happened to this
/// file*, which is a claim this answer cannot make.
#[test]
fn a_file_nothing_named_says_so_rather_than_answering_nothing() {
    let found = what_touched(&an_afternoon(), "/home/anna/Taxes/2025.pdf");

    assert!(found.found().is_empty());
    let said = found
        .and_nothing_names_it()
        .expect("an empty answer says it is empty");
    assert_eq!(said.named(), "record.nothing-touched-it");
}

/// **And an answer that found something does not say nothing names it.**
#[test]
fn an_answer_that_found_something_does_not_claim_otherwise() {
    let found = what_touched(&an_afternoon(), "/home/anna/Invoices/march.pdf");
    assert!(found.and_nothing_names_it().is_none());
}

/// **Every answer says it is what alo did**, whether it found anything or not.
///
/// The one sentence protecting a person from concluding that a file nothing
/// named is a file nothing changed. No shape of the type can omit it, which is
/// what this asserts on both shapes.
#[test]
fn every_answer_says_it_covers_what_alo_did_and_not_what_a_person_did() {
    for file in ["/home/anna/Invoices/march.pdf", "/home/anna/Taxes/2025.pdf"] {
        let found = what_touched(&an_afternoon(), file);
        assert_eq!(
            found.and_only_what_alo_did().named(),
            "record.only-what-alo-did",
            "{file}"
        );
    }
}

/// **A path that is almost the right one matches nothing.**
///
/// `Asking::touching` matches a recorded argument exactly, and this says so
/// rather than leaving it to be discovered: a prefix is not a match, so
/// neither the folder above nor a longer path below answers.
#[test]
fn a_path_that_is_almost_the_right_one_is_not_a_match() {
    let record = an_afternoon();
    for near in [
        "/home/anna/Invoices/march",
        "/home/anna/Invoices/march.pdf.bak",
        "/home/anna/Invoices/",
        "march.pdf",
    ] {
        assert!(
            what_touched(&record, near).found().is_empty(),
            "{near} matched, and only the exact path should"
        );
    }
}

/// **An empty record answers, rather than failing.**
#[test]
fn an_empty_record_answers_that_nothing_names_it() {
    let found = what_touched(&Record::default(), "/home/anna/invoice.pdf");
    assert!(found.found().is_empty());
    assert!(found.and_nothing_names_it().is_some());
}

/// **An entry with no call is skipped rather than refused.**
///
/// A pairing, an update, a command that ran: none can have matched `touching`,
/// which matches a call's arguments, and an entry that cannot describe a call
/// is not evidence of one.
#[test]
fn an_entry_that_describes_no_call_contributes_nothing() {
    let mut record = an_afternoon();
    record.keep(Entry::paired("the studio machine", noon() + hour() * 2));
    record.keep(Entry::updated("1.2.0", "1.3.0", noon() + hour() * 3));

    let found = what_touched(&record, "/home/anna/Invoices/march.pdf");
    assert_eq!(found.found().len(), 1, "{:?}", found.found());
}

/// **The one reader of the call answers the path it was asked about.**
#[test]
fn the_reader_of_the_call_answers_the_file_it_names() {
    let grants = granting_both();
    let verbs = crate::record_verbs().unwrap();
    let call = verbs
        .call(THE_VERB, &[(THE_FILE, Given::text("/home/anna/Invoices"))])
        .unwrap();
    let authorised = Authorised::read(&call, &files(), &grants, noon()).unwrap();

    assert_eq!(
        asked_about(&authorised).unwrap(),
        std::path::Path::new("/home/anna/Invoices")
    );
}

/// **Another verb's authority is refused, and the refusal says which.**
#[test]
fn another_verbs_authority_is_not_read_as_this_one() {
    let grants = granting_both();
    let authorised = Authorised::read(&listing_invoices(), &files(), &grants, noon()).unwrap();

    let refused = asked_about(&authorised).unwrap_err();
    assert_eq!(
        refused,
        NotAsked::AnotherVerb {
            verb: "list_folder".to_owned()
        }
    );
    let (word, gaps) = refused.said();
    assert_eq!(word.named(), "record.verb.not-this-crates");
    assert_eq!(gaps, vec![("verb", "list_folder".to_owned())]);
}

/// **A malformed call is worded as a verb to write again.**
///
/// Unreachable through the door — `alo_capability` validated the argument as a
/// path — so this asserts the wording rather than reaching it through one.
#[test]
fn a_malformed_call_is_worded_for_whoever_wrote_it() {
    let refused = NotAsked::Malformed { argument: THE_FILE };
    let (word, gaps) = refused.said();
    assert_eq!(word.named(), "record.verb.missing");
    assert_eq!(
        gaps,
        vec![
            ("verb", THE_VERB.to_owned()),
            ("argument", THE_FILE.to_owned())
        ]
    );
}
