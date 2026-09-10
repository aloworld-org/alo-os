//! What this crate says, against the vocabulary alo OS really loads.
//!
//! The crate's own tests are written against a fixture holding this crate's
//! list alone. This one asks the question that fixture cannot: does what
//! this crate says survive being put beside every other crate's strings,
//! and does a person read one refusal that never says which of two facts it
//! is refusing over.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_accounts::{EVERY_WORD, NotSignedIn, Word, declare_into};
use alo_strings::{Language, Strings, Translation, Vocabulary};

/// Everything the machine can say, which is what a process really holds.
fn everything_this_machine_can_say() -> Vocabulary {
    alo_saying::everything_this_machine_can_say().unwrap()
}

/// German, as `alo-strings` names a language.
fn german() -> Language {
    Language::written("de").unwrap()
}

/// The machine's vocabulary with these words translated, and German
/// preferred.
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

/// **Every string this crate can say is in the machine's one vocabulary.**
/// A word declared here and left out of `alo-saying`'s list is a sentence
/// that reaches a person as a key, and a key is what `alo-strings` calls a
/// bug.
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

/// **This crate's list goes in beside everybody else's**, which is only
/// true while no two crates have claimed one key.
#[test]
fn this_crates_words_do_not_collide_with_anybody_elses() {
    let mut machine = everything_this_machine_can_say();
    // Already in it, because `alo-saying` collects this crate: declaring it
    // a second time is what a second claim on a key looks like, and it is
    // refused.
    assert!(declare_into(&mut machine).is_err());
}

/// **The sign-in refusal reaches a German reader in German**, still as one
/// sentence that never says which of the two facts it refuses over — the
/// property the translator's note asks every language to keep.
#[test]
fn a_refused_sign_in_is_refused_in_the_readers_own_language() {
    let strings = machine_reading_german(&[(
        alo_accounts::NOT_SIGNED_IN,
        "dieser Name und dieses Passwort melden auf dieser Maschine niemanden an — prüfen Sie \
         beides und versuchen Sie es erneut",
    )]);
    let said = NotSignedIn::Refused.said(&strings);

    assert!(said.is_translated(), "{said}");
    assert!(said.text().contains("niemanden an"), "{said}");
}

/// **A sentence nobody has translated is still a sentence**, and the two
/// refusals stay two different sentences — retype, or go and fix the
/// machine — in the fallback English as much as anywhere.
#[test]
fn the_two_refusals_reach_a_person_as_two_different_sentences() {
    let strings = machine_reading_german(&[]);
    let refused = NotSignedIn::Refused.said(&strings);
    let not_the_person = NotSignedIn::NotThePerson {
        signed_in: 1001,
        described: 1000,
    }
    .said(&strings);

    assert!(!refused.is_a_bug(), "{refused}");
    assert!(!not_the_person.is_a_bug(), "{not_the_person}");
    assert_ne!(refused.text(), not_the_person.text());
    // And the machine-fixing refusal carries no number into the sentence:
    // the uids stay in the value, for whoever reconciles the description.
    assert!(!not_the_person.text().contains("1001"), "{not_the_person}");
    assert!(!not_the_person.text().contains("1000"), "{not_the_person}");
}
