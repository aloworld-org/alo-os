//! *"I can't open this file"* is recorded like any other outcome, so *what
//! happened when I opened that* is answerable afterwards.
//!
//! Task 4 of `docs/autonomy/v0-5-documents-and-paper-plan.md`. Converting is the
//! verb on this machine that opens a document a person was sent, under a grant,
//! and every conversion that ran is recorded with what the person was told
//! (ADR 0039 §6). So a document this machine cannot open is walked here through
//! the whole verb — declared, proposed, approved once, redeemed, resolved,
//! carried out — and then asked of the record by the file it touched: the entry
//! is there, and it keeps exactly the explanation the person read — what the
//! file is, why it cannot be opened, what would open it, and that nothing was
//! converted.
//!
//! No converting service is running for any of these, on purpose: a file that
//! cannot be opened is decided about before anything is asked to convert it, so
//! the answer is about the file and never about a missing service.

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod making;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::{Approvals, Given, Grant, Grantee, Grants, Proposal, Reach};
use alo_converting::recording::entry;
use alo_converting::{ConvertingService, Done, NotConverted, convert, converting_verbs};
use alo_files::{OnThisMachine, Resolving, Touching};
use alo_opening::{Cannot, Container, Decided, Kind, Named, Outcome, Would};
use alo_record::{Asking, Only, Record};
use alo_strings::{Filling, Strings, Vocabulary};

/// A fixed moment, so that expiry is arithmetic rather than a wait.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the grants and the approval last.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The agent the tests grant things to.
fn agent() -> Grantee {
    Grantee::named("@documents")
}

/// Every sentence a conversion can be told in, from every crate that words one.
fn in_english() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    alo_converting::declare_into(&mut vocabulary).unwrap();
    alo_opening::declare_into(&mut vocabulary).unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_files::words::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A folder of this test's own, resolved.
fn a_folder(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-converting-cannot-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    OnThisMachine.real(&folder).unwrap().into_path_buf()
}

/// These bytes, sent under this name into a granted folder, converted into
/// another granted folder with no converting service anywhere — and the record
/// that conversion was kept in.
fn sent_and_converted(named: &str, bytes: &[u8]) -> (PathBuf, PathBuf, Done, Record) {
    let strings = in_english();
    let received = a_folder("received");
    let chosen = a_folder("chosen");
    let file = received.join(named);
    fs::write(&file, bytes).unwrap();
    let mut grants = Grants::default();
    for reach in [Reach::Folder(received), Reach::Folder(chosen.clone())] {
        grants.grant(Grant::checked("@documents", reach, noon(), hour()).unwrap());
    }

    let call = converting_verbs()
        .unwrap()
        .call(
            "convert_document",
            &[
                ("file", Given::text(file.to_string_lossy().into_owned())),
                ("into", Given::text(chosen.to_string_lossy().into_owned())),
            ],
        )
        .unwrap();
    let mut approvals = Approvals::default();
    let id =
        approvals.propose(Proposal::checked(&call, &agent(), &grants, noon(), hour()).unwrap());
    let authorised = approvals
        .approve(id, noon())
        .unwrap()
        .redeem(&grants, noon())
        .unwrap();
    let touching = Touching::of(authorised, &grants, &OnThisMachine, &strings).unwrap();
    let nowhere = a_folder("no-service").join("socket");
    let done = convert(&ConvertingService::at(&nowhere), touching, &grants).unwrap();

    let mut record = Record::default();
    record.keep(entry(&done, &strings));
    (file, chosen, done, record)
}

/// What the record answers about the file, asked afterwards by the file
/// itself: the one entry that touched it, as the lines it keeps.
fn what_the_record_says_happened(record: &Record, file: &Path) -> Vec<String> {
    let asking = Asking::anything().by("@documents").only(Only::Executions);
    let about: Vec<_> = record
        .answering(&asking)
        .filter(|entry| {
            entry
                .what()
                .is_some_and(|what| what.touched(&file.to_string_lossy()))
        })
        .collect();
    assert_eq!(about.len(), 1, "the record holds {} entries", record.len());
    about
        .into_iter()
        .flat_map(|entry| entry.told().iter().map(ToString::to_string))
        .collect()
}

/// The whole of what a person reads, as text.
fn read(done: &Done, strings: &Strings) -> Vec<String> {
    done.said(strings)
        .iter()
        .map(|said| said.text().to_owned())
        .collect()
}

/// **Each reason a document cannot be opened is recorded with its whole
/// explanation** — an empty file, a damaged one, a program, and bytes nothing
/// recognises — and nothing is created, the original is untouched, and the
/// record answers from the file's own path.
#[test]
fn each_reason_is_recorded_with_what_the_person_was_told() {
    let strings = in_english();
    let whole = making::a_word_document_in("Garamond");
    let half = whole.get(..whole.len() / 2).unwrap().to_vec();
    for (named, bytes, reason, would) in [
        ("minutes", Vec::new(), Cannot::Empty, Would::ACompleteCopy),
        (
            "contract",
            half,
            Cannot::Damaged(Container::Compressed),
            Would::ACompleteCopy,
        ),
        (
            "setup",
            b"\x7fELF\x02\x01\x01\0\0\0\0\0\0\0\0\0".to_vec(),
            Cannot::AProgram,
            Would::TheDocumentItself,
        ),
        (
            // Bytes of no kind at all. This case held an MP4 file until
            // `alo-opening` learned the containers people are sent: a film is
            // now recognised as the film it is, and is refused as a kind this
            // converter does not convert rather than as something nothing
            // recognises.
            "whatever",
            b"\x07\x0e\x13\x2b\x91\xa0\x00\xff\xfe\x01".to_vec(),
            Cannot::Unrecognised,
            Would::WhoeverMadeIt,
        ),
    ] {
        let (file, chosen, done, record) = sent_and_converted(named, &bytes);
        assert_eq!(
            done.not_converted(),
            Some(&NotConverted::CannotBeOpened(reason)),
            "{named}"
        );
        assert_eq!(reason.would(), would);

        let told = read(&done, &strings);
        let mut expected: Vec<String> = reason
            .explained(&strings)
            .iter()
            .map(|said| said.text().to_owned())
            .collect();
        assert_eq!(
            expected.last().map(String::as_str),
            Some(would.said(&strings).text())
        );
        expected.push(
            strings
                .say(
                    &alo_converting::words::NOTHING_WAS_CONVERTED.key(),
                    &Filling::nothing(),
                )
                .text()
                .to_owned(),
        );
        assert_eq!(told, expected, "{named}");

        assert_eq!(
            what_the_record_says_happened(&record, &file),
            told,
            "{named}"
        );
        assert_eq!(fs::read_dir(&chosen).unwrap().count(), 0, "{named}");
        assert_eq!(fs::read(&file).unwrap(), bytes, "{named}");
    }
}

/// **A document whose name lies and cannot be opened is recorded with the
/// finding first and the whole explanation after it** — the lie never cuts the
/// way out of the record.
#[test]
fn a_name_that_lies_is_recorded_with_the_explanation_after_it() {
    let strings = in_english();
    let (file, chosen, done, record) =
        sent_and_converted("invoice.docx", b"\x7fELF\x02\x01\x01\0\0\0\0\0\0\0\0\0");
    let Some(NotConverted::NotWhatItsNameSays(decided)) = done.not_converted() else {
        panic_free_failure(&done);
        return;
    };
    assert_eq!(
        *decided,
        Decided::NotWhatItsNameSays {
            named: Named::A(Kind::WordDocument),
            outcome: Outcome::CannotOpen(Cannot::AProgram),
        }
    );
    let told = what_the_record_says_happened(&record, &file);
    assert_eq!(told, read(&done, &strings));
    assert_eq!(
        told.first().map(String::as_str),
        Some("This file is named as a Word document, but it is a program")
    );
    assert_eq!(
        told.get(1..3).map(<[String]>::to_vec),
        Some(
            Cannot::AProgram
                .explained(&strings)
                .iter()
                .map(|said| said.text().to_owned())
                .collect()
        )
    );
    assert_eq!(fs::read_dir(&chosen).unwrap().count(), 0);
}

/// Fail a test with what was decided instead, without a panic the lints refuse.
fn panic_free_failure(done: &Done) {
    assert_eq!(
        format!("{:?}", done.not_converted()),
        "Some(NotWhatItsNameSays(..))"
    );
}
