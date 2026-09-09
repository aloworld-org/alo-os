//! What a record does with an entry whose kind this version does not know.
//!
//! `docs/contracts/record-file.md` settles the policy: **a new kind of
//! `happened` is additive and does not raise `format`**, because raising it
//! would make the whole file unreadable to an older reader rather than one line
//! of it. What that reader does instead is *report that line as one it could not
//! read, with its line number, alongside everything it could* — and two things
//! are said to make that safe: an older writer goes on appending beside entries
//! it cannot read and loses nothing, and an older shortening refuses, so the
//! version that does not understand an entry is also the version that will not
//! remove it.
//!
//! This file holds that up. `alo-record` has just gained
//! `Happened::NeverPutAnywhere`, which is exactly such an addition.
//!
//! # What this is evidence of, stated exactly
//!
//! The unknown entry here is a **synthetic tag** — `from-a-later-alo-os`, a kind
//! no version of this repository writes. That makes it a test of the
//! *mechanism*: what this reader does with any tag it does not know, which is
//! what an older binary meeting a newer one's entry would face.
//!
//! **It is not an execution of an older binary.** No older `alo-record` is
//! built, run or linked here, and nothing in this file can show what a
//! particular past version did. What it shows is that the behaviour the contract
//! promises is the behaviour this code has — which is the half that can be
//! checked from inside one checkout, and the half that would break first if
//! somebody made the reader skip what it could not parse.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::Grantee;
use alo_keeping::{Keeping, NotKept, Reading, THE_FORMAT, Writing};
use alo_record::{Entry, Line};

/// A fixed moment, so everything about time here is arithmetic.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// A day, as a retention rule counts one.
fn day() -> Duration {
    Duration::from_secs(24 * 60 * 60)
}

/// A folder of this test's own.
fn a_folder(called: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let at = std::env::temp_dir().join(format!(
        "alo-unknown-tag-{}-{called}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    drop(fs::remove_dir_all(&at));
    fs::create_dir_all(&at).unwrap();
    at
}

/// An entry every version of this repository can read.
fn a_known_entry(at: SystemTime) -> Entry {
    Entry::turned_away(
        "open-file",
        "there is no such verb",
        &Grantee::named("@files"),
        at,
    )
}

/// The tag this version writes for a refused question.
const A_KIND_WE_KNOW: &str = "never-put-anywhere";

/// A tag no version of this repository writes.
const A_KIND_FROM_LATER: &str = "from-a-later-alo-os";

/// A real entry's line with **only its kind changed** to one this version does
/// not know.
///
/// Built by serialising an entry through the ordinary writer and swapping the
/// tag in the text, rather than by typing a line out here. That matters: a
/// hand-written line could fail to parse because its *shape* was wrong — a
/// mis-spelled time field, a missing member — and the test would then be about
/// malformed JSON rather than about an unrecognised kind, while passing exactly
/// the same way. Everything but the tag is what this version really writes.
fn with_its_kind_made_unknown(line: &str) -> String {
    assert!(
        line.contains(A_KIND_WE_KNOW),
        "the entry this fixture rewrites is not the kind it expects: {line}"
    );
    line.replace(A_KIND_WE_KNOW, A_KIND_FROM_LATER)
}

/// A record whose middle line is a kind this version does not know.
///
/// Built through `Writing` so the head and the known entries are exactly what
/// this version writes, with the unknown line spliced between them — which is
/// what a file written by a newer version and read by an older one looks like.
fn a_record_with_an_unknown_entry(called: &str) -> (PathBuf, PathBuf) {
    let folder = a_folder(called);
    let path = folder.join("record.jsonl");

    // Three real entries, the middle one of a kind whose tag is then made
    // unknown — so line three has exactly the shape this version writes and
    // differs from a readable entry in the tag alone.
    let mut writing = Writing::opening(&path).unwrap();
    writing.keep(&a_known_entry(noon())).unwrap();
    writing
        .keep(&Entry::never_put_anywhere(
            &Grantee::named("@mail"),
            "something this version has no word for",
            noon() + day(),
        ))
        .unwrap();
    writing.keep(&a_known_entry(noon() + day() * 2)).unwrap();
    drop(writing);

    // Line 1 is the head, line 2 the first entry, line 3 the one to disguise,
    // line 4 the last — so there is a known entry on each side of it.
    let written = fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = written.lines().collect();
    assert_eq!(
        lines.len(),
        4,
        "expected a head and three entries: {written}"
    );
    let disguised: Vec<String> = lines
        .iter()
        .enumerate()
        .map(|(at, line)| {
            if at == 2 {
                with_its_kind_made_unknown(line)
            } else {
                (*line).to_owned()
            }
        })
        .collect();
    fs::write(&path, format!("{}\n", disguised.join("\n"))).unwrap();

    (folder, path)
}

/// **Everything this version knows is still read, on both sides of the entry it
/// does not.**
///
/// The failure this guards against is a reader that stops at the first line it
/// cannot parse, which would let one unknown entry hide every entry after it.
#[test]
fn entries_before_and_after_an_unknown_one_are_still_read() {
    let (folder, path) = a_record_with_an_unknown_entry("both-sides");

    let reading = Reading::at(&path).unwrap();

    assert_eq!(
        reading.record().len(),
        2,
        "the known entries on both sides of the unknown one were not both read"
    );
    assert_eq!(
        reading.head().format(),
        THE_FORMAT,
        "the format was raised, which the contract says an additive kind does not do"
    );

    drop(fs::remove_dir_all(&folder));
}

/// **The unknown entry is reported, with its line number.**
///
/// Not skipped: `alo-keeping`'s damage exists so a line nobody could read leaves
/// a mark, because a reader that quietly stepped over one could be made to lose
/// an entry by corrupting it.
#[test]
fn the_unknown_entry_is_reported_at_its_line() {
    let (folder, path) = a_record_with_an_unknown_entry("reported");

    let reading = Reading::at(&path).unwrap();
    let damage = reading.damage();

    assert_eq!(
        damage.unreadable(),
        &[3],
        "the unknown entry is the third line and should be reported as that one"
    );
    assert!(
        !damage.last_line_is_unfinished(),
        "a whole line in the middle is not a write that was interrupted"
    );

    drop(fs::remove_dir_all(&folder));
}

/// **Appending keeps every byte that was there, the unknown entry included.**
///
/// The contract's *an older writer is never endangered*: the file is appended to
/// and never rewritten, so a version that cannot read an entry still cannot lose
/// it. Asserted on the bytes rather than on a count of entries.
#[test]
fn appending_preserves_every_byte_including_the_unknown_entry() {
    let (folder, path) = a_record_with_an_unknown_entry("appending");
    let before = fs::read_to_string(&path).unwrap();

    let mut writing = Writing::opening(&path).unwrap();
    writing.keep(&a_known_entry(noon() + day() * 2)).unwrap();
    drop(writing);

    let after = fs::read_to_string(&path).unwrap();
    assert!(
        after.starts_with(&before),
        "appending changed bytes that were already there"
    );
    assert!(
        after.contains(A_KIND_FROM_LATER),
        "the entry this version cannot read did not survive an append"
    );
    assert!(
        after.len() > before.len(),
        "nothing was appended, so this proves nothing"
    );

    drop(fs::remove_dir_all(&folder));
}

/// **Shortening refuses, and leaves the file exactly as it was.**
///
/// The contract's *an older shortening refuses*: the version that does not
/// understand an entry is the version that will not remove it. Rewriting a file
/// with an unreadable line would tidy away the one thing anybody could have
/// looked at.
///
/// The retention rule here would otherwise remove every entry in the file, so a
/// shortening that went ahead would visibly change it — which is what makes the
/// byte comparison mean something.
#[test]
fn shortening_refuses_and_changes_nothing() {
    let (folder, path) = a_record_with_an_unknown_entry("shortening");
    let before = fs::read_to_string(&path).unwrap();

    let much_later = noon() + day() * 400;
    let mut writing = Writing::opening(&path).unwrap();
    let refused = writing.prune(Keeping::for_days(1).unwrap(), much_later);
    drop(writing);

    assert!(
        matches!(refused, Err(NotKept::Damaged { .. })),
        "a record with an entry this version cannot read was shortened anyway: {refused:?}"
    );
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        before,
        "the file changed while shortening was refusing"
    );

    drop(fs::remove_dir_all(&folder));
}

/// **A refused question round-trips, with the agent and the moment it was given.**
///
/// `Happened::NeverPutAnywhere` is the addition this file's policy is about, and
/// this is it written and read back: the moment comes from the caller — the
/// turn, once there is one — and the agent from the `Grantee`, so neither is
/// invented by the record.
///
/// **Nothing emits one in production.** The constructor is called by this test
/// and by nothing else until the daemon integration lands.
#[test]
fn a_refused_question_round_trips_with_its_agent_and_moment() {
    let folder = a_folder("round-trip");
    let path = folder.join("record.jsonl");
    let asked_at = noon() + day();

    let mut writing = Writing::opening(&path).unwrap();
    writing
        .keep(&Entry::never_put_anywhere(
            &Grantee::named("@mail"),
            "your organisation's rule refused that",
            asked_at,
        ))
        .unwrap();
    drop(writing);

    let reading = Reading::at(&path).unwrap();
    assert!(
        reading.damage().unreadable().is_empty(),
        "this version could not read back an entry it had just written"
    );

    let record = reading.into_record();
    let entry = record.everything().next().unwrap();
    assert_eq!(entry.at(), asked_at, "the moment was not the one given");
    assert_eq!(
        entry.agent().map(Line::as_str),
        Some("@mail"),
        "the agent was not the one given"
    );
    assert_eq!(
        entry.happened().why_stopped().map(Line::as_str),
        Some("your organisation's rule refused that"),
        "the refusal did not come back as it was written"
    );
    assert!(
        entry.what().is_none(),
        "a refused question came back looking like a call"
    );

    drop(fs::remove_dir_all(&folder));
}

/// **The bytes on the disk carry the refusal and nothing else of the person's.**
///
/// Read off the file rather than off the entry's fields: a field named `why`
/// says nothing about what somebody put in it, and this is the check that the
/// stored line has no question, no credential and no address in it.
#[test]
fn the_stored_bytes_hold_no_question_no_key_and_no_endpoint() {
    let folder = a_folder("stored-bytes");
    let path = folder.join("record.jsonl");

    // What a refusal is worded from: our own sentence, naming the source the
    // person configured. The three beside it are what must not be there.
    const THE_REFUSAL: &str = "a question was not put anywhere, because a rule refused Mistral";
    const THE_QUESTION: &str = "may the tenant sublet the flat above the bakery";
    const A_KEY: &str = "sk-live-RECORD-FIXTURE-ONLY-77d1";
    const AN_ENDPOINT: &str = "https://api.mistral.ai";

    let mut writing = Writing::opening(&path).unwrap();
    writing
        .keep(&Entry::never_put_anywhere(
            &Grantee::named("@mail"),
            THE_REFUSAL,
            noon(),
        ))
        .unwrap();
    drop(writing);

    let bytes = fs::read(&path).unwrap();
    let written = String::from_utf8_lossy(&bytes);

    assert!(
        written.contains("a rule refused Mistral"),
        "the refusal itself is not on the disk, so this is not looking at the right file"
    );
    for (what, needle) in [
        ("the question", THE_QUESTION),
        ("a credential", A_KEY),
        ("an endpoint", AN_ENDPOINT),
    ] {
        assert!(
            !written.contains(needle),
            "{what} is in the bytes this entry wrote to the disk"
        );
    }

    drop(fs::remove_dir_all(&folder));
}
