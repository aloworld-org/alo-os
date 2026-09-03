//! A whole turn on a real filesystem, with the record on a real disk.
//!
//! The crate's own tests take each door apart against a record held in memory.
//! This is the other half of that bargain: one turn, in the order `alo-agentd`
//! will run it in, with files that really move and a record file that is
//! written, closed, and read back by the crate that reads records — because a
//! record nothing can read afterwards answers no question anybody will ask of
//! it.
//!
//! **What it is really asking** is the sentence in `CLAUDE.md`'s gate that no
//! single crate can demonstrate: *every execution and every refusal leaves a
//! record*. Here a turn does one of each, and the file on the disk is what
//! says so.
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

use alo_capability::{Given, Grant, Grantee, Grants, Reach};
use alo_context::{Context, Document, Focused, Selection};
use alo_files::{OnThisMachine, Resolving, file_words};
use alo_keeping::{Reading, Writing};
use alo_record::{Asking, Only, Record};
use alo_strings::Strings;
use alo_turn::{Machine, Turning};

/// The moment the person pressed the key.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the turn, the grants and the question stand.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The paragraph the person had selected, which is theirs and goes nowhere
/// near the record.
const SELECTED: &str = "Northstar have not paid the March invoice, account 41-8823.";

/// A folder of this test's own, resolved — a grant is over a place, and on
/// Windows the resolved spelling is the one a grant has to be made with.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-turn-whole-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    OnThisMachine.real(&folder).unwrap().into_path_buf()
}

/// The words this machine reads: every crate's, in one vocabulary, which is the
/// arrangement a shell is really in.
fn on_this_machine() -> Strings {
    let mut vocabulary = file_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_keeping::declare_into(&mut vocabulary).unwrap();
    alo_turn::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// Grants to `@files` over these folders, made at noon and lasting an hour.
fn granting(folders: &[&Path]) -> Grants {
    let mut grants = Grants::default();
    for folder in folders {
        grants.grant(
            Grant::checked(
                "@files",
                Reach::Folder((*folder).to_path_buf()),
                noon(),
                hour(),
            )
            .unwrap(),
        );
    }
    grants
}

/// A path as a verb's argument arrives: text.
fn as_given(path: &Path) -> Given {
    Given::text(path.to_string_lossy().into_owned())
}

/// **The whole thing.** Anna presses the key with an invoice open, a paragraph
/// selected and Blender in front of her. The agent lists the folder, is refused
/// a folder nobody granted, proposes moving the invoice into the archive, and
/// she approves it. The turn ends, the record is closed, and every question ADR
/// 0001 §7 asks is answered from the file that is left on the disk.
#[test]
fn one_turn_moves_a_file_and_the_record_on_the_disk_says_what_happened() {
    let strings = on_this_machine();
    let invoices = a_folder_of_our_own("invoices");
    let archive = a_folder_of_our_own("archive");
    let nobodys = a_folder_of_our_own("nobodys");
    let march = invoices.join("march.pdf");
    fs::write(&march, "March, 4180.00").unwrap();

    let kept_at = a_folder_of_our_own("record").join("record.jsonl");
    let mut grants = granting(&[&invoices, &archive]);
    let approval;
    {
        let mut writing = Writing::opening(&kept_at).unwrap();
        let mut machine =
            Machine::carrying_out_file_verbs(&strings, &OnThisMachine, &mut writing).unwrap();

        let context = Context::at_invocation(noon())
            .and_document(Document::open(&march).unwrap())
            .and_selection(Selection::of(SELECTED).unwrap())
            .and_window(Focused::titled("org.blender.Blender", "untitled.blend").unwrap());
        let mut turning =
            Turning::beginning(context, "@files", hour(), &mut grants, &mut machine).unwrap();

        // A read, inside the turn, with nobody asked about anything.
        let listing = turning
            .reading(
                "list_folder",
                &[("folder", as_given(&invoices))],
                &grants,
                noon(),
            )
            .unwrap();
        assert_eq!(listing.listed().unwrap().things().len(), 1);

        // A folder nobody granted, refused before the disk is touched.
        let refused = turning
            .reading(
                "list_folder",
                &[("folder", as_given(&nobodys))],
                &grants,
                noon(),
            )
            .unwrap_err();
        assert!(refused.was_refused(), "{refused:?}");

        // A change, put to the person in one sentence, and approved.
        let id = turning
            .proposing(
                "move_file",
                &[("file", as_given(&march)), ("into", as_given(&archive))],
                &grants,
                hour(),
                noon(),
            )
            .unwrap();
        let waiting = turning.waiting_at(noon()).next().unwrap();
        assert_eq!(
            waiting.proposal.sentence(&strings).text(),
            format!("move {} into {}", march.display(), archive.display())
        );
        approval = id.as_u64();

        let moved = turning.approving(id, &grants, noon()).unwrap();
        assert_eq!(moved.now_at(), Some(archive.join("march.pdf").as_path()));

        // And the turn ends, taking the grant the invocation made with it.
        assert!(
            turning.ending(&mut grants),
            "the document grant outlived it"
        );
    }

    // The file really moved.
    assert!(archive.join("march.pdf").is_file());
    assert!(!march.exists());

    // And the record on the disk says so, read back by the crate that reads
    // records rather than by this test parsing lines of its own.
    let read = Reading::at(&kept_at).unwrap();
    assert!(read.damage().nothing_wrong(), "{:?}", read.damage());
    let record = read.record();
    assert_eq!(record.len(), 3, "one read, one refusal, one change");

    let executions = Asking::anything().only(Only::Executions);
    let ran: Vec<_> = record.answering(&executions).collect();
    assert_eq!(ran.len(), 2);
    let change = ran
        .iter()
        .find(|entry| entry.what().is_some_and(|what| what.verb().is("move_file")))
        .unwrap();
    assert!(change.agent().is_some_and(|agent| agent.is("@files")));
    assert_eq!(change.happened().from_approval(), Some(approval));
    assert_eq!(
        change.what().unwrap().sentence().as_str(),
        format!("move {} into {}", march.display(), archive.display())
    );
    assert!(
        !change.happened().against().is_empty(),
        "nothing says which grant permitted it"
    );

    // The refusal is there too, and it is the one the person read.
    assert_eq!(
        record
            .answering(&Asking::anything().only(Only::Refusals))
            .count(),
        1
    );

    // And what was on the screen is not what the record kept.
    let everything = fs::read_to_string(&kept_at).unwrap();
    for private in ["Northstar", "41-8823", "untitled.blend", "org.blender"] {
        assert!(
            !everything.contains(private),
            "{private} reached the record file"
        );
    }
}

/// **Nothing is handed back before it is on the disk**, asserted against the
/// disk rather than against the order the code is written in.
///
/// `alo_keeping::Writing::keep` syncs before it answers, so a line that is
/// there when a door answers was there before the caller was told anything. The
/// entry counted here after each door is what a security review would find if
/// the machine lost power in the same instant.
///
/// **What this cannot ask** is the other half — a disk that refuses — because
/// no portable way to make one refuse exists: on Windows a record file whose
/// folder has been removed goes on accepting writes through the open handle.
/// `docs/quirks.md` records it, and the crate's own
/// `a_turn_that_could_not_write_something_down_does_nothing_else` covers the
/// closing against a record that refuses everything.
#[test]
fn nothing_is_handed_back_before_it_is_on_the_disk() {
    let strings = on_this_machine();
    let invoices = a_folder_of_our_own("in-order-invoices");
    let march = invoices.join("march.pdf");
    fs::write(&march, "March, 4180.00").unwrap();
    let nobodys = a_folder_of_our_own("in-order-nobodys");
    let kept_at = a_folder_of_our_own("in-order-record").join("record.jsonl");

    /// How many things the record on the disk says have happened.
    fn on_the_disk(path: &Path) -> usize {
        fs::read_to_string(path)
            .unwrap()
            .lines()
            .skip(1) // the first line says where the record starts.
            .filter(|line| !line.trim().is_empty())
            .count()
    }

    let mut grants = granting(&[&invoices]);
    let mut writing = Writing::opening(&kept_at).unwrap();
    let mut machine =
        Machine::carrying_out_file_verbs(&strings, &OnThisMachine, &mut writing).unwrap();
    let mut turning = Turning::beginning(
        Context::at_invocation(noon()),
        "@files",
        hour(),
        &mut grants,
        &mut machine,
    )
    .unwrap();
    assert_eq!(on_the_disk(&kept_at), 0, "a turn wrote something to begin");

    // A read that answers.
    turning
        .reading(
            "list_folder",
            &[("folder", as_given(&invoices))],
            &grants,
            noon(),
        )
        .unwrap();
    assert_eq!(on_the_disk(&kept_at), 1);

    // A refusal, which is written down as carefully.
    turning
        .reading(
            "list_folder",
            &[("folder", as_given(&nobodys))],
            &grants,
            noon(),
        )
        .unwrap_err();
    assert_eq!(on_the_disk(&kept_at), 2);

    // A question, which is not a thing that happened.
    let id = turning
        .proposing(
            "rename_file",
            &[
                ("file", as_given(&march)),
                ("name", Given::text("march-final.pdf")),
            ],
            &grants,
            hour(),
            noon(),
        )
        .unwrap();
    assert_eq!(
        on_the_disk(&kept_at),
        2,
        "a question nobody has answered was written down as something that happened"
    );

    // And the answer to it, which is.
    turning.declining(id, noon()).unwrap();
    assert_eq!(on_the_disk(&kept_at), 3);
    assert!(!turning.is_closed());
}

/// **A machine with somewhere to keep its record in memory writes the same
/// entries**, which is what makes the crate's own tests worth anything: the
/// entry a turn makes does not depend on where it goes.
#[test]
fn what_a_turn_writes_does_not_depend_on_where_it_is_written() {
    let strings = on_this_machine();
    let invoices = a_folder_of_our_own("same-entries");
    fs::write(invoices.join("march.pdf"), "March, 4180.00").unwrap();
    let kept_at = a_folder_of_our_own("same-entries-record").join("record.jsonl");

    let mut in_memory = Record::default();
    let mut on_a_disk = Writing::opening(&kept_at).unwrap();
    for kept in [
        &mut in_memory as &mut dyn alo_turn::Kept,
        &mut on_a_disk as &mut dyn alo_turn::Kept,
    ] {
        let mut grants = granting(&[&invoices]);
        let mut machine = Machine::carrying_out_file_verbs(&strings, &OnThisMachine, kept).unwrap();
        let mut turning = Turning::beginning(
            Context::at_invocation(noon()),
            "@files",
            hour(),
            &mut grants,
            &mut machine,
        )
        .unwrap();
        turning
            .reading(
                "list_folder",
                &[("folder", as_given(&invoices))],
                &grants,
                noon(),
            )
            .unwrap();
        assert!(!turning.ending(&mut grants));
    }
    drop(on_a_disk);

    let read = Reading::at(&kept_at).unwrap();
    assert_eq!(read.record().len(), in_memory.len());
    assert_eq!(
        read.record().everything().next(),
        in_memory.everything().next(),
        "the same turn wrote two different entries"
    );
}

/// A machine with no agent has no grants to begin a turn with, so there is no
/// turn to record anything about (ADR 0009).
#[test]
fn a_machine_where_somebody_declined_an_agent_has_no_turn_to_begin() {
    let mut declined = alo_capability::Agent::declined();
    assert!(declined.grants_mut().is_none());
    assert!(declined.grants().is_none());
    assert!(!declined.has_an_agent());
    assert!(!declined.permits(
        &Grantee::named("@files"),
        &alo_capability::Ask::path("/home/anna/Invoices"),
        noon()
    ));
}
