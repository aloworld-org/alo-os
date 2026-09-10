//! The ledger, read as entries rather than admired as a document.
//!
//! `docs/autonomy/v0-01-evidence.md` explains itself in prose first and then
//! lists the promises under one heading. Only what is under that heading is
//! read, so the prose can name a test file as an example without that example
//! becoming an entry — and so a heading somebody moved shows up as *nothing to
//! reconcile* rather than as a ledger that quietly checks nothing.

use crate::entry::Entry;

/// The heading the entries live under.
pub const THE_ENTRIES: &str = "## Every v0.01 promise, one at a time";

/// How an entry begins.
const AN_ENTRY: &str = "### ";

/// How a section that is not the entries begins.
const ANOTHER_SECTION: &str = "## ";

/// Every entry in a ledger document, in the order it makes them.
///
/// An entry is a `###` heading under [`THE_ENTRIES`], and everything written
/// beneath it until the next heading is its body.
#[must_use]
pub fn entries_in(document: &str) -> Vec<Entry> {
    let mut entries: Vec<Entry> = Vec::new();
    let mut reading = false;
    let mut promise: Option<String> = None;
    let mut body = String::new();

    for line in document.lines() {
        if let Some(quoted) = line.strip_prefix(AN_ENTRY) {
            if reading {
                finish(&mut promise, &mut body, &mut entries);
                promise = Some(quoted.trim().to_owned());
            }
            continue;
        }
        if line.starts_with(ANOTHER_SECTION) {
            finish(&mut promise, &mut body, &mut entries);
            reading = line.trim_end() == THE_ENTRIES;
            continue;
        }
        if promise.is_some() {
            body.push_str(line);
            body.push('\n');
        }
    }
    finish(&mut promise, &mut body, &mut entries);
    entries
}

/// The entry being read, put away.
///
/// Nothing is put away where no heading has been seen, which is what keeps the
/// prose in front of the first entry out of the ledger.
fn finish(promise: &mut Option<String>, body: &mut String, into: &mut Vec<Entry>) {
    if let Some(quoted) = promise.take() {
        into.push(Entry::under(&quoted, body));
    }
    body.clear();
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A ledger with prose in front of it, two entries, and a section after
    /// them — which is exactly the document this crate reads.
    fn a_ledger() -> String {
        format!(
            "\
# v0.01, against the evidence

Prose that names `crates/alo-files/tests/doing.rs` as an example of what an
entry looks like.

{THE_ENTRIES}

### File verbs: list, read, find, rename, move, archive

**Shown by:** `crates/alo-files/tests/doing.rs`

**Still owed:** the verbs have never run on a certified machine, which is the
half only hardware gives.

### No telemetry

**Shown by:** `crates/alo-egress/src/errand.rs`

## What this ledger does not do

It does not judge whether `crates/alo-files/tests/doing.rs` proves anything.
"
        )
    }

    /// **The entries are the ones under the heading**, and the prose around them
    /// is prose.
    #[test]
    fn the_entries_are_the_ones_under_the_heading() {
        let entries = entries_in(&a_ledger());
        assert_eq!(
            entries
                .iter()
                .map(Entry::promise)
                .collect::<Vec<_>>()
                .as_slice(),
            [
                "File verbs: list, read, find, rename, move, archive",
                "No telemetry"
            ],
            "the ledger was read as more or fewer entries than it has, which \
             would reconcile a promise nobody wrote about"
        );
        assert_eq!(
            entries.first().unwrap().names(),
            ["crates/alo-files/tests/doing.rs".to_owned()]
        );
        assert!(entries.first().unwrap().owed().is_some());
        assert!(
            entries.last().unwrap().owed().is_none(),
            "the section written after the entries was read into the last one"
        );
    }

    /// And a heading that moved leaves nothing to reconcile, rather than a
    /// ledger that silently checks nothing.
    #[test]
    fn a_heading_that_moved_leaves_nothing() {
        let renamed = a_ledger().replace(THE_ENTRIES, "## The promises");
        assert!(
            entries_in(&renamed).is_empty(),
            "entries were found under a heading this crate does not know, so a \
             renamed section would pass while checking nothing"
        );
    }
}
