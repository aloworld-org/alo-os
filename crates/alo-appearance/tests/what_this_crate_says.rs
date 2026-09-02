//! Everything this crate can say, put through the whole path a translation
//! takes: declared, checked, partly translated, and drawn as the picker a person
//! actually reads.
//!
//! The crate's own tests take one string at a time. This is the other half: the
//! real vocabulary — not a fixture that resembles it — walked as a whole
//! appearance panel, which is eleven colour names and the sentence that comes
//! back when one of them cannot be had.
//!
//! **German for both halves of this file, which is a departure.**
//! `alo-shortcuts`' test reads a full panel in German and a half-finished one in
//! Maltese, because a keyboard is printed differently in each country. Colours
//! are not: what makes this list hard is that a language either has a word for a
//! colour or does not, and German is the language whose answers the notes in
//! `alo_appearance::words` were written against. It has an ordinary word where
//! English borrows one — *Grünspan* for verdigris, *Anthrazit* for charcoal,
//! which is a mineral rather than burnt wood — and borrows where English has an
//! ordinary word. Writing the second half in a language nobody here reads would
//! have been a test of our guessing rather than of this crate.
//!
//! `alo-strings`' own integration test carried copies of four of these strings,
//! because it was built before any of its users existed. All three of them exist
//! now, so that file has gone and this is where its last four tests live: a
//! translation checked and shown, what is not translated being visible, a key
//! nobody declared saying it is a bug, and the two words that need a decision
//! rather than a dictionary carrying their note.
//!
//! It is not the hardware verification `CLAUDE.md` asks for. Nothing here has
//! been seen: there is no compositor, no screen, and there are still no
//! translations in this repository.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_appearance::words::{self, EVERY_WORD};
use alo_appearance::{
    Accent, AccentError, Colour, DisplayId, TextScale, Token, Word, appearance_words,
};
use alo_strings::{CameFrom, Filling, Language, Phrase, Showing, Strings, Translation, Vocabulary};

/// One of the tests' languages.
fn language(tag: &str) -> Language {
    Language::written(tag).unwrap()
}

/// This crate's words, with nothing translated.
fn in_english() -> Strings {
    Strings::of(appearance_words().unwrap())
}

/// This crate's words, with these translated into the given language and that
/// language preferred.
fn reading(tag: &str, words: &[(Word, &str)]) -> Strings {
    let vocabulary = appearance_words().unwrap();
    let mut translation = Translation::into_language(language(tag));
    for (word, says) in words {
        translation = translation.says(word.key(), *says);
    }
    let speaking = vocabulary.check(translation).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[language(tag)]);
    strings
}

/// Every colour this crate names, in German. Nothing else is translated, which
/// is what the half-translated test below counts.
fn die_farben() -> Strings {
    reading(
        "de",
        &[
            (words::NAVY, "Marineblau"),
            (words::TERRACOTTA, "Terrakotta"),
            (words::CREAM, "Cremeweiß"),
            (words::PORCELAIN, "Porzellanweiß"),
            (words::CHARCOAL, "Anthrazit"),
            (words::WARM_STONE, "Warmer Stein"),
            (words::VERDIGRIS, "Grünspan"),
            (words::INDIGO, "Indigo"),
            (words::VIOLET, "Violett"),
            (words::MOSS, "Moosgrün"),
            (words::ROSE, "Altrosa"),
        ],
    )
}

/// **Every string this crate can say declares**, and it declares into a
/// vocabulary that already holds somebody else's.
///
/// The area at the front of a key is what makes that safe, and this is the test
/// that says so: a shell has one vocabulary, and every crate puts its own into
/// it.
#[test]
fn everything_this_crate_says_joins_one_vocabulary_beside_another_crate() {
    let mut vocabulary = Vocabulary::empty();
    vocabulary
        .says(
            Phrase::says(
                alo_strings::Key::named("shortcuts.action.the-agent").unwrap(),
                "Ask the agent",
            )
            .unwrap(),
        )
        .unwrap();
    alo_appearance::declare_into(&mut vocabulary).unwrap();

    assert_eq!(vocabulary.how_many(), EVERY_WORD.len() + 1);
    assert_eq!(vocabulary.counted().count(), 0, "nothing here counts");
    for word in EVERY_WORD {
        assert!(vocabulary.phrase(&word.key()).is_some(), "{}", word.key());
    }
}

/// **The whole colour picker, read on a German machine.** Eleven rows: the six
/// the system is built out of and the five a person can choose between, each
/// named the way somebody reading German would name that colour rather than the
/// way English does.
///
/// *Grünspan* and *Anthrazit* are the two that make the point. English borrowed
/// verdigris from French and named a grey after burnt wood; German has an
/// ordinary word for the first and names the second after a mineral. Neither
/// list could have been reached from the other by translating word for word,
/// which is what the notes on these eleven strings exist to say.
#[test]
fn the_whole_colour_picker_is_read_in_the_language_the_person_reads() {
    let strings = die_farben();

    let built_out_of: Vec<String> = Token::ALL
        .iter()
        .map(|token| token.said(&strings).into_text())
        .collect();
    assert_eq!(
        built_out_of,
        [
            "Marineblau",
            "Terrakotta",
            "Cremeweiß",
            "Porzellanweiß",
            "Anthrazit",
            "Warmer Stein",
        ]
    );

    let to_choose_from: Vec<String> = Accent::ALL
        .iter()
        .map(|accent| accent.said(&strings).into_text())
        .collect();
    assert_eq!(
        to_choose_from,
        ["Grünspan", "Indigo", "Violett", "Moosgrün", "Altrosa"]
    );

    for token in Token::ALL {
        assert!(token.said(&strings).is_translated(), "{token:?}");
    }
    for accent in Accent::ALL {
        assert!(accent.said(&strings).is_translated(), "{accent:?}");
    }
}

/// **A refusal and everything inside it are in one language.** The colour the
/// sentence is about is one of this crate's own strings, so a German machine
/// does not read a German sentence with an English colour in the middle of it —
/// which is the property `alo-shortcuts` established for an action's name inside
/// a clash, reaching the colours.
#[test]
fn a_refusal_and_the_colour_inside_it_are_in_one_language() {
    let strings = reading(
        "de",
        &[
            (words::CHARCOAL, "Anthrazit"),
            (
                words::NOT_AN_ACCENT,
                "{colour} ist eine Grund- oder Strukturfarbe und keine Akzentfarbe — wählen Sie \
                 Grünspan, Indigo, Violett, Moosgrün oder Altrosa",
            ),
        ],
    );
    let said = Accent::of_colour(Token::Charcoal.colour())
        .unwrap_err()
        .said(&strings);
    assert_eq!(
        said.text(),
        "Anthrazit ist eine Grund- oder Strukturfarbe und keine Akzentfarbe — wählen Sie \
         Grünspan, Indigo, Violett, Moosgrün oder Altrosa"
    );
    assert!(said.is_translated());
    assert!(said.unfilled().is_empty());
}

/// **What is inside a refusal and is not ours stays as it is.** A hex somebody
/// typed, a screen's own name and a file off their disk are theirs, whatever
/// language the sentence around them is written in — and a translator is told so
/// in the note on each.
#[test]
fn what_came_off_somebodys_own_machine_is_not_translated() {
    let strings = die_farben();

    let invented = Colour::written("#123456").unwrap();
    let refused = Accent::of_colour(invented).unwrap_err();
    assert_eq!(refused, AccentError::NotOffered(invented));
    assert!(refused.said(&strings).text().contains("#123456"));

    let spaced = DisplayId::named("DP-1 ").unwrap_err().said(&strings);
    assert!(spaced.text().contains("\"DP-1 \""), "{spaced}");

    let mistyped = Colour::written("blau").unwrap_err().said(&strings);
    assert!(mistyped.text().contains("blau"), "{mistyped}");
}

/// **A half-translated panel says which half.** A shell being built in German
/// can count what is left without knowing what it was looking for, and what
/// reaches a person meanwhile is marked in development rather than passed off as
/// German.
#[test]
fn what_nobody_has_translated_yet_is_visible_rather_than_silently_english() {
    let mut strings = die_farben();
    let colours = 11;
    assert_eq!(strings.unanswered().len(), EVERY_WORD.len() - colours);
    assert_eq!(
        strings.missing_from(&language("de")).len(),
        EVERY_WORD.len() - colours
    );

    strings.shown(Showing::InDevelopment);
    let translated = Token::Terracotta.said(&strings);
    assert_eq!(translated.text(), "Terrakotta");
    assert_eq!(
        translated.came_from(),
        &CameFrom::Translation(language("de"))
    );

    let untranslated = TextScale::percent(10).unwrap_err().said(&strings);
    assert_eq!(
        untranslated.text(),
        "«10% is smaller than this screen can be read at — 75% is as small as it goes»"
    );
    assert_eq!(untranslated.came_from(), &CameFrom::TheSource);
    assert!(untranslated.unfilled().is_empty(), "and it is still filled");
}

/// A key that nothing declares is a mistake in this repository and says so,
/// rather than showing an empty row where a colour should be. It is the failure
/// a crate that half-moved its strings would produce, which is why 9b, 9c and 9d
/// were one crate each.
#[test]
fn a_colour_nobody_declared_says_it_is_a_bug() {
    let strings = in_english();
    let ochre = alo_strings::Key::named("appearance.accent.ochre").unwrap();
    let said = strings.say(&ochre, &Filling::nothing());
    assert!(said.is_a_bug());
    assert_eq!(said.came_from(), &CameFrom::NoPhrase);
    assert_eq!(said.text(), "«appearance.accent.ochre»");
}

/// **The two words `docs/autonomy/QUEUE.md` called out as needing a decision
/// rather than a dictionary carry their note**, and so does every other string
/// in this crate. One word with no note is where a translation goes wrong
/// quietly: nobody here can read the language it went wrong in.
#[test]
fn the_words_that_need_a_decision_carry_their_note() {
    let vocabulary = appearance_words().unwrap();
    for word in EVERY_WORD {
        let phrase = vocabulary.phrase(&word.key()).unwrap();
        assert!(phrase.note().is_some(), "{}", word.key());
    }

    let terracotta = vocabulary.phrase(&words::TERRACOTTA.key()).unwrap();
    assert!(
        terracotta
            .note()
            .is_some_and(|note| note.contains("orange-brown")),
        "the colour is described rather than named"
    );
    let verdigris = vocabulary.phrase(&words::VERDIGRIS.key()).unwrap();
    assert!(
        verdigris
            .note()
            .is_some_and(|note| note.contains("weathered copper"))
    );
}

/// A machine with no translations at all is the machine this repository ships
/// today, and on it every one of these keys still answers with the string the
/// code declared rather than with the key.
///
/// The gaps are deliberately left empty here — this asks whether the string is
/// *there*, and a sentence with `{colour}` still in it is what an unfilled gap
/// looks like. That the callers fill them is each error type's own test.
#[test]
fn with_no_translations_at_all_every_string_is_still_a_string() {
    let strings = in_english();
    for word in EVERY_WORD {
        let said = strings.say(&word.key(), &Filling::nothing());
        assert_eq!(said.came_from(), &CameFrom::TheSource, "{}", word.key());
        assert_eq!(said.text(), word.says(), "{}", word.key());
    }
}
