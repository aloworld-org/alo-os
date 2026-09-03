//! Everything this crate can say, put through the whole path a translation
//! takes: declared, checked, partly translated, counted, and shown.
//!
//! The crate's own tests take one sentence at a time. This is the other half:
//! the real vocabulary — not a fixture that resembles it — walked in languages
//! that make the exercise worth doing. German puts the verb where English does
//! not; Polish counts in three forms where English has two, so a countable
//! string written against English's habits fails here rather than in Warsaw.
//!
//! It is not the hardware verification `CLAUDE.md` asks for. Nothing here has
//! been read by anybody: there is no screen, and there are still no
//! translations in this repository.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None, Err or index is the failure being reported"
)]

use alo_context::words::{self, EVERY_WORD, SELECTION_SHORTENED};
use alo_context::{Context, Document, Focused, MOST, NotOffered, Selection, context_words};
use alo_strings::{
    CameFrom, Filling, Form, Key, Language, Phrase, Showing, Strings, Translation, Vocabulary,
};
use std::time::{Duration, SystemTime};

/// One of the tests' languages.
fn language(tag: &str) -> Language {
    Language::written(tag).unwrap()
}

/// This crate's words, with nothing translated.
fn in_english() -> Strings {
    Strings::of(context_words().unwrap())
}

/// A fixed moment, so nothing here waits for anything.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// **Every string this crate can say declares**, and it declares into a
/// vocabulary that already holds somebody else's.
///
/// The area at the front of a key is what makes that safe: a shell has one
/// vocabulary, and every crate puts its own into it.
#[test]
fn everything_this_crate_says_joins_one_vocabulary_beside_another_crates() {
    let mut vocabulary = Vocabulary::empty();
    vocabulary
        .says(
            Phrase::says(
                Key::named("files.verb.read-file.purpose").unwrap(),
                "read what is in a file",
            )
            .unwrap(),
        )
        .unwrap();
    words::declare_into(&mut vocabulary).unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();

    for word in EVERY_WORD {
        assert!(vocabulary.phrase(&word.key()).is_some(), "{}", word.key());
    }
    assert!(vocabulary.plural(&SELECTION_SHORTENED.key()).is_some());
    assert!(
        vocabulary
            .phrase(&Key::named("files.verb.read-file.purpose").unwrap())
            .is_some()
    );
}

/// **The rows a person reads before they answer, in their own language.**
/// This is the visible half of *context is offered, never watched*: if the
/// rows do not read properly in Greek then a Greek speaker cannot check the
/// promise, and an unverifiable promise is just a sentence in a document.
#[test]
fn what_was_offered_reads_as_rows_in_the_readers_own_language() {
    let vocabulary = context_words().unwrap();
    let german = vocabulary
        .check(
            Translation::into_language(language("de"))
                .says(words::THE_DOCUMENT.key(), "geöffnetes Dokument: {document}")
                .says(words::THE_SELECTION.key(), "der markierte Text")
                .says(words::THE_WINDOW.key(), "das Fenster davor: {window}")
                .says(words::WINDOW_CALLED.key(), "{title} – {application}"),
        )
        .unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(german).unwrap();
    strings.prefers(&[language("de")]);

    let rows = Context::at_invocation(noon())
        .and_document(Document::open("/home/anna/Rechnungen/März.pdf").unwrap())
        .and_selection(Selection::of("die Rechnung von März").unwrap())
        .and_window(Focused::titled("org.blender.Blender", "untitled.blend").unwrap())
        .shown(&strings);

    assert_eq!(rows.len(), 3);
    assert!(rows.iter().all(alo_strings::Said::is_translated));
    assert_eq!(
        rows[0].text(),
        "geöffnetes Dokument: /home/anna/Rechnungen/März.pdf"
    );
    assert_eq!(rows[1].text(), "der markierte Text");
    assert_eq!(
        rows[2].text(),
        "das Fenster davor: untitled.blend – org.blender.Blender"
    );
}

/// **A translation that dropped the path would never have loaded.** The row
/// exists to say *which* document went with the question, and one that named no
/// document would be worse than no row at all.
#[test]
fn a_row_that_lost_the_thing_it_names_is_refused_before_anybody_reads_it() {
    let wrongs = context_words()
        .unwrap()
        .check(
            Translation::into_language(language("de"))
                .says(words::THE_DOCUMENT.key(), "das geöffnete Dokument"),
        )
        .unwrap_err();
    assert!(wrongs.to_string().contains("{document}"), "{wrongs}");
}

/// **How much of a selection was left out is counted the reader's own way.**
/// Polish has three forms for a whole number where English has two, so a
/// sentence built by sticking a number into one English string is wrong in
/// Poland for every number except one — and CLDR, not memory, is where the
/// three come from.
#[test]
fn what_was_left_out_of_a_selection_is_counted_in_the_readers_own_forms() {
    let vocabulary = context_words().unwrap();
    let polish = vocabulary
        .check(
            Translation::into_language(language("pl"))
                .says(
                    SELECTION_SHORTENED.key().for_form(Form::One),
                    "pominięto {characters} znak",
                )
                .says(
                    SELECTION_SHORTENED.key().for_form(Form::Few),
                    "pominięto {characters} znaki",
                )
                .says(
                    SELECTION_SHORTENED.key().for_form(Form::Many),
                    "pominięto {characters} znaków",
                ),
        )
        .unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(polish).unwrap();
    strings.prefers(&[language("pl")]);

    let said = |over: usize| {
        Selection::of(&"a".repeat(MOST + over))
            .unwrap()
            .shortened(&strings)
            .unwrap()
            .into_text()
    };
    assert_eq!(said(1), "pominięto 1 znak");
    assert_eq!(said(2), "pominięto 2 znaki");
    assert_eq!(said(7), "pominięto 7 znaków");
}

/// **A half-translated crate says which half.** A shell being built in Greek
/// can count what is left, and what reaches a person meanwhile is marked in
/// development rather than passed off as Greek.
#[test]
fn what_nobody_has_translated_yet_is_visible_rather_than_silently_english() {
    let vocabulary = context_words().unwrap();
    let greek = vocabulary
        .check(
            Translation::into_language(language("el"))
                .says(
                    words::NOTHING_OFFERED.key(),
                    "τίποτα από την οθόνη σας δεν στάλθηκε",
                )
                .says(words::THE_SELECTION.key(), "το κείμενο που είχατε επιλέξει"),
        )
        .unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(greek).unwrap();
    strings.prefers(&[language("el")]);

    // Two of the plain strings are done. The countable one is not, and it
    // counts as one thing owed however many forms Greek needs for it.
    assert_eq!(strings.unanswered().len(), EVERY_WORD.len() + 1 - 2);

    let rows = Context::at_invocation(noon()).shown(&strings);
    assert!(rows[0].is_translated());
    assert_eq!(rows[0].text(), "τίποτα από την οθόνη σας δεν στάλθηκε");

    // One nobody has reached yet is marked, rather than looking like Greek
    // somebody wrote.
    strings.shown(Showing::InDevelopment);
    let untranslated = NotOffered::NoDocument.said(&strings);
    assert!(untranslated.text().starts_with('«'), "{untranslated}");
    assert!(!untranslated.is_translated());
}

/// A machine with no translations at all is the machine this repository ships
/// today, and on it every one of these keys still answers with the sentence the
/// code declared rather than with the key.
///
/// The gaps are deliberately left empty here — this asks whether the string is
/// *there*, and a sentence with `{document}` still in it is what an unfilled
/// gap looks like. That the callers fill them is the crate's own tests'.
#[test]
fn with_no_translations_at_all_every_sentence_is_still_a_sentence() {
    let strings = in_english();
    for word in EVERY_WORD {
        let said = strings.say(&word.key(), &Filling::nothing());
        assert_eq!(said.came_from(), &CameFrom::TheSource, "{}", word.key());
        assert_eq!(said.text(), word.says(), "{}", word.key());
    }
    // And the honest count of what is owed, the countable string included.
    assert_eq!(strings.unanswered().len(), EVERY_WORD.len() + 1);
}

/// **Nothing this crate can say is a key nobody declared** — the test every
/// crate that declares words owes, asked of the real vocabulary rather than of
/// a fixture.
///
/// Every refusal and every row is walked, because a string that reaches a
/// person as `«context.something»` is a bug they meet at the worst moment: the
/// one where they are deciding whether to hand over a document.
#[test]
fn nothing_this_crate_says_is_a_key_nobody_declared() {
    let strings = in_english();

    for why in [
        NotOffered::NoWindow,
        NotOffered::NotAnIdentifier {
            offered: "/usr/bin/blender".to_owned(),
        },
        NotOffered::NoDocument,
        NotOffered::NotAFullPath {
            offered: "march.pdf".to_owned(),
        },
        NotOffered::CouldLeadElsewhere {
            offered: "/home/anna/../root".to_owned(),
        },
        NotOffered::NotADocument {
            offered: "/".to_owned(),
        },
    ] {
        let said = why.said(&strings);
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.unfilled().is_empty(), "{:?}", said.unfilled());
    }

    let everything = Context::at_invocation(noon())
        .and_document(Document::open("/home/anna/Invoices/march.pdf").unwrap())
        .and_selection(Selection::of("the invoice from March").unwrap())
        .and_window(Focused::titled("org.blender.Blender", "untitled.blend").unwrap());
    for row in everything.shown(&strings) {
        assert!(!row.is_a_bug(), "{row}");
        assert!(row.unfilled().is_empty(), "{:?}", row.unfilled());
    }
    for row in Context::at_invocation(noon()).shown(&strings) {
        assert!(!row.is_a_bug(), "{row}");
    }
    let shortened = Selection::of(&"a".repeat(MOST + 2))
        .unwrap()
        .shortened(&strings)
        .unwrap();
    assert!(!shortened.is_a_bug(), "{shortened}");
    assert!(
        shortened.unfilled().is_empty(),
        "{:?}",
        shortened.unfilled()
    );
}
