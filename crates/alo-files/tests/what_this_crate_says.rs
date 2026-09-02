//! Everything this crate can say, put through the whole path a translation
//! takes: declared, checked, partly translated, counted, and shown.
//!
//! The crate's own tests take one sentence at a time. This is the other half:
//! the real vocabulary — not a fixture that resembles it — walked in the
//! languages that make the exercise worth doing. Polish counts in three for a
//! whole number and never says `other` about one; Irish counts in five; Latvian
//! has a form for none at all; and German moves the words of an approval
//! sentence around the two paths in it.
//!
//! `alo-strings`' own integration test used to carry copies of three of these
//! strings, because it was built before its first user existed. They have left
//! it, which is what that file says happens when a crate moves.
//!
//! It is not the hardware verification `CLAUDE.md` asks for. Nothing here has
//! been read by anybody: there is no screen, and there are still no
//! translations in this repository.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_capability::Given;
use alo_files::words::{self, EVERY_WORD, TOO_BIG};
use alo_files::{Failed, file_verbs, file_words};
use alo_strings::{
    Counting, Filling, Form, Key, Language, Showing, Strings, Translation, Vocabulary,
};

/// One of the tests' languages.
fn language(tag: &str) -> Language {
    Language::written(tag).unwrap()
}

/// This crate's words, with nothing translated.
fn in_english() -> Strings {
    Strings::of(file_words().unwrap())
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
            alo_strings::Phrase::says(Key::named("shortcuts.modifier.super").unwrap(), "Super")
                .unwrap(),
        )
        .unwrap();
    words::declare_into(&mut vocabulary).unwrap();

    assert_eq!(vocabulary.how_many(), EVERY_WORD.len() + 2);
    assert_eq!(vocabulary.counted().count(), 1);
    for word in EVERY_WORD {
        assert!(vocabulary.phrase(&word.key()).is_some(), "{}", word.key());
    }
}

/// **The message about a file that is too big, counted each language's own
/// way.** As this crate wrote it before `alo-strings` existed it said "1 bytes"
/// for a one-byte file, in English, on every machine.
#[test]
fn the_size_of_a_file_is_said_in_the_readers_own_forms() {
    let vocabulary = file_words().unwrap();
    let key = TOO_BIG.key();
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

    let too_big = |bytes: u64| Failed::TooBig {
        path: "/home/anna/Invoices/scan.tiff".to_owned(),
        bytes,
        most: 1_048_576,
    };

    strings.prefers(&[language("pl")]);
    for (bytes, expected) in [
        (
            1_u64,
            "/home/anna/Invoices/scan.tiff ma jeden bajt, a 1048576",
        ),
        (3, "/home/anna/Invoices/scan.tiff ma 3 bajty, a 1048576"),
        (7, "/home/anna/Invoices/scan.tiff ma 7 bajtów, a 1048576"),
    ] {
        let said = too_big(bytes).said(&strings);
        assert_eq!(said.text(), expected, "{bytes}");
        assert!(said.is_translated() && !said.is_a_bug(), "{bytes}");
    }

    strings.prefers(&[language("ga")]);
    for (bytes, expected) in [
        (
            1_u64,
            "/home/anna/Invoices/scan.tiff: beart amháin, 1048576",
        ),
        (2, "/home/anna/Invoices/scan.tiff: dhá bheart, 1048576"),
        (4, "/home/anna/Invoices/scan.tiff: 4 bheart, 1048576"),
        (8, "/home/anna/Invoices/scan.tiff: 8 mbeart, 1048576"),
        (40, "/home/anna/Invoices/scan.tiff: 40 beart, 1048576"),
    ] {
        assert_eq!(too_big(bytes).said(&strings).text(), expected, "{bytes}");
    }

    // Latvian has none of it, and what a translator is handed is the three
    // forms Latvian uses — one of which is a word for none at all.
    assert_eq!(
        strings
            .missing_from(&language("lv"))
            .iter()
            .filter(|missing| missing.as_str().starts_with(TOO_BIG.key().as_str()))
            .map(Key::to_string)
            .collect::<Vec<String>>(),
        [
            "files.failed.too-big.one",
            "files.failed.too-big.other",
            "files.failed.too-big.zero",
        ]
    );
}

/// **The sentence a person approves, translated, with the arguments still in
/// it.** `alo_capability::Verb::checked` refuses an English sentence that does
/// not name every argument, because a person approves the sentence; this is
/// that same rule holding in German, enforced by the crate whose job it is.
#[test]
fn an_approval_sentence_survives_translation_with_every_argument_in_it() {
    let vocabulary = file_words().unwrap();
    let german = vocabulary
        .check(
            Translation::into_language(language("de"))
                .says(
                    words::ARCHIVE_FOLDER_SENTENCE.key(),
                    "ein Archiv von {folder} namens {name} in {into} anlegen",
                )
                .says(
                    words::MOVE_FILE_SENTENCE.key(),
                    "{file} nach {into} verschieben",
                ),
        )
        .unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(german).unwrap();
    strings.prefers(&[language("de")]);

    let call = file_verbs()
        .unwrap()
        .call(
            "archive_folder",
            &[
                ("folder", Given::text("/home/anna/Invoices")),
                ("into", Given::text("/home/anna/Archive")),
                ("name", Given::text("invoices-2026.zip")),
            ],
        )
        .unwrap();
    let said = call.sentence(&strings);
    assert_eq!(
        said.text(),
        "ein Archiv von /home/anna/Invoices namens invoices-2026.zip in /home/anna/Archive anlegen"
    );
    assert!(said.is_translated());
    assert!(said.unfilled().is_empty(), "{:?}", said.unfilled());

    // And a German sentence that dropped one of the three would never have
    // loaded, so nobody could be shown it.
    let wrongs = file_words()
        .unwrap()
        .check(Translation::into_language(language("de")).says(
            words::ARCHIVE_FOLDER_SENTENCE.key(),
            "ein Archiv von {folder} in {into} anlegen",
        ))
        .unwrap_err();
    assert!(
        wrongs
            .to_string()
            .contains("put {name} back into the sentence"),
        "{wrongs}"
    );
}

/// **A half-translated crate says which half.** A shell being built in Maltese
/// can count what is left without knowing what it was looking for, and what
/// reaches a person meanwhile is marked in development rather than passed off
/// as Maltese.
#[test]
fn what_nobody_has_translated_yet_is_visible_rather_than_silently_english() {
    let vocabulary = file_words().unwrap();
    let maltese = vocabulary
        .check(
            Translation::into_language(language("mt"))
                .says(words::NOT_A_FOLDER.key(), "{path} mhuwiex folder")
                .says(words::READ_FILE.key(), "aqra x'hemm f'fajl"),
        )
        .unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(maltese).unwrap();
    strings.prefers(&[language("mt")]);

    // Two of the plain ones are answered; the countable one is not answered at
    // all, and is listed once under its own name.
    assert_eq!(strings.unanswered().len(), EVERY_WORD.len() + 1 - 2);

    strings.shown(Showing::InDevelopment);
    let gone = Failed::Gone {
        path: "/home/anna/Invoices/march.pdf".to_owned(),
    }
    .said(&strings);
    assert!(gone.text().starts_with('«'), "{gone}");
    assert!(!gone.is_translated());

    let not_a_folder = Failed::NotAFolder {
        path: "/home/anna/Invoices/march.pdf".to_owned(),
    }
    .said(&strings);
    assert_eq!(
        not_a_folder.text(),
        "/home/anna/Invoices/march.pdf mhuwiex folder"
    );
}

/// A machine with no translations at all is the machine this repository ships
/// today, and on it every one of these keys still answers with the sentence the
/// code declared rather than with the key.
///
/// The gaps are deliberately left empty here — this asks whether the string is
/// *there*, and a sentence with `{path}` still in it is what an unfilled gap
/// looks like. That the callers fill them is
/// `failed::tests::every_failure_is_something_this_crate_can_say`'s.
#[test]
fn with_no_translations_at_all_every_sentence_is_still_a_sentence() {
    let strings = in_english();
    for word in EVERY_WORD {
        let said = strings.say(&word.key(), &Filling::nothing());
        assert_eq!(
            said.came_from(),
            &alo_strings::CameFrom::TheSource,
            "{}",
            word.key()
        );
        assert_eq!(said.text(), word.says(), "{}", word.key());
    }
    let said = strings.count(&TOO_BIG.key(), &Counting::of(1), &Filling::nothing());
    assert!(said.text().contains("one byte"), "{said}");
    assert_eq!(said.came_from(), &alo_strings::CameFrom::TheSource);
}
