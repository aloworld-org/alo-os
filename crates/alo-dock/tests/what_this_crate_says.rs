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
const TRANSLATED_INTO_GERMAN: usize = 3;

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
            // **The placement a dock has.** This fixture translated
            // `NAMES_UNDER` and `NAMES_GAVE_WAY` until 2026-10-10, when the
            // owner removed the row of names from the bar and both states
            // became unreachable. One placement is left and it is the tooltip.
            (
                words::NAMES_BESIDE,
                "jedes Symbol zeigt seinen Namen daneben, wenn Sie darauf zeigen oder es mit der \
                 Tastatur erreichen",
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
    let line = dock.layout_on(laptop).labels().said(&strings);

    // **The placement a dock has, in German.** This read *jedes Symbol hat
    // seinen Namen darunter* — the name under the icon — until 2026-10-10, when
    // the owner removed that row. There is one placement now and it is the
    // tooltip.
    assert_eq!(
        line.text(),
        "jedes Symbol zeigt seinen Namen daneben, wenn Sie darauf zeigen oder es mit der \
         Tastatur erreichen"
    );
    assert!(line.is_translated());
}

/// **The sentence a person reads about where the names are is read in their own
/// language.**
///
/// This test was about names *disappearing*: a dock on the smallest screen at
/// 300% text gave its labels up, and the sentence said so with the percentage
/// placed where German puts it — *300 %*, with a space. The owner removed that
/// state on 2026-10-10, so there is no disappearance to describe.
///
/// **What it was really protecting survives and is asserted here**: the whole
/// sentence is the translator's, nothing is assembled from parts, and a
/// translated string comes back translated with no gap left unfilled.
#[test]
fn the_sentence_about_where_the_names_are_survives_being_translated() {
    let strings = das_dock();

    let labels = Dock::shipped().layout_on(Screen::the_smallest()).labels();
    assert_eq!(labels, Labels::Beside);

    let said = labels.said(&strings);
    assert!(said.text().starts_with("jedes Symbol"), "{said}");
    assert!(said.text().contains("Tastatur"), "{said}");
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
    // **456 and not 384**, which moved with the bar on 2026-10-10. The shortest
    // side a screen may have is the dock's share times its thickness, so a bar
    // measured at 76 rather than a proposed 64 asks for a taller screen: 6 × 76
    // instead of 6 × 64. No machine in `docs/hardware.md` is affected — the
    // smallest this crate lays out for is 1366 × 768 — but a display between
    // 384 and 455 in either direction is refused where it was not before, and
    // that is a consequence of the measurement rather than a decision taken
    // here.
    assert!(said.text().contains("456"), "{said}");
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
/// **`dock.edge.middle` is the permanent example, and it is the only one left.**
/// It names an edge that does not exist, so a shell asking for it — an old
/// panel, a stale translation file, a typo in a theme — gets a sentence saying
/// it is a bug rather than a blank row where a picker should be.
///
/// **`dock.edge.bottom` left this list on 2026-10-10**, exactly as the note it
/// replaced said it would: *so `dock.edge.bottom` leaves this list when a
/// person can choose an edge, and the list is not evidence that it should stay
/// gone.* Four keys are declared now — bottom, left, right and top — with a
/// fifth for the heading over them, and a dock that lays out and draws on each.
///
/// That note also recorded why they had been absent, and it is worth keeping
/// because it is the one thing a reader gets wrong here: **not ADR 0076.** That
/// record's bottom-only ruling was reversed by the owner on 2026-09-30 —
/// *bottom should be the default; the person can choose bottom, left, right, or
/// top*. The keys were absent because of the **order of work** set on
/// 2026-10-04: the structure comes first, and *nonfunctional edge choices are
/// not exposed as finished settings*. A declared `dock.edge.left` with no
/// working left dock behind it was exactly that exposure, in the one file a
/// translator reads.
///
/// **`dock.labels.beside` left on 2026-10-04 and the four edges on 2026-10-10**,
/// for the same reason in two steps: the owner gave a side dock's names their
/// measurement — 200 logical pixels in a tooltip beside the icon — which made
/// all four edges lay out, and the draw then took an edge. Three members of this
/// list have now been both wrong to declare and right to declare, which is why
/// **the note rather than the list is the thing to read**.
#[test]
fn a_key_nobody_declared_says_it_is_a_bug() {
    let strings = in_english();
    // **One name, so no loop.** It was a list of two until `dock.edge.bottom`
    // was declared; clippy refuses a `for` over a single element, and spelling
    // it out once is clearer than a one-element array pretending there are more
    // to come. If a second permanently-undeclared key ever arrives, this goes
    // back to a loop and the note above says which ones left and why.
    let named = "dock.edge.middle";
    let key = alo_strings::Key::named(named).unwrap();
    let said = strings.say(&key, &Filling::nothing());
    assert!(said.is_a_bug(), "{named}");
    assert_eq!(said.came_from(), &CameFrom::NoPhrase, "{named}");
    assert_eq!(said.text(), format!("«{named}»"));

    // **And the four that are declared are not bugs**, which is the other half
    // of the same claim and the half this test could not make until today: a
    // list of undeclared keys proves nothing about the declared ones.
    for edge in alo_dock::Edge::EVERY {
        let said = edge.said(&strings);
        assert!(
            !said.is_a_bug(),
            "{:?} has no phrase, so a person picking an edge would read «{}»",
            edge,
            said.text()
        );
        assert!(!said.text().is_empty(), "{edge:?} says nothing");
    }

    // **And no two of them say the same thing.** Four rows a person picks from,
    // and two reading alike is a list nobody can use — which a per-row check
    // cannot see.
    let mut saying: Vec<String> = alo_dock::Edge::EVERY
        .into_iter()
        .map(|edge| edge.said(&strings).text().to_owned())
        .collect();
    saying.sort();
    let before = saying.len();
    saying.dedup();
    assert_eq!(
        saying.len(),
        before,
        "two edges read the same in this language: {saying:?}"
    );
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
