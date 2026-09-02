//! The scaffolding, walked with the strings this repository is actually going
//! to have to move.
//!
//! `docs/autonomy/QUEUE.md` item 9 names three lists of hardcoded English —
//! `alo-files`' refusals and verb sentences, `alo-shortcuts`' action purposes
//! and key labels, and `alo-appearance`' colour names and errors — and the
//! whole risk of building scaffolding before its first user is that it turns
//! out not to fit them. So the awkward ones are carried through the whole path
//! here, verbatim, rather than being represented by strings invented to suit
//! the crate.
//!
//! These are copies. Moving the crates themselves onto `alo-strings` is items
//! 9b, 9c and 9d, and when a crate moves its strings leave here.
//!
//! **`alo-files` has moved (item 9b) and `alo-shortcuts` has moved (item 9c),
//! so their strings have gone.** They are declared for real in those crates'
//! own `words` modules now, and what they do in Polish, Irish, Latvian, Maltese
//! and German is asserted there, against the vocabulary the code actually uses
//! rather than against a copy of it — including the countable one, the refusal
//! a translator meets when they drop a gap out of a sentence somebody has to
//! approve, and a whole shortcuts panel read on a German keyboard. What is left
//! here is `alo-appearance`', which has not moved.
//!
//! Those are labels rather than sentences: no gaps, nothing counted, and one
//! word each where a word is the hardest thing to translate. That is why the
//! note is what this file now mostly asserts.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_strings::{
    CameFrom, Filling, Key, Language, Phrase, Showing, Strings, Translation, Vocabulary,
};

/// One of the test's own keys.
fn key(named: &str) -> Key {
    Key::named(named).unwrap()
}

/// One of the test's own languages.
fn language(tag: &str) -> Language {
    Language::written(tag).unwrap()
}

/// One of the strings this repository already has, declared.
fn phrase(vocabulary: &mut Vocabulary, named: &str, source: &str) {
    vocabulary
        .says(Phrase::says(key(named), source).unwrap())
        .unwrap();
}

/// The strings item 9d singles out in the one crate that still holds its own
/// English, as they are written today.
///
/// Two colour names and one refusal: short, and two of the three need a
/// decision rather than a dictionary.
fn what_the_repository_says() -> Vocabulary {
    let mut vocabulary = Vocabulary::empty();

    phrase(&mut vocabulary, "appearance.token.terracotta", "Terracotta");
    phrase(&mut vocabulary, "appearance.token.warm-stone", "Warm stone");
    phrase(&mut vocabulary, "appearance.accent.verdigris", "Verdigris");
    phrase(
        &mut vocabulary,
        "appearance.accent.not-offered",
        "{colour} is not one of the accents this system offers — choose verdigris, indigo, \
         violet, moss or rose, each of which is drawn to read on a light ground and on a dark one",
    );

    vocabulary
}

/// Three labels and one refusal: the labels name nothing, and the refusal names
/// the colour it is about, which is the thing a translator has to keep.
#[test]
fn every_string_the_repository_has_survives_being_named() {
    let vocabulary = what_the_repository_says();
    assert_eq!(vocabulary.how_many(), 4);
    let with_gaps = vocabulary
        .phrases()
        .filter(|phrase| !phrase.source().gaps().is_empty())
        .count();
    assert_eq!(
        with_gaps, 1,
        "a label names nothing and a refusal names one"
    );
    assert_eq!(vocabulary.counted().count(), 0, "and nothing counts");
}

/// A translation of them is checked and then shown, which is the whole path a
/// contributed language takes.
#[test]
fn a_translation_of_them_is_checked_and_then_shown() {
    let vocabulary = what_the_repository_says();
    let german = vocabulary
        .check(
            Translation::into_language(language("de"))
                .says(key("appearance.token.warm-stone"), "Warmer Stein"),
        )
        .unwrap();

    let mut strings = Strings::of(vocabulary);
    strings.speaks(german).unwrap();
    strings.prefers(&[language("de")]);

    let said = strings.say(&key("appearance.token.warm-stone"), &Filling::nothing());
    assert_eq!(said.text(), "Warmer Stein");
    assert!(said.is_translated());
    assert!(!said.is_a_bug());
}

/// **Item 9's test, on the real list.** Three of these four are not translated,
/// and a person building a German shell can see which three without knowing
/// what they were looking for.
#[test]
fn what_is_not_translated_yet_is_visible_rather_than_silently_english() {
    let vocabulary = what_the_repository_says();
    let german = vocabulary
        .check(
            Translation::into_language(language("de"))
                .says(key("appearance.token.warm-stone"), "Warmer Stein"),
        )
        .unwrap();

    let mut strings = Strings::of(vocabulary);
    strings.speaks(german).unwrap();
    strings.prefers(&[language("de")]);

    assert_eq!(strings.unanswered().len(), 3);

    strings.shown(Showing::InDevelopment);
    let terracotta = strings.say(&key("appearance.token.terracotta"), &Filling::nothing());
    assert_eq!(terracotta.text(), "«Terracotta»");
    assert_eq!(terracotta.came_from(), &CameFrom::TheSource);

    let translated = strings.say(&key("appearance.token.warm-stone"), &Filling::nothing());
    assert_eq!(translated.text(), "Warmer Stein");
}

/// A key that nothing declares is a mistake in this repository and says so,
/// rather than showing an empty line where a label should be. It is the failure
/// a crate that half-moved its strings would produce, and it is why 9c and 9d
/// are one crate each.
#[test]
fn a_label_nobody_declared_says_it_is_a_bug() {
    let strings = Strings::of(what_the_repository_says());
    let said = strings.say(&key("appearance.accent.moss"), &Filling::nothing());
    assert!(said.is_a_bug());
    assert_eq!(said.came_from(), &CameFrom::NoPhrase);
    assert_eq!(said.text(), "«appearance.accent.moss»");
}

/// The two `docs/autonomy/QUEUE.md` calls out as needing a translator's
/// judgement rather than their typing. Both are one word, and one word with no
/// note is where a translation goes wrong quietly.
///
/// The third one this file used to carry — `alo-shortcuts`' name for the key
/// between Ctrl and Alt — has moved into that crate, where it is a note on a
/// string the code actually uses.
#[test]
fn the_two_words_that_need_a_decision_carry_their_note() {
    let terracotta = Phrase::says(key("appearance.token.terracotta"), "Terracotta")
        .unwrap()
        .noting(
            "The colour of fired clay: an orange-brown. Several languages have no ordinary word \
             for it and the nearest loanword may name a different colour, so describe the colour \
             rather than borrowing the word.",
        )
        .unwrap();
    assert!(
        terracotta
            .note()
            .is_some_and(|note| note.contains("orange-brown"))
    );

    let verdigris = Phrase::says(key("appearance.accent.verdigris"), "Verdigris")
        .unwrap()
        .noting(
            "The blue-green of weathered copper — a church roof, an old statue. Two words in some \
             languages and none in others; this is a colour a person picks from a list, so name \
             it in whatever way they would recognise it.",
        )
        .unwrap();
    assert!(verdigris.note().is_some());
}
