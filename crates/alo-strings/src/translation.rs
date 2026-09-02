//! One language's strings, as they arrive from whoever wrote them.
//!
//! **A translation as it arrives is not a translation that can be shown.** This
//! type deserialises, and there is nothing on it that answers a lookup: the only
//! way to a usable one is [`crate::Vocabulary::check`], which answers with a
//! [`crate::Speaking`] or with everything that is wrong. It is the same shape
//! `alo-files` uses for a path — resolve it, then ask about the resolved thing —
//! and it is here for the same reason. A translation is a file a person edited
//! by hand, in a language nobody on this team reads, and the moment to find a
//! mistake in it is when it is loaded rather than when somebody's disk is full
//! and the sentence that says so comes out with a hole in it.
//!
//! **A partial translation is normal and must stay shippable.** Missing keys
//! are not errors — a language is translated a few hundred strings at a time,
//! and refusing the file until it is complete would mean nobody ever sees the
//! first half. What *is* refused is a string that would come out wrong:
//! a gap the source has and the translation dropped, a gap the translation
//! invented, a sentence that is empty, and a key nothing in the running code
//! says.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::key::Key;
use crate::language::Language;
use crate::template::TemplateError;

/// One language's strings, as written down.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Translation {
    /// Which language this is.
    language: Language,
    /// Key to sentence, sorted, which is the order a translator works in.
    texts: BTreeMap<Key, String>,
}

impl Translation {
    /// An empty translation into this language.
    #[must_use]
    pub fn into_language(language: Language) -> Self {
        Self {
            language,
            texts: BTreeMap::new(),
        }
    }

    /// One string translated. The last one written for a key is the one kept,
    /// because a file with a key in it twice has to mean something and the
    /// alternative is refusing a file over a duplicate a person cannot see.
    #[must_use]
    pub fn says(mut self, key: Key, text: impl Into<String>) -> Self {
        self.texts.insert(key, text.into());
        self
    }

    /// Which language this is.
    #[must_use]
    pub fn language(&self) -> &Language {
        &self.language
    }

    /// Every key and sentence in it, in key order.
    pub fn texts(&self) -> impl Iterator<Item = (&Key, &str)> {
        self.texts.iter().map(|(key, text)| (key, text.as_str()))
    }

    /// How many strings it holds.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.texts.len()
    }

    /// Whether nothing has been translated yet.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.texts.is_empty()
    }
}

/// One thing wrong with a translation, and which string it is wrong in.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("{key}: {amiss}")]
pub struct Wrong {
    /// The string it is wrong in.
    key: Key,
    /// What is wrong with it.
    amiss: Amiss,
}

impl Wrong {
    /// One thing wrong with one string.
    pub(crate) fn with(key: Key, amiss: Amiss) -> Self {
        Self { key, amiss }
    }

    /// Which string.
    #[must_use]
    pub fn key(&self) -> &Key {
        &self.key
    }

    /// What is wrong with it.
    #[must_use]
    pub fn amiss(&self) -> &Amiss {
        &self.amiss
    }
}

/// What is wrong with one translated string.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum Amiss {
    /// A key nothing in the running code says.
    #[error(
        "nothing in this system says this any more — the string was renamed or removed, so take the line out, and if you translated it under an older name look for it under the new one"
    )]
    NotSaidHere,

    /// A sentence that is not one.
    #[error("{0}")]
    NotASentence(#[from] TemplateError),

    /// A gap the source has and this translation does not.
    #[error(
        "put {{{name}}} back into the sentence — it is where the file name or the number goes, and without it the person reading this is told something is wrong but not what"
    )]
    GapDropped {
        /// The gap that went missing.
        name: String,
    },

    /// A gap this translation has and the source does not.
    #[error(
        "there is nothing to put in {{{name}}} — the English sentence has no such gap, so it would come out on the screen written exactly like that"
    )]
    GapInvented {
        /// The gap that was made up.
        name: String,
    },

    /// A sentence about a number of something, in a language whose plural rules
    /// alo OS does not have.
    #[error(
        "alo OS does not know how {language} counts, so it cannot tell which of these sentences to show for which number — the rules are CLDR's and have to be read rather than guessed at, so add {language} to the table in alo-strings and this file will load; everything in it that does not count a thing is already fine"
    )]
    CountingUnknown {
        /// The language nobody has read the rules for.
        language: Language,
    },

    /// A form this language does not use for any whole number.
    #[error(
        "nothing would ever show this: counting a whole number, this language uses {forms}, so put the sentence under one of those and take this line out"
    )]
    FormNotCounted {
        /// The forms it does use, in order.
        forms: String,
    },
}

/// Everything wrong with one translation, which is what a person fixing it
/// reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wrongs {
    /// Which language was being checked.
    language: Language,
    /// Everything wrong with it, in key order, always one or more.
    wrongs: Vec<Wrong>,
}

impl Wrongs {
    /// Everything wrong with one language's translation.
    pub(crate) fn in_language(language: Language, wrongs: Vec<Wrong>) -> Self {
        Self { language, wrongs }
    }

    /// Which language.
    #[must_use]
    pub fn language(&self) -> &Language {
        &self.language
    }

    /// Everything wrong, in key order.
    #[must_use]
    pub fn wrongs(&self) -> &[Wrong] {
        &self.wrongs
    }

    /// How many things are wrong.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.wrongs.len()
    }
}

impl fmt::Display for Wrongs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the {} translation cannot be shown as it is:",
            self.language
        )?;
        for wrong in &self.wrongs {
            write!(f, "\n  {wrong}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Wrongs {}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn german() -> Language {
        Language::written("de").unwrap()
    }

    #[test]
    fn a_translation_holds_a_language_and_its_strings() {
        let translation = Translation::into_language(german())
            .says(Key::named("files.gone").unwrap(), "Es ist nicht mehr da")
            .says(
                Key::named("files.not-a-folder").unwrap(),
                "{path} ist kein Ordner",
            );
        assert_eq!(translation.language(), &german());
        assert_eq!(translation.how_many(), 2);
        assert!(!translation.is_empty());
    }

    /// A translator works through a file in key order, so that is the order it
    /// is read back in — the same file, whoever wrote it and in whatever order
    /// they typed it.
    #[test]
    fn the_strings_come_back_in_key_order() {
        let translation = Translation::into_language(german())
            .says(Key::named("files.not-a-folder").unwrap(), "b")
            .says(Key::named("files.gone").unwrap(), "a");
        let keys: Vec<&str> = translation.texts().map(|(key, _)| key.as_str()).collect();
        assert_eq!(keys, ["files.gone", "files.not-a-folder"]);
    }

    #[test]
    fn a_key_written_twice_keeps_the_last_sentence() {
        let translation = Translation::into_language(german())
            .says(Key::named("files.gone").unwrap(), "erst")
            .says(Key::named("files.gone").unwrap(), "dann");
        assert_eq!(
            translation.texts().next(),
            Some((&Key::named("files.gone").unwrap(), "dann"))
        );
    }

    /// A translation is a file somebody edited by hand, so it is read back the
    /// way a file is read back, and a language or a key that is not one dies at
    /// the door.
    #[test]
    fn a_translation_is_read_back_from_a_file() {
        let written = r#"{"language":"de","texts":{"files.gone":"Es ist nicht mehr da"}}"#;
        let translation: Translation = serde_json::from_str(written).unwrap();
        assert_eq!(translation.language(), &german());
        assert_eq!(translation.how_many(), 1);
        assert_eq!(serde_json::to_string(&translation).unwrap(), written);

        assert!(
            serde_json::from_str::<Translation>(r#"{"language":"deutsch","texts":{}}"#).is_err()
        );
        assert!(
            serde_json::from_str::<Translation>(r#"{"language":"de","texts":{"gone":"x"}}"#)
                .is_err()
        );
    }

    /// Every refusal is addressed to the person who will fix it, who is a
    /// translator rather than a programmer and is not reading Rust.
    #[test]
    fn what_is_wrong_is_said_to_the_person_who_will_fix_it() {
        let wrongs = Wrongs::in_language(
            german(),
            vec![
                Wrong::with(
                    Key::named("files.too-big").unwrap(),
                    Amiss::GapDropped {
                        name: "bytes".to_owned(),
                    },
                ),
                Wrong::with(Key::named("files.old-name").unwrap(), Amiss::NotSaidHere),
            ],
        );
        let said = wrongs.to_string();
        assert!(said.starts_with("the de translation cannot be shown as it is:"));
        assert!(said.contains("files.too-big: put {bytes} back into the sentence"));
        assert!(said.contains("files.old-name: nothing in this system says this any more"));
        assert_eq!(wrongs.how_many(), 2);
    }

    #[test]
    fn an_invented_gap_says_what_it_would_look_like() {
        let wrong = Wrong::with(
            Key::named("files.gone").unwrap(),
            Amiss::GapInvented {
                name: "folder".to_owned(),
            },
        );
        assert!(
            wrong
                .to_string()
                .contains("there is nothing to put in {folder}")
        );
        assert_eq!(wrong.key().as_str(), "files.gone");
    }
}
