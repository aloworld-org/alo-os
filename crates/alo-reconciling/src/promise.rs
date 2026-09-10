//! A promise the release makes, read out of the only list of what gets built.
//!
//! `docs/features.md` writes one promise per line and gives each a tier. The
//! v0.01 tier is what this crate reconciles, so the promises are taken from the
//! marker rather than from a heading: a promise that moved between sections is
//! still the same promise, and a section that was renamed is not a release
//! change.

/// How `docs/features.md` marks a promise as v0.01.
///
/// The tier is written into the line itself, which is why it can be read
/// without knowing anything about the document's shape. The leading `- ` is part
/// of it so that the sentence in the file's own preamble explaining what the
/// tiers mean is not read as a promise.
const AT_V0_01: &str = "- [v0.01]";

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

/// Every v0.01 promise in a features document, in the order it makes them.
///
/// Duplicates are not collapsed. Two identical promises would each need
/// reconciling and the ledger would have to say so twice, which is the honest
/// reading of a definition that made the same commitment in two places.
#[must_use]
pub fn promises_in(document: &str) -> Vec<Promise> {
    document
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix(AT_V0_01))
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

    /// A document written the way `docs/features.md` is written, with one
    /// promise at each tier and the preamble that explains the tiers.
    const A_DEFINITION: &str = "\
**[v0.01]** = it boots and the agent acts · **[v0.5]** = a person can work on it

- [v0.01] ★ **File verbs**: list, read, find, rename, move, archive
- [v0.5] Lock screen, suspend and resume
- [v0.01] Every execution recorded with its origin, approval and grant
";

    /// **The promises are the v0.01 lines and nothing else**, which is the whole
    /// of what this parser is trusted with.
    #[test]
    fn the_v0_01_lines_are_the_promises() {
        let promises = promises_in(A_DEFINITION);
        assert_eq!(
            promises.len(),
            2,
            "the parser did not find both v0.01 promises, or found something \
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

    /// And the sentence explaining what the tiers mean is not a promise — it
    /// names every tier, so a parser reading tiers rather than lines would take
    /// it for one and this audit would demand evidence for a definition.
    #[test]
    fn the_preamble_is_not_a_promise() {
        assert!(
            !promises_in(A_DEFINITION)
                .iter()
                .any(|promise| promise.words().contains("it boots and the agent acts")),
            "the sentence defining the tiers was read as a promise"
        );
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
