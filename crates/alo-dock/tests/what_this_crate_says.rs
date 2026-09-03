//! Everything this crate can say, put through the whole path a translation
//! takes: declared, checked, partly translated, and drawn as the panel a person
//! actually reads.
//!
//! The crate's own tests take one string at a time. This is the other half: the
//! real vocabulary — not a fixture that resembles it — walked as a whole dock
//! settings panel, which is four edges to pick between and one line underneath
//! saying what picking one did to the names.
//!
//! **German and Greek, for two different reasons.** German because it writes
//! *200 %* with a space where English writes *200%*, which is the one thing a
//! translator decides about the numbered strings in this crate. Greek because it
//! is written in an alphabet that is not Latin, and the row a person picks their
//! dock's edge from has to be readable by somebody who reads only that — which
//! is the whole reason a colour, an edge or a key label is a string rather than
//! a word in the source.
//!
//! It is not the hardware verification `CLAUDE.md` asks for. Nothing here has
//! been seen: there is no compositor, no screen, and there are still no
//! translations in this repository.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_appearance::TextScale;
use alo_dock::words::{self, EVERY_WORD};
use alo_dock::{Dock, Edge, Labels, Screen, Word, dock_words};
use alo_strings::{CameFrom, Direction, Filling, Language, Phrase, Showing, Strings, Vocabulary};

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

/// The four edges and the three things that can become of the names, in German.
fn das_dock() -> Strings {
    reading(
        "de",
        &[
            (words::BOTTOM, "Unten"),
            (words::LEFT, "Links"),
            (words::RIGHT, "Rechts"),
            (words::TOP, "Oben"),
            (words::NAMES_UNDER, "jedes Symbol hat seinen Namen darunter"),
            (words::NAMES_BESIDE, "jedes Symbol hat seinen Namen daneben"),
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

/// **The whole dock panel, read on a German machine**: the four edges a person
/// picks between, and the line underneath telling them what picking one did.
///
/// This is the test that ties the layout to the words. The line under the picker
/// is not a caption somebody wrote — it is what [`alo_dock::Layout`] worked out,
/// said in the language of whoever is reading the panel.
#[test]
fn the_whole_panel_is_read_in_the_language_the_person_reads() {
    let strings = das_dock();
    let laptop = Screen::the_smallest();
    let ordinary = TextScale::ordinary();

    let picker: Vec<String> = Edge::ALL
        .iter()
        .map(|edge| edge.said(&strings).into_text())
        .collect();
    assert_eq!(picker, ["Unten", "Links", "Rechts", "Oben"]);

    let mut dock = Dock::shipped();
    let line = |dock: &Dock, text| {
        dock.layout_on(laptop, text, Direction::LeftToRight)
            .labels()
            .said(&strings)
    };

    assert_eq!(
        line(&dock, ordinary).text(),
        "jedes Symbol hat seinen Namen darunter"
    );
    dock.set_edge(Edge::Left);
    assert_eq!(
        line(&dock, ordinary).text(),
        "jedes Symbol hat seinen Namen daneben"
    );

    for edge in Edge::ALL {
        dock.set_edge(edge);
        assert!(line(&dock, ordinary).is_translated(), "{edge:?}");
    }
}

/// **The sentence somebody reads when their names disappear is read in their
/// own language, and the percent sign is where their language puts it.** German
/// writes *300 %* with a space; the number arrives bare, so it can.
///
/// The half that matters survives the round trip: the name is still announced.
#[test]
fn the_sentence_about_names_disappearing_survives_being_translated() {
    let strings = das_dock();
    let dock = Dock::shipped().with({
        let mut changes = alo_dock::Changes::untouched();
        changes.set_edge(Edge::Right);
        changes
    });
    let large = TextScale::percent(300).unwrap();

    let labels = dock
        .layout_on(Screen::the_smallest(), large, Direction::LeftToRight)
        .labels();
    assert_eq!(labels, Labels::GaveWay(300));

    let said = labels.said(&strings);
    assert!(said.text().starts_with("bei 300 % Textgröße"), "{said}");
    assert!(said.text().contains("Screenreader"), "{said}");
    assert!(said.is_translated());
    assert!(said.unfilled().is_empty());
}

/// **A person who reads only Greek can pick where their dock goes.** Four rows
/// in an alphabet that is not Latin, which is what an edge being a string rather
/// than a word in the source is for.
#[test]
fn the_picker_is_readable_by_somebody_who_reads_no_latin() {
    let strings = reading(
        "el",
        &[
            (words::BOTTOM, "Κάτω"),
            (words::LEFT, "Αριστερά"),
            (words::RIGHT, "Δεξιά"),
            (words::TOP, "Πάνω"),
        ],
    );
    let picker: Vec<String> = Edge::ALL
        .iter()
        .map(|edge| edge.said(&strings).into_text())
        .collect();
    assert_eq!(picker, ["Κάτω", "Αριστερά", "Δεξιά", "Πάνω"]);
    for edge in Edge::ALL {
        assert!(edge.said(&strings).is_translated(), "{edge:?}");
        assert!(
            !edge.said(&strings).text().is_ascii(),
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
    let translated = 7;
    assert_eq!(strings.unanswered().len(), EVERY_WORD.len() - translated);
    assert_eq!(
        strings.missing_from(&language("de")).len(),
        EVERY_WORD.len() - translated
    );

    strings.shown(Showing::InDevelopment);
    assert_eq!(
        Edge::Top.said(&strings).came_from(),
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
/// rather than showing an empty row where an edge should be.
#[test]
fn an_edge_nobody_declared_says_it_is_a_bug() {
    let strings = in_english();
    let middle = alo_strings::Key::named("dock.edge.middle").unwrap();
    let said = strings.say(&middle, &Filling::nothing());
    assert!(said.is_a_bug());
    assert_eq!(said.came_from(), &CameFrom::NoPhrase);
    assert_eq!(said.text(), "«dock.edge.middle»");
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
