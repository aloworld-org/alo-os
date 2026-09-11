//! What this crate says, against the vocabulary alo OS really loads.
//!
//! The crate's own list is tested where it is declared; this asks the two
//! questions that list cannot. Whether `alo-saying` collects it at all — task 3
//! records a crate declaring nine strings that reached nothing, which
//! `alo-collected` now refuses repository-wide — and whether the sentences a
//! greeter shows reach a person in their own language, which is the whole of
//! why a key crosses the wire from `alo-sessiond` rather than a sentence.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_greeting::{EVERY_WORD, MAKE_AN_ACCOUNT, NOTHING_LISTENING, Standing, declare_into};
use alo_strings::{Language, Strings, Translation, Vocabulary};

/// Everything the machine can say, which is what a real surface holds.
fn everything_this_machine_can_say() -> Vocabulary {
    alo_saying::everything_this_machine_can_say().unwrap()
}

/// **Every string this crate can say is in the machine's one vocabulary.** A
/// word declared here and left out of `alo-saying`'s list is a sentence that
/// reaches a person as a key, at the one screen they cannot get past.
#[test]
fn everything_this_crate_says_is_collected_by_the_machine() {
    let vocabulary = everything_this_machine_can_say();
    for word in EVERY_WORD {
        assert!(
            vocabulary.phrase(&word.key()).is_some(),
            "{} is not collected: the machine cannot say it",
            word.named()
        );
    }
}

/// **This crate's list goes in beside everybody else's**, which is only true
/// while no two crates have claimed one key.
#[test]
fn this_crates_words_do_not_collide_with_anybody_elses() {
    let mut machine = everything_this_machine_can_say();
    // Already in it, because `alo-saying` collects this crate: declaring it a
    // second time is what a second claim on a key looks like, and it is
    // refused.
    assert!(declare_into(&mut machine).is_err());
}

/// **The first screen an alo OS machine ever shows reaches a person in their
/// own language.** It is the sentence with the least English around it — there
/// is no account, no name and nothing else on the screen — so a key arriving
/// here is a machine that looks broken on its first morning.
#[test]
fn the_first_screen_is_read_in_the_readers_own_language() {
    let german = Language::written("de").unwrap();
    let vocabulary = everything_this_machine_can_say();
    let translation = Translation::into_language(german.clone())
        .says(
            MAKE_AN_ACCOUNT.key(),
            "Auf diesem Rechner hat noch niemand ein Konto. Legen Sie eines an, um sich \
             anzumelden — das Konto gehört diesem Rechner und wird nirgendwohin gesendet",
        )
        .says(
            NOTHING_LISTENING.key(),
            "Die Anmeldung wurde nicht abgeschlossen: Der Teil dieses Rechners, der eine \
             Sitzung startet, läuft nicht. Sie haben nichts falsch eingegeben",
        );
    let speaking = vocabulary.check(translation).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german]);

    let said = Standing::MakeAnAccount.said(&strings).unwrap();
    assert!(said.is_translated(), "{said}");
    assert!(said.text().starts_with("Auf diesem Rechner"), "{said}");
    assert!(!said.is_a_bug(), "{said}");
}
