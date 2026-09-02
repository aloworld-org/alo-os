//! Everything the running code can say, and the one place a translation is
//! checked against it.
//!
//! A vocabulary is built by the code that owns the strings: `alo-files` knows
//! its own refusals, `alo-shortcuts` knows what its actions are called, and each
//! of them hands over a vocabulary that the shell joins into one. That is why
//! [`Key`] insists on an area — joining two crates' vocabularies is how a
//! collision would happen, and [`VocabularyError::AlreadySaid`] is what happens
//! instead.
//!
//! **A translation is checked here because this is the only place that knows
//! what the sentence was supposed to say.** The checks are in
//! [`crate::translation`]; what matters about doing them here is that
//! [`crate::Speaking`] has no other door, so a lookup cannot be handed a
//! translation nobody looked at.
//!
//! **Missing strings are not an error.** A language arrives a few hundred
//! strings at a time, and a check that refused an incomplete file would mean
//! nobody ever saw the first half of anybody's work. What is missing is a
//! question the lookup answers — [`crate::Strings::missing_from`] — rather than
//! a reason to throw a file away.

use std::collections::BTreeMap;

use crate::key::Key;
use crate::phrase::Phrase;
use crate::speaking::Speaking;
use crate::template::Template;
use crate::translation::{Amiss, Translation, Wrong, Wrongs};

/// Everything the running code can say.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Vocabulary {
    /// Key to phrase, in key order, which is the order a translator's file is
    /// written in.
    phrases: BTreeMap<Key, Phrase>,
}

impl Vocabulary {
    /// A vocabulary that says nothing yet.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Add one phrase.
    ///
    /// # Errors
    ///
    /// [`VocabularyError::AlreadySaid`] when the key is taken. One key says one
    /// thing: a second phrase under a key would be shown or not depending on
    /// which was registered first, which is a decision nobody made.
    pub fn says(&mut self, phrase: Phrase) -> Result<(), VocabularyError> {
        if let Some(already) = self.phrases.get(phrase.key()) {
            return Err(VocabularyError::AlreadySaid {
                key: phrase.key().clone(),
                already: already.source().as_written().to_owned(),
            });
        }
        self.phrases.insert(phrase.key().clone(), phrase);
        Ok(())
    }

    /// The same vocabulary with one more phrase in it, for building one in an
    /// expression.
    ///
    /// # Errors
    ///
    /// As [`Vocabulary::says`].
    pub fn and(mut self, phrase: Phrase) -> Result<Self, VocabularyError> {
        self.says(phrase)?;
        Ok(self)
    }

    /// Take in everything another crate's vocabulary says.
    ///
    /// # Errors
    ///
    /// [`VocabularyError::AlreadySaid`] when the two name one string. Nothing
    /// is taken in when that happens: half a joined vocabulary would be worse
    /// than none, because the half that arrived would look complete.
    pub fn join(&mut self, other: Self) -> Result<(), VocabularyError> {
        for phrase in other.phrases.values() {
            if let Some(already) = self.phrases.get(phrase.key()) {
                return Err(VocabularyError::AlreadySaid {
                    key: phrase.key().clone(),
                    already: already.source().as_written().to_owned(),
                });
            }
        }
        self.phrases.extend(other.phrases);
        Ok(())
    }

    /// What this key says, if anything says it.
    #[must_use]
    pub fn phrase(&self, key: &Key) -> Option<&Phrase> {
        self.phrases.get(key)
    }

    /// Everything the code can say, in key order — which is what is written out
    /// for a translator to work from.
    pub fn phrases(&self) -> impl Iterator<Item = &Phrase> {
        self.phrases.values()
    }

    /// How many strings there are.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.phrases.len()
    }

    /// Whether nothing has been declared.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.phrases.is_empty()
    }

    /// Check a translation against what the code actually says, and answer with
    /// the only type the lookup accepts.
    ///
    /// # Errors
    ///
    /// [`Wrongs`] — everything wrong with the file, in key order, so that
    /// whoever fixes it fixes all of it once rather than being told about the
    /// next mistake each time they try again.
    pub fn check(&self, translation: Translation) -> Result<Speaking, Wrongs> {
        let mut texts: BTreeMap<Key, Template> = BTreeMap::new();
        let mut wrongs = Vec::new();
        for (key, text) in translation.texts() {
            let Some(phrase) = self.phrases.get(key) else {
                wrongs.push(Wrong::with(key.clone(), Amiss::NotSaidHere));
                continue;
            };
            let template = match Template::written(text) {
                Ok(template) => template,
                Err(why) => {
                    wrongs.push(Wrong::with(key.clone(), Amiss::NotASentence(why)));
                    continue;
                }
            };
            let mut sound = true;
            for gap in phrase.source().gaps() {
                if !template.has(gap) {
                    wrongs.push(Wrong::with(
                        key.clone(),
                        Amiss::GapDropped { name: gap.clone() },
                    ));
                    sound = false;
                }
            }
            for gap in template.gaps() {
                if !phrase.source().has(gap) {
                    wrongs.push(Wrong::with(
                        key.clone(),
                        Amiss::GapInvented { name: gap.clone() },
                    ));
                    sound = false;
                }
            }
            if sound {
                texts.insert(key.clone(), template);
            }
        }
        if wrongs.is_empty() {
            Ok(Speaking::checked(translation.language().clone(), texts))
        } else {
            Err(Wrongs::in_language(translation.language().clone(), wrongs))
        }
    }
}

/// Why a phrase could not be added.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum VocabularyError {
    /// Two phrases under one key.
    #[error(
        "{key} already says \"{already}\" — one key says one thing, so give this one a name of its own, and if the two really are the same sentence say it once and use it twice"
    )]
    AlreadySaid {
        /// The key both wanted.
        key: Key,
        /// What the first one says, so the person reading this can tell whether
        /// they have found a duplicate or a collision.
        already: String,
    },
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::language::Language;

    fn key(named: &str) -> Key {
        Key::named(named).unwrap()
    }

    fn files() -> Vocabulary {
        Vocabulary::empty()
            .and(Phrase::says(key("files.gone"), "It is not there any more").unwrap())
            .unwrap()
            .and(
                Phrase::says(
                    key("files.too-big"),
                    "{path} holds {bytes} bytes and a verb reads at most {most}",
                )
                .unwrap(),
            )
            .unwrap()
    }

    fn into_german() -> Translation {
        Translation::into_language(Language::written("de").unwrap())
    }

    #[test]
    fn a_vocabulary_holds_what_the_code_can_say() {
        let vocabulary = files();
        assert_eq!(vocabulary.how_many(), 2);
        assert!(!vocabulary.is_empty());
        assert_eq!(
            vocabulary
                .phrase(&key("files.gone"))
                .unwrap()
                .source()
                .as_written(),
            "It is not there any more"
        );
        assert!(vocabulary.phrase(&key("files.nothing")).is_none());
        let keys: Vec<&str> = vocabulary.phrases().map(|p| p.key().as_str()).collect();
        assert_eq!(keys, ["files.gone", "files.too-big"]);
    }

    /// One key says one thing. Two would be shown or not depending on which was
    /// registered first, and nothing anywhere decided that.
    #[test]
    fn one_key_says_one_thing() {
        let mut vocabulary = files();
        let refused = vocabulary
            .says(Phrase::says(key("files.gone"), "Something else entirely").unwrap())
            .unwrap_err();
        assert!(matches!(refused, VocabularyError::AlreadySaid { .. }));
        assert!(refused.to_string().contains("It is not there any more"));
        assert_eq!(vocabulary.how_many(), 2);
    }

    /// Each crate declares its own strings and the shell joins them, which is
    /// exactly where two crates would collide — so this is where the area in a
    /// key earns its place.
    #[test]
    fn two_crates_vocabularies_join() {
        let mut all = files();
        let shortcuts = Vocabulary::empty()
            .and(Phrase::says(key("shortcuts.action.close"), "Close the window").unwrap())
            .unwrap();
        all.join(shortcuts).unwrap();
        assert_eq!(all.how_many(), 3);
    }

    /// Nothing is taken in from a vocabulary that collides, because a half-taken
    /// one would look complete to everything downstream.
    #[test]
    fn a_colliding_vocabulary_is_refused_whole() {
        let mut all = files();
        let clashing = Vocabulary::empty()
            .and(Phrase::says(key("files.gone"), "Gone").unwrap())
            .unwrap()
            .and(Phrase::says(key("files.new-one"), "Something new").unwrap())
            .unwrap();
        assert!(all.join(clashing).is_err());
        assert_eq!(all.how_many(), 2, "nothing was taken in");
    }

    #[test]
    fn a_translation_that_matches_becomes_something_that_can_be_shown() {
        let speaking = files()
            .check(into_german().says(key("files.gone"), "Es ist nicht mehr da"))
            .unwrap();
        assert!(speaking.says(&key("files.gone")));
        assert!(!speaking.says(&key("files.too-big")));
    }

    /// **A partial translation is shippable.** A language arrives a few hundred
    /// strings at a time, and refusing the file until it was complete would mean
    /// nobody ever saw the first half of anybody's work.
    #[test]
    fn a_partial_translation_is_not_an_error() {
        let speaking = files().check(into_german()).unwrap();
        assert_eq!(speaking.how_many(), 0);
        assert_eq!(speaking.language().tag(), "de");
    }

    /// **A dropped gap is refused**, and this is the check the whole type
    /// exists for: without it, a person is told their disk is full in their own
    /// language and not told which file.
    #[test]
    fn a_translation_that_dropped_a_gap_is_refused() {
        let wrongs = files()
            .check(into_german().says(
                key("files.too-big"),
                "{path} ist zu groß und wird höchstens {most} gelesen",
            ))
            .unwrap_err();
        assert_eq!(wrongs.how_many(), 1);
        assert_eq!(
            wrongs.wrongs().first().unwrap().amiss(),
            &Amiss::GapDropped {
                name: "bytes".to_owned()
            }
        );
    }

    #[test]
    fn a_translation_that_invented_a_gap_is_refused() {
        let wrongs = files()
            .check(into_german().says(key("files.gone"), "{path} ist weg"))
            .unwrap_err();
        assert_eq!(
            wrongs.wrongs().first().unwrap().amiss(),
            &Amiss::GapInvented {
                name: "path".to_owned()
            }
        );
    }

    /// A key nothing says any more is usually a rename, and the message says so
    /// rather than telling a translator their file is broken.
    #[test]
    fn a_translation_of_something_nothing_says_is_refused() {
        let wrongs = files()
            .check(into_german().says(key("files.old-name"), "Alt"))
            .unwrap_err();
        assert_eq!(
            wrongs.wrongs().first().unwrap().amiss(),
            &Amiss::NotSaidHere
        );
    }

    #[test]
    fn a_sentence_that_is_not_one_is_refused() {
        let wrongs = files()
            .check(into_german().says(key("files.gone"), "   "))
            .unwrap_err();
        assert!(matches!(
            wrongs.wrongs().first().unwrap().amiss(),
            Amiss::NotASentence(_)
        ));
    }

    /// Everything wrong comes back at once. Being told about the next mistake
    /// each time you try again is how a translator gives up.
    #[test]
    fn everything_wrong_comes_back_at_once() {
        let wrongs = files()
            .check(
                into_german()
                    .says(key("files.gone"), "{path} ist weg")
                    .says(key("files.old-name"), "Alt")
                    .says(key("files.too-big"), "{path} ist zu groß"),
            )
            .unwrap_err();
        assert_eq!(wrongs.how_many(), 4, "{wrongs}");
        assert_eq!(wrongs.language().tag(), "de");
    }

    /// A translation with one bad string keeps none of it, because a file that
    /// half-loaded would put a language into service with holes nobody was told
    /// about.
    #[test]
    fn nothing_is_shown_from_a_translation_that_has_something_wrong_with_it() {
        assert!(
            files()
                .check(
                    into_german()
                        .says(key("files.gone"), "Es ist nicht mehr da")
                        .says(key("files.old-name"), "Alt")
                )
                .is_err()
        );
    }
}
