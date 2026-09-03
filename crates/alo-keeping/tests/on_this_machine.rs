//! The whole journey onto a real disk: a verb, a grant, an approval, an
//! execution, a refusal — written down, the machine turned off, and the record
//! read back and asked what happened.
//!
//! The crate's own tests take each half apart. This is the other half of that
//! bargain: entries that came out of `alo-capability` rather than out of a
//! fixture that only looks like one, on the filesystem the tests are running
//! on, answering the questions ADR 0001 §7 says a record exists to answer.
//!
//! It is not the hardware verification `CLAUDE.md` asks for: that is a
//! certified machine, and this is whatever the tests were run on.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::{
    Approvals, Arg, Authorised, Call, Effect, Given, Grant, Grantee, Grants, Proposal, Reach,
    Requires, Takes, Verb,
};
use alo_keeping::{Keeping, NotKept, Reading, THE_FORMAT, Writing, keeping_words};
use alo_record::{Asking, Entry, Only, Record};
use alo_strings::{Strings, Vocabulary, Word};

/// A fixed moment, so that everything about time here is arithmetic.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// A day, as a retention rule counts one.
fn day() -> Duration {
    Duration::from_secs(24 * 60 * 60)
}

/// The agent the record in these tests is about.
fn files() -> Grantee {
    Grantee::named("@files")
}

/// What the change these tests follow does.
const MOVING: Word = Word::saying(
    "testing.verb.move-file.purpose",
    "move a file into a folder",
);
/// The sentence a person approves before that file moves.
const MOVING_SENTENCE: Word =
    Word::saying("testing.verb.move-file.sentence", "move {file} into {into}");
/// What it moves.
const MOVING_FILE: Word = Word::saying("testing.verb.move-file.argument.file", "the file to move");
/// Where it moves it.
const MOVING_INTO: Word = Word::saying(
    "testing.verb.move-file.argument.into",
    "the folder it goes into",
);
/// What the read these tests follow does.
const LISTING: Word = Word::saying(
    "testing.verb.list-folder.purpose",
    "list what is in a folder",
);
/// What a person is shown while it happens.
const LISTING_SENTENCE: Word = Word::saying(
    "testing.verb.list-folder.sentence",
    "list what is in {folder}",
);
/// What it lists.
const LISTING_FOLDER: Word = Word::saying(
    "testing.verb.list-folder.argument.folder",
    "the folder to list",
);

/// Everything the two fixture verbs can say, beside everything this crate can.
///
/// A record renders the sentence a person approved (item 9g), so a vocabulary
/// missing the verb's words would write a key into the record where the
/// sentence belongs — and the file would then be evidence of a bug rather than
/// of an afternoon.
fn what_the_machine_can_say() -> Strings {
    let mut vocabulary: Vocabulary = keeping_words().unwrap();
    for word in [
        MOVING,
        MOVING_SENTENCE,
        MOVING_FILE,
        MOVING_INTO,
        LISTING,
        LISTING_SENTENCE,
        LISTING_FOLDER,
    ] {
        vocabulary.says(word.phrase().unwrap()).unwrap();
    }
    Strings::of(vocabulary)
}

/// A change: moving one file into one folder, so two things must be granted.
fn move_file() -> Verb {
    Verb::checked(
        "move_file",
        MOVING,
        Effect::Change,
        vec![
            Arg::taking("file", MOVING_FILE, Takes::Path),
            Arg::taking("into", MOVING_INTO, Takes::Path),
        ],
        Requires::grants_over(["file", "into"]),
        MOVING_SENTENCE,
    )
    .unwrap()
}

/// A read: it answers inside the turn and is never proposed.
fn list_folder() -> Verb {
    Verb::checked(
        "list_folder",
        LISTING,
        Effect::Read,
        vec![Arg::taking("folder", LISTING_FOLDER, Takes::Path)],
        Requires::grants_over(["folder"]),
        LISTING_SENTENCE,
    )
    .unwrap()
}

/// The change every test here follows: March's invoice into the archive.
fn archiving_march() -> Call {
    Call::of(
        &move_file(),
        &[
            ("file", Given::text("/home/anna/Invoices/march.pdf")),
            ("into", Given::text("/home/anna/Archive")),
        ],
    )
    .unwrap()
}

/// The read every test here follows.
fn listing_invoices() -> Call {
    Call::of(
        &list_folder(),
        &[("folder", Given::text("/home/anna/Invoices"))],
    )
    .unwrap()
}

/// Grants to `@files` over these folders, made at this moment and lasting a
/// day.
fn granting(reaches: &[&str], at: SystemTime) -> Grants {
    let mut grants = Grants::default();
    for reach in reaches {
        grants.grant(
            Grant::checked("@files", Reach::Folder(PathBuf::from(reach)), at, day()).unwrap(),
        );
    }
    grants
}

/// A folder of this test's own, on the disk the tests are running on.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-keeping-real-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    drop(fs::remove_dir_all(&folder));
    fs::create_dir_all(&folder).unwrap();
    folder
}

/// An afternoon on this machine, written straight to a real record as it
/// happens: a read that ran, a change somebody approved, and a refusal.
fn an_afternoon_onto(path: &Path, strings: &Strings, at: SystemTime) -> Writing {
    let mut writing = Writing::opening(path).unwrap();
    let grants = granting(&["/home/anna/Invoices", "/home/anna/Archive"], at);

    // A read answers inside the turn: nobody is asked, and it is recorded.
    let read = Authorised::read(&listing_invoices(), &files(), &grants, at).unwrap();
    writing.keep(&Entry::ran(&read, strings)).unwrap();

    // A change waits for one approval, and the sentence approved is the
    // sentence written down.
    let mut approvals = Approvals::default();
    let proposal = Proposal::checked(&archiving_march(), &files(), &grants, at, day()).unwrap();
    let id = approvals.propose(proposal);
    let approved = approvals.approve(id, at).unwrap();
    let running = approved.redeem(&grants, at).unwrap();
    writing.keep(&Entry::ran(&running, strings)).unwrap();

    // And something the grants refused, which is the entry a security review
    // is actually reading for.
    let refused = Authorised::read(
        &listing_invoices(),
        &files(),
        &granting(&["/home/anna/Taxes"], at),
        at,
    )
    .unwrap_err();
    writing
        .keep(&Entry::refused(&refused, &files(), strings, at))
        .unwrap();

    writing
}

/// **A record outlives the session that wrote it.** Everything that happened is
/// on the disk, in the order it happened, and reads back as what it was.
#[test]
fn what_happened_this_afternoon_is_still_there_after_the_machine_is_turned_off() {
    let folder = a_folder_of_our_own("afternoon");
    let path = folder.join("record.jsonl");
    let strings = what_the_machine_can_say();

    let writing = an_afternoon_onto(&path, &strings, noon());
    // The machine is turned off.
    drop(writing);

    let reading = Reading::at(&path).unwrap();
    assert_eq!(reading.head().format(), THE_FORMAT);
    assert!(reading.head().is_whole());
    assert!(reading.damage().nothing_wrong());
    assert_eq!(reading.record().len(), 3);

    let record: Record = reading.into_record();
    assert_eq!(
        record.everything().filter(|e| e.happened().ran()).count(),
        2
    );
    assert_eq!(
        record
            .everything()
            .filter(|e| e.happened().was_stopped())
            .count(),
        1
    );
}

/// **"Explain what it did" is a query, and it is the same query after a
/// reboot.** ADR 0001 §7's four answers survive the disk: what ran, under whose
/// authority, from which approval, against which grant.
#[test]
fn a_record_read_back_answers_what_it_answered_before_it_was_written() {
    let folder = a_folder_of_our_own("asking");
    let path = folder.join("record.jsonl");
    let strings = what_the_machine_can_say();
    drop(an_afternoon_onto(&path, &strings, noon()));

    let record = Reading::at(&path).unwrap().into_record();

    let by_the_agent = Asking::anything().by("@files");
    assert_eq!(record.answering(&by_the_agent).count(), 3);
    assert_eq!(record.answering(&Asking::anything().by("@mail")).count(), 0);

    let refusals = Asking::anything().only(Only::Refusals);
    let stopped: Vec<_> = record.answering(&refusals).collect();
    assert_eq!(stopped.len(), 1);
    assert!(
        stopped
            .first()
            .and_then(|entry| entry.what())
            .is_some_and(|what| what.touched("/home/anna/Invoices"))
    );

    // The change that ran came from an approval, and the record says which.
    let approved = record
        .everything()
        .filter_map(|entry| entry.happened().from_approval())
        .count();
    assert_eq!(approved, 1);

    // Nothing that happened before this afternoon is in it.
    let earlier = Asking::anything().between(noon() - day() * 2, noon() - day());
    assert_eq!(record.answering(&earlier).count(), 0);
}

/// **The sentence a person approved is what is on the disk.** Since item 9g a
/// record renders it rather than keeping a copy, so what a shell shows and what
/// a security review reads are one string looked up twice.
#[test]
fn the_sentence_somebody_approved_is_what_the_record_says_afterwards() {
    let folder = a_folder_of_our_own("sentence");
    let path = folder.join("record.jsonl");
    let strings = what_the_machine_can_say();
    drop(an_afternoon_onto(&path, &strings, noon()));

    let record = Reading::at(&path).unwrap().into_record();
    let moved = record
        .everything()
        .find(|entry| entry.what().is_some_and(|what| what.verb().is("move_file")))
        .unwrap();
    assert!(
        moved
            .what()
            .is_some_and(|what| what.touched("/home/anna/Archive"))
    );

    // The file holds the words, so a tool that is not alo OS can read what was
    // approved without knowing anything about vocabularies.
    let written = fs::read_to_string(&path).unwrap();
    assert!(
        written.contains("move /home/anna/Invoices/march.pdf into /home/anna/Archive"),
        "{written}"
    );
}

/// **A record that has been shortened says so, and a machine that did nothing
/// says something different.** This is the distinction the crate exists for:
/// after ageing out a fortnight, the file must not read like a quiet week.
#[test]
fn a_shortened_record_never_reads_like_a_machine_that_did_nothing() {
    let folder = a_folder_of_our_own("shortened");
    let path = folder.join("record.jsonl");
    let strings = what_the_machine_can_say();

    // A fortnight ago, and again today.
    let mut writing = an_afternoon_onto(&path, &strings, noon() - day() * 14);
    drop(writing);
    writing = an_afternoon_onto(&path, &strings, noon());
    assert_eq!(Reading::at(&path).unwrap().record().len(), 6);

    let pruned = writing
        .prune(Keeping::for_days(7).unwrap(), noon())
        .unwrap();
    assert_eq!(pruned.removed(), 3);
    assert_eq!(pruned.kept(), 3);

    let reading = Reading::at(&path).unwrap();
    assert!(!reading.head().is_whole());
    assert_eq!(reading.head().since(), Some(noon() - day() * 7));
    assert!(
        reading
            .head()
            .said(&strings)
            .text()
            .contains("does not go all the way back")
    );

    // The question about a fortnight ago now finds nothing, and the record is
    // the thing that says why.
    let record = reading.into_record();
    let a_fortnight_ago = Asking::anything().between(noon() - day() * 15, noon() - day() * 13);
    assert_eq!(record.answering(&a_fortnight_ago).count(), 0);
}

/// **A record nobody can read all of is not shortened**, on a real disk. The
/// refusal is in the reader's language, and nothing is removed.
#[test]
fn a_record_with_something_wrong_in_it_is_looked_at_rather_than_tidied() {
    let folder = a_folder_of_our_own("wrong");
    let path = folder.join("record.jsonl");
    let strings = what_the_machine_can_say();
    drop(an_afternoon_onto(&path, &strings, noon() - day() * 14));

    // A line that was written whole and is not whole now.
    let text = fs::read_to_string(&path).unwrap();
    let damaged = text.replacen("\"at\"", "\"at", 1);
    fs::write(&path, &damaged).unwrap();

    let reading = Reading::at(&path).unwrap();
    assert!(reading.damage().must_be_looked_at());
    assert_eq!(reading.damage().how_many_unreadable(), 1);
    let sentences = reading.damage().said(&strings);
    assert!(
        sentences
            .first()
            .is_some_and(|said| said.text().contains("not all of what happened"))
    );

    let mut writing = Writing::opening(&path).unwrap();
    let refused = writing
        .prune(Keeping::for_days(1).unwrap(), noon())
        .unwrap_err();
    assert!(matches!(refused, NotKept::Damaged { .. }), "{refused:?}");
    assert!(refused.said(&strings).text().contains("cannot be read"));
    assert_eq!(fs::read_to_string(&path).unwrap(), damaged);
}

/// **★ No telemetry, on a real disk and after the machine is turned off.**
/// What alo OS did with nobody having asked — fetching a model — is written
/// down like anything else and reads back saying what it was, where it reached
/// and that nobody was behind it.
///
/// The entry is made from the errand the indicator showed and from nothing
/// else, which is the guarantee `alo-record` carries and this walks: there is
/// no way here to write down a departure that never appeared to anybody.
#[test]
fn what_the_machine_did_on_its_own_is_on_the_disk_and_names_nobody() {
    let folder = a_folder_of_our_own("errand");
    let path = folder.join("record.jsonl");
    let strings = what_the_machine_can_say();

    let mut indicator = alo_egress::Indicator::default();
    let mut writing = an_afternoon_onto(&path, &strings, noon());
    let underway = indicator.beginning_on_its_own(
        alo_egress::OnItsOwn::for_(
            alo_egress::Errand::FetchingAModel,
            alo_egress::Destination::at("models.alo.example").unwrap(),
        ),
        noon() + Duration::from_secs(60),
    );
    writing.keep(&Entry::left_on_its_own(&underway)).unwrap();
    indicator.ended_on_its_own(underway);
    // The machine is turned off.
    drop(writing);

    let reading = Reading::at(&path).unwrap();
    assert!(reading.damage().nothing_wrong());
    let record = reading.into_record();
    assert_eq!(record.len(), 4);

    let on_its_own = Asking::anything().only(Only::OnItsOwn);
    let errands: Vec<_> = record.answering(&on_its_own).collect();
    assert_eq!(errands.len(), 1);
    let errand = errands.first().unwrap();
    assert_eq!(errand.at(), noon() + Duration::from_secs(60));
    assert_eq!(
        errand.happened().errand(),
        Some(alo_egress::Errand::FetchingAModel)
    );
    assert!(errand.happened().caused_egress());

    // **Nobody, and nobody in the file either.** A tool that is not alo OS
    // reads one line with no agent on it, so nothing downstream can attribute
    // the machine's own errand to something that was granted anything.
    assert_eq!(errand.agent(), None);
    assert_eq!(
        record.answering(&Asking::anything().by("@files")).count(),
        3
    );
    let written = fs::read_to_string(&path).unwrap();
    let line = written
        .lines()
        .find(|line| line.contains("left-on-its-own"))
        .unwrap();
    assert!(line.contains("models.alo.example"), "{line}");
    assert!(!line.contains("agent"), "{line}");
}

/// **A record that is not there is refused rather than answered with
/// nothing.** A machine whose record has been deleted must not read as a
/// machine that has done nothing.
#[test]
fn a_missing_record_is_not_an_empty_one() {
    let folder = a_folder_of_our_own("missing");
    let path = folder.join("record.jsonl");
    let strings = what_the_machine_can_say();

    let refused = Reading::at(&path).unwrap_err();
    assert!(matches!(refused, NotKept::NotThere { .. }));
    let message = refused.said(&strings).into_text();
    assert!(message.contains("has done nothing"), "{message}");

    // Making one is a deliberate act, and the new record says it is whole —
    // which it is: it is a record of nothing, starting now.
    let writing = Writing::opening(&path).unwrap();
    assert!(writing.head().is_whole());
    drop(writing);
    assert!(Reading::at(&path).unwrap().record().is_empty());
}
