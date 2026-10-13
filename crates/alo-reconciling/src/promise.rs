//! A promise the release makes, read out of the only list of what gets built.
//!
//! `docs/features.md` writes one promise per line and gives each a tier. The
//! v0.01 tier is what this crate reconciles, so the promises are taken from the
//! marker rather than from a heading: a promise that moved between sections is
//! still the same promise, and a section that was renamed is not a release
//! change.

use crate::tier::Tier;

/// One `[v0.01]` promise, in the words `docs/features.md` uses.
///
/// Kept whole rather than summarised. A ledger entry quotes a phrase out of it,
/// and the quote is checked against this — so a promise that is reworded stops
/// matching and has to be reconciled again by whoever reworded it, which is the
/// one moment somebody knows whether the evidence still holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Promise {
    /// The promise's line, trimmed of the list marker and nothing else.
    words: String,
}

impl Promise {
    /// The promise as the definition words it.
    #[must_use]
    pub fn words(&self) -> &str {
        &self.words
    }

    /// Whether this promise contains a phrase, exactly as it is written.
    ///
    /// Substring rather than fuzzy: a ledger entry quotes the definition, and a
    /// quotation that has drifted from what it quotes is what this audit exists
    /// to catch rather than to tolerate.
    #[must_use]
    pub fn carries(&self, phrase: &str) -> bool {
        !phrase.is_empty() && self.words.contains(phrase)
    }
}

/// Every promise in a features document, in the order it makes them.
///
/// Duplicates are not collapsed. Two identical promises would each need
/// reconciling and the ledger would have to say so twice, which is the honest
/// reading of a definition that made the same commitment in two places.
///
/// # It was *every v0.01 promise* until 2026-10-12
///
/// The owner removed the version tiers that day — [ADR
/// 0105](../../../docs/decisions/0105-there-are-no-versions-until-there-is-a-launch.md)
/// — and this selector keyed on `[v0.01]`. **With the markers gone it matched
/// nothing**, and a reconciliation of the empty set passes: exactly the shape
/// `docs/misreadings/a-null-result-and-a-negative-result-look-the-same.md`
/// describes. The test above it caught that at once, because it asserts a
/// positive count before it asserts agreement.
///
/// So the subject moves. A promise is now **any bullet in the features
/// document**, and the launch list in `ROADMAP.md` is what says which of them
/// stand between today and a usable machine.
#[must_use]
pub fn promises_in(document: &str) -> Vec<Promise> {
    document
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("- "))
        .map(|words| Promise {
            words: words.trim().to_owned(),
        })
        .filter(|promise| !promise.words.is_empty())
        .collect()
}

/// Every item on the launch list, in the order it is written.
///
/// **The list that replaced the tiers.** `ROADMAP.md`'s *launch list* section
/// holds the five things that stand between today and a machine somebody
/// outside this team can use, and this is what an audit of *are we there* has
/// to walk. A tier was attached to a feature and consulted when somebody wanted
/// to build it; this is attached to the launch and consulted when somebody asks
/// whether it is done. The second gets looked at.
///
/// Reads only the bullets directly under the heading, stopping at the next
/// heading of any depth — so the explanatory sub-sections under it are prose
/// and not items.
#[must_use]
pub fn the_launch_list(roadmap: &str) -> Vec<Promise> {
    let mut inside = false;
    let mut items = Vec::new();
    for line in roadmap.lines() {
        if line.starts_with("## ") {
            inside = line.trim() == "## The launch list";
            continue;
        }
        if inside && line.starts_with("###") {
            break;
        }
        if !inside {
            continue;
        }
        if let Some(words) = line.strip_prefix("- ") {
            items.push(Promise {
                words: words.trim().to_owned(),
            });
        }
    }
    items
}

/// Every promise a features document makes at one tier, in the order it makes
/// them.
///
/// The tier is a [`Tier`] rather than a string a caller passes, because the
/// markers are one character apart: `[v0.5]` and `[v0.01]` differ by a zero, and
/// a caller that assembled one itself would be a typo away from reconciling an
/// empty set and reporting that every promise was answered.
#[must_use]
pub fn promises_at(document: &str, tier: Tier) -> Vec<Promise> {
    document
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix(tier.marker()))
        .map(|words| Promise {
            words: words.trim().to_owned(),
        })
        .collect()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A document written the way `docs/features.md` is written since the
    /// tiers were removed: bullets, and a preamble that is not one.
    const A_DEFINITION: &str = "\
Feature inventory: what alo OS is, in one list. It boots and the agent acts.

- ★ **File verbs**: list, read, find, rename, move, archive
- Lock screen, suspend and resume
- Every execution recorded with its origin, approval and grant
";

    /// A roadmap written the way `ROADMAP.md` is, with the launch list first
    /// and other sections holding bullets of their own.
    const A_ROADMAP: &str = "\
# alo OS — ROADMAP.md

## The launch list

- **It installs from a release page onto a PC**, without erasing what was there.
- **It boots and a person signs in.**

### What done means for each of them

- This is prose under a sub-heading and must not be an item.

## Where we are

- Nor is this, which is another section entirely.
";

    /// **The promises are the bullets and nothing else**, which is the whole of
    /// what this parser is trusted with.
    #[test]
    fn the_bullets_are_the_promises() {
        let promises = promises_in(A_DEFINITION);
        assert_eq!(
            promises.len(),
            3,
            "the parser did not find every promise, or found something \
             that is not one: {promises:?}"
        );
        assert!(
            promises
                .first()
                .unwrap()
                .words()
                .starts_with("★ **File verbs**"),
            "the marker was not trimmed off the promise, so every quotation \
             would have to include it"
        );
    }

    /// And the preamble is not a promise. It is prose about the file, and a
    /// parser that took it for an item would demand evidence for a definition.
    #[test]
    fn the_preamble_is_not_a_promise() {
        assert!(
            !promises_in(A_DEFINITION)
                .iter()
                .any(|promise| promise.words().contains("it boots and the agent acts")),
            "the sentence describing the file was read as a promise"
        );
    }

    /// **The launch list is the section's own bullets and nothing else.**
    ///
    /// Two failures are possible and both are silent. Reading past the
    /// sub-headings would make every line of the explanation an item somebody
    /// owes evidence for; reading past the section would make the whole roadmap
    /// the launch list, which is the circle the tiers were removed to escape.
    #[test]
    fn the_launch_list_is_its_own_section() {
        let items = the_launch_list(A_ROADMAP);
        assert_eq!(
            items.len(),
            2,
            "the launch list is not the two items under its heading: {items:?}"
        );
        assert!(
            items
                .first()
                .unwrap()
                .words()
                .starts_with("**It installs from a release page"),
            "the first item is not the first bullet under the heading"
        );
        assert!(
            !items
                .iter()
                .any(|item| item.words().contains("another section entirely")),
            "the reader ran past the launch list into the rest of the roadmap"
        );
        assert!(
            !items
                .iter()
                .any(|item| item.words().contains("prose under")),
            "the reader took the explanation under a sub-heading for an item"
        );
    }

    /// **A roadmap with no launch list answers nothing rather than
    /// everything.**
    ///
    /// The dangerous direction: a reader that fell back to every bullet in the
    /// file would reconcile three hundred lines against a ledger of eighty and
    /// drown the real answer. An empty answer is caught by the caller, which
    /// asserts a positive count before it asserts agreement.
    #[test]
    fn a_roadmap_without_the_heading_has_no_launch_list() {
        assert!(the_launch_list("# A roadmap\n\n- a bullet\n").is_empty());
    }

    /// A quotation matches the promise it was taken from, and an empty one
    /// matches nothing — an entry quoting nothing would otherwise match the
    /// first promise in the file and reconcile it by accident.
    #[test]
    fn a_quotation_matches_what_it_was_taken_from() {
        let promises = promises_in(A_DEFINITION);
        let verbs = promises.first().unwrap();
        assert!(verbs.carries("list, read, find, rename"));
        assert!(!verbs.carries("Lock screen"));
        assert!(
            !verbs.carries(""),
            "an entry that quotes nothing matched a promise, which would \
             reconcile whatever happened to be first"
        );
    }
}
