//! Everything this crate can say, put through the whole path a translation
//! takes: declared, checked, partly translated, and drawn as the panel a person
//! actually reads.
//!
//! The crate's own tests take one string at a time. This is the other half: the
//! real vocabulary — not a fixture that resembles it — walked as a whole dock
//! settings panel, which since ADR 0076 is two rows to pick between and one line
//! underneath saying what the text size did to the names.
//!
//! **It was four rows above those two**, for the edge a person put the dock on.
//! The edges went with the decision; the shape of the test did not change, because
//! *a list of rows and a line underneath* is still what the panel is.
//!
//! **German and Greek, for two different reasons.** German because it writes
//! *200 %* with a space where English writes *200%*, which is the one thing a
//! translator decides about the numbered strings in this crate. Greek because it
//! is written in an alphabet that is not Latin, and a row a person picks from has
//! to be readable by somebody who reads only that — which is the whole reason a
//! colour, a row or a key label is a string rather than a word in the source.
//!
//! It is not the hardware verification `CLAUDE.md` asks for. Nothing here has
//! been seen: there is no screen in this test, and there are still no translations
//! in this repository.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_appearance::TextScale;
use alo_dock::words::{self, EVERY_WORD};
use alo_dock::{Dock, Hiding, Labels, Screen, Word, dock_words};
use alo_strings::{CameFrom, Filling, Language, Phrase, Showing, Strings, Vocabulary};

/// One of the tests' languages.
fn language(tag: &str) -> Language {
    Language::written(tag).unwrap()
}

/// This crate's words, with nothing translated.
fn in_english() -> Strings {
    Strings::of(dock_words().unwrap())
}

/// This crate's words, with these translated into the given language and that
/// language preferred.
fn reading(tag: &str, words: &[(Word, &str)]) -> Strings {
    let vocabulary = dock_words().unwrap();
    let mut translation = alo_strings::Translation::into_language(language(tag));
    for (word, says) in words {
        translation = translation.says(word.key(), *says);
    }
    let speaking = vocabulary.check(translation).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[language(tag)]);
    strings
}

/// How many of this crate's words `das_dock` translates, named once so the test
/// that counts what is left does not carry the number twice.
const TRANSLATED_INTO_GERMAN: usize = 4;

/// The two rows of the panel and the two things that can become of the names, in
/// German.
fn das_dock() -> Strings {
    reading(
        "de",
        &[
            (words::ALWAYS_SHOWN, "Immer sichtbar"),
            (
                words::GIVES_WAY_TO_A_WINDOW,
                "Weicht zurück, wenn ein Fenster den Platz braucht",
            ),
            (words::NAMES_UNDER, "jedes Symbol hat seinen Namen darunter"),
            (
                words::NAMES_GAVE_WAY,
                "bei {percent} % Textgröße ist kein Platz für Namen — das Dock zeigt Symbole; wer \
                 auf einem verweilt, bekommt weiterhin seinen Namen, und ein Screenreader liest \
                 ihn weiterhin vor",
            ),
        ],
    )
}

/// **Every string this crate can say declares**, and it declares into a
/// vocabulary that already holds somebody else's.
///
/// The area at the front of a key is what makes that safe: a shell has one
/// vocabulary, and every crate puts its own into it.
#[test]
fn everything_this_crate_says_joins_one_vocabulary_beside_another_crate() {
    let mut vocabulary = Vocabulary::empty();
    vocabulary
        .says(
            Phrase::says(
                alo_strings::Key::named("appearance.token.charcoal").unwrap(),
                "Charcoal",
            )
            .unwrap(),
        )
        .unwrap();
    alo_dock::declare_into(&mut vocabulary).unwrap();

    assert_eq!(vocabulary.how_many(), EVERY_WORD.len() + 1);
    assert_eq!(vocabulary.counted().count(), 0, "nothing here counts");
    for word in EVERY_WORD {
        assert!(vocabulary.phrase(&word.key()).is_some(), "{}", word.key());
    }
}

/// **The whole dock panel, read on a German machine**: the rows a person picks
/// between, and the line underneath telling them what the text size did.
///
/// This is the test that ties the layout to the words. The line under the picker
/// is not a caption somebody wrote — it is what [`alo_dock::Layout`] worked out,
/// said in the language of whoever is reading the panel.
#[test]
fn the_whole_panel_is_read_in_the_language_the_person_reads() {
    let strings = das_dock();
    let laptop = Screen::the_smallest();
    let ordinary = TextScale::ordinary();

    let picker: Vec<String> = Hiding::ALL
        .iter()
        .map(|hiding| hiding.said(&strings).into_text())
        .collect();
    assert_eq!(
        picker,
        [
            "Immer sichtbar",
            "Weicht zurück, wenn ein Fenster den Platz braucht"
        ]
    );
    for hiding in Hiding::ALL {
        assert!(hiding.said(&strings).is_translated(), "{hiding:?}");
    }

    let dock = Dock::shipped();
    let line = |text| dock.layout_on(laptop, text).labels().said(&strings);

    assert_eq!(
        line(ordinary).text(),
        "jedes Symbol hat seinen Namen darunter"
    );
    assert!(line(ordinary).is_translated());
}

/// **The sentence somebody reads when their names disappear is read in their
/// own language, and the percent sign is where their language puts it.** German
/// writes *300 %* with a space; the number arrives bare, so it can.
///
/// The half that matters survives the round trip: the name is still announced.
#[test]
fn the_sentence_about_names_disappearing_survives_being_translated() {
    let strings = das_dock();
    let large = TextScale::percent(300).unwrap();

    let labels = Dock::shipped()
        .layout_on(Screen::the_smallest(), large)
        .labels();
    assert_eq!(labels, Labels::GaveWay(300));

    let said = labels.said(&strings);
    assert!(said.text().starts_with("bei 300 % Textgröße"), "{said}");
    assert!(said.text().contains("Screenreader"), "{said}");
    assert!(said.is_translated());
    assert!(said.unfilled().is_empty());
}

/// **A person who reads only Greek can pick what their dock does.** Two rows in
/// an alphabet that is not Latin, which is what a row being a string rather than
/// a word in the source is for.
#[test]
fn the_picker_is_readable_by_somebody_who_reads_no_latin() {
    let strings = reading(
        "el",
        &[
            (words::ALWAYS_SHOWN, "Πάντα ορατό"),
            (
                words::GIVES_WAY_TO_A_WINDOW,
                "Αποσύρεται όταν ένα παράθυρο χρειάζεται τον χώρο",
            ),
        ],
    );
    let picker: Vec<String> = Hiding::ALL
        .iter()
        .map(|hiding| hiding.said(&strings).into_text())
        .collect();
    assert_eq!(
        picker,
        [
            "Πάντα ορατό",
            "Αποσύρεται όταν ένα παράθυρο χρειάζεται τον χώρο"
        ]
    );
    for hiding in Hiding::ALL {
        assert!(hiding.said(&strings).is_translated(), "{hiding:?}");
        assert!(
            !hiding.said(&strings).text().is_ascii(),
            "and it is genuinely not Latin"
        );
    }
}

/// **What came off somebody's own machine is not translated.** A screen's
/// measurements are numbers a compositor reported, whatever language the
/// sentence around them is written in.
#[test]
fn what_came_off_the_machine_is_not_translated() {
    let strings = reading(
        "de",
        &[(
            words::SCREEN_TOO_SMALL,
            "{width} × {height} ist kleiner, als alo OS auslegt — ein Bildschirm braucht in jeder \
             Richtung mindestens {least}",
        )],
    );
    let said = Screen::of(320, 240).unwrap_err().said(&strings);
    assert!(said.text().contains("320 × 240"), "{said}");
    assert!(said.text().contains("384"), "{said}");
    assert!(
        said.text().contains("alo OS"),
        "the name is never translated"
    );
    assert!(said.is_translated());
    assert!(said.unfilled().is_empty());
}

/// **A half-translated panel says which half.** A shell being built in German
/// can count what is left without knowing what it was looking for, and what
/// reaches a person meanwhile is marked in development rather than passed off as
/// German.
#[test]
fn what_nobody_has_translated_yet_is_visible_rather_than_silently_english() {
    let mut strings = das_dock();
    assert_eq!(
        strings.unanswered().len(),
        EVERY_WORD.len() - TRANSLATED_INTO_GERMAN
    );
    assert_eq!(
        strings.missing_from(&language("de")).len(),
        EVERY_WORD.len() - TRANSLATED_INTO_GERMAN
    );

    strings.shown(Showing::InDevelopment);
    assert_eq!(
        Hiding::Never.said(&strings).came_from(),
        &CameFrom::Translation(language("de"))
    );

    let untranslated = Screen::of(0, 768).unwrap_err().said(&strings);
    assert_eq!(
        untranslated.text(),
        "«a screen has a width and a height — 0 by 768 is not one»"
    );
    assert_eq!(untranslated.came_from(), &CameFrom::TheSource);
    assert!(untranslated.unfilled().is_empty(), "and it is still filled");
}

/// A key that nothing declares is a mistake in this repository and says so,
/// rather than showing an empty row where a setting should be.
///
/// **`dock.edge.middle` is the permanent example; the other two are temporary and
/// this note is the only thing that says so.** No `dock.edge.*` key is declared
/// today, so a shell still asking for one — an old panel, a stale translation
/// file — gets a sentence saying it is a bug rather than a blank row where an edge
/// picker used to be.
///
/// **The reason is no longer ADR 0076, and that matters to whoever edits this
/// next.** That record's bottom-only ruling was reversed by the owner on
/// 2026-09-30 — *bottom should be the default; the person can choose bottom,
/// left, right, or top* — so the keys are not absent because the choice was
/// withdrawn. They are absent because of the **order of work** the owner set on
/// 2026-10-04: the structure comes first, and *nonfunctional edge choices are not
/// exposed as finished settings*. A declared `dock.edge.left` with no working left
/// dock behind it is exactly that exposure, in the one file a translator reads.
///
/// So `dock.edge.bottom` **leaves this list** when a person can choose an edge, and
/// the list is not evidence that it should stay gone. `dock.edge.middle` names an
/// edge that does not exist and stays forever.
///
/// **`dock.labels.beside` left on 2026-10-04**, exactly as the note above said it
/// would: the owner gave the side placement its measurement — 200 logical pixels
/// of usable text width in a tooltip — so a dock down a side lays out, its names
/// have a placement, and the key is declared again in `crate::words`. One member
/// of this list has now been both wrong to declare and right to declare, eighteen
/// hours apart, which is why the note rather than the list is the thing to read.
#[test]
fn a_key_nobody_declared_says_it_is_a_bug() {
    let strings = in_english();
    for named in ["dock.edge.middle", "dock.edge.bottom"] {
        let key = alo_strings::Key::named(named).unwrap();
        let said = strings.say(&key, &Filling::nothing());
        assert!(said.is_a_bug(), "{named}");
        assert_eq!(said.came_from(), &CameFrom::NoPhrase, "{named}");
        assert_eq!(said.text(), format!("«{named}»"));
    }
}

/// A machine with no translations at all is the machine this repository ships
/// today, and on it every one of these keys still answers with the string the
/// code declared rather than with the key.
///
/// The gaps are deliberately left empty here — this asks whether the string is
/// *there*. That the callers fill them is each type's own test.
#[test]
fn with_no_translations_at_all_every_string_is_still_a_string() {
    let strings = in_english();
    for word in EVERY_WORD {
        let said = strings.say(&word.key(), &Filling::nothing());
        assert_eq!(said.came_from(), &CameFrom::TheSource, "{}", word.key());
        assert_eq!(said.text(), word.says(), "{}", word.key());
    }
}
