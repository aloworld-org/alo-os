//! What `docs/features.md` promises, read out of the only list of what gets
//! built.
//!
//! A by-hand answer is not a sentence somebody wrote here; it is a **promise the
//! definition already makes**, quoted. That is the whole reason this file
//! exists. ADR 0009 does not say a verb's author should have an idea about how a
//! person might manage without the agent — it says *no surface may be left out
//! because an agent can do it instead*, and the only place a surface is
//! committed to is `docs/features.md` with a tier. So the answer is checked
//! against the definition, and the release that owns it is read off the line
//! rather than asserted beside it.
//!
//! Every tier is read, not only v0.01: a plain way that arrives at v0.5 is an
//! answer, and *which release* is the thing a reader most wants to know.

use crate::release::Release;
use crate::wrapping::one_line;

/// How `docs/features.md` begins a promise.
///
/// The tier is inside the line, which is why the promises can be read without
/// knowing anything about the document's sections. `- [` is the list marker and
/// the opening of the tier together, so a sentence in the file's own preamble
/// explaining what the tiers mean is not read as a promise.
const A_PROMISE: &str = "- [";

/// How a tier ends.
const AND_ITS_TIER: char = ']';

/// One promise of the definition: the release it is for, and its own words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Promised {
    /// The tier the line carries.
    release: Release,

    /// The promise's line, trimmed of the marker and the tier and nothing else.
    words: String,
}

impl Promised {
    /// The release this promise is made for.
    #[must_use]
    pub fn release(&self) -> &Release {
        &self.release
    }

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
/// Duplicates are not collapsed and lines that are not promises are not read:
/// a list item whose brackets hold something that is not a tier is an ordinary
/// list item, and [`Release::named`] is what says so.
#[must_use]
pub fn promises_in(document: &str) -> Vec<Promised> {
    document
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix(A_PROMISE))
        .filter_map(|rest| rest.split_once(AND_ITS_TIER))
        .filter_map(|(tier, words)| {
            Release::named(tier).map(|release| Promised {
                release,
                words: words.trim().to_owned(),
            })
        })
        .collect()
}

/// Every release the definition makes a promise for, in the order they first
/// appear.
///
/// This is the list a named release is checked against, and it is the
/// definition's rather than this crate's — a verb owed at a release nobody
/// ships is a verb owed at nothing.
#[must_use]
pub fn releases_among(promises: &[Promised]) -> Vec<Release> {
    let mut releases: Vec<Release> = Vec::new();
    for promise in promises {
        if !releases.contains(promise.release()) {
            releases.push(promise.release().clone());
        }
    }
    releases
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A document written the way `docs/features.md` is written: a preamble that
    /// names every tier, promises at two of them, and a list item that is not a
    /// promise at all.
    const A_DEFINITION: &str = "\
**[v0.01]** = it boots and the agent acts · **[v0.5]** = a person can work on it

- [v0.01] ★ **File verbs**: list, read, find, rename, move, archive
- [v0.5] A file manager, with trash, and archives that open
- [ ] Something somebody left a checkbox on
- [v0.5] **A terminal.** Law 2 forbids the *agent* running arbitrary commands
";

    /// **The promises are the lines with a tier on them**, and each carries the
    /// release it is for.
    #[test]
    fn every_promise_carries_the_release_it_is_for() {
        let promises = promises_in(A_DEFINITION);
        assert_eq!(
            promises.len(),
            3,
            "the parser did not find the three promises, or found something that \
             is not one: {promises:?}"
        );
        assert_eq!(promises.first().unwrap().release().as_written(), "v0.01");
        assert!(
            promises
                .first()
                .unwrap()
                .words()
                .starts_with("★ **File verbs**"),
            "the tier was not trimmed off the promise, so every quotation would \
             have to include it"
        );
        assert_eq!(promises.last().unwrap().release().as_written(), "v0.5");
    }

    /// The sentence explaining what the tiers mean is not a promise, and neither
    /// is a checkbox. Both would otherwise arrive as things a verb could be
    /// answered by.
    #[test]
    fn what_is_not_a_promise_is_not_read_as_one() {
        let promises = promises_in(A_DEFINITION);
        assert!(
            !promises
                .iter()
                .any(|promise| promise.words().contains("it boots and the agent acts")),
            "the sentence defining the tiers was read as a promise"
        );
        assert!(
            !promises
                .iter()
                .any(|promise| promise.words().contains("left a checkbox")),
            "an ordinary markdown checkbox was read as a promise for a release \
             called nothing"
        );
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

    /// The releases are the definition's, each named once and in the order it
    /// first promises something.
    #[test]
    fn the_releases_are_the_ones_the_definition_uses() {
        let releases: Vec<String> = releases_among(&promises_in(A_DEFINITION))
            .iter()
            .map(|release| release.as_written().to_owned())
            .collect();
        assert_eq!(releases, ["v0.01".to_owned(), "v0.5".to_owned()]);
    }
}
