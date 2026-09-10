//! One entry of the ledger: a promise quoted, what shows it, and what is owed.
//!
//! An entry is written as a heading and two marked lines, rather than as a row
//! of a table, for one reason: the promises in `docs/features.md` are whole
//! paragraphs and the sentences about them are longer. A forty-one row table
//! nobody can read is a ledger nobody reads, and the reader is the half of this
//! audit no test replaces.

use crate::owed::Owed;

/// How an entry says what shows its promise.
const SHOWN_BY: &str = "**Shown by:**";

/// How an entry says what is still owed on it.
const STILL_OWED: &str = "**Still owed:**";

/// One promise of the release, and this repository's answer about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The phrase quoted out of `docs/features.md`, which is how the entry says
    /// which promise it is about.
    promise: String,

    /// Every file named as showing it, verbatim, before anything decides
    /// whether a name is evidence.
    ///
    /// Kept raw so that a path which is neither a test nor a report is a finding
    /// with the name in it, rather than a name silently dropped on the way in.
    names: Vec<String>,

    /// What is still owed, where the entry says anything.
    owed: Option<Owed>,
}

impl Entry {
    /// An entry, from the phrase it quotes and the body written under it.
    ///
    /// The body is read for the two markers. Neither is required here: an entry
    /// with neither is exactly the finding this audit exists to produce, and
    /// refusing to parse it would hide the promise it is about.
    #[must_use]
    pub fn under(promise: &str, body: &str) -> Self {
        let (shown, owed) = match body.split_once(STILL_OWED) {
            Some((before, after)) => (before, Some(after)),
            None => (body, None),
        };
        let names = match shown.split_once(SHOWN_BY) {
            Some((_, named)) => backticked_in(named),
            None => Vec::new(),
        };
        Self {
            promise: promise.trim().to_owned(),
            names,
            owed: owed.and_then(Owed::said),
        }
    }

    /// The phrase this entry quotes out of the definition.
    #[must_use]
    pub fn promise(&self) -> &str {
        &self.promise
    }

    /// Every file it names as showing that promise, in the order it names them.
    #[must_use]
    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// What it says is still owed, if it says anything.
    #[must_use]
    pub fn owed(&self) -> Option<&Owed> {
        self.owed.as_ref()
    }

    /// Whether it answers about its promise at all.
    ///
    /// Naming nothing and owing nothing is the promise with neither, which is
    /// the finding.
    #[must_use]
    pub fn says_something(&self) -> bool {
        !self.names.is_empty() || self.owed.is_some()
    }
}

/// Everything written in backticks in a piece of text, in order.
///
/// The odd-numbered pieces of a split on the backtick are what was between a
/// pair of them; an unclosed backtick therefore contributes its tail and is
/// caught downstream as a path that is not evidence, which names it.
fn backticked_in(text: &str) -> Vec<String> {
    text.split('`')
        .skip(1)
        .step_by(2)
        .map(|named| named.trim().to_owned())
        .filter(|named| !named.is_empty())
        .collect()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The shape every entry in the ledger is written in, including the line
    /// break inside a list of files that markdown wraps.
    const A_WHOLE_ENTRY: &str = "\
**Shown by:** `crates/alo-picking/tests/a_person_picks_a_folder.rs`,
`docs/autonomy/updates/native-folder-selection.md`

**Still owed:** nothing lists what has been granted where a person could find
it, which is the surface half of the promise.
";

    /// **Both halves are read**, and the files are the ones in backticks.
    #[test]
    fn an_entry_carries_what_shows_it_and_what_is_owed() {
        let entry = Entry::under("Grants: pick a folder", A_WHOLE_ENTRY);
        assert_eq!(entry.promise(), "Grants: pick a folder");
        assert_eq!(
            entry.names(),
            [
                "crates/alo-picking/tests/a_person_picks_a_folder.rs".to_owned(),
                "docs/autonomy/updates/native-folder-selection.md".to_owned(),
            ],
            "a list of files that wrapped onto a second line lost one"
        );
        assert!(entry.owed().unwrap().is_an_answer());
        assert!(entry.says_something());
    }

    /// A promise wholly owed says so and names nothing, and a promise wholly
    /// shown names files and owes nothing. Both are ordinary.
    #[test]
    fn an_entry_may_be_all_of_one_half() {
        let owed = Entry::under(
            "Copy, cut and paste",
            "**Still owed:** nothing in this repository implements a clipboard.",
        );
        assert!(owed.names().is_empty());
        assert!(owed.owed().is_some());
        assert!(owed.says_something());

        let shown = Entry::under(
            "No telemetry",
            "**Shown by:** `crates/alo-egress/src/errand.rs`",
        );
        assert_eq!(shown.names().len(), 1);
        assert!(shown.owed().is_none());
        assert!(shown.says_something());
    }

    /// And an entry that answers with neither is parsed rather than refused —
    /// the finding names the promise, which needs the promise to have been read.
    #[test]
    fn an_entry_with_neither_half_is_still_read() {
        let empty = Entry::under("The GPU works on first boot", "Some prose about it.\n");
        assert!(
            !empty.says_something(),
            "an entry with no evidence and nothing owed was taken for an answer"
        );
        assert_eq!(empty.promise(), "The GPU works on first boot");
    }

    /// The file names come from the *shown* half only. A path quoted inside
    /// what is owed is a thing that does not exist yet, and counting it would
    /// let a promise be shown by the very file it says is missing.
    #[test]
    fn a_path_named_in_what_is_owed_shows_nothing() {
        let entry = Entry::under(
            "Boots on one certified machine",
            "**Still owed:** no certified machine has booted this; \
             `docs/hardware.md` would name it with a date.",
        );
        assert!(
            entry.names().is_empty(),
            "a file named inside what is still owed was counted as showing the \
             promise"
        );
    }
}
