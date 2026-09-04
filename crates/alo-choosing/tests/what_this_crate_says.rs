//! What this crate says, against the vocabulary alo OS really loads.
//!
//! The crate's own tests are written against a fixture holding this crate's
//! list alone. This one asks the question that fixture cannot: does what this
//! crate says survive being put beside every other crate's strings, and does a
//! person who has translated some of the machine and not the rest read a whole
//! sentence.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;

use alo_choosing::{EVERY_WORD, NotSet, Word, choosing_words, declare_into};
use alo_strings::{Language, Strings, Translation, Vocabulary};

/// Everything the machine can say, which is what a process really holds.
fn everything_this_machine_can_say() -> Vocabulary {
    alo_saying::everything_this_machine_can_say().unwrap()
}

/// German, as `alo-strings` names a language.
fn german() -> Language {
    Language::written("de").unwrap()
}

/// The machine's vocabulary with these words translated, and German preferred.
fn machine_reading_german(words: &[(Word, &str)]) -> Strings {
    let vocabulary = everything_this_machine_can_say();
    let mut translation = Translation::into_language(german());
    for (word, says) in words {
        translation = translation.says(word.key(), *says);
    }
    let speaking = vocabulary.check(translation).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german()]);
    strings
}

/// The file every refusal in these tests names.
fn somewhere() -> PathBuf {
    PathBuf::from("/home/ada/.config/alo/settings.toml")
}

/// **Every string this crate can say is in the machine's one vocabulary.** A
/// word declared in this crate and left out of `alo-saying`'s list is a
/// sentence that reaches a person as a key, and a key is what `alo-strings`
/// calls a bug.
#[test]
fn everything_this_crate_says_is_something_the_machine_can_say() {
    let vocabulary = everything_this_machine_can_say();
    for word in EVERY_WORD {
        assert!(
            vocabulary.phrase(&word.key()).is_some(),
            "the machine cannot say {}",
            word.named()
        );
    }
}

/// **This crate's list goes in beside everybody else's**, which is only true
/// while no two crates have claimed one key.
#[test]
fn this_crates_words_do_not_collide_with_anybody_elses() {
    let alone = choosing_words().unwrap().how_many();
    assert_eq!(alone, EVERY_WORD.len());

    let mut machine = everything_this_machine_can_say();
    // Already in it, because `alo-saying` collects this crate: declaring it a
    // second time is what a second claim on a key looks like, and it is
    // refused.
    assert!(declare_into(&mut machine).is_err());
    assert!(machine.how_many() > alone);
}

/// **A settings file that is not settings is refused in the language the
/// machine is showing**, with the path a person has to open in it — against the
/// whole machine's vocabulary rather than this crate's own.
#[test]
fn a_file_that_is_not_settings_is_refused_in_the_readers_own_language() {
    let strings = machine_reading_german(&[(
        alo_choosing::EVERY_WORD[1],
        "Ihre Einstellungen in {path} sind keine Einstellungen, die alo OS lesen kann",
    )]);
    let said = NotSet::NotUnderstood {
        at: somewhere(),
        why: Box::new(toml::from_str::<toml::Table>("=").unwrap_err()),
    }
    .said(&strings);

    assert!(said.is_translated(), "{said}");
    assert!(said.text().contains("Einstellungen"), "{said}");
    assert!(
        said.text().contains("/home/ada/.config/alo/settings.toml"),
        "{said}"
    );
}

/// **A sentence nobody has translated is still a sentence**, and it says so.
/// A machine with the rest of its strings in German and this one not is the
/// ordinary state of a translation somebody is part way through, and the person
/// reads the English rather than the key.
#[test]
fn a_sentence_nobody_translated_reaches_a_person_as_a_sentence() {
    let strings = machine_reading_german(&[]);
    let said = NotSet::Nameless { at: somewhere() }.said(&strings);

    assert!(!said.is_translated(), "{said}");
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("name a list of models"), "{said}");
    assert!(
        said.text().contains("/home/ada/.config/alo/settings.toml"),
        "{said}"
    );
}
