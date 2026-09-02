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
    CameFrom, Counting, Filling, Form, Key, Language, Phrase, Plural, Showing, Strings,
    Translation, Vocabulary,
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
    // The one item 9a exists for: as `alo-files` writes it today this message
    // says "1 bytes" for a one-byte file, and it is three sentences in Polish.
    vocabulary
        .counts(
            Plural::counting(
                key("files.failed.too-big"),
                "bytes",
                "{path} holds one byte and a verb reads at most {most} — open it in an \
                 application, or ask for part of the folder instead",
                "{path} holds {bytes} bytes and a verb reads at most {most} — open it in an \
                 application, or ask for part of the folder instead",
            )
            .unwrap(),
        )
        .unwrap();
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
    assert_eq!(with_gaps, 2, "two of the plain ones name something");
    assert_eq!(
        vocabulary.counted().count(),
        1,
        "and one of them counts something"
    );
}

/// **The message item 9a exists for, in the three languages that make it worth
/// doing.** Polish counts in three and never says `other` about a whole number,
/// Irish counts in five, and Latvian has a word for none of them — and every
/// one of those is a sentence a person is shown when their disk says no.
#[test]
fn the_message_that_counts_is_counted_in_each_languages_own_way() {
    let vocabulary = what_the_repository_says();
    let key = key("files.failed.too-big");

    let polish = vocabulary
        .check(
            Translation::into_language(language("pl"))
                .says(key.for_form(Form::One), "{path} ma jeden bajt, a {most}")
                .says(key.for_form(Form::Few), "{path} ma {bytes} bajty, a {most}")
                .says(
                    key.for_form(Form::Many),
                    "{path} ma {bytes} bajtów, a {most}",
                ),
        )
        .unwrap();
    let irish = vocabulary
        .check(
            Translation::into_language(language("ga"))
                .says(key.for_form(Form::One), "{path}: beart amháin, {most}")
                .says(key.for_form(Form::Two), "{path}: dhá bheart, {most}")
                .says(key.for_form(Form::Few), "{path}: {bytes} bheart, {most}")
                .says(key.for_form(Form::Many), "{path}: {bytes} mbeart, {most}")
                .says(key.for_form(Form::Other), "{path}: {bytes} beart, {most}"),
        )
        .unwrap();

    let mut strings = Strings::of(vocabulary);
    strings.speaks(polish).unwrap();
    strings.speaks(irish).unwrap();

    let filling = Filling::of("path", "/home/ada/notes.txt").and("most", "1 000 000");

    strings.prefers(&[language("pl")]);
    for (how_many, expected) in [
        (1_u64, "/home/ada/notes.txt ma jeden bajt, a 1 000 000"),
        (3, "/home/ada/notes.txt ma 3 bajty, a 1 000 000"),
        (7, "/home/ada/notes.txt ma 7 bajtów, a 1 000 000"),
    ] {
        let said = strings.count(&key, &Counting::of(how_many), &filling);
        assert_eq!(said.text(), expected, "{how_many}");
        assert!(said.is_translated() && !said.is_a_bug(), "{how_many}");
    }

    strings.prefers(&[language("ga")]);
    for (how_many, expected) in [
        (1_u64, "/home/ada/notes.txt: beart amháin, 1 000 000"),
        (2, "/home/ada/notes.txt: dhá bheart, 1 000 000"),
        (4, "/home/ada/notes.txt: 4 bheart, 1 000 000"),
        (8, "/home/ada/notes.txt: 8 mbeart, 1 000 000"),
        (40, "/home/ada/notes.txt: 40 beart, 1 000 000"),
    ] {
        assert_eq!(
            strings
                .count(&key, &Counting::of(how_many), &filling)
                .text(),
            expected,
            "{how_many}"
        );
    }

    // Latvian has translated none of it, and what a translator is handed is the
    // three forms Latvian uses — one of which is a word for none.
    assert_eq!(
        strings
            .missing_from(&language("lv"))
            .iter()
            .filter(|missing| missing.as_str().starts_with("files.failed.too-big"))
            .map(Key::to_string)
            .collect::<Vec<String>>(),
        [
            "files.failed.too-big.one",
            "files.failed.too-big.other",
            "files.failed.too-big.zero",
        ]
    );
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
                    key("files.failed.too-big").for_form(Form::Other),
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

    let said = strings.count(
        &key("files.failed.too-big"),
        &Counting::written_as(4_000_000, "4 000 000"),
        &Filling::of("path", "/home/ada/notes.txt").and("most", "1 000 000"),
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
            key("files.failed.too-big").for_form(Form::Other),
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
