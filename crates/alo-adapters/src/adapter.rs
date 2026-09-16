//! An adapter, as declared data.
//!
//! `docs/contracts/app-adapters.md`, *What an adapter declares*: the
//! application, the mechanism, the verbs and the grants — and, from
//! *Versioning*, the application releases it supports. Every field is data an
//! author writes; nothing an adapter declares is code this machine runs. What
//! runs is this crate's own, generic over the declaration: [`crate::loading`]
//! checks it, and [`crate::driving`] builds the one message a verb becomes.
//!
//! **Declared in Rust, as constants.** A verb's words are `alo_strings::Word`s —
//! a key, the English and a translator's note — and the verb contract requires
//! them to be, so that the sentence a person approves is the string a
//! translator was handed. A `Word` is `'static`, so an adapter is too. Reading
//! one from a signed file at run time is the v1 *adapter allowlist* and *SDK*
//! lines, and needs words that arrive at run time; the task report says what
//! that asks of `alo-strings`.

use alo_strings::Word;

use crate::adapter_verb::AdapterVerb;
use crate::mechanism::Mechanism;

/// One adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Adapter {
    /// What an agent is called by — `text_editor` is `@text_editor`, and each
    /// verb is `text_editor.open_document` on the machine's list.
    pub name: &'static str,
    /// The application, by the identifier the machine knows it by.
    pub application: &'static str,
    /// The release series it supports: `50` covers `50`, `50.1` and `50.2`.
    pub releases: &'static [&'static str],
    /// How it reaches the application.
    pub mechanism: Mechanism,
    /// Every word the adapter's verbs are declared with.
    pub words: &'static [Word],
    /// What it can do.
    pub verbs: &'static [AdapterVerb],
}

impl Adapter {
    /// Whether this adapter supports the application at this release.
    #[must_use]
    pub fn supports(&self, release: &str) -> bool {
        self.releases.iter().any(|series| {
            !series.is_empty()
                && (release == *series
                    || release
                        .strip_prefix(series)
                        .is_some_and(|rest| rest.starts_with('.')))
        })
    }

    /// The name the machine's list knows this verb by.
    #[must_use]
    pub fn verb_named(&self, verb: &AdapterVerb) -> String {
        format!("{}.{}", self.name, verb.name)
    }
}

#[cfg(test)]
mod tests {
    use crate::text_editor::TEXT_EDITOR;

    #[test]
    fn a_release_series_covers_its_releases_and_no_other() {
        assert!(TEXT_EDITOR.supports("50"));
        assert!(TEXT_EDITOR.supports("50.1"));
        assert!(!TEXT_EDITOR.supports("500.1"));
        assert!(!TEXT_EDITOR.supports("49.3"));
        assert!(!TEXT_EDITOR.supports(""));
    }
}
