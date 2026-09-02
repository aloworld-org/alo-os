//! What this crate's own tests are written against.
//!
//! Two things: the vocabulary every sentence here is rendered with, and an
//! afternoon of entries to write down.
//!
//! **The entries are the cheapest ones `alo-record` makes.** Nothing in this
//! crate looks inside an entry — it writes one down, reads one back, and asks
//! it when it happened — so the fixture uses the two constructors that need
//! nothing but an agent and a moment. The whole journey with grants, approvals
//! and refusals in it is `alo-record`'s own fixture and is tested there.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::Grantee;
use alo_record::Entry;
use alo_strings::{Said, Strings};

use crate::failing::NotKept;
use crate::words::keeping_words;

/// Everything this crate can say, with nothing translated.
pub(crate) fn in_english() -> Strings {
    Strings::of(keeping_words().unwrap())
}

/// What a sentence reads as, for a test that is about the words in it.
pub(crate) fn said(said: &Said) -> String {
    said.text().to_owned()
}

/// A fixed moment, so that ageing out is arithmetic rather than a wait.
pub(crate) fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// A day, as this crate counts one.
pub(crate) fn day() -> Duration {
    Duration::from_secs(24 * 60 * 60)
}

/// The agent the entries in these tests were made under.
fn mail() -> Grantee {
    Grantee::named("@mail")
}

/// Something that never became a call, which is the cheapest entry there is.
pub(crate) fn turned_away(verb: &str, why: &str) -> Entry {
    Entry::turned_away(verb, why, &mail(), noon())
}

/// The same, at a moment of the test's choosing.
pub(crate) fn turned_away_at(at: SystemTime) -> Entry {
    Entry::turned_away("delete_everything", "there is no such verb", &mail(), at)
}

/// An afternoon: four things that happened, in the order they happened.
pub(crate) fn an_afternoon() -> Vec<Entry> {
    vec![
        Entry::answered_here(&mail(), noon()),
        turned_away(
            "delete_everything",
            "there is no verb called delete_everything",
        ),
        Entry::answered_here(&mail(), noon() + Duration::from_secs(60)),
        turned_away("run", "there is no verb called run"),
    ]
}

/// One entry as this crate writes it, for a test that builds a file by hand.
pub(crate) fn a_line(entry: &Entry) -> String {
    crate::writing::a_line(entry).unwrap()
}

/// A path no test writes to, for the reading that is done from a string.
pub(crate) fn nowhere() -> &'static Path {
    Path::new("/var/lib/alo/record.jsonl")
}

/// One example of every way this crate can fail, so that a variant added
/// without a word for it is caught by the tests that walk this.
pub(crate) fn every_way_it_can_fail() -> Vec<NotKept> {
    let path = nowhere().display().to_string();
    vec![
        NotKept::NotThere { path: path.clone() },
        NotKept::NotARecord { path: path.clone() },
        NotKept::FromANewerAlo {
            path: path.clone(),
            format: 9,
        },
        NotKept::Damaged { path: path.clone() },
        NotKept::NotOpened {
            path: path.clone(),
            why: "permission denied".to_owned(),
        },
        NotKept::NotAddedTo {
            path: path.clone(),
            why: "no space left on device".to_owned(),
        },
        NotKept::NotRead {
            path: path.clone(),
            why: "input/output error".to_owned(),
        },
        NotKept::NotShortened {
            path,
            why: "read-only file system".to_owned(),
        },
    ]
}

/// A folder of this test's own, on the disk the tests are running on.
///
/// `alo-files` has the same fixture and for the same reason: a test about a
/// file has to be about a real one, and two of them must not meet.
pub(crate) fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-keeping-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    drop(fs::remove_dir_all(&folder));
    fs::create_dir_all(&folder).unwrap();
    folder
}
