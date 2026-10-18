//! **Three roads, and all three come back with an entry.** The two refusals
//! matter more than the answer: a question an agent asked about one of a
//! person's files must be written down whether it was answered or not.

use super::*;
use crate::machine::Machine;
use crate::testing::{a_folder_of_our_own, files, granting, in_english, noon};
use alo_capability::{Given, Grants};
use alo_egress::Indicator;
use alo_files::OnThisMachine;
use alo_keeping::NotKept;
use alo_record::{Entry as AnEntry, Record};
use std::path::Path;

/// A call of the record's verb, about this file.
fn asking_about(file: &Path) -> alo_capability::Call {
    alo_record::record_verbs()
        .expect("the record's verb declares")
        .call(
            alo_record::what_touched_a_file::THE_VERB,
            &[(
                alo_record::what_touched_a_file::THE_FILE,
                Given::text(file.display().to_string()),
            )],
        )
        .expect("the verb takes one file")
}

/// A record in which one folder was listed.
fn a_record_that_listed(folder: &Path, grants: &Grants) -> Record {
    let mut record = Record::default();
    let listing = crate::testing::listing(folder);
    let read =
        Authorised::read(&listing, &files(), grants, noon()).expect("the grant covers the folder");
    record.keep(AnEntry::ran(&read, &in_english()));
    record
}

/// Put the question to a machine whose record is this one.
fn on_a_machine<T>(kept: &mut dyn crate::Shortening, doing: impl FnOnce(&Machine<'_>) -> T) -> T {
    let strings = in_english();
    let mut bounding = crate::testing::NothingIsBounded;
    let mut indicator = Indicator::default();
    let machine = Machine::carrying_out_file_verbs(
        &strings,
        &OnThisMachine,
        &mut bounding,
        &mut indicator,
        kept,
    )
    .expect("the six declare");
    doing(&machine)
}

/// **The question is answered out of the record, and the asking is recorded.**
#[test]
fn what_the_record_says_comes_back_and_the_question_is_written_down() {
    let folder = a_folder_of_our_own("asked-and-answered");
    let grants = granting(&[&folder]);
    let mut record = a_record_that_listed(&folder, &grants);

    let call = asking_about(&folder);
    let authorised = Authorised::read(&call, &files(), &grants, noon())
        .expect("the grant covers the file the question names");

    let (entry, found) = on_a_machine(&mut record, |machine| {
        asking_the_record(machine, authorised)
    });

    let found = found.expect("the record answers");
    assert_eq!(found.found().len(), 1, "{:?}", found.found());
    assert_eq!(
        found.found()[0].effect(),
        alo_capability::Effect::Read,
        "listing a folder changed nothing"
    );
    assert!(found.and_nothing_names_it().is_none());

    // **The read is recorded exactly as a change is.** A person looking at
    // what their agent did should see the questions it asked about them.
    assert_eq!(
        entry.what().expect("the entry describes the call").verb(),
        &alo_record::Line::of(alo_record::what_touched_a_file::THE_VERB)
    );
}

/// **A file the record does not name answers, and says that it does not.**
#[test]
fn a_file_the_record_does_not_name_is_answered_rather_than_refused() {
    let folder = a_folder_of_our_own("asked-about-nothing");
    let grants = granting(&[&folder]);
    let mut record = Record::default();

    let call = asking_about(&folder);
    let authorised =
        Authorised::read(&call, &files(), &grants, noon()).expect("the grant covers it");

    let (_entry, found) = on_a_machine(&mut record, |machine| {
        asking_the_record(machine, authorised)
    });

    let found = found.expect("an empty record answers");
    assert!(found.found().is_empty());
    assert!(
        found.and_nothing_names_it().is_some(),
        "an empty answer has to say it is empty"
    );
}

/// A record on a disk that cannot be read back.
struct ARecordThatCannotBeRead;

impl crate::Kept for ARecordThatCannotBeRead {
    fn keep(&mut self, _entry: AnEntry) -> Result<(), NotKept> {
        Ok(())
    }

    fn written_down(&self) -> Result<Record, NotKept> {
        Err(NotKept::NotThere {
            path: "/var/lib/alo/record.jsonl".to_owned(),
        })
    }
}

impl crate::Shortening for ARecordThatCannotBeRead {
    fn shorten(
        &mut self,
        _keeping: alo_keeping::Keeping,
        _now: std::time::SystemTime,
    ) -> Result<crate::Shortened, NotKept> {
        Ok(crate::Shortened::NotOnADisk)
    }
}

/// **A record that cannot be read is reported, never answered as empty.**
///
/// The sharpest test here. *Nothing was written down* and *what was written
/// down could not be read* are different facts, and an executor that folded
/// the second into the first would tell a person their file was untouched
/// because a disk was unreadable.
#[test]
fn a_record_that_cannot_be_read_is_a_refusal_and_not_an_empty_answer() {
    let folder = a_folder_of_our_own("unreadable-record");
    let grants = granting(&[&folder]);
    let mut kept = ARecordThatCannotBeRead;

    let call = asking_about(&folder);
    let authorised =
        Authorised::read(&call, &files(), &grants, noon()).expect("the grant covers it");

    let (entry, found) = on_a_machine(&mut kept, |machine| asking_the_record(machine, authorised));

    let refused = found.expect_err("an unreadable record is not an answer");
    assert!(
        matches!(
            refused,
            NotDone::CouldNotAskTheRecord(WentWrongAsking::NotReadBack(_))
        ),
        "{refused:?}"
    );
    // And the failed question is still written down.
    assert!(entry.what().is_some());
}

/// **What a person reads says nothing was found out, either way.**
///
/// Not *nothing touched the file*: the sentence has to leave that open,
/// because nothing was learnt about the file at all.
#[test]
fn the_sentence_a_person_reads_does_not_claim_the_file_was_untouched() {
    let strings = in_english();
    let said = NotDone::CouldNotAskTheRecord(WentWrongAsking::NotReadBack(NotKept::NotThere {
        path: "/var/lib/alo/record.jsonl".to_owned(),
    }))
    .said(&strings);

    assert!(!said.is_a_bug(), "{said}");
    assert!(
        said.text().contains("record could not be read"),
        "{}",
        said.text()
    );
    assert!(
        !said.text().contains("nothing was done"),
        "the sentence must not claim the file was untouched: {}",
        said.text()
    );
}

/// **Another verb's authority is refused, and the attempt is recorded.**
#[test]
fn another_verbs_authority_does_not_reach_the_record() {
    let folder = a_folder_of_our_own("wrong-verb-at-the-record");
    let grants = granting(&[&folder]);
    let mut record = Record::default();

    let listing = crate::testing::listing(&folder);
    let authorised =
        Authorised::read(&listing, &files(), &grants, noon()).expect("the grant covers it");

    let (entry, found) = on_a_machine(&mut record, |machine| {
        asking_the_record(machine, authorised)
    });

    let refused = found.expect_err("a file verb is not the record's question");
    assert!(
        matches!(
            refused,
            NotDone::CouldNotAskTheRecord(WentWrongAsking::NotItsQuestion(_))
        ),
        "{refused:?}"
    );
    assert!(entry.what().is_some(), "the misrouting is written down");
}

/// **The record's verb is on the list a whole machine offers.**
///
/// The fault this repository has met four times: a verb the machine carries
/// out and the agent is never told about. Routed in `crate::carrying` and
/// declared here, so the two cannot drift.
#[test]
fn a_whole_machine_offers_the_record_verb_it_can_carry_out() {
    let offered =
        crate::machine::the_verbs_a_whole_machine_offers().expect("the twelve verbs declare");
    assert!(
        offered
            .of(alo_record::what_touched_a_file::THE_VERB)
            .is_some(),
        "the machine carries this verb out and must offer it"
    );
}
