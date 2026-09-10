//! One entry of `docs/by-hand.md`: a verb, and this machine's answer about it.
//!
//! An entry is a heading naming the verb and one or two marked lines under it.
//! The heading is the verb's **name**, matched exactly, for the same reason
//! `alo_capability::Verbs` matches a call's name exactly: the name is the
//! identity, and an entry that fuzzily resembles a verb is an entry that answers
//! about whichever verb it reached first.
//!
//! Both halves may be written. A verb whose plain way is partly promised and
//! partly owed is an ordinary state — `find_in_folder` is one — and a document
//! that could hold only one of the two would push the other half into prose
//! nothing reads.

use crate::by_hand::ByHand;
use crate::owed::Owed;

/// How an entry says how a person does the same thing.
const BY_HAND: &str = "**By hand:**";

/// How an entry says the plain way is owed, and by which release.
const OWED_AT: &str = "**Owed at:**";

/// One verb, and what this machine says about doing the same thing by hand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The verb's name, as the heading writes it.
    verb: String,

    /// How a person does the same thing, where the entry says.
    by_hand: Option<ByHand>,

    /// What is owed, where the entry says.
    owed: Option<Owed>,
}

impl Entry {
    /// An entry, from the verb its heading names and the body written under it.
    ///
    /// Backticks are trimmed off the heading, so `### list_folder` and
    /// ``### `list_folder` `` are the same entry — a document is read by people
    /// as well, and which of the two somebody writes is not a fact about the
    /// verb.
    ///
    /// Neither half is required here. An entry with neither is exactly the
    /// finding this check exists to produce, and refusing to parse it would hide
    /// the verb it is about.
    #[must_use]
    pub fn under(verb: &str, body: &str) -> Self {
        let (before, owed) = match body.split_once(OWED_AT) {
            Some((before, after)) => (before, Owed::said(after)),
            None => (body, None),
        };
        let by_hand = match before.split_once(BY_HAND) {
            Some((_, said)) => ByHand::said(said),
            None => None,
        };
        Self {
            verb: verb.trim().trim_matches('`').trim().to_owned(),
            by_hand,
            owed,
        }
    }

    /// The verb this entry is about.
    #[must_use]
    pub fn verb(&self) -> &str {
        &self.verb
    }

    /// How a person does the same thing, if the entry says.
    #[must_use]
    pub fn by_hand(&self) -> Option<&ByHand> {
        self.by_hand.as_ref()
    }

    /// What is owed, if the entry says.
    #[must_use]
    pub fn owed(&self) -> Option<&Owed> {
        self.owed.as_ref()
    }

    /// Whether it answers about its verb at all.
    ///
    /// Naming no plain way and owing nothing is the verb with neither, which is
    /// the finding.
    #[must_use]
    pub fn says_something(&self) -> bool {
        self.by_hand.is_some() || self.owed.is_some()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **Both halves are read**, and the quotations come out of the by-hand half.
    #[test]
    fn an_entry_carries_the_plain_way_and_what_is_owed() {
        let entry = Entry::under(
            "find_in_folder",
            "\
**By hand:** a person searches in the file manager —
`Search your own files, without asking anything`.

**Owed at:** [v0.5] the search is v0.5 work and nothing on this machine indexes
anything yet.
",
        );
        assert_eq!(entry.verb(), "find_in_folder");
        assert_eq!(
            entry.by_hand().unwrap().naming(),
            ["Search your own files, without asking anything".to_owned()]
        );
        assert_eq!(entry.owed().unwrap().release(), Some("v0.5"));
        assert!(entry.says_something());
    }

    /// Either half alone is ordinary: a verb whose plain way is promised owes
    /// nothing, and one whose plain way nothing promises quotes nothing.
    #[test]
    fn an_entry_may_be_all_of_one_half() {
        let plain = Entry::under(
            "open_application",
            "**By hand:** a person opens it from the launcher, as on any desktop — \
             `Launcher and window management: open, focus, close, tile`.",
        );
        assert!(plain.by_hand().is_some());
        assert!(plain.owed().is_none());

        let owed = Entry::under(
            "archive_folder",
            "**Owed at:** [v0.5] nothing in the definition promises making an \
             archive by hand, only opening one.",
        );
        assert!(owed.by_hand().is_none());
        assert!(owed.owed().is_some());
        assert!(owed.says_something());
    }

    /// And an entry that answers with neither is parsed rather than refused. It
    /// is the verb somebody added and said nothing about, which is the whole
    /// point.
    #[test]
    fn an_entry_with_neither_half_is_still_read() {
        let empty = Entry::under("`delete_folder`", "Some prose about how useful it is.\n");
        assert_eq!(
            empty.verb(),
            "delete_folder",
            "the backticks a heading may be written with became part of the name"
        );
        assert!(
            !empty.says_something(),
            "an entry with no plain way and nothing owed was taken for an answer"
        );
    }

    /// A quotation written inside what is *owed* names nothing. It is a surface
    /// that does not exist, and counting it would let a verb be answered by the
    /// very promise it says is missing.
    #[test]
    fn a_quotation_inside_what_is_owed_names_nothing() {
        let entry = Entry::under(
            "archive_folder",
            "**Owed at:** [v0.5] `A file manager, with trash, and archives that \
             open` promises opening one and not making one.",
        );
        assert!(
            entry.by_hand().is_none(),
            "a phrase quoted inside what is still owed was read as the plain way"
        );
    }
}
