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

/// The strings item 9 singles out, from all three crates, as they are written
/// today.
fn what_the_repository_says() -> Vocabulary {
    let mut vocabulary = Vocabulary::empty();

    // alo-files: the longest list, and the one with the most gaps in it.
    phrase(
        &mut vocabulary,
        "files.failed.not-a-folder",
        "{path} is not a folder — read it if it is a file, or list the folder it is in",
    );
    phrase(
        &mut vocabulary,
        "files.failed.too-big",
        "{path} holds {bytes} bytes and a verb reads at most {most} — open it in an application, \
         or ask for part of the folder instead",
    );
    phrase(
        &mut vocabulary,
        "files.failed.into-itself",
        "the archive would be written inside {folder}, which is the folder it is an archive of — \
         put it somewhere that is not inside it",
    );

    // alo-shortcuts: what a person reads every time they open the panel.
    phrase(
        &mut vocabulary,
        "shortcuts.action.snap-left",
        "Put the window on the left half",
    );
    phrase(&mut vocabulary, "shortcuts.modifier.super", "Super");

    // alo-appearance: short, and two of them need a decision rather than a
    // dictionary.
    phrase(&mut vocabulary, "appearance.token.terracotta", "Terracotta");
    phrase(&mut vocabulary, "appearance.token.warm-stone", "Warm stone");

    vocabulary
}

/// **The sentences a person approves are among these**, which is why the gaps
/// are checked rather than trusted: `alo-capability` refuses a verb whose
/// sentence does not name every argument, and a translation that dropped one
/// would put that refusal back on the wrong side of the boundary.
#[test]
fn every_string_the_repository_has_survives_being_named() {
    let vocabulary = what_the_repository_says();
    assert_eq!(vocabulary.how_many(), 7);
    let with_gaps = vocabulary
        .phrases()
        .filter(|phrase| !phrase.source().gaps().is_empty())
        .count();
    assert_eq!(with_gaps, 3, "three of them name something");
}

/// A translation of the awkward ones, checked and shown. The German is real
/// enough to move the gaps around, which is the thing the check exists for.
#[test]
fn a_translation_of_them_is_checked_and_then_shown() {
    let vocabulary = what_the_repository_says();
    let german = vocabulary
        .check(
            Translation::into_language(language("de"))
                .says(
                    key("files.failed.too-big"),
                    "{path} ist {bytes} Bytes groß, und ein Verb liest höchstens {most} — öffnen \
                     Sie die Datei in einer Anwendung, oder fragen Sie nach einem Teil des Ordners",
                )
                .says(
                    key("shortcuts.action.snap-left"),
                    "Fenster auf die linke Hälfte legen",
                ),
        )
        .unwrap();

    let mut strings = Strings::of(vocabulary);
    strings.speaks(german).unwrap();
    strings.prefers(&[language("de")]);

    let said = strings.say(
        &key("files.failed.too-big"),
        &Filling::of("path", "/home/ada/notes.txt")
            .and("bytes", "4 000 000")
            .and("most", "1 000 000"),
    );
    assert!(
        said.text()
            .starts_with("/home/ada/notes.txt ist 4 000 000 Bytes groß")
    );
    assert!(said.text().contains("höchstens 1 000 000"));
    assert!(said.is_translated());
    assert!(!said.is_a_bug());
}

/// **Item 9's test, on the real list.** Four of these seven are not translated,
/// and a person building a German shell can see which four without knowing what
/// they were looking for.
#[test]
fn what_is_not_translated_yet_is_visible_rather_than_silently_english() {
    let vocabulary = what_the_repository_says();
    let german = vocabulary
        .check(Translation::into_language(language("de")).says(
            key("shortcuts.action.snap-left"),
            "Fenster auf die linke Hälfte legen",
        ))
        .unwrap();

    let mut strings = Strings::of(vocabulary);
    strings.speaks(german).unwrap();
    strings.prefers(&[language("de")]);

    assert_eq!(strings.unanswered().len(), 6);

    strings.shown(Showing::InDevelopment);
    let terracotta = strings.say(&key("appearance.token.terracotta"), &Filling::nothing());
    assert_eq!(terracotta.text(), "«Terracotta»");
    assert_eq!(terracotta.came_from(), &CameFrom::TheSource);

    let translated = strings.say(&key("shortcuts.action.snap-left"), &Filling::nothing());
    assert_eq!(translated.text(), "Fenster auf die linke Hälfte legen");
}

/// The refusal that matters most for these strings: a translator who moved
/// *bytes* out of the sentence would leave a person told their file is too big
/// and not told how big, in their own language, with nothing anywhere saying so.
#[test]
fn a_translation_that_lost_the_size_is_refused_before_anybody_sees_it() {
    let wrongs = what_the_repository_says()
        .check(Translation::into_language(language("de")).says(
            key("files.failed.too-big"),
            "{path} ist zu groß — öffnen Sie die Datei in einer Anwendung",
        ))
        .unwrap_err();
    assert_eq!(wrongs.how_many(), 2);
    let said = wrongs.to_string();
    assert!(
        said.contains("put {bytes} back into the sentence"),
        "{said}"
    );
    assert!(said.contains("put {most} back into the sentence"), "{said}");
}

/// The two `docs/autonomy/QUEUE.md` calls out as needing a translator's
/// judgement rather than their typing. Both are one word, and one word with no
/// note is where a translation goes wrong quietly.
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

    let modifier = Phrase::says(key("shortcuts.modifier.super"), "Super")
        .unwrap()
        .noting(
            "The key between Ctrl and Alt. Name it whatever is printed on it on the keyboards \
             most of your readers have, which is not always what we call it.",
        )
        .unwrap();
    assert!(modifier.note().is_some());
}
