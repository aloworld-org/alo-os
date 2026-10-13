//! What `docs/features.md` promises, read out of the only list of what gets
//! built.
//!
//! A by-hand answer is not a sentence somebody wrote here; it is a **promise the
//! definition already makes**, quoted. That is the whole reason this file
//! exists. ADR 0009 does not say a verb's author should have an idea about how a
//! person might manage without the agent — it says *no surface may be left out
//! because an agent can do it instead*, and the only place a surface is
//! committed to is `docs/features.md`. So the answer is checked against the
//! definition rather than asserted beside it.
//!
//! # A promise used to carry a tier, and the tier is gone
//!
//! Until 2026-10-12 every line of the definition carried `[v0.01]`, `[v0.5]` or
//! `[v1]`, this file read the tier off the line, and a verb's debt named the
//! release that owed it. The owner removed the versions — [ADR
//! 0105](../../../docs/decisions/0105-there-are-no-versions-until-there-is-a-launch.md)
//! — and **this parser went quietly blind**: it looked for `- [`, found nothing,
//! and reported that the definition made no promises at all. Forty verbs were
//! then said to have lost their plain way, which was false about every one of
//! them. The promises had not moved; the thing reading them had.
//!
//! *That is the shape to watch in a parser keyed to a document's formatting:
//! when the format changes it does not fail, it finds nothing — and finding
//! nothing reads exactly like finding a fault.*
//!
//! So a promise is now **any bullet in the definition**, which is what
//! `alo_reconciling::promises_in` was changed to in the same removal. The two
//! read the same document and now read it the same way.

use crate::wrapping::one_line;

/// How `docs/features.md` begins a promise.
///
/// Any list item. Over-inclusive on purpose: a bullet that is not a promise can
/// only make [`Promised::carries`] more permissive, and the alternative —
/// tracking which section of the document a line is in — is a second parser
/// keyed to formatting, which is the thing that just broke.
const A_PROMISE: &str = "- ";

/// One promise of the definition, in its own words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Promised {
    /// The promise's line, trimmed of its list marker and nothing else.
    words: String,
}

impl Promised {
    /// The promise as the definition words it.
    #[must_use]
    pub fn words(&self) -> &str {
        &self.words
    }

    /// Whether this promise contains a phrase, exactly as it is written, once
    /// both are read as one line.
    ///
    /// Substring rather than fuzzy: a by-hand answer quotes the definition, and
    /// a quotation that has drifted from what it quotes is what this check
    /// exists to catch rather than to tolerate. A promise reworded stops being
    /// quoted, and whoever reworded it is the one person who knows whether the
    /// plain way survived the rewording.
    ///
    /// [`crate::wrapping`] is why the whitespace is the one thing not compared
    /// exactly.
    #[must_use]
    pub fn carries(&self, phrase: &str) -> bool {
        let phrase = one_line(phrase);
        !phrase.is_empty() && one_line(&self.words).contains(&phrase)
    }
}

/// Every promise in a features document, in the order it makes them.
///
/// Duplicates are not collapsed and empty bullets are not read. **A positive
/// count is asserted by the test below before anything is compared**, because
/// an empty list is how this file last went wrong: nothing failed, nothing was
/// found, and every verb looked broken.
#[must_use]
pub fn promises_in(document: &str) -> Vec<Promised> {
    document
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix(A_PROMISE))
        .map(|words| Promised {
            words: words.trim().to_owned(),
        })
        .filter(|promise| !promise.words.is_empty())
        .collect()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A document written the way `docs/features.md` is written **now**: a
    /// preamble that is prose, promises with no tier on them, and a list item
    /// somebody left a checkbox on.
    const A_DEFINITION: &str = "\
Everything below is what gets built, and nothing else is.

- \u{2605} **File verbs**: list, read, find, rename, move, archive
- A file manager, with trash, and archives that open
- [ ] Something somebody left a checkbox on
- **A terminal.** Law 2 forbids the *agent* running arbitrary commands
";

    /// **Every bullet is a promise, and there is at least one.**
    ///
    /// The positive count comes first deliberately. An empty list is how this
    /// parser last went wrong — it found nothing when the tiers were removed,
    /// and finding nothing read exactly like finding forty faults.
    #[test]
    fn every_bullet_is_a_promise_and_the_count_is_positive() {
        let promises = promises_in(A_DEFINITION);

        assert!(
            !promises.is_empty(),
            "the parser found no promises at all, which is the failure it is \
             least able to tell from a real one"
        );
        assert_eq!(
            promises.len(),
            4,
            "the parser did not find the four bullets: {promises:?}"
        );
    }

    /// **Prose is not a promise.** A line that is not a list item is not read,
    /// so the document's own preamble cannot answer a verb.
    #[test]
    fn prose_is_not_read_as_a_promise() {
        let promises = promises_in(A_DEFINITION);

        assert!(
            !promises
                .iter()
                .any(|promise| promise.words().contains("Everything below is what")),
            "a line of prose was read as a promise"
        );
    }

    /// **A marker is kept in the words and a quotation still matches through
    /// it.** The star and the bold markers are the definition's own text, and a
    /// by-hand answer quotes the sentence rather than the formatting.
    #[test]
    fn a_quotation_matches_through_the_markers() {
        let promises = promises_in(A_DEFINITION);
        let verbs = promises.first().unwrap();

        assert!(
            verbs.words().starts_with('\u{2605}'),
            "the marker was trimmed: {}",
            verbs.words()
        );
        assert!(verbs.carries("**File verbs**: list, read, find"));
    }

    /// A quotation matches the promise it was taken from, and an empty one
    /// matches nothing — an answer that quoted nothing would otherwise be
    /// answered by whichever promise happened to be first.
    #[test]
    fn a_quotation_matches_what_it_was_taken_from() {
        let promises = promises_in(A_DEFINITION);
        let manager = promises.get(1).unwrap();

        assert!(manager.carries("A file manager, with trash"));
        assert!(!manager.carries("A terminal"));
        assert!(!manager.carries(""));
    }

    /// **A checkbox is read as a promise, and that is a known over-inclusion.**
    ///
    /// Until 2026-10-12 a promise had to carry a tier, so `- [ ]` was excluded
    /// by the shape of the line. With any bullet accepted it is included, and
    /// its text could in principle answer a verb.
    ///
    /// Recorded as a test rather than left to be discovered: the cost is that
    /// [`Promised::carries`] is more permissive, and the alternative — tracking
    /// which section a line sits in — is a second parser keyed to formatting,
    /// which is the thing that just broke.
    #[test]
    fn a_checkbox_is_read_too_and_that_is_known() {
        let promises = promises_in(A_DEFINITION);

        assert!(
            promises
                .iter()
                .any(|promise| promise.words().contains("left a checkbox on")),
            "the over-inclusion this test documents is not happening, so either \
             the parser got stricter or the fixture changed"
        );
    }
}
