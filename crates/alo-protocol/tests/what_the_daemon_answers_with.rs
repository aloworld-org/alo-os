//! What a real turn did, carried back onto the wire.
//!
//! The unit tests beside each file say what an answer is. These say that what
//! the rest of the workspace produces is what this crate can carry: a real
//! folder is really listed, a real file is really read and really renamed, a
//! real refusal is worded by the crate that made it, and every one of those
//! crosses as one line and reads back as itself.
//!
//! It is the answering half of `what_a_client_may_ask.rs`, and it exists for
//! the same reason: a protocol tested only against values a test composed is a
//! description of a protocol rather than the protocol.
//!
//! **The record is in memory**, which is `alo-turn`'s `Kept` for a `Record`.
//! Writing one to a real file is `alo-keeping`'s and is tested there.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::{Grant, Grants, Reach};
use alo_context::Context;
use alo_egress::Indicator;
use alo_files::{OnThisMachine, Resolving as _};
use alo_protocol::{Done, FromAPerson, Kind, NotUnderstood, ToAPerson, ToAnAgent};
use alo_record::Record;
use alo_strings::{Language, Strings, Translation, Vocabulary};
use alo_turn::{Machine, Turning};

/// A fixed moment, so that a question standing for an hour is arithmetic.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long a turn, a grant and a question last in these tests.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// Every word a machine answering these messages has loaded.
fn everything_this_machine_says() -> Vocabulary {
    let mut vocabulary = alo_files::file_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_egress::declare_into(&mut vocabulary).unwrap();
    alo_turn::declare_into(&mut vocabulary).unwrap();
    alo_protocol::declare_into(&mut vocabulary).unwrap();
    vocabulary
}

/// A folder of this test's own with two files in it, resolved.
///
/// Resolved because a grant is over a place, and on Windows a resolved path
/// carries a prefix the typed one does not — `docs/quirks.md` records it.
fn a_folder_with_invoices(what: &str) -> (PathBuf, PathBuf) {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-protocol-answers-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    let folder = OnThisMachine.real(&folder).unwrap().into_path_buf();
    let invoice = folder.join("march.pdf");
    fs::write(&invoice, "March\n\t4180.00\n").unwrap();
    fs::write(folder.join("april.pdf"), "April, 220.00").unwrap();
    (folder, invoice)
}

/// The arrangement every test here needs: a machine that offers the six, a
/// grant over one folder, and a turn under way.
fn on_a_machine<T>(
    what: &str,
    strings: &Strings,
    doing: impl FnOnce(&mut Turning<'_, '_>, &mut Grants, &Path, &Path) -> T,
) -> T {
    let (folder, invoice) = a_folder_with_invoices(what);
    let mut indicator = Indicator::default();
    let mut record = Record::default();
    let mut machine =
        Machine::carrying_out_file_verbs(strings, &OnThisMachine, &mut indicator, &mut record)
            .unwrap();
    let mut grants = Grants::default();
    grants.grant(Grant::checked("@files", Reach::Folder(folder.clone()), noon(), hour()).unwrap());
    let mut turning = Turning::beginning(
        Context::at_invocation(noon()),
        "@files",
        hour(),
        &mut grants,
        &mut machine,
    )
    .unwrap();
    doing(&mut turning, &mut grants, &folder, &invoice)
}

/// **What a real folder holds crosses as a shape and not as a sentence.** The
/// names are the disk's, the kinds are what the disk said, and a client reads
/// them back as what the machine found.
#[test]
fn what_a_real_folder_holds_crosses_as_what_the_machine_found() {
    let strings = Strings::of(everything_this_machine_says());
    let told = on_a_machine("a-listing", &strings, |turning, grants, folder, _| {
        let answer = turning
            .reading(
                "list_folder",
                &[(
                    "folder",
                    alo_capability::Given::text(folder.display().to_string()),
                )],
                grants,
                noon(),
            )
            .unwrap();
        ToAnAgent::did(&answer)
    });

    let written = told.written().unwrap();
    assert!(!written.contains('\n'), "{written}");
    let back = ToAnAgent::read(&written).unwrap();
    assert_eq!(back, told);

    let Some(Done::Listed { things, .. }) = back.done().cloned() else {
        unreachable!("a listing is what a listing answers with")
    };
    let mut named: Vec<&str> = things.iter().map(alo_protocol::Thing::name).collect();
    named.sort_unstable();
    assert_eq!(named, ["april.pdf", "march.pdf"]);
    assert!(things.iter().all(|thing| thing.kind() == Kind::File));
    assert!(!back.done().unwrap().left_something_out());
}

/// **A file's contents cross as they are, line breaks and all**, because a
/// message is one line of JSON rather than one line of text — and because
/// contents are contents, not a name inside a sentence.
#[test]
fn a_real_file_crosses_with_its_line_breaks_intact() {
    let strings = Strings::of(everything_this_machine_says());
    let told = on_a_machine("a-read", &strings, |turning, grants, _, invoice| {
        let answer = turning
            .reading(
                "read_file",
                &[(
                    "file",
                    alo_capability::Given::text(invoice.display().to_string()),
                )],
                grants,
                noon(),
            )
            .unwrap();
        ToAnAgent::did(&answer)
    });

    let written = told.written().unwrap();
    assert!(!written.contains('\n'), "a message is one line: {written}");
    let back = ToAnAgent::read(&written).unwrap();
    assert_eq!(
        back.done().unwrap(),
        &Done::Read {
            text: "March\n\t4180.00\n".to_owned()
        }
    );
}

/// **A change crosses as a number and the sentence it waits on**, to the agent
/// that proposed it and to the person who will answer it — and what the person
/// is drawn is the turn's own list rather than one assembled beside it.
#[test]
fn a_change_crosses_as_a_number_and_the_sentence_the_person_is_asked() {
    let strings = Strings::of(everything_this_machine_says());
    let (to_the_agent, to_the_person, done) =
        on_a_machine("a-change", &strings, |turning, grants, _, invoice| {
            let id = turning
                .proposing(
                    "rename_file",
                    &[
                        (
                            "file",
                            alo_capability::Given::text(invoice.display().to_string()),
                        ),
                        ("name", alo_capability::Given::text("march-final.pdf")),
                    ],
                    grants,
                    hour(),
                    noon(),
                )
                .unwrap();

            let waiting = turning.waiting_at(noon()).next().unwrap();
            let to_the_agent = ToAnAgent::proposed(waiting, &strings, noon());
            let to_the_person = ToAPerson::waiting(turning.waiting_at(noon()), &strings, noon());

            // The person answers by the number they were shown, off the wire.
            let answered = FromAPerson::read(&format!(
                r#"{{"format":1,"asks":{{"approve":{{"number":{}}}}}}}"#,
                id.as_u64()
            ))
            .unwrap();
            assert_eq!(answered.number(), Some(id.as_u64()));
            let did = turning.approving(id, grants, noon()).unwrap();
            (to_the_agent, to_the_person, ToAPerson::did(&did))
        });

    assert!(to_the_agent.waits_for_a_person());
    let changes = to_the_person.changes().unwrap();
    assert_eq!(changes.len(), 1);
    let only = changes.first().unwrap();
    assert!(
        only.sentence().text().contains("march-final.pdf"),
        "{only:?}"
    );
    assert_eq!(only.lapses_in(), Some(3600));

    // And all three cross and read back as themselves.
    assert_eq!(
        ToAnAgent::read(&to_the_agent.written().unwrap()).unwrap(),
        to_the_agent
    );
    assert_eq!(
        ToAPerson::read(&to_the_person.written().unwrap()).unwrap(),
        to_the_person
    );
    let renamed = ToAPerson::read(&done.written().unwrap()).unwrap();
    let Some(Done::Renamed { now_at }) = renamed.done().cloned() else {
        unreachable!("a rename answers with where the file now is")
    };
    assert!(
        now_at.unwrap().ends_with("march-final.pdf"),
        "the person is told where the file went"
    );
}

/// **A refusal crosses in the words of whoever made it, and says whether
/// anybody translated it.** The whole 9-series, at the last boundary before a
/// person reads the sentence: the daemon renders with the machine's vocabulary,
/// and what arrives is that sentence with its provenance rather than a string
/// nobody can account for.
#[test]
fn a_refusal_crosses_in_the_persons_own_language_and_says_so() {
    let vocabulary = everything_this_machine_says();
    let translation = Translation::into_language(Language::written("de").unwrap())
        .says(
            alo_capability::words::NEVER_GRANTED.key(),
            "{agent} hat keinen Zugriff auf {wanted} — Zugriff entsteht, indem man einen Ordner \
             auswählt, nie indem danach gefragt wird",
        )
        .says(
            alo_capability::words::A_FOLDER.key(),
            "{path} und alles darin",
        )
        .says(alo_capability::words::A_FILE.key(), "die Datei {path}");
    let speaking = vocabulary.check(translation).unwrap();
    let mut strings = Strings::of(everything_this_machine_says());
    strings.speaks(speaking).unwrap();
    strings.prefers(&[Language::written("de").unwrap()]);

    let told = on_a_machine("a-refusal", &strings, |turning, grants, _, _| {
        let refused = turning
            .reading(
                "list_folder",
                &[("folder", alo_capability::Given::text("/home/anna/Secrets"))],
                grants,
                noon(),
            )
            .unwrap_err();
        assert!(refused.was_refused());
        ToAnAgent::refused(&refused.said(&strings))
    });

    let back = ToAnAgent::read(&told.written().unwrap()).unwrap();
    let refusal = back.refusal().unwrap();
    assert!(
        refusal.text().starts_with("@files hat keinen"),
        "{refusal:?}"
    );
    assert!(refusal.is_translated(), "{refusal:?}");
    assert!(!refusal.is_a_bug());
}

/// **A daemon cannot put one side's answer on the other side's connection.**
/// The division of the doors, on the way back: what a person is told about
/// their own list has no shape an agent can read, and what an agent is told
/// about a model has none a person's shell can.
#[test]
fn an_answer_for_one_side_is_refused_by_the_other() {
    let strings = Strings::of(everything_this_machine_says());
    let (to_the_agent, to_the_person) =
        on_a_machine("two-doors", &strings, |turning, grants, _, invoice| {
            turning
                .proposing(
                    "rename_file",
                    &[
                        (
                            "file",
                            alo_capability::Given::text(invoice.display().to_string()),
                        ),
                        ("name", alo_capability::Given::text("march-final.pdf")),
                    ],
                    grants,
                    hour(),
                    noon(),
                )
                .unwrap();
            let waiting = turning.waiting_at(noon()).next().unwrap();
            (
                ToAnAgent::proposed(waiting, &strings, noon()),
                ToAPerson::waiting(turning.waiting_at(noon()), &strings, noon()),
            )
        });

    assert_eq!(
        ToAPerson::read(&to_the_agent.written().unwrap()),
        Err(NotUnderstood::NotAnAnswerForAPerson)
    );
    assert_eq!(
        ToAnAgent::read(&to_the_person.written().unwrap()),
        Err(NotUnderstood::NotAnAnswerForAnAgent)
    );
    for refused in [
        NotUnderstood::NotAnAnswerForAPerson,
        NotUnderstood::NotAnAnswerForAnAgent,
    ] {
        assert!(refused.is_about_who_it_was_for());
        assert!(!refused.said(&strings).is_a_bug());
    }
}

/// A search over a real folder crosses with its paths and its counts, and an
/// answer that found everything says it found everything.
#[test]
fn a_search_crosses_with_its_paths_and_its_counts() {
    let strings = Strings::of(everything_this_machine_says());
    let told = on_a_machine("a-search", &strings, |turning, grants, folder, _| {
        let answer = turning
            .reading(
                "find_in_folder",
                &[
                    (
                        "folder",
                        alo_capability::Given::text(folder.display().to_string()),
                    ),
                    ("named", alo_capability::Given::text(".pdf")),
                    ("most", alo_capability::Given::number(10)),
                ],
                grants,
                noon(),
            )
            .unwrap();
        ToAnAgent::did(&answer)
    });

    let back = ToAnAgent::read(&told.written().unwrap()).unwrap();
    let Some(Done::Found {
        files,
        could_not_be_named,
        cut_short,
    }) = back.done().cloned()
    else {
        unreachable!("a search answers with what it found")
    };
    assert_eq!(files.len(), 2);
    assert_eq!(could_not_be_named, 0);
    assert!(!cut_short);
    assert!(files.iter().all(|file| file.ends_with(".pdf")));
}

/// **An archive crosses with where it is and what it left out.** The one answer
/// whose three numbers are the whole of what a person needs to know about a
/// change they approved, and the only way to produce a real one is to make a
/// real archive.
#[test]
fn an_archive_crosses_with_its_path_and_its_counts() {
    let strings = Strings::of(everything_this_machine_says());
    let told = on_a_machine("an-archive", &strings, |turning, grants, folder, _| {
        let old = folder.join("old");
        fs::create_dir_all(&old).unwrap();
        fs::write(old.join("february.pdf"), "February, 90.00").unwrap();

        let id = turning
            .proposing(
                "archive_folder",
                &[
                    (
                        "folder",
                        alo_capability::Given::text(old.display().to_string()),
                    ),
                    (
                        "into",
                        alo_capability::Given::text(folder.display().to_string()),
                    ),
                    ("name", alo_capability::Given::text("old.zip")),
                ],
                grants,
                hour(),
                noon(),
            )
            .unwrap();
        let did = turning.approving(id, grants, noon()).unwrap();
        ToAPerson::did(&did)
    });

    let back = ToAPerson::read(&told.written().unwrap()).unwrap();
    let Some(Done::Archived {
        at,
        things,
        left_out,
        bytes,
    }) = back.done().cloned()
    else {
        unreachable!("an archive answers with the archive it made")
    };
    assert!(at.unwrap().ends_with("old.zip"));
    assert_eq!(things, 1);
    assert_eq!(left_out, 0);
    assert!(bytes > 0);
    assert!(!back.done().unwrap().left_something_out());
}
