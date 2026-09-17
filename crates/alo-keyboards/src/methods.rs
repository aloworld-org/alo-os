//! Input methods, for the scripts a keyboard cannot type.
//!
//! Chinese, Japanese and Korean are not typed by pressing the key with the
//! letter on it, because there is no such key: a person types sounds and
//! chooses characters. That is an input method, it is a large and specialised
//! piece of software, and alo OS rents it. [`THE_FRAMEWORK`] is IBus — the
//! framework every Linux desktop uses and every one of these engines is built
//! for — configured and never patched (ADR 0011).
//!
//! # A person adds one without knowing its name
//!
//! Nobody should have to know that Japanese is `mozc-jp` and Korean is
//! `hangul`. [`Writing::for_language`] takes a language — the same
//! [`alo_strings::Language`] the rest of the machine uses — and names the
//! engine itself. The engine names never reach a person: they are how this
//! crate and the rented framework agree, the way a layout name is.
//!
//! # And nothing starts on a machine that does not need one
//!
//! [`crate::Keyboards::what_the_session_starts`] answers `None` until a person
//! has added an input method, so a machine that types Latin script runs no
//! extra daemon at all. A framework started for everybody in case somebody
//! needs it is a process reading every keystroke on machines where nobody asked
//! for it, which is not a thing this product starts by default.

use alo_strings::Language;
use serde::{Deserialize, Serialize};

/// The rented input-method framework.
///
/// Its name is never said to a person: this crate's plan is explicit that no
/// sentence names an input-method framework.
pub const THE_FRAMEWORK: &str = "ibus";

/// What the session starts when a person has added an input method — the rented
/// framework's own command.
pub const IT_IS_STARTED_BY: &str = "ibus-daemon";

/// A way of writing that a keyboard layout cannot type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Writing {
    /// Chinese in simplified characters, typed by sound.
    ChineseSimplified,
    /// Chinese in traditional characters, typed by sound.
    ChineseTraditional,
    /// Japanese, typed by sound and converted.
    Japanese,
    /// Korean, typed by the parts of a syllable.
    Korean,
}

impl Writing {
    /// Every one alo OS can set up.
    pub const ALL: [Self; 4] = [
        Self::ChineseSimplified,
        Self::ChineseTraditional,
        Self::Japanese,
        Self::Korean,
    ];

    /// The rented engine that does this, as [`THE_FRAMEWORK`] names it.
    #[must_use]
    pub const fn engine(self) -> &'static str {
        match self {
            Self::ChineseSimplified => "libpinyin",
            Self::ChineseTraditional => "chewing",
            Self::Japanese => "mozc-jp",
            Self::Korean => "hangul",
        }
    }

    /// The way this language is written, when it needs an input method.
    ///
    /// `None` for every language that is typed on a keyboard, which is every
    /// language [`crate::offering`] has an offer for and a good many it has
    /// not.
    #[must_use]
    pub fn for_language(language: &Language) -> Option<Self> {
        match (language.primary(), language.tag()) {
            ("zh", "zh-TW" | "zh-HK" | "zh-MO" | "zh-Hant") => Some(Self::ChineseTraditional),
            ("zh", _) => Some(Self::ChineseSimplified),
            ("ja", _) => Some(Self::Japanese),
            ("ko", _) => Some(Self::Korean),
            _ => None,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// **A person names a language and gets a way of typing it**, without ever
    /// meeting the engine's name.
    #[test]
    fn a_language_names_the_way_it_is_written() {
        for (tag, writing) in [
            ("ja", Writing::Japanese),
            ("ja-JP", Writing::Japanese),
            ("ko", Writing::Korean),
            ("zh", Writing::ChineseSimplified),
            ("zh-CN", Writing::ChineseSimplified),
            ("zh-TW", Writing::ChineseTraditional),
            ("zh-Hant", Writing::ChineseTraditional),
        ] {
            let language = Language::written(tag).unwrap();
            assert_eq!(Writing::for_language(&language), Some(writing), "{tag}");
        }
    }

    /// **A language typed on a keyboard needs no input method**, so every one
    /// of the 24 answers `None` and is offered a keyboard instead.
    #[test]
    fn a_language_typed_on_a_keyboard_needs_no_input_method() {
        for official in alo_strings::union::OFFICIAL {
            let language = Language::written(official.tag).unwrap();
            assert_eq!(
                Writing::for_language(&language),
                None,
                "{}",
                official.in_english
            );
        }
    }

    /// Every engine is a different engine, and each is one the rented framework
    /// knows by that name.
    #[test]
    fn every_way_of_writing_has_its_own_engine() {
        let engines: BTreeSet<&str> = Writing::ALL.iter().map(|w| w.engine()).collect();
        assert_eq!(engines.len(), Writing::ALL.len());
        for engine in engines {
            assert!(!engine.is_empty());
            assert!(
                engine
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c == '-' || c.is_ascii_digit())
            );
        }
    }

    /// A way of writing reads back from a person's own file as itself.
    #[test]
    fn a_way_of_writing_reads_back_as_itself() {
        for writing in Writing::ALL {
            let written = toml::to_string(&Held { writing }).unwrap();
            assert_eq!(toml::from_str::<Held>(&written).unwrap().writing, writing);
        }
        assert!(toml::from_str::<Held>("writing = \"Cantonese\"").is_err());
    }

    /// One, so that TOML has a table to hold it.
    #[derive(Debug, Serialize, Deserialize)]
    struct Held {
        writing: Writing,
    }
}
