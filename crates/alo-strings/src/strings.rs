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

use crate::cldr::{self, Counting};
use crate::filling::Filling;
use crate::form::Form;
use crate::key::Key;
use crate::language::Language;
use crate::plural::Plural;
use crate::said::{CameFrom, Said};
use crate::speaking::Speaking;
use crate::union;
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

    /// What this string says about this many of something, with its gaps filled
    /// in and the number put where the sentence says it goes.
    ///
    /// The form is chosen with **the language's own rules**, language by
    /// language down the chain: Polish's `few` is not English's, so a Polish
    /// translation is asked for the Polish form of this number and a Russian
    /// one after it for the Russian form. A language whose rules alo OS does
    /// not have is stepped over rather than guessed at.
    ///
    /// Never fails and never panics, as [`Strings::say`] does not. Asking for a
    /// countable string with [`Strings::say`], or a plain one here, shows the
    /// key and answers [`crate::Said::is_a_bug`] — the mistake is in the calling
    /// code and there is no honest sentence to show for it.
    #[must_use]
    pub fn count(&self, key: &Key, counting: &Counting, filling: &Filling) -> Said {
        let Some(plural) = self.vocabulary.plural(key) else {
            return Said::new(format!("«{key}»"), CameFrom::NoPhrase, Vec::new());
        };
        // The number that picked the form is the number the sentence shows, so
        // it is filled in here rather than left to every call site to remember.
        let filling = filling.clone().and(plural.number(), counting.written());
        for language in &self.chain {
            let Some(speaking) = self
                .speaking
                .iter()
                .find(|speaking| speaking.language() == language)
            else {
                continue;
            };
            let Some(form) = cldr::form_for(language, counting.how_many()) else {
                continue;
            };
            if let Some(template) = speaking.text(&key.for_form(form)) {
                let filled = template.fill(&filling);
                return Said::new(
                    filled.text().to_owned(),
                    CameFrom::Translation(language.clone()),
                    filled.unfilled().to_vec(),
                );
            }
        }
        let filled = plural.source(source_form(counting)).fill(&filling);
        Said::new(
            self.showing.mark(filled.text()),
            CameFrom::TheSource,
            filled.unfilled().to_vec(),
        )
    }

    /// Every string that would reach a person in the source language, in key
    /// order — which is what a release note counts and what a translator is
    /// handed next.
    ///
    /// **A countable string is listed once, under its own name**, and is
    /// counted as answered only where one language in the chain has every form
    /// that language needs. Two half-translated languages might between them
    /// cover every number, and this says they do not: erring towards *finish
    /// it* is the right way round for a list a translator is handed.
    #[must_use]
    pub fn unanswered(&self) -> Vec<Key> {
        let plain = self
            .vocabulary
            .phrases()
            .map(crate::phrase::Phrase::key)
            .filter(|key| {
                !self.chain.iter().any(|language| {
                    self.speaking
                        .iter()
                        .any(|speaking| speaking.language() == language && speaking.says(key))
                })
            })
            .cloned();
        let counted = self
            .vocabulary
            .counted()
            .filter(|plural| !self.counted_in_full(plural))
            .map(Plural::key)
            .cloned();
        let mut keys: Vec<Key> = plain.chain(counted).collect();
        keys.sort();
        keys
    }

    /// Every string this language has not translated, in key order, whatever
    /// anybody's preference is — which is what one translator asks for.
    ///
    /// **A countable string is listed by the forms this language needs** —
    /// `files.too-big.few` and the rest — because that is what the person doing
    /// the work has to write, and a Polish file that has `one` and `other` is
    /// not two thirds done, it is missing `few` and `many` and has an `other`
    /// no whole number reaches. A language whose plural rules alo OS does not
    /// have is listed the base key instead: nobody can translate a sentence
    /// that counts until somebody adds how the language counts.
    #[must_use]
    pub fn missing_from(&self, language: &Language) -> Vec<Key> {
        let speaking = self
            .speaking
            .iter()
            .find(|speaking| speaking.language() == language);
        let mut keys: Vec<Key> = self
            .vocabulary
            .phrases()
            .map(crate::phrase::Phrase::key)
            .filter(|key| speaking.is_none_or(|speaking| !speaking.says(key)))
            .cloned()
            .collect();
        for plural in self.vocabulary.counted() {
            match cldr::forms(language) {
                None => keys.push(plural.key().clone()),
                Some(forms) => {
                    for key in forms.iter().map(|form| plural.key().for_form(*form)) {
                        if speaking.is_none_or(|speaking| !speaking.says(&key)) {
                            keys.push(key);
                        }
                    }
                }
            }
        }
        keys.sort();
        keys
    }

    /// Whether one language in the chain has every form it needs for this
    /// countable string.
    fn counted_in_full(&self, plural: &Plural) -> bool {
        self.chain.iter().any(|language| {
            let Some(forms) = cldr::forms(language) else {
                return false;
            };
            self.speaking.iter().any(|speaking| {
                speaking.language() == language
                    && forms
                        .iter()
                        .all(|form| speaking.says(&plural.key().for_form(*form)))
            })
        })
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

/// Which of the source language's two forms this many takes.
///
/// English counts in one and other, which
/// `the_source_language_counts_and_counts_in_two` asserts rather than assumes.
/// [`Form::Other`] is the general sentence, so a source language whose rules
/// were somehow missing would answer with the sentence that fits every number
/// rather than with nothing.
fn source_form(counting: &Counting) -> Form {
    Language::written(union::THE_SOURCE)
        .ok()
        .and_then(|source| cldr::form_for(&source, counting.how_many()))
        .unwrap_or(Form::Other)
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
    use crate::plural::Plural;
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
            [key("shortcuts.close")],
            "one string is still English"
        );
        assert_eq!(
            strings.missing_from(&language("de")),
            [key("shortcuts.close")]
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

    /// One countable string, and the languages that make it interesting:
    /// Polish, which counts in three and never says `other`; Irish, which
    /// counts in five; and Latvian, which has a word for none.
    fn counting_vocabulary() -> Vocabulary {
        Vocabulary::empty()
            .and(Phrase::says(key("files.gone"), "It is not there any more").unwrap())
            .unwrap()
            .counting(
                Plural::counting(key("files.found"), "how_many", "1 file", "{how_many} files")
                    .unwrap(),
            )
            .unwrap()
    }

    fn polish() -> Strings {
        let vocabulary = counting_vocabulary();
        let polish = speaking(
            &vocabulary,
            "pl",
            &[
                ("files.found.one", "1 plik"),
                ("files.found.few", "{how_many} pliki"),
                ("files.found.many", "{how_many} plików"),
            ],
        );
        let mut strings = Strings::of(vocabulary);
        strings.speaks(polish).unwrap();
        strings.prefers(&[language("pl")]);
        strings
    }

    /// **The form is the reader's language's, not English's.** Polish's `few`
    /// covers 2 to 4 and not 22 to 24's teens; its `many` covers 0, 5 to 19 and
    /// 100. Nothing about English decides any of that.
    #[test]
    fn a_counted_string_takes_the_readers_own_form() {
        let strings = polish();
        for (how_many, expected) in [
            (1_u64, "1 plik"),
            (2, "2 pliki"),
            (4, "4 pliki"),
            (5, "5 plików"),
            (0, "0 plików"),
            (12, "12 plików"),
            (22, "22 pliki"),
            (100, "100 plików"),
        ] {
            let said = strings.count(
                &key("files.found"),
                &Counting::of(how_many),
                &Filling::nothing(),
            );
            assert_eq!(said.text(), expected, "{how_many}");
            assert!(said.is_translated(), "{how_many}");
        }
    }

    /// **The number that picked the form is the number the sentence shows.**
    /// It is filled in from the same value, so no call site can pass four to
    /// the rules and write three on the screen — and how it is written is still
    /// whoever knows the region's business.
    #[test]
    fn the_number_shown_is_the_number_that_picked_the_form() {
        let strings = polish();
        let said = strings.count(
            &key("files.found"),
            &Counting::written_as(4_000_000, "4 000 000"),
            &Filling::of("how_many", "nine hundred"),
        );
        assert_eq!(said.text(), "4 000 000 plików");
        assert!(said.unfilled().is_empty());
    }

    /// A person reading a language nobody has translated gets English, in the
    /// English form for that number, and the answer says so.
    #[test]
    fn an_untranslated_countable_string_falls_back_to_the_source_in_the_sources_own_forms() {
        let mut strings = Strings::of(counting_vocabulary());
        strings.prefers(&[language("ga")]);
        assert_eq!(
            strings
                .count(&key("files.found"), &Counting::of(1), &Filling::nothing())
                .text(),
            "1 file"
        );
        let two = strings.count(&key("files.found"), &Counting::of(2), &Filling::nothing());
        assert_eq!(two.text(), "2 files");
        assert_eq!(two.came_from(), &CameFrom::TheSource);
        assert!(!two.is_translated());
    }

    /// **A language whose plural rules alo OS does not have is stepped over,
    /// not guessed at**, and the person meets the next language they read — or
    /// the source, which is honest — rather than a Icelandic sentence in
    /// whichever form English would have used.
    #[test]
    fn a_language_we_cannot_count_in_is_stepped_over() {
        let vocabulary = counting_vocabulary();
        // Icelandic can hold the plain strings; a countable one could not have
        // been checked into it at all.
        let icelandic = speaking(&vocabulary, "is", &[("files.gone", "Það er farið")]);
        let mut strings = Strings::of(vocabulary);
        strings.speaks(icelandic).unwrap();
        strings.prefers(&[language("is")]);
        assert_eq!(
            strings
                .count(&key("files.found"), &Counting::of(2), &Filling::nothing())
                .came_from(),
            &CameFrom::TheSource
        );
        assert!(
            strings
                .say(&key("files.gone"), &Filling::nothing())
                .is_translated(),
            "everything that does not count is still translated"
        );
    }

    /// A form the person's language does have but the translator has not
    /// written falls through to the next language, exactly as a missing plain
    /// string does.
    #[test]
    fn a_form_nobody_has_written_yet_falls_through() {
        let vocabulary = counting_vocabulary();
        let half = speaking(&vocabulary, "pl", &[("files.found.one", "1 plik")]);
        let mut strings = Strings::of(vocabulary);
        strings.speaks(half).unwrap();
        strings.prefers(&[language("pl")]);
        assert_eq!(
            strings
                .count(&key("files.found"), &Counting::of(1), &Filling::nothing())
                .text(),
            "1 plik"
        );
        let five = strings.count(&key("files.found"), &Counting::of(5), &Filling::nothing());
        assert_eq!(five.text(), "5 files");
        assert_eq!(five.came_from(), &CameFrom::TheSource);
    }

    /// Asking the wrong way round is a mistake in this repository, and it is
    /// reported the way every other one is: the key on the screen, and
    /// [`Said::is_a_bug`].
    #[test]
    fn asking_the_wrong_way_round_shows_the_key() {
        let strings = polish();
        let counted_as_plain = strings.say(&key("files.found"), &Filling::nothing());
        assert_eq!(counted_as_plain.text(), "«files.found»");
        assert!(counted_as_plain.is_a_bug());

        let plain_as_counted =
            strings.count(&key("files.gone"), &Counting::of(1), &Filling::nothing());
        assert_eq!(plain_as_counted.text(), "«files.gone»");
        assert!(plain_as_counted.is_a_bug());
    }

    /// **A half-translated countable string is not a translated one.** A Polish
    /// file holding `one` and `other` looks two thirds done and is missing the
    /// two forms most numbers take — so it is listed until every form that
    /// language needs is there.
    #[test]
    fn what_a_translator_is_handed_is_the_forms_their_own_language_needs() {
        let strings = polish();
        assert_eq!(
            strings.unanswered(),
            [key("files.gone")],
            "the countable one is complete in Polish"
        );
        assert_eq!(
            strings.missing_from(&language("pl")),
            [key("files.gone")],
            "one plain string left"
        );
        assert_eq!(
            strings.missing_from(&language("ga")),
            [
                key("files.found.few"),
                key("files.found.many"),
                key("files.found.one"),
                key("files.found.other"),
                key("files.found.two"),
                key("files.gone"),
            ],
            "Irish counts in five and has translated none of them"
        );
        assert!(
            !strings
                .missing_from(&language("pl"))
                .iter()
                .any(|key| key.as_str().ends_with(".other")),
            "Polish is never asked for a form no whole number reaches"
        );
    }

    /// A language whose rules nobody has read is handed the countable string
    /// under its own name: the first thing to do is add how the language
    /// counts, not translate five sentences into forms nothing would show.
    #[test]
    fn a_language_we_cannot_count_in_is_handed_the_string_itself() {
        let strings = polish();
        assert_eq!(
            strings.missing_from(&language("is")),
            [key("files.found"), key("files.gone")]
        );
    }

    /// The source language has to be countable, or every countable string falls
    /// back to a form nothing chose. English counts in two, which is what
    /// [`Plural`] is built on.
    #[test]
    fn the_source_language_counts_and_counts_in_two() {
        let source = Language::written(union::THE_SOURCE).unwrap();
        assert_eq!(cldr::forms(&source), Some(&[Form::One, Form::Other][..]));
        assert_eq!(source_form(&Counting::of(1)), Form::One);
        assert_eq!(source_form(&Counting::of(0)), Form::Other);
        assert_eq!(source_form(&Counting::of(2)), Form::Other);
    }

    /// A countable string is marked in a development build exactly as a plain
    /// one is, because falling back to English is the same gap either way.
    #[test]
    fn an_untranslated_countable_string_is_marked_in_development() {
        let mut strings = Strings::of(counting_vocabulary());
        strings.shown(Showing::InDevelopment);
        assert_eq!(
            strings
                .count(&key("files.found"), &Counting::of(3), &Filling::nothing())
                .text(),
            "«3 files»"
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
