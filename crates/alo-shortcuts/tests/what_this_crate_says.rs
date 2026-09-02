//! Everything this crate can say, put through the whole path a translation
//! takes: declared, checked, partly translated, and drawn as the panel a person
//! actually reads.
//!
//! The crate's own tests take one string at a time. This is the other half: the
//! real vocabulary — not a fixture that resembles it — walked as a whole
//! shortcuts panel in German, which is the language whose keyboard prints
//! something different on half the keys this crate names. *Entf*, *Einfg*,
//! *Pos1*, *Strg*, *Bild ↑*: every one of them is a row that would have read
//! wrong on a German machine yesterday, naming a key that is not printed on the
//! keyboard in front of the person.
//!
//! `alo-strings`' own integration test used to carry copies of two of these
//! strings, because it was built before its first user existed. They have left
//! it, which is what that file says happens when a crate moves.
//!
//! It is not the hardware verification `CLAUDE.md` asks for. Nothing here has
//! been pressed: there is no compositor, no screen, and there are still no
//! translations in this repository.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_shortcuts::words::{self, EVERY_WORD, Word};
use alo_shortcuts::{Action, Chord, Key, Modifier, Modifiers, Shortcuts, shortcut_words};
use alo_strings::{CameFrom, Filling, Language, Phrase, Showing, Strings, Translation, Vocabulary};

/// One of the tests' languages.
fn language(tag: &str) -> Language {
    Language::written(tag).unwrap()
}

/// This crate's words, with nothing translated.
fn in_english() -> Strings {
    Strings::of(shortcut_words().unwrap())
}

/// This crate's words, with these translated into the given language and that
/// language preferred.
fn reading(tag: &str, words: &[(Word, &str)]) -> Strings {
    let vocabulary = shortcut_words().unwrap();
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

/// Enough German to draw the whole shipped panel.
fn auf_deutsch() -> Strings {
    reading(
        "de",
        &[
            (words::THE_AGENT, "Den Assistenten fragen"),
            (words::LAUNCHER, "Das Startmenü öffnen"),
            (words::CLOSE_WINDOW, "Fenster schließen"),
            (words::MINIMISE_WINDOW, "Fenster minimieren"),
            (
                words::MAXIMISE_WINDOW,
                "Fenster maximieren oder zurücksetzen",
            ),
            (words::SNAP_LEFT, "Fenster auf die linke Hälfte legen"),
            (words::SNAP_RIGHT, "Fenster auf die rechte Hälfte legen"),
            (words::NEXT_WINDOW, "Nächstes Fenster"),
            (words::PREVIOUS_WINDOW, "Vorheriges Fenster"),
            (words::NEXT_APPLICATION, "Nächste Anwendung"),
            (words::PREVIOUS_APPLICATION, "Vorherige Anwendung"),
            (words::CTRL, "Strg"),
            (words::SHIFT, "Umschalt"),
            (words::SPACE, "Leertaste"),
            (words::TAB, "Tabulator"),
            (words::DELETE, "Entf"),
            (words::INSERT, "Einfg"),
            (words::HOME, "Pos1"),
            (words::PAGE_UP, "Bild ↑"),
            (words::PAGE_DOWN, "Bild ↓"),
            (words::LEFT, "Pfeil links"),
            (words::RIGHT, "Pfeil rechts"),
            (words::UP, "Pfeil hoch"),
            (words::DOWN, "Pfeil runter"),
            (
                words::TAKEN,
                "{chord} ist bereits {action} — ändern Sie zuerst diese Zuordnung, oder nehmen \
                 Sie eine andere Taste",
            ),
            (
                words::CLASH,
                "{chord} ist für mehr als eine Sache eingestellt — ändern Sie eine davon",
            ),
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
                alo_strings::Key::named("appearance.token.terracotta").unwrap(),
                "Terracotta",
            )
            .unwrap(),
        )
        .unwrap();
    alo_shortcuts::declare_into(&mut vocabulary).unwrap();

    assert_eq!(vocabulary.how_many(), EVERY_WORD.len() + 1);
    assert_eq!(vocabulary.counted().count(), 0, "nothing here counts");
    for word in EVERY_WORD {
        assert!(vocabulary.phrase(&word.key()).is_some(), "{}", word.key());
    }
}

/// **The whole shipped panel, read on a German machine.** Eleven rows, each
/// naming what it does and the keys that do it — and every key named the way it
/// is printed on the keyboard in front of the person, which is the thing this
/// crate said from the start and could not do until now.
#[test]
fn the_whole_panel_is_read_in_the_language_the_person_reads() {
    let strings = auf_deutsch();
    let shortcuts = Shortcuts::shipped();

    let rows: Vec<(String, String)> = shortcuts
        .bindings()
        .map(|binding| {
            (
                binding.action.said(&strings).into_text(),
                binding
                    .chord
                    .map_or_else(String::new, |chord| chord.shown(&strings)),
            )
        })
        .collect();

    assert_eq!(
        rows,
        [
            ("Den Assistenten fragen".to_owned(), "Super+A".to_owned()),
            (
                "Das Startmenü öffnen".to_owned(),
                "Super+Leertaste".to_owned()
            ),
            ("Fenster schließen".to_owned(), "Alt+F4".to_owned()),
            (
                "Fenster minimieren".to_owned(),
                "Super+Pfeil runter".to_owned()
            ),
            (
                "Fenster maximieren oder zurücksetzen".to_owned(),
                "Super+Pfeil hoch".to_owned()
            ),
            (
                "Fenster auf die linke Hälfte legen".to_owned(),
                "Super+Pfeil links".to_owned()
            ),
            (
                "Fenster auf die rechte Hälfte legen".to_owned(),
                "Super+Pfeil rechts".to_owned()
            ),
            ("Nächstes Fenster".to_owned(), "Alt+Tabulator".to_owned()),
            (
                "Vorheriges Fenster".to_owned(),
                "Alt+Umschalt+Tabulator".to_owned()
            ),
            ("Nächste Anwendung".to_owned(), "Super+Tabulator".to_owned()),
            (
                "Vorherige Anwendung".to_owned(),
                "Super+Umschalt+Tabulator".to_owned()
            ),
        ]
    );
}

/// **The refusal a person meets while setting a shortcut, in German** — and the
/// row it names is German too, because the name of the action that already has
/// the chord is one of this crate's own strings rather than a word borrowed
/// from somewhere that has not moved.
#[test]
fn a_refusal_and_everything_inside_it_are_in_one_language() {
    let strings = auf_deutsch();
    let mut shortcuts = Shortcuts::shipped();
    let taken = shortcuts.chord_for(Action::SnapLeft).unwrap();
    let refused = shortcuts.bind(Action::TheAgent, taken).unwrap_err();
    let said = refused.said(&strings);
    assert_eq!(
        said.text(),
        "Super+Pfeil links ist bereits Fenster auf die linke Hälfte legen — ändern Sie zuerst \
         diese Zuordnung, oder nehmen Sie eine andere Taste"
    );
    assert!(said.is_translated());
    assert!(said.unfilled().is_empty());

    // And the report of a clash nobody could have refused at bind time.
    let both = Chord::checked(
        Modifiers::just(Modifier::Ctrl).and(Modifier::Alt),
        Key::Delete,
    )
    .unwrap();
    let mut changes = alo_shortcuts::Changes::none();
    changes.set(Action::MaximiseWindow, Some(both));
    changes.set(Action::MinimiseWindow, Some(both));
    let clashes = Shortcuts::shipped().with(changes).clashes();
    let clash = clashes.first().unwrap();
    assert_eq!(
        clash.said(&strings).text(),
        "Strg+Alt+Entf ist für mehr als eine Sache eingestellt — ändern Sie eine davon"
    );
}

/// **A key that prints a mark is not a string, in any language.** Fifty-three of
/// the sixty-nine keys are absent from the vocabulary on purpose: translating
/// `Q` would name a *position* on a keyboard, which is the model this crate
/// exists to reject, and `Strings::unanswered` would report fifty-three strings
/// nobody should ever translate.
#[test]
fn the_keys_that_print_a_mark_are_in_no_vocabulary_at_all() {
    let vocabulary = shortcut_words().unwrap();
    let strings = auf_deutsch();
    let mut marks = 0_usize;
    for key in Key::ALL {
        let Some(mark) = key.mark() else { continue };
        marks = marks.saturating_add(1);
        assert_eq!(key.word(), None, "{key:?}");
        assert_eq!(key.said(&strings), None, "{key:?}");
        assert_eq!(key.shown(&strings), mark, "{key:?}");
        assert!(
            vocabulary
                .phrases()
                .all(|phrase| phrase.source().as_written() != mark),
            "{key:?} is declared as a string"
        );
    }
    assert_eq!(marks, 53);
    assert_eq!(
        vocabulary
            .phrases()
            .filter(|phrase| phrase.key().as_str().starts_with("shortcuts.key."))
            .count(),
        16
    );
}

/// **A half-translated panel says which half.** A shell being built in Maltese
/// can count what is left without knowing what it was looking for, and what
/// reaches a person meanwhile is marked in development rather than passed off as
/// Maltese.
#[test]
fn what_nobody_has_translated_yet_is_visible_rather_than_silently_english() {
    let mut strings = reading(
        "mt",
        &[
            (words::CLOSE_WINDOW, "Agħlaq it-tieqa"),
            (words::SPACE, "Spazju"),
        ],
    );
    assert_eq!(strings.unanswered().len(), EVERY_WORD.len() - 2);
    assert_eq!(
        strings.missing_from(&language("mt")).len(),
        EVERY_WORD.len() - 2
    );

    strings.shown(Showing::InDevelopment);
    let translated = Action::CloseWindow.said(&strings);
    assert_eq!(translated.text(), "Agħlaq it-tieqa");
    assert_eq!(
        translated.came_from(),
        &CameFrom::Translation(language("mt"))
    );

    let untranslated = Action::SnapLeft.said(&strings);
    assert_eq!(untranslated.text(), "«Put the window on the left half»");
    assert_eq!(untranslated.came_from(), &CameFrom::TheSource);
}

/// A machine with no translations at all is the machine this repository ships
/// today, and on it every one of these keys still answers with the string the
/// code declared rather than with the key.
///
/// The gaps are deliberately left empty here — this asks whether the string is
/// *there*, and a sentence with `{key}` still in it is what an unfilled gap
/// looks like. That the callers fill them is
/// `refusing::tests::every_refusal_says_something_of_its_own`'s.
#[test]
fn with_no_translations_at_all_every_string_is_still_a_string() {
    let strings = in_english();
    for word in EVERY_WORD {
        let said = strings.say(&word.key(), &Filling::nothing());
        assert_eq!(said.came_from(), &CameFrom::TheSource, "{}", word.key());
        assert_eq!(said.text(), word.says(), "{}", word.key());
    }
}
