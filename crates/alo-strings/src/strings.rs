//! The lookup: what a person is asked to read, in the language they read.
//!
//! Everything else in this crate exists so that this file can be short. A
//! `Strings` holds the vocabulary the code declared, the translations that have
//! been checked against it, and the languages the person in front of the
//! machine asked for. [`Strings::say`] answers, and the answer says where it
//! came from.
//!
//! **The chain is stated rather than guessed.** A person names the languages
//! they read, in the order they read them, and each of them brings its broader
//! forms with it: `pt-BR` brings `pt`. So somebody in Latvia who also reads
//! Russian says so, and meets Russian before English rather than after it —
//! which for the languages `docs/features.md` exists to serve is the difference
//! between a machine that speaks to them and one that does not. Nothing infers
//! a second language from a first, because *you are Latvian so you must read
//! Russian* is not a thing software gets to decide.
//!
//! **English is the end of the chain and is not on it.** It is not a preference
//! anybody expressed; it is the language the code happens to be written in. So
//! a string that reaches a person in English has fallen off the end, and every
//! one of the three ways of noticing that is here: [`crate::Said`] says so on
//! every answer, [`Strings::unanswered`] lists them for a release note, and
//! [`Showing::InDevelopment`] marks them on the screen.
//!
//! ```
//! use alo_strings::{Filling, Key, Language, Phrase, Strings, Translation, Vocabulary};
//!
//! let gone = Key::named("files.gone")?;
//! let mut vocabulary = Vocabulary::empty();
//! vocabulary.says(Phrase::says(gone.clone(), "{path} is not there any more")?)?;
//!
//! let german = vocabulary
//!     .check(
//!         Translation::into_language(Language::written("de")?)
//!             .says(gone.clone(), "{path} ist nicht mehr da"),
//!     )
//!     .map_err(|wrongs| wrongs.to_string())?;
//!
//! let mut strings = Strings::of(vocabulary);
//! strings.speaks(german)?;
//! strings.prefers(&[Language::written("de-AT")?]);
//!
//! // Austrian German is not translated; German is, and that is what is shown.
//! let said = strings.say(&gone, &Filling::of("path", "/home/ada/notes"));
//! assert_eq!(said.text(), "/home/ada/notes ist nicht mehr da");
//! assert!(said.is_translated());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::filling::Filling;
use crate::key::Key;
use crate::language::Language;
use crate::said::{CameFrom, Said};
use crate::speaking::Speaking;
use crate::vocabulary::Vocabulary;

/// Who is being shown these strings.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Showing {
    /// Somebody using the machine. An untranslated string is shown as the plain
    /// English it is: marking it would tell a person nothing they can act on
    /// and would make their machine look broken rather than unfinished.
    #[default]
    ToAPerson,
    /// Somebody building the system. An untranslated string is wrapped in
    /// guillemets, so that walking through a Latvian build shows at a glance
    /// which screens are not Latvian yet.
    ///
    /// Guillemets rather than English quotation marks, because a marked string
    /// has to be distinguishable from a string that is quoting something.
    InDevelopment,
}

impl Showing {
    /// The text as this audience should see it.
    fn mark(self, text: &str) -> String {
        match self {
            Self::ToAPerson => text.to_owned(),
            Self::InDevelopment => format!("«{text}»"),
        }
    }
}

/// Every string the system can say, and the language it says them in.
#[derive(Debug, Clone)]
pub struct Strings {
    /// What the code can say, and the English it says it in.
    vocabulary: Vocabulary,
    /// Every language that has been checked, in the order they were added.
    speaking: Vec<Speaking>,
    /// The languages to try, in order, with the broader forms already worked
    /// out. The source is not in it; it is what happens after it.
    chain: Vec<Language>,
    /// Who is looking.
    showing: Showing,
}

impl Strings {
    /// Everything the code says, with nothing translated and nobody's
    /// preference expressed yet — which is what a machine has before it has
    /// been signed into.
    #[must_use]
    pub fn of(vocabulary: Vocabulary) -> Self {
        Self {
            vocabulary,
            speaking: Vec::new(),
            chain: Vec::new(),
            showing: Showing::ToAPerson,
        }
    }

    /// Take on a language that has been checked.
    ///
    /// # Errors
    ///
    /// [`StringsError::AlreadySpeaks`] for a language taken on twice. Which of
    /// the two answered would depend on the order they were added, and nobody
    /// decided that.
    pub fn speaks(&mut self, speaking: Speaking) -> Result<(), StringsError> {
        if self
            .speaking
            .iter()
            .any(|already| already.language() == speaking.language())
        {
            return Err(StringsError::AlreadySpeaks {
                language: speaking.language().clone(),
            });
        }
        self.speaking.push(speaking);
        Ok(())
    }

    /// The languages this person reads, in the order they read them.
    ///
    /// Each one brings its broader forms with it, so asking for `pt-BR` also
    /// asks for `pt`, and nothing is added that the person did not name.
    pub fn prefers(&mut self, languages: &[Language]) {
        self.chain.clear();
        for language in languages {
            let mut asked = Some(language.clone());
            while let Some(next) = asked {
                if !self.chain.contains(&next) {
                    self.chain.push(next.clone());
                }
                asked = next.broader();
            }
        }
    }

    /// The languages that will be tried, in order. The source language is not
    /// among them: it is what happens when they all run out.
    #[must_use]
    pub fn chain(&self) -> &[Language] {
        &self.chain
    }

    /// Who is looking at these strings.
    pub fn shown(&mut self, showing: Showing) {
        self.showing = showing;
    }

    /// What this string says, with its gaps filled in.
    ///
    /// Never fails and never panics: there is always something to put on the
    /// screen, and what there was to say about it is on the [`Said`].
    #[must_use]
    pub fn say(&self, key: &Key, filling: &Filling) -> Said {
        let Some(phrase) = self.vocabulary.phrase(key) else {
            // No sentence was found, so there is no gap to have left unfilled;
            // what is wrong here is the key, and `CameFrom::NoPhrase` says so.
            return Said::new(format!("«{key}»"), CameFrom::NoPhrase, Vec::new());
        };
        for language in &self.chain {
            let Some(speaking) = self
                .speaking
                .iter()
                .find(|speaking| speaking.language() == language)
            else {
                continue;
            };
            if let Some(template) = speaking.text(key) {
                let filled = template.fill(filling);
                return Said::new(
                    filled.text().to_owned(),
                    CameFrom::Translation(language.clone()),
                    filled.unfilled().to_vec(),
                );
            }
        }
        let filled = phrase.source().fill(filling);
        Said::new(
            self.showing.mark(filled.text()),
            CameFrom::TheSource,
            filled.unfilled().to_vec(),
        )
    }

    /// Every string that would reach a person in the source language, in key
    /// order — which is what a release note counts and what a translator is
    /// handed next.
    #[must_use]
    pub fn unanswered(&self) -> Vec<&Key> {
        self.vocabulary
            .phrases()
            .map(crate::phrase::Phrase::key)
            .filter(|key| {
                !self.chain.iter().any(|language| {
                    self.speaking
                        .iter()
                        .any(|speaking| speaking.language() == language && speaking.says(key))
                })
            })
            .collect()
    }

    /// Every string this language has not translated, in key order, whatever
    /// anybody's preference is — which is what one translator asks for.
    #[must_use]
    pub fn missing_from(&self, language: &Language) -> Vec<&Key> {
        let speaking = self
            .speaking
            .iter()
            .find(|speaking| speaking.language() == language);
        self.vocabulary
            .phrases()
            .map(crate::phrase::Phrase::key)
            .filter(|key| speaking.is_none_or(|speaking| !speaking.says(key)))
            .collect()
    }

    /// Every language that has been taken on.
    pub fn languages(&self) -> impl Iterator<Item = &Language> {
        self.speaking.iter().map(Speaking::language)
    }

    /// What the code can say, for whoever is writing a file out for a
    /// translator.
    #[must_use]
    pub fn vocabulary(&self) -> &Vocabulary {
        &self.vocabulary
    }
}

/// Why a language could not be taken on.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum StringsError {
    /// One language, twice.
    #[error(
        "{language} has already been taken on — put the two files together into one before loading it, because which of them answered would depend on the order they were read"
    )]
    AlreadySpeaks {
        /// The language offered twice.
        language: Language,
    },
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

    fn key(named: &str) -> Key {
        Key::named(named).unwrap()
    }

    fn language(tag: &str) -> Language {
        Language::written(tag).unwrap()
    }

    /// Three strings, one of them with gaps in it, which is the shape most of
    /// `alo-files` has.
    fn vocabulary() -> Vocabulary {
        Vocabulary::empty()
            .and(Phrase::says(key("files.gone"), "It is not there any more").unwrap())
            .unwrap()
            .and(Phrase::says(key("files.not-a-folder"), "{path} is not a folder").unwrap())
            .unwrap()
            .and(Phrase::says(key("shortcuts.close"), "Close the window").unwrap())
            .unwrap()
    }

    fn speaking(vocabulary: &Vocabulary, tag: &str, texts: &[(&str, &str)]) -> Speaking {
        let mut translation = Translation::into_language(language(tag));
        for (named, text) in texts {
            translation = translation.says(key(named), *text);
        }
        vocabulary.check(translation).unwrap()
    }

    fn german() -> Strings {
        let vocabulary = vocabulary();
        let german = speaking(
            &vocabulary,
            "de",
            &[
                ("files.gone", "Es ist nicht mehr da"),
                ("files.not-a-folder", "{path} ist kein Ordner"),
            ],
        );
        let mut strings = Strings::of(vocabulary);
        strings.speaks(german).unwrap();
        strings.prefers(&[language("de")]);
        strings
    }

    #[test]
    fn a_translated_string_is_answered_in_the_translation() {
        let strings = german();
        let said = strings.say(&key("files.not-a-folder"), &Filling::of("path", "/tmp/x"));
        assert_eq!(said.text(), "/tmp/x ist kein Ordner");
        assert!(said.is_translated());
        assert_eq!(said.came_from(), &CameFrom::Translation(language("de")));
    }

    /// **The test item 9 asks for.** A string nobody has translated reaches a
    /// person as English — there is nothing else to show them — but it is
    /// marked while the system is being built, so that walking through a German
    /// build shows at a glance which screens are not German yet.
    #[test]
    fn a_missing_translation_is_visible_in_development() {
        let mut strings = german();

        let to_a_person = strings.say(&key("shortcuts.close"), &Filling::nothing());
        assert_eq!(to_a_person.text(), "Close the window");
        assert!(!to_a_person.is_translated());
        assert_eq!(to_a_person.came_from(), &CameFrom::TheSource);

        strings.shown(Showing::InDevelopment);
        let in_development = strings.say(&key("shortcuts.close"), &Filling::nothing());
        assert_eq!(in_development.text(), "«Close the window»");
        assert!(!in_development.is_translated());

        // And the one that *was* translated is not marked, whoever is looking.
        assert_eq!(
            strings.say(&key("files.gone"), &Filling::nothing()).text(),
            "Es ist nicht mehr da"
        );
    }

    /// Whether a person is reading a translation is on every answer, whoever is
    /// looking — which is what makes English impossible to show silently even
    /// in a release build.
    #[test]
    fn every_answer_says_whether_it_was_translated() {
        let strings = german();
        assert!(
            strings
                .say(&key("files.gone"), &Filling::nothing())
                .is_translated()
        );
        assert!(
            !strings
                .say(&key("shortcuts.close"), &Filling::nothing())
                .is_translated()
        );
    }

    /// A person who asked for Brazilian Portuguese and met a string only
    /// European Portuguese has is shown European Portuguese.
    #[test]
    fn a_narrower_language_falls_back_to_the_broader_one() {
        let vocabulary = vocabulary();
        let portuguese = speaking(&vocabulary, "pt", &[("files.gone", "Já não está lá")]);
        let mut strings = Strings::of(vocabulary);
        strings.speaks(portuguese).unwrap();
        strings.prefers(&[language("pt-BR")]);

        assert_eq!(strings.chain(), [language("pt-BR"), language("pt")]);
        let said = strings.say(&key("files.gone"), &Filling::nothing());
        assert_eq!(said.text(), "Já não está lá");
        assert_eq!(said.came_from(), &CameFrom::Translation(language("pt")));
    }

    /// **A person names their own second language.** Nothing infers one from
    /// the other, and a person who reads Latvian and Russian meets Russian
    /// before English rather than after it.
    #[test]
    fn a_second_language_comes_before_the_source() {
        let vocabulary = vocabulary();
        let latvian = speaking(&vocabulary, "lv", &[("files.gone", "Tā vairs nav")]);
        let russian = speaking(
            &vocabulary,
            "ru",
            &[
                ("files.gone", "Этого больше нет"),
                ("shortcuts.close", "Закрыть окно"),
            ],
        );
        let mut strings = Strings::of(vocabulary);
        strings.speaks(latvian).unwrap();
        strings.speaks(russian).unwrap();
        strings.prefers(&[language("lv"), language("ru")]);

        assert_eq!(
            strings
                .say(&key("files.gone"), &Filling::nothing())
                .came_from(),
            &CameFrom::Translation(language("lv"))
        );
        assert_eq!(
            strings
                .say(&key("shortcuts.close"), &Filling::nothing())
                .came_from(),
            &CameFrom::Translation(language("ru"))
        );
    }

    #[test]
    fn the_chain_holds_no_language_twice_and_nothing_nobody_named() {
        let mut strings = Strings::of(vocabulary());
        strings.prefers(&[language("pt-BR"), language("pt"), language("de-AT")]);
        assert_eq!(
            strings.chain(),
            [
                language("pt-BR"),
                language("pt"),
                language("de-AT"),
                language("de")
            ]
        );
    }

    /// A machine nobody has signed into yet answers in the source language, and
    /// says that is what it is doing.
    #[test]
    fn with_no_preference_everything_comes_from_the_source() {
        let strings = Strings::of(vocabulary());
        assert!(strings.chain().is_empty());
        let said = strings.say(&key("files.gone"), &Filling::nothing());
        assert_eq!(said.text(), "It is not there any more");
        assert_eq!(said.came_from(), &CameFrom::TheSource);
    }

    /// A key nothing declares is a mistake here rather than in a translation,
    /// and the person sees the key rather than a blank space, in every build.
    #[test]
    fn a_key_nothing_declares_shows_the_key_and_says_it_is_a_bug() {
        let strings = german();
        let said = strings.say(&key("files.never-declared"), &Filling::nothing());
        assert_eq!(said.text(), "«files.never-declared»");
        assert_eq!(said.came_from(), &CameFrom::NoPhrase);
        assert!(said.is_a_bug());
    }

    /// A gap nobody filled survives the lookup rather than being swallowed by
    /// it, in a translation as much as in the source.
    #[test]
    fn an_unfilled_gap_reaches_the_answer() {
        let strings = german();
        let said = strings.say(&key("files.not-a-folder"), &Filling::nothing());
        assert_eq!(said.text(), "{path} ist kein Ordner");
        assert_eq!(said.unfilled(), ["path"]);
        assert!(said.is_a_bug());
    }

    /// What a release note counts, and what a translator is handed next.
    #[test]
    fn what_is_still_in_the_source_language_can_be_listed() {
        let strings = german();
        assert_eq!(
            strings.unanswered(),
            [&key("shortcuts.close")],
            "one string is still English"
        );
        assert_eq!(
            strings.missing_from(&language("de")),
            [&key("shortcuts.close")]
        );
        assert_eq!(
            strings.missing_from(&language("lv")).len(),
            3,
            "a language nobody has translated is missing all of them"
        );
        assert_eq!(strings.languages().collect::<Vec<_>>(), [&language("de")]);
    }

    /// One language, one file. Which of two answered would depend on the order
    /// they were read, and nobody decided that.
    #[test]
    fn one_language_cannot_be_taken_on_twice() {
        let vocabulary = vocabulary();
        let first = speaking(&vocabulary, "de", &[("files.gone", "Weg")]);
        let second = speaking(&vocabulary, "de", &[("files.gone", "Anders")]);
        let mut strings = Strings::of(vocabulary);
        strings.speaks(first).unwrap();
        strings.prefers(&[language("de")]);
        let refused = strings.speaks(second).unwrap_err();
        assert!(matches!(refused, StringsError::AlreadySpeaks { .. }));
        assert_eq!(
            strings.say(&key("files.gone"), &Filling::nothing()).text(),
            "Weg"
        );
    }

    /// Naming preferences again replaces them rather than adding to them: a
    /// person changing their language in Settings is changing it, not adding
    /// one.
    #[test]
    fn naming_preferences_again_replaces_them() {
        let mut strings = german();
        strings.prefers(&[language("fr")]);
        assert_eq!(strings.chain(), [language("fr")]);
        assert_eq!(
            strings
                .say(&key("files.gone"), &Filling::nothing())
                .came_from(),
            &CameFrom::TheSource
        );
    }
}
