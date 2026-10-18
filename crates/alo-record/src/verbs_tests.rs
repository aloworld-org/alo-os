//! **The refusals outnumber the happy path**, and the one that matters most is
//! that the list stays closed: a verb that runs something must not be
//! reachable from the record's list any more than from anywhere else.

use super::*;
use alo_capability::{Ask, CallError, Given};
use alo_strings::Strings;

/// This crate's words beside the capability model's, with nothing translated —
/// a refusal at the door is worded by the deciding crate.
fn in_english() -> Strings {
    let mut vocabulary = crate::record_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A call of the verb, about this file.
fn asking(file: &str) -> Result<alo_capability::Call, CallError> {
    record_verbs()
        .unwrap()
        .call(THE_VERB, &[(THE_FILE, Given::text(file))])
}

/// The list is the one verb, and nothing arrived with it.
#[test]
fn the_list_is_the_one_record_verb() {
    let verbs = record_verbs().unwrap();
    let names: Vec<_> = verbs.all().map(Verb::name).collect();
    assert_eq!(names, [THE_VERB]);
}

/// **A question about the record is a read.** It runs inside the turn, nobody
/// approves it, and that is a property of the verb rather than of its caller.
#[test]
fn asking_the_record_is_a_read_that_nobody_is_asked_to_approve() {
    let verbs = record_verbs().unwrap();
    let verb = verbs.of(THE_VERB).unwrap();
    assert_eq!(verb.effect(), Effect::Read);
    assert!(!verb.effect().waits_for_approval());
    assert!(
        !asking("/home/anna/invoice.pdf")
            .unwrap()
            .waits_for_approval()
    );
}

/// **The grant is over the file the question names**, and a call asks the
/// grants about exactly that path.
///
/// This is the test that would fail if somebody made the verb need no grant,
/// which is the change that would let an agent ask about any path on the disk.
#[test]
fn the_grant_is_over_the_file_the_question_is_about() {
    let verbs = record_verbs().unwrap();
    let verb = verbs.of(THE_VERB).unwrap();
    assert_eq!(
        verb.requires(),
        &Requires::Grants(vec![THE_FILE.to_owned()])
    );
    let call = asking("/home/anna/invoice.pdf").unwrap();
    assert_eq!(call.asks(), &[Ask::path("/home/anna/invoice.pdf")]);
}

/// The sentence a person reads while it runs names the file.
#[test]
fn what_a_person_reads_is_a_sentence_naming_the_file() {
    let said = asking("/home/anna/invoice.pdf")
        .unwrap()
        .sentence(&in_english());
    assert_eq!(
        said.text(),
        "look in the record for what was done to /home/anna/invoice.pdf"
    );
    assert!(!said.is_a_bug());
    assert!(said.unfilled().is_empty());
}

/// A call that does not survive the door is not a call: a file named from
/// somewhere rather than from the root, and a missing argument.
#[test]
fn a_call_that_does_not_survive_the_door_is_not_a_call() {
    let strings = in_english();

    let relative = asking("invoice.pdf").unwrap_err();
    assert!(matches!(relative, CallError::Argument(_)), "{relative:?}");
    assert!(relative.said(&strings).text().contains("full path"));

    let missing = record_verbs().unwrap().call(THE_VERB, &[]).unwrap_err();
    assert!(missing.said(&strings).text().contains(THE_FILE));
}

/// The list is closed, and this verb does not open it.
#[test]
fn nothing_that_runs_something_is_on_the_list() {
    let verbs = record_verbs().unwrap();
    for asked in [
        "exec",
        "run_command",
        "search_the_record",
        "forget_an_entry",
        "read_file",
    ] {
        assert!(verbs.of(asked).is_none(), "{asked}");
        assert!(matches!(
            verbs.call(asked, &[]),
            Err(CallError::NoSuchVerb { .. })
        ));
    }
}

/// A list that already holds the name keeps its own.
#[test]
fn a_name_already_taken_is_not_quietly_replaced() {
    let mut verbs = record_verbs().unwrap();
    let again = declare_into(&mut verbs).unwrap_err();
    assert!(matches!(again, Declaring::List(_)), "{again}");
    assert!(again.to_string().contains(THE_VERB), "{again}");
    assert_eq!(verbs.len(), 1);
}

/// The verb sits beside the file verbs on one list, which is the arrangement a
/// daemon has: one registry, every crate declaring into it.
#[test]
fn it_joins_a_list_that_already_holds_the_file_verbs() {
    let mut verbs = alo_files::file_verbs().unwrap();
    let before = verbs.len();
    declare_into(&mut verbs).unwrap();
    assert_eq!(verbs.len(), before + 1);
    assert!(verbs.of(THE_VERB).is_some());
    assert!(verbs.of("read_file").is_some());
}

/// **Every word the verb is declared with is one this crate declares.** A
/// constant left out of [`crate::words`]' list would compile, declare and
/// reach a person as a key in the place where the sentence belongs.
#[test]
fn everything_the_verb_says_is_something_this_crate_declares() {
    let strings = in_english();
    for verb in record_verbs().unwrap().all() {
        let purpose = verb.purpose(&strings);
        assert!(!purpose.is_a_bug(), "{}: {purpose}", verb.name());
        for arg in verb.args() {
            let said = arg.purpose(&strings);
            assert!(!said.is_a_bug(), "{} {}: {said}", verb.name(), arg.name());
        }
        assert_eq!(
            verb.sentence().key(),
            &words::WHAT_TOUCHED_THIS_FILE_SENTENCE.key()
        );
    }
}

/// Nothing arrives declared.
#[test]
fn nothing_is_declared_until_somebody_declares_it() {
    let empty = Verbs::default();
    assert!(empty.of(THE_VERB).is_none());
    let mut mine = Verbs::default();
    declare_into(&mut mine).unwrap();
    assert_eq!(mine.len(), 1);
}
