//! A translation the vocabulary has checked: the only type that means *these
//! words may be shown to somebody*.
//!
//! There is no public constructor and no `Deserialize`. The only way to one is
//! [`crate::Vocabulary::check`], which is what makes the checks in
//! [`crate::translation`] impossible to skip rather than merely rude to skip.
//! It is the shape `alo-egress` uses for `Departing` and `alo-files` uses for
//! `Touching`: the guarantee is carried by a type somebody has to hold, not by
//! whoever writes the next lookup remembering to call the checker first.
//!
//! ```compile_fail
//! use std::collections::BTreeMap;
//! use alo_strings::{Language, Speaking};
//!
//! // The fields are private and nothing outside the crate builds one:
//! // a translation that was never checked cannot be shown.
//! let speaking = Speaking {
//!     language: Language::written("de").unwrap(),
//!     texts: BTreeMap::new(),
//! };
//! ```
//!
//! The twin that must pass, so the pair cannot rot into a test of a typo:
//!
//! ```
//! use alo_strings::{Key, Language, Phrase, Translation, Vocabulary};
//!
//! let key = Key::named("files.gone")?;
//! let mut vocabulary = Vocabulary::empty();
//! vocabulary.says(Phrase::says(key.clone(), "It is not there any more")?)?;
//!
//! let speaking = vocabulary
//!     .check(Translation::into_language(Language::written("de")?).says(key.clone(), "Es ist weg"))
//!     .map_err(|wrongs| wrongs.to_string())?;
//! assert!(speaking.says(&key));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::collections::BTreeMap;

use crate::key::Key;
use crate::language::Language;
use crate::template::Template;

/// One language's strings, checked against the vocabulary and ready to be
/// shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Speaking {
    /// Which language.
    language: Language,
    /// Key to sentence, already parsed, and every gap already known to match
    /// the source's.
    texts: BTreeMap<Key, Template>,
}

impl Speaking {
    /// Made by [`crate::Vocabulary::check`] and by nothing else.
    pub(crate) fn checked(language: Language, texts: BTreeMap<Key, Template>) -> Self {
        Self { language, texts }
    }

    /// Which language this is.
    #[must_use]
    pub fn language(&self) -> &Language {
        &self.language
    }

    /// Whether this language has this string.
    #[must_use]
    pub fn says(&self, key: &Key) -> bool {
        self.texts.contains_key(key)
    }

    /// The sentence for this string, if this language has it.
    pub(crate) fn text(&self, key: &Key) -> Option<&Template> {
        self.texts.get(key)
    }

    /// How many strings this language has.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.texts.len()
    }

    /// Every string it has, in key order — which is what a progress line in a
    /// release note counts.
    pub fn keys(&self) -> impl Iterator<Item = &Key> {
        self.texts.keys()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::phrase::Phrase;
    use crate::translation::Translation;
    use crate::vocabulary::Vocabulary;

    #[test]
    fn a_checked_translation_answers_for_what_it_holds() {
        let gone = Key::named("files.gone").unwrap();
        let folder = Key::named("files.not-a-folder").unwrap();
        let mut vocabulary = Vocabulary::empty();
        vocabulary
            .says(Phrase::says(gone.clone(), "It is not there any more").unwrap())
            .unwrap();
        vocabulary
            .says(Phrase::says(folder.clone(), "{path} is not a folder").unwrap())
            .unwrap();

        let speaking = vocabulary
            .check(
                Translation::into_language(Language::written("de").unwrap())
                    .says(gone.clone(), "Es ist nicht mehr da"),
            )
            .unwrap();
        assert_eq!(speaking.language().tag(), "de");
        assert!(speaking.says(&gone));
        assert!(!speaking.says(&folder));
        assert_eq!(speaking.how_many(), 1);
        assert_eq!(speaking.keys().collect::<Vec<_>>(), [&gone]);
    }
}
