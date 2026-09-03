//! Everything this crate can say, put through the whole path a translation
//! takes: declared, checked, partly translated, and shown.
//!
//! The crate's own tests take one sentence at a time. This is the other half:
//! the real vocabulary — not a fixture that resembles it — walked in languages
//! that make the exercise worth doing. German moves the verb to the end of the
//! sentence a person approves; Maltese is one of the two official languages a
//! product selling "English plus the big five" would have skipped, and it is
//! here for that reason as much as any other.
//!
//! It is not the hardware verification `CLAUDE.md` asks for. Nothing here has
//! been read by anybody: there is no screen, and there are still no
//! translations in this repository.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_applications::words::{self, EVERY_WORD};
use alo_applications::{
    Application, NotAnApplication, NotInstalled, application_verbs, application_words,
};
use alo_capability::Given;
use alo_strings::{Key, Language, Phrase, Showing, Strings, Translation, Vocabulary};

/// One of the tests' languages.
fn language(tag: &str) -> Language {
    Language::written(tag).unwrap()
}

/// This crate's words, with nothing translated.
fn in_english() -> Strings {
    Strings::of(application_words().unwrap())
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
    assert!(
        vocabulary
            .phrase(&Key::named("files.verb.read-file.purpose").unwrap())
            .is_some()
    );
}

/// **The sentence a person approves, translated, with its argument still in
/// it.** `alo_capability::Verb::checked` refuses a sentence that does not name
/// every argument; `alo_strings::Vocabulary::check` refuses a translation that
/// drops a gap the source has. They are the same rule about the same string,
/// and this is it holding in German.
#[test]
fn an_approval_sentence_survives_translation_with_its_argument_in_it() {
    let vocabulary = application_words().unwrap();
    let german = vocabulary
        .check(
            Translation::into_language(language("de"))
                .says(
                    words::CLOSE_APPLICATION_SENTENCE.key(),
                    "{application} bitten, sich zu schließen",
                )
                .says(
                    words::FOCUS_APPLICATION_SENTENCE.key(),
                    "{application} in den Vordergrund holen",
                ),
        )
        .unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(german).unwrap();
    strings.prefers(&[language("de")]);

    let verbs = application_verbs().unwrap();
    for (verb, expected) in [
        (
            "close_application",
            "org.blender.Blender bitten, sich zu schließen",
        ),
        (
            "focus_application",
            "org.blender.Blender in den Vordergrund holen",
        ),
    ] {
        let said = verbs
            .call(verb, &[("application", Given::text("org.blender.Blender"))])
            .unwrap()
            .sentence(&strings);
        assert_eq!(said.text(), expected, "{verb}");
        assert!(said.is_translated(), "{verb}");
        assert!(said.unfilled().is_empty(), "{:?}", said.unfilled());
    }

    // And a translation that dropped the identifier would never have loaded, so
    // nobody could be asked to approve a sentence that does not say which
    // application it is about.
    let wrongs = application_words()
        .unwrap()
        .check(Translation::into_language(language("de")).says(
            words::OPEN_APPLICATION_SENTENCE.key(),
            "die Anwendung öffnen",
        ))
        .unwrap_err();
    assert!(wrongs.to_string().contains("{application}"), "{wrongs}");
}

/// **A half-translated crate says which half.** A shell being built in Maltese
/// can count what is left, and what reaches a person meanwhile is marked in
/// development rather than passed off as Maltese.
#[test]
fn what_nobody_has_translated_yet_is_visible_rather_than_silently_english() {
    let vocabulary = application_words().unwrap();
    let maltese = vocabulary
        .check(
            Translation::into_language(language("mt"))
                .says(
                    words::NOT_INSTALLED.key(),
                    "{application} mhuwiex installat fuq din il-magna",
                )
                .says(words::OPEN_APPLICATION.key(), "iftaħ applikazzjoni"),
        )
        .unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(maltese).unwrap();
    strings.prefers(&[language("mt")]);

    assert_eq!(strings.unanswered().len(), EVERY_WORD.len() - 2);

    let missing = NotInstalled::wanting("org.blender.Blender").said(&strings);
    assert_eq!(
        missing.text(),
        "org.blender.Blender mhuwiex installat fuq din il-magna"
    );
    assert!(missing.is_translated());

    // One nobody has reached yet is marked, rather than looking like Maltese
    // somebody wrote.
    strings.shown(Showing::InDevelopment);
    let untranslated = NotAnApplication::NoIdentifier.said(&strings);
    assert!(untranslated.text().starts_with('«'), "{untranslated}");
    assert!(!untranslated.is_translated());
}

/// **What is approved is never translated, and what is around it always is.**
/// An identifier is a name off this machine: a translation of it would name a
/// different application, or none.
#[test]
fn the_identifier_is_the_machines_and_the_sentence_around_it_is_the_readers() {
    let vocabulary = application_words().unwrap();
    let german = vocabulary
        .check(
            Translation::into_language(language("de"))
                .says(words::CALLED.key(), "{called} [{application}]")
                .says(
                    words::OPEN_APPLICATION_SENTENCE.key(),
                    "{application} öffnen",
                ),
        )
        .unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(german).unwrap();
    strings.prefers(&[language("de")]);

    let said = application_verbs()
        .unwrap()
        .call(
            "open_application",
            &[("application", Given::text("org.blender.Blender"))],
        )
        .unwrap()
        .sentence(&strings);
    assert_eq!(said.text(), "org.blender.Blender öffnen");

    // The name the packager gave it is not ours to translate either, and it is
    // shown beside the identifier rather than in place of it.
    assert_eq!(
        Application::called("org.blender.Blender", "Blender")
            .unwrap()
            .shown(&strings),
        "Blender [org.blender.Blender]"
    );
}

/// A machine with no translations at all is the machine this repository ships
/// today, and on it every one of these keys still answers with the sentence the
/// code declared rather than with the key.
///
/// The gaps are deliberately left empty here — this asks whether the string is
/// *there*, and a sentence with `{application}` still in it is what an unfilled
/// gap looks like. That the callers fill them is the crate's own tests'.
#[test]
fn with_no_translations_at_all_every_sentence_is_still_a_sentence() {
    let strings = in_english();
    for word in EVERY_WORD {
        let said = strings.say(&word.key(), &alo_strings::Filling::nothing());
        assert_eq!(
            said.came_from(),
            &alo_strings::CameFrom::TheSource,
            "{}",
            word.key()
        );
        assert_eq!(said.text(), word.says(), "{}", word.key());
    }
    // And the honest count of what is owed: on this machine nobody has
    // translated any of it, and the shell can say so without knowing what it
    // was looking for.
    assert_eq!(strings.unanswered().len(), EVERY_WORD.len());
}

/// **Every word the four verbs are declared with is one this crate declares**,
/// asked of the real vocabulary rather than of a fixture — the test
/// `docs/contracts/agent-verbs.md` asks of every crate that declares verbs.
///
/// Since item 11a that reaches an argument's *options* as well. An arrangement
/// left out of [`EVERY_WORD`] would compile, declare and reach a person as a
/// key in the middle of the sentence they are approving, which is one place
/// further in than the hole this test was written for.
#[test]
fn nothing_a_verb_says_is_a_key_nobody_declared() {
    let strings = in_english();
    let verbs = application_verbs().unwrap();
    assert_eq!(verbs.len(), 4);
    let given = [
        ("application", Given::text("org.blender.Blender")),
        ("where", Given::text("left_half")),
    ];
    for verb in verbs.all() {
        assert!(!verb.purpose(&strings).is_a_bug(), "{}", verb.name());
        for arg in verb.args() {
            assert!(
                !arg.purpose(&strings).is_a_bug(),
                "{} {}",
                verb.name(),
                arg.name()
            );
        }
        let takes_where = verb.arg("where").is_some();
        let said = verbs
            .call(verb.name(), if takes_where { &given } else { &given[..1] })
            .unwrap()
            .sentence(&strings);
        assert!(!said.is_a_bug(), "{}: {said}", verb.name());
        assert!(
            said.text().contains("org.blender.Blender"),
            "{}",
            verb.name()
        );
    }
}

/// **The arrangement inside the sentence is a string this crate declares too.**
///
/// The failure this closes is quieter than a missing sentence: the verb's own
/// sentence would still be there, so a shell asking "is this a bug" would be
/// told no while a person read *put org.blender.Blender
/// «applications.where.left-half»*.
#[test]
fn an_arrangement_nobody_declared_would_be_a_bug_and_is_not_one() {
    let strings = in_english();
    let verbs = application_verbs().unwrap();
    for sent in ["left_half", "right_half", "whole_screen"] {
        let said = verbs
            .call(
                "arrange_application",
                &[
                    ("application", Given::text("org.blender.Blender")),
                    ("where", Given::text(sent)),
                ],
            )
            .unwrap()
            .sentence(&strings);
        assert!(!said.is_a_bug(), "{sent}: {said}");
        assert!(!said.text().contains('«'), "{sent}: {said}");
        assert!(
            said.text().starts_with("put org.blender.Blender "),
            "{said}"
        );
    }
}
