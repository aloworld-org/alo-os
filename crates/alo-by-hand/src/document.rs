//! `docs/by-hand.md`, read as entries rather than admired as a document.
//!
//! The document explains itself in prose first and then answers about the verbs
//! under one heading. Only what is under that heading is read, so the prose can
//! name a verb as an example without that example becoming an answer — and so a
//! heading somebody moved shows up as *nothing to check* rather than as a check
//! that quietly holds nothing.
//!
//! That last failure is the one worth the module. A document this crate cannot
//! find is a green gate, and a green gate is exactly what somebody adding a verb
//! would take for permission.

use crate::entry::Entry;

/// The heading the entries live under.
pub const THE_ANSWERS: &str = "## Every verb, one at a time";

/// How an entry begins.
const AN_ENTRY: &str = "### ";

/// How a section that is not the entries begins.
const ANOTHER_SECTION: &str = "## ";

/// Every entry in the document, in the order it makes them.
///
/// An entry is a `###` heading under [`THE_ANSWERS`], and everything written
/// beneath it until the next heading is its body.
#[must_use]
pub fn entries_in(document: &str) -> Vec<Entry> {
    let mut entries: Vec<Entry> = Vec::new();
    let mut reading = false;
    let mut verb: Option<String> = None;
    let mut body = String::new();

    for line in document.lines() {
        if let Some(named) = line.strip_prefix(AN_ENTRY) {
            if reading {
                finish(&mut verb, &mut body, &mut entries);
                verb = Some(named.trim().to_owned());
            }
            continue;
        }
        if line.starts_with(ANOTHER_SECTION) {
            finish(&mut verb, &mut body, &mut entries);
            reading = line.trim_end() == THE_ANSWERS;
            continue;
        }
        if verb.is_some() {
            body.push_str(line);
            body.push('\n');
        }
    }
    finish(&mut verb, &mut body, &mut entries);
    entries
}

/// The entry being read, put away.
///
/// Nothing is put away where no heading has been seen, which is what keeps the
/// prose in front of the first entry out of the document's answers.
fn finish(verb: &mut Option<String>, body: &mut String, into: &mut Vec<Entry>) {
    if let Some(named) = verb.take() {
        into.push(Entry::under(&named, body));
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

    /// A document with prose in front of it, two entries and a section after
    /// them — which is exactly the shape this crate reads.
    fn a_document() -> String {
        format!(
            "\
# Every verb, by hand

Prose that names `list_folder` as an example of what an entry looks like.

{THE_ANSWERS}

### list_folder

**By hand:** a person opens the folder in the file manager and reads what is in
it — `A file manager, with trash, and archives that open`.

### archive_folder

**Owed at:** [v0.5] the definition promises archives that open and not archives
a person makes.

## What this document does not do

It does not decide whether `A file manager, with trash, and archives that open`
is a good answer.
"
        )
    }

    /// **The entries are the ones under the heading**, and the prose around them
    /// is prose.
    #[test]
    fn the_entries_are_the_ones_under_the_heading() {
        let entries = entries_in(&a_document());
        assert_eq!(
            entries.iter().map(Entry::verb).collect::<Vec<_>>(),
            ["list_folder", "archive_folder"],
            "the document was read as more or fewer entries than it has, which \
             would answer about a verb nobody wrote about"
        );
        assert!(entries.first().unwrap().by_hand().is_some());
        assert!(
            entries.last().unwrap().by_hand().is_none(),
            "the section written after the entries was read into the last one"
        );
        assert_eq!(
            entries.last().unwrap().owed().unwrap().release(),
            Some("v0.5")
        );
    }

    /// And a heading that moved leaves nothing to check, rather than a document
    /// that silently holds nothing.
    #[test]
    fn a_heading_that_moved_leaves_nothing() {
        let renamed = a_document().replace(THE_ANSWERS, "## The verbs");
        assert!(
            entries_in(&renamed).is_empty(),
            "entries were found under a heading this crate does not know, so a \
             renamed section would pass while checking nothing"
        );
    }
}
