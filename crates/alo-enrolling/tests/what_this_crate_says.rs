//! What this crate says, against the vocabulary alo OS really loads.
//!
//! The crate's own tests read this crate's list alone. This one asks the
//! question that list cannot: does what this crate says survive being put beside
//! every other crate's sentences, and is a person who never learns what LUKS is
//! told everything they need at the two moments they need it — while their disk
//! is being encrypted, and in front of a machine that will not open.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_encrypting::TheDiskRefused;
use alo_enrolling::{EVERY_WORD, about_what_the_disk_refused, in_the_persons_language};
use alo_strings::{Language, Strings, Translation, Vocabulary};

/// Everything the machine can say, which is what a process really holds.
fn everything_this_machine_can_say() -> Vocabulary {
    match alo_saying::everything_this_machine_can_say() {
        Ok(vocabulary) => vocabulary,
        Err(why) => panic!("alo OS's own words are wrong: {why}"),
    }
}

/// **Every sentence this crate can say is in the machine's one vocabulary.** A
/// word declared here and left out of `alo-saying`'s list reaches a person as a
/// key, and a key is what `alo-strings` calls a bug.
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

/// **No sentence names anything alo OS has rather than the person.** Not LUKS,
/// not a TPM, not the tools, not a keyslot, not a device, not *root*. This is
/// the rule task 7 of `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`
/// applies to every crate in this workstream, held here for this one so that a
/// sentence written later fails beside the sentence it was written with.
#[test]
fn no_sentence_names_anything_but_what_the_person_has() {
    for word in EVERY_WORD {
        let said = format!("{} {}", word.says(), word.note().unwrap_or_default());
        for never in [
            "LUKS",
            "luks",
            "TPM",
            "cryptsetup",
            "cryptenroll",
            "keyslot",
            "key slot",
            "/dev/",
            "root",
            "socket",
            "partition",
            "systemd",
            "PCR",
        ] {
            assert!(!said.contains(never), "{}: {never}", word.key());
        }
    }
}

/// **The sentence a person reads in front of a machine that will not open says
/// what to do**, in whatever language they read — held against the machine's
/// vocabulary rather than against this crate's constant, because what they read
/// is what the assembled machine says.
#[test]
fn the_machine_says_what_to_do_when_it_will_not_open() {
    let word = about_what_the_disk_refused(TheDiskRefused::WhatWasGivenDoesNotOpenIt);
    let vocabulary = everything_this_machine_can_say();
    assert!(
        vocabulary.phrase(&word.key()).is_some(),
        "the machine cannot say {}",
        word.named()
    );

    // In English, which is what somebody whose language nobody has written yet
    // reads — and it is still a sentence rather than a key.
    let english = Strings::of(everything_this_machine_can_say());
    let said = in_the_persons_language(word, &english);
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("key you wrote down"), "{said}");

    // And in a language that is not English, once somebody has written it.
    let Ok(german) = Language::written("de") else {
        panic!("de is a language")
    };
    let translation = Translation::into_language(german.clone()).says(
        word.key(),
        "das öffnet diesen Rechner nicht — nimm den Schlüssel, den du aufgeschrieben hast",
    );
    let speaking = match vocabulary.check(translation) {
        Ok(speaking) => speaking,
        Err(why) => panic!("a correct line was refused: {why:?}"),
    };
    let mut strings = Strings::of(everything_this_machine_can_say());
    if let Err(why) = strings.speaks(speaking) {
        panic!("the machine would not speak it: {why}");
    }
    strings.prefers(&[german]);
    let said = in_the_persons_language(word, &strings);
    assert!(said.is_translated(), "{said}");
    assert!(said.text().contains("aufgeschrieben"), "{said}");
}
