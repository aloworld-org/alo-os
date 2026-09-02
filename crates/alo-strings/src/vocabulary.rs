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

use crate::cldr;
use crate::form::{self, EVERY_FORM, Form};
use crate::key::Key;
use crate::phrase::Phrase;
use crate::plural::Plural;
use crate::speaking::Speaking;
use crate::template::Template;
use crate::translation::{Amiss, Translation, Wrong, Wrongs};

/// Everything the running code can say.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Vocabulary {
    /// Key to phrase, in key order, which is the order a translator's file is
    /// written in.
    phrases: BTreeMap<Key, Phrase>,
    /// Key to countable string, in key order. Kept apart from the phrases
    /// because a translator answers one of these with a line per form and how
    /// many that is depends on their language rather than on ours.
    plurals: BTreeMap<Key, Plural>,
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
        self.free(phrase.key())?;
        self.phrases.insert(phrase.key().clone(), phrase);
        Ok(())
    }

    /// Add one countable string — a sentence with a number in it, which takes a
    /// different shape for different numbers.
    ///
    /// # Errors
    ///
    /// [`VocabularyError::AlreadySaid`] or [`VocabularyError::AlreadyCounted`]
    /// when the key is taken, in either direction: a countable string owns
    /// every `key.form` beneath it, so a phrase called `files.too-big.one`
    /// and a countable `files.too-big` cannot both exist. Which of them
    /// answered would depend on the order they were registered.
    pub fn counts(&mut self, plural: Plural) -> Result<(), VocabularyError> {
        self.free(plural.key())?;
        for form in EVERY_FORM {
            self.free(&plural.key().for_form(form))?;
        }
        self.plurals.insert(plural.key().clone(), plural);
        Ok(())
    }

    /// The same vocabulary with one more countable string in it, for building
    /// one in an expression.
    ///
    /// # Errors
    ///
    /// As [`Vocabulary::counts`].
    pub fn counting(mut self, plural: Plural) -> Result<Self, VocabularyError> {
        self.counts(plural)?;
        Ok(self)
    }

    /// Whether nothing has claimed this key, in either direction.
    fn free(&self, key: &Key) -> Result<(), VocabularyError> {
        if let Some(already) = self.phrases.get(key) {
            return Err(VocabularyError::AlreadySaid {
                key: key.clone(),
                already: already.source().as_written().to_owned(),
            });
        }
        if self.plurals.contains_key(key) {
            return Err(VocabularyError::AlreadyCounted { key: key.clone() });
        }
        if let Some((base, _)) = key.without_form()
            && self.plurals.contains_key(&base)
        {
            return Err(VocabularyError::AlreadyCounted { key: base });
        }
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
        for key in other.phrases.keys() {
            self.free(key)?;
        }
        for key in other.plurals.keys() {
            self.free(key)?;
            for form in EVERY_FORM {
                self.free(&key.for_form(form))?;
            }
        }
        self.phrases.extend(other.phrases);
        self.plurals.extend(other.plurals);
        Ok(())
    }

    /// What this key says, if anything says it.
    #[must_use]
    pub fn phrase(&self, key: &Key) -> Option<&Phrase> {
        self.phrases.get(key)
    }

    /// What this key says about a number of something, if it counts.
    #[must_use]
    pub fn plural(&self, key: &Key) -> Option<&Plural> {
        self.plurals.get(key)
    }

    /// Everything the code can say that does not count, in key order — which is
    /// half of what is written out for a translator to work from.
    pub fn phrases(&self) -> impl Iterator<Item = &Phrase> {
        self.phrases.values()
    }

    /// Everything the code can say about a number of something, in key order —
    /// the other half.
    pub fn counted(&self) -> impl Iterator<Item = &Plural> {
        self.plurals.values()
    }

    /// How many strings there are. **A countable string counts once**, whatever
    /// how many forms it turns into: it is one thing the code can say, and how
    /// many lines it costs a translator depends on their language rather than
    /// on this number.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.phrases.len().saturating_add(self.plurals.len())
    }

    /// Whether nothing has been declared.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.phrases.is_empty() && self.plurals.is_empty()
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
            // What the sentence is measured against: the English for a plain
            // string, and the general form for one that counts — which is the
            // form that carries every gap.
            let Some(against) = self.against(key, translation.language(), &mut wrongs) else {
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
            for gap in against.source.gaps() {
                if Some(gap.as_str()) == against.number {
                    // A form that is exactly one number may spell it out —
                    // *ein Ordner* — so this is the one gap such a form is
                    // allowed to do without.
                    continue;
                }
                if !template.has(gap) {
                    wrongs.push(Wrong::with(
                        key.clone(),
                        Amiss::GapDropped { name: gap.clone() },
                    ));
                    sound = false;
                }
            }
            for gap in template.gaps() {
                if !against.source.has(gap) {
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

    /// What one translated line is checked against, or `None` with the reason
    /// already written down.
    fn against(
        &self,
        key: &Key,
        language: &crate::language::Language,
        wrongs: &mut Vec<Wrong>,
    ) -> Option<Against<'_>> {
        if let Some(phrase) = self.phrases.get(key) {
            return Some(Against {
                source: phrase.source(),
                number: None,
            });
        }
        let Some((base, form)) = key.without_form() else {
            wrongs.push(Wrong::with(key.clone(), Amiss::NotSaidHere));
            return None;
        };
        let Some(plural) = self.plurals.get(&base) else {
            wrongs.push(Wrong::with(key.clone(), Amiss::NotSaidHere));
            return None;
        };
        let Some(forms) = cldr::forms(language) else {
            wrongs.push(Wrong::with(
                key.clone(),
                Amiss::CountingUnknown {
                    language: language.clone(),
                },
            ));
            return None;
        };
        if !forms.contains(&form) {
            wrongs.push(Wrong::with(
                key.clone(),
                Amiss::FormNotCounted {
                    forms: form::listed(forms),
                },
            ));
            return None;
        }
        Some(Against {
            source: plural.source(Form::Other),
            // *Ein Ordner* is how the sentence is written — but only where this
            // form is one number and not, as Croatian's *one* is, every number
            // ending in 1.
            number: cldr::names_one_number(language, form).then(|| plural.number()),
        })
    }
}

/// What one translated line is measured against.
#[derive(Debug, Clone, Copy)]
struct Against<'a> {
    /// The English sentence its gaps have to match.
    source: &'a Template,
    /// The gap holding the number, when this line is one form of a countable
    /// string. It is the only gap a translation is allowed to leave out.
    number: Option<&'a str>,
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

    /// A key a countable string already owns, or a countable string over a key
    /// something else has.
    #[error(
        "{key} already counts something, and a string that counts owns every form beneath it — {key}.one, {key}.other and the rest — so name this one something else"
    )]
    AlreadyCounted {
        /// The countable string in the way.
        key: Key,
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
    use crate::plural::Plural;

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

    /// A vocabulary with a string that counts in it, which is the shape
    /// `alo-files`' *too big* message needs.
    fn counting() -> Vocabulary {
        files()
            .counting(
                Plural::counting(
                    key("files.too-many"),
                    "how_many",
                    "1 file is too big to read",
                    "{how_many} files are too big to read",
                )
                .unwrap(),
            )
            .unwrap()
    }

    fn into_language(tag: &str) -> Translation {
        Translation::into_language(Language::written(tag).unwrap())
    }

    /// A countable string is one thing the code can say, however many lines it
    /// costs a translator — and the two halves are written out separately
    /// because a translator answers them differently.
    #[test]
    fn a_countable_string_is_one_thing_the_code_can_say() {
        let vocabulary = counting();
        assert_eq!(vocabulary.how_many(), 3);
        assert!(vocabulary.plural(&key("files.too-many")).is_some());
        assert!(vocabulary.phrase(&key("files.too-many")).is_none());
        assert_eq!(vocabulary.counted().count(), 1);
        assert_eq!(vocabulary.phrases().count(), 2);
    }

    /// **A countable string owns every form beneath it.** Two things answering
    /// `files.too-many.one` would be shown or not depending on which was
    /// registered first, which is the same reason one key says one thing.
    #[test]
    fn a_countable_string_and_a_phrase_cannot_claim_one_key() {
        let mut vocabulary = counting();
        let refused = vocabulary
            .says(Phrase::says(key("files.too-many.one"), "Just the one").unwrap())
            .unwrap_err();
        assert!(matches!(refused, VocabularyError::AlreadyCounted { .. }));

        let mut other_way_round = files();
        other_way_round
            .says(Phrase::says(key("files.too-many.few"), "A few").unwrap())
            .unwrap();
        let refused = other_way_round
            .counts(Plural::counting(key("files.too-many"), "how_many", "1", "{how_many}").unwrap())
            .unwrap_err();
        assert!(matches!(refused, VocabularyError::AlreadySaid { .. }));

        let mut twice = counting();
        assert!(matches!(
            twice
                .counts(
                    Plural::counting(key("files.too-many"), "how_many", "1", "{how_many}").unwrap()
                )
                .unwrap_err(),
            VocabularyError::AlreadyCounted { .. }
        ));
    }

    /// A translation is checked against the forms **its own language** uses.
    /// Polish needs three and none of them is `other`.
    #[test]
    fn a_countable_translation_is_checked_against_its_own_languages_forms() {
        let speaking = counting()
            .check(
                into_language("pl")
                    .says(key("files.too-many.one"), "1 plik jest za duży")
                    .says(key("files.too-many.few"), "{how_many} pliki są za duże")
                    .says(
                        key("files.too-many.many"),
                        "{how_many} plików jest za dużych",
                    ),
            )
            .unwrap();
        assert_eq!(speaking.how_many(), 3);
    }

    /// **A form no whole number reaches is refused**, and the refusal says
    /// which forms the language does use — because a translator writing a
    /// Polish `other` has written a sentence nothing will ever show and has no
    /// way to find that out.
    #[test]
    fn a_form_the_language_never_uses_is_refused() {
        let wrongs = counting()
            .check(into_language("pl").says(key("files.too-many.other"), "{how_many} plików"))
            .unwrap_err();
        assert_eq!(
            wrongs.wrongs().first().unwrap().amiss(),
            &Amiss::FormNotCounted {
                forms: "one, few and many".to_owned()
            }
        );
        assert!(wrongs.to_string().contains("one, few and many"));
    }

    /// **A language whose plural rules alo OS does not have is refused rather
    /// than guessed at**, in words addressed to whoever is contributing it, and
    /// the message says what to do: add the rules. Everything in the file that
    /// does not count is untouched by the reason.
    #[test]
    fn a_countable_string_in_a_language_we_cannot_count_in_is_refused() {
        let wrongs = counting()
            .check(
                into_language("is")
                    .says(key("files.gone"), "Það er farið")
                    .says(key("files.too-many.one"), "1 skrá"),
            )
            .unwrap_err();
        assert_eq!(wrongs.how_many(), 1);
        assert_eq!(
            wrongs.wrongs().first().unwrap().amiss(),
            &Amiss::CountingUnknown {
                language: Language::written("is").unwrap()
            }
        );
        assert!(wrongs.to_string().contains("does not know how is counts"));

        // The same file without the countable string loads.
        assert!(
            counting()
                .check(into_language("is").says(key("files.gone"), "Það er farið"))
                .is_ok()
        );
    }

    /// **The number is the one gap a form may leave out**, because *ein Ordner*
    /// is how the sentence is written and spelling it `1 Ordner` to satisfy a
    /// check would be this crate writing a translator's sentence for them.
    #[test]
    fn a_form_may_spell_the_number_out_and_may_not_drop_anything_else() {
        let plural = Plural::counting(
            key("files.too-big"),
            "bytes",
            "{path} holds one byte, and a verb reads at most {most}",
            "{path} holds {bytes} bytes, and a verb reads at most {most}",
        )
        .unwrap();
        let vocabulary = Vocabulary::empty().counting(plural).unwrap();

        let speaking = vocabulary
            .check(into_language("de").says(
                key("files.too-big.one"),
                "{path} ist ein Byte groß, und ein Verb liest höchstens {most}",
            ))
            .unwrap();
        assert_eq!(speaking.how_many(), 1);

        let wrongs = vocabulary
            .check(into_language("de").says(
                key("files.too-big.other"),
                "{path} ist zu groß, und ein Verb liest höchstens {most}",
            ))
            .unwrap_err();
        assert_eq!(
            wrongs.wrongs().first().unwrap().amiss(),
            &Amiss::GapDropped {
                name: "bytes".to_owned()
            },
            "the general form is the one that has to say how many"
        );

        let wrongs = vocabulary
            .check(into_language("de").says(
                key("files.too-big.one"),
                "{folder} ist ein Byte groß, und ein Verb liest höchstens {most}",
            ))
            .unwrap_err();
        assert_eq!(wrongs.how_many(), 2, "{wrongs}");
    }

    /// **And *one* is not always one number.** Croatian's `one` covers 1, 21,
    /// 31 and 101, so a Croatian sentence that spells the number out would tell
    /// somebody with twenty-one files about one file. The exemption is asked of
    /// the rules, not of the form's name.
    #[test]
    fn a_form_that_is_not_one_number_may_not_spell_it_out() {
        let vocabulary = Vocabulary::empty()
            .counting(
                Plural::counting(key("files.found"), "how_many", "1 file", "{how_many} files")
                    .unwrap(),
            )
            .unwrap();

        assert!(
            vocabulary
                .check(into_language("de").says(key("files.found.one"), "Eine Datei"))
                .is_ok(),
            "German's one is one file and nothing else"
        );

        let wrongs = vocabulary
            .check(into_language("hr").says(key("files.found.one"), "Jedna datoteka"))
            .unwrap_err();
        assert_eq!(
            wrongs.wrongs().first().unwrap().amiss(),
            &Amiss::GapDropped {
                name: "how_many".to_owned()
            }
        );
    }

    /// A form of something nothing counts is a key nothing says, and gets the
    /// message a renamed string gets.
    #[test]
    fn a_form_of_something_nothing_counts_is_refused() {
        let wrongs = counting()
            .check(into_language("pl").says(key("files.nothing.one"), "Jeden"))
            .unwrap_err();
        assert_eq!(
            wrongs.wrongs().first().unwrap().amiss(),
            &Amiss::NotSaidHere
        );
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
