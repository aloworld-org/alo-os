//! Which language, written the way the rest of the world writes it.
//!
//! A language is a BCP 47 tag — `de`, `pt-BR`, `sr-Latn-RS` — because that is
//! what a keyboard layout, a font, a spell checker and a translator's tools all
//! already speak, and inventing a second spelling would mean translating
//! between them somewhere.
//!
//! **A tag is normalised when it is made**, so `PT-br` and `pt-BR` are one
//! language and not two entries in a settings panel. That matters more here
//! than it looks: two spellings of one language would each get their own
//! translation file and a person would be shown whichever loaded first.
//!
//! **A language knows which way it is read.** No official EU language is
//! written right to left, so nothing today depends on it — which is exactly
//! when to write it down. `docs/features.md` promises the shell is
//! right-to-left ready *so that adding a language later is translation rather
//! than rework*, and a promise in that document is a test in this one.
//!
//! What is not here: how a date, a number or a size is written. That belongs to
//! the region and not to the language, and [`crate::Filling`] says why.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::union;

/// Which language something is written in.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Language {
    /// The tag, normalised: `pt`, `pt-BR`, `sr-Latn-RS`.
    tag: String,
}

impl Language {
    /// A language, if the tag is one.
    ///
    /// # Errors
    ///
    /// [`LanguageError`], which says how a tag is written.
    pub fn written(tag: &str) -> Result<Self, LanguageError> {
        if tag.is_empty() {
            return Err(LanguageError::Empty);
        }
        let mut subtags = tag.split('-');
        let Some(primary) = subtags.next() else {
            return Err(LanguageError::Empty);
        };
        if !(matches!(primary.len(), 2..=3) && primary.chars().all(|c| c.is_ascii_alphabetic())) {
            return Err(LanguageError::NotALanguage {
                subtag: primary.to_owned(),
            });
        }
        let mut normalised = primary.to_ascii_lowercase();
        for subtag in subtags {
            normalised.push('-');
            normalised.push_str(&normalise(subtag)?);
        }
        Ok(Self { tag: normalised })
    }

    /// The tag.
    #[must_use]
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// The language itself, without the script or the region: `pt` for
    /// `pt-BR`.
    #[must_use]
    pub fn primary(&self) -> &str {
        self.tag.split('-').next().unwrap_or(&self.tag)
    }

    /// The same language with the last subtag dropped — `pt-BR` becomes `pt` —
    /// or `None` when there is nothing left to drop.
    ///
    /// This is the whole of the fallback chain's arithmetic: a person who asked
    /// for Brazilian Portuguese and met a string only European Portuguese has
    /// should be shown European Portuguese rather than English.
    #[must_use]
    pub fn broader(&self) -> Option<Self> {
        let (broader, _) = self.tag.rsplit_once('-')?;
        Some(Self {
            tag: broader.to_owned(),
        })
    }

    /// Which way this language is read.
    #[must_use]
    pub fn direction(&self) -> Direction {
        if RIGHT_TO_LEFT.contains(&self.primary()) {
            Direction::RightToLeft
        } else {
            Direction::LeftToRight
        }
    }

    /// What this language is called in itself — `Deutsch`, `Ελληνικά`,
    /// `Gaeilge` — for the 24 in [`crate::union`].
    ///
    /// **A language is named in its own language and never in ours.** A picker
    /// that listed *Greek* would be a picker a person who reads only Greek
    /// cannot use, which is the whole population the entry is there for.
    ///
    /// `None` for a language somebody contributed that is not one of the 24;
    /// a picker shows its tag until the name is added beside it in
    /// [`crate::union`].
    #[must_use]
    pub fn in_its_own_language(&self) -> Option<&'static str> {
        union::OFFICIAL
            .iter()
            .find(|official| official.tag == self.primary())
            .map(|official| official.calls_itself)
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.tag)
    }
}

impl TryFrom<String> for Language {
    type Error = LanguageError;

    fn try_from(tag: String) -> Result<Self, Self::Error> {
        Self::written(&tag)
    }
}

impl From<Language> for String {
    fn from(language: Language) -> Self {
        language.tag
    }
}

/// A subtag after the first, written the way its kind is written: a script in
/// title case, a region in capitals, a variant in lowercase.
fn normalise(subtag: &str) -> Result<String, LanguageError> {
    let letters = subtag.chars().all(|c| c.is_ascii_alphabetic());
    let digits = subtag.chars().all(|c| c.is_ascii_digit());
    let alphanumeric = subtag.chars().all(|c| c.is_ascii_alphanumeric());
    match subtag.len() {
        4 if letters => {
            let mut script = subtag.to_ascii_lowercase();
            if let Some(first) = script.get_mut(0..1) {
                first.make_ascii_uppercase();
            }
            Ok(script)
        }
        2 if letters => Ok(subtag.to_ascii_uppercase()),
        3 if digits => Ok(subtag.to_owned()),
        5..=8 if alphanumeric => Ok(subtag.to_ascii_lowercase()),
        _ => Err(LanguageError::NotASubtag {
            subtag: subtag.to_owned(),
        }),
    }
}

/// The languages alo OS knows are read right to left.
///
/// None of them is an official EU language, and the list is short on purpose:
/// it holds the ones a contributed translation is actually likely to be in.
/// Adding one is a line here, which is the rework `docs/features.md`'s
/// right-to-left promise exists to avoid.
const RIGHT_TO_LEFT: [&str; 8] = ["ar", "arc", "ckb", "dv", "fa", "he", "ur", "yi"];

/// Which way a language is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    /// Every official EU language, and most others.
    LeftToRight,
    /// Arabic, Hebrew, Persian and the rest.
    RightToLeft,
}

/// Why something is not a language.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum LanguageError {
    /// Nothing at all.
    #[error("name the language — de, pt-BR or sr-Latn-RS, the way the rest of the world writes it")]
    Empty,

    /// A first subtag that is not a language.
    #[error(
        "start the tag with the language itself: two or three letters, so de rather than {subtag}"
    )]
    NotALanguage {
        /// What was offered.
        subtag: String,
    },

    /// A subtag after the first that is none of the kinds a tag holds.
    #[error(
        "{subtag} is not a script, a region or a variant — write a script as four letters (Latn), a region as two letters or three digits (BR, 419), and anything else as five to eight letters or digits"
    )]
    NotASubtag {
        /// What was offered.
        subtag: String,
    },
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_language_is_a_language() {
        let german = Language::written("de").unwrap();
        assert_eq!(german.tag(), "de");
        assert_eq!(german.primary(), "de");
        assert_eq!(german.broader(), None);
    }

    /// **One language, one spelling.** Two would each get their own
    /// translation file, and a person would be shown whichever was loaded
    /// first.
    #[test]
    fn a_tag_is_normalised_when_it_is_made() {
        for written in ["pt-BR", "PT-br", "pt-br", "Pt-Br"] {
            assert_eq!(
                Language::written(written).unwrap().tag(),
                "pt-BR",
                "{written}"
            );
        }
        assert_eq!(Language::written("SR-latn-rs").unwrap().tag(), "sr-Latn-RS");
    }

    /// The whole of the fallback chain's arithmetic: a person who asked for
    /// Brazilian Portuguese and met a string only European Portuguese has is
    /// shown European Portuguese, not English.
    #[test]
    fn a_language_gets_broader_one_subtag_at_a_time() {
        let precise = Language::written("sr-Latn-RS").unwrap();
        let script = precise.broader().unwrap();
        assert_eq!(script.tag(), "sr-Latn");
        let plain = script.broader().unwrap();
        assert_eq!(plain.tag(), "sr");
        assert_eq!(plain.broader(), None);
    }

    #[test]
    fn something_that_is_not_a_tag_is_refused() {
        assert_eq!(Language::written(""), Err(LanguageError::Empty));
        for written in ["e", "deutsch", "1e"] {
            assert!(
                matches!(
                    Language::written(written),
                    Err(LanguageError::NotALanguage { .. })
                ),
                "{written}"
            );
        }
        for written in ["de-D", "de-l@tin", "de-abcdefghi"] {
            assert!(
                matches!(
                    Language::written(written),
                    Err(LanguageError::NotASubtag { .. })
                ),
                "{written}"
            );
        }
    }

    /// `docs/features.md` promises the shell is right-to-left ready so that
    /// adding a language later is translation rather than rework. Nothing in
    /// the Union needs it, which is precisely why it is asserted now rather
    /// than discovered later.
    #[test]
    fn no_official_language_is_read_right_to_left_and_the_ones_that_are_say_so() {
        for official in union::OFFICIAL {
            let language = Language::written(official.tag).unwrap();
            assert_eq!(
                language.direction(),
                Direction::LeftToRight,
                "{}",
                official.in_english
            );
        }
        for written in ["ar", "he", "fa", "ur", "ar-EG"] {
            assert_eq!(
                Language::written(written).unwrap().direction(),
                Direction::RightToLeft,
                "{written}"
            );
        }
    }

    /// A language is named in its own language. A region does not change the
    /// name, so Brazilian Portuguese is still *Português*.
    #[test]
    fn a_language_is_named_in_itself() {
        assert_eq!(
            Language::written("de").unwrap().in_its_own_language(),
            Some("Deutsch")
        );
        assert_eq!(
            Language::written("pt-BR").unwrap().in_its_own_language(),
            Some("Português")
        );
        assert_eq!(Language::written("is").unwrap().in_its_own_language(), None);
    }

    #[test]
    fn a_language_is_checked_when_it_is_read_back() {
        let written = serde_json::to_string(&Language::written("pt-BR").unwrap()).unwrap();
        assert_eq!(written, "\"pt-BR\"");
        assert_eq!(
            serde_json::from_str::<Language>("\"PT-br\"").unwrap(),
            Language::written("pt-BR").unwrap()
        );
        assert!(serde_json::from_str::<Language>("\"deutsch\"").is_err());
    }
}
