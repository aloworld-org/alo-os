//! Which sentence each refusal on the road is.
//!
//! One function per refusal `alo-encrypting` can make, each of them an
//! exhaustive match. That is the whole mechanism: a variant added to any of
//! those refusals stops this crate compiling until somebody has written the
//! sentence for it, so *every refusal on the road is a sentence in the
//! vocabulary* is held by the compiler rather than by a test that counts.
//!
//! # Why the mapping is here and not in the refusals
//!
//! Because `alo-encrypting` depends on nothing — it holds a recovery key for
//! the length of one screen and must not be able to serialise, log or send
//! anything — and a vocabulary is a dependency. The refusals keep their English
//! `Display` for a service log, the way `alo-drives`' refusals do, and what a
//! person reads is this.

use alo_encrypting::{
    NoChipToSealTo, NotADisk, NotAPassphrase, NotAPin, NotARecoveryKey, NotWhatWasShown,
    TheDiskRefused,
};
use alo_strings::{Filling, Said, Strings, Word};

use crate::words;

/// One of these sentences, in the language the person reads.
///
/// None of them is filled in with anything: every number a person needs beside
/// one — how many characters a PIN takes, the recovery key itself — is shown
/// beside the line rather than written into it, which is what the translator's
/// notes say and what keeps a secret out of a template.
#[must_use]
pub fn in_the_persons_language(word: Word, strings: &Strings) -> Said {
    strings.say(&word.key(), &Filling::nothing())
}

/// What a person is told when what they typed was not a PIN.
#[must_use]
pub const fn about_a_pin(why: NotAPin) -> Word {
    match why {
        NotAPin::Nothing => words::NO_PIN_WAS_TYPED,
        NotAPin::TooShort { .. } => words::THE_PIN_IS_TOO_SHORT,
        NotAPin::TheTwoDoNotMatch => words::THE_TWO_PINS_DIFFER,
    }
}

/// What a person is told when what they typed was not a passphrase.
#[must_use]
pub const fn about_a_passphrase(why: NotAPassphrase) -> Word {
    match why {
        NotAPassphrase::Nothing => words::NO_PASSPHRASE_WAS_TYPED,
        NotAPassphrase::TooShort { .. } => words::THE_PASSPHRASE_IS_TOO_SHORT,
        NotAPassphrase::TheTwoDoNotMatch => words::THE_TWO_PASSPHRASES_DIFFER,
    }
}

/// What a person is told when what the machine made was not a recovery key.
///
/// Both of these are alo OS's own fault rather than anything the person did,
/// and both of them stop the encryption rather than going on without a key
/// anybody could recover the machine with.
#[must_use]
pub const fn about_a_recovery_key(why: NotARecoveryKey) -> Word {
    match why {
        NotARecoveryKey::Nothing => words::NO_RECOVERY_KEY_WAS_MADE,
        NotARecoveryKey::NotEightGroups { .. }
        | NotARecoveryKey::AGroupIsNotEight { .. }
        | NotARecoveryKey::NotTheAlphabet => words::THE_RECOVERY_KEY_IS_THE_WRONG_SHAPE,
    }
}

/// What a person is told when what they typed back was not the key they were
/// shown.
#[must_use]
pub const fn about_what_was_typed_back(why: NotWhatWasShown) -> Word {
    match why {
        NotWhatWasShown::Nothing => words::THE_KEY_WAS_NOT_TYPED_BACK,
        NotWhatWasShown::NotThisKey => words::THAT_IS_NOT_THE_KEY_SHOWN,
    }
}

/// What a person is told when their machine's chip cannot hold the key.
///
/// One sentence for all three of the ways a chip cannot: a chip that is absent,
/// one that is not ready and one nothing would say about are the same fact to
/// somebody being asked for a passphrase, and telling them which would be
/// telling them about a part they cannot do anything about.
#[must_use]
pub const fn about_the_chip(_why: &NoChipToSealTo) -> Word {
    words::THIS_MACHINE_HAS_NO_CHIP_FOR_THE_KEY
}

/// What a person is told when the disk that was chosen cannot be used.
#[must_use]
pub const fn about_the_disk(why: NotADisk) -> Word {
    match why {
        NotADisk::NotAName => words::THAT_IS_NOT_A_DISK,
        NotADisk::APartition => words::THAT_IS_PART_OF_A_DISK,
        NotADisk::NotAPartitionNumber => words::THERE_IS_NO_SUCH_PART_OF_THE_DISK,
    }
}

/// What a person is told when the disk itself refused.
#[must_use]
pub const fn about_what_the_disk_refused(why: TheDiskRefused) -> Word {
    match why {
        TheDiskRefused::WhatWasGivenDoesNotOpenIt => words::THAT_DOES_NOT_OPEN_THIS_MACHINE,
        TheDiskRefused::ThisIsNotAnEncryptedVolume => words::THIS_DISK_IS_NOT_ENCRYPTED,
        TheDiskRefused::ItIsAlreadyOpen => words::IT_IS_ALREADY_OPEN,
        TheDiskRefused::TheRentedToolWouldNotDoIt => words::THIS_MACHINE_COULD_NOT_DO_IT,
        TheDiskRefused::TheRentedToolIsNotOnThisMachine => words::PART_OF_THIS_MACHINE_IS_MISSING,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alo_encrypting::{A_PASSPHRASE_IS_AT_LEAST, A_PIN_IS_AT_LEAST, HowItUnlocks, Pin, TheChip};

    /// Every refusal `alo-encrypting` can make, as a key of this crate's.
    fn every_refusal() -> Vec<String> {
        let mut keys: Vec<String> = Vec::new();
        for why in [
            NotAPin::Nothing,
            NotAPin::TooShort {
                at_least: A_PIN_IS_AT_LEAST,
            },
            NotAPin::TheTwoDoNotMatch,
        ] {
            keys.push(about_a_pin(why).key().to_string());
        }
        for why in [
            NotAPassphrase::Nothing,
            NotAPassphrase::TooShort {
                at_least: A_PASSPHRASE_IS_AT_LEAST,
            },
            NotAPassphrase::TheTwoDoNotMatch,
        ] {
            keys.push(about_a_passphrase(why).key().to_string());
        }
        for why in [
            NotARecoveryKey::Nothing,
            NotARecoveryKey::NotEightGroups { found: 2 },
            NotARecoveryKey::AGroupIsNotEight { found: 7 },
            NotARecoveryKey::NotTheAlphabet,
        ] {
            keys.push(about_a_recovery_key(why).key().to_string());
        }
        for why in [NotWhatWasShown::Nothing, NotWhatWasShown::NotThisKey] {
            keys.push(about_what_was_typed_back(why).key().to_string());
        }
        for why in [
            NotADisk::NotAName,
            NotADisk::APartition,
            NotADisk::NotAPartitionNumber,
        ] {
            keys.push(about_the_disk(why).key().to_string());
        }
        for why in TheDiskRefused::ALL {
            keys.push(about_what_the_disk_refused(why).key().to_string());
        }
        keys
    }

    /// **Every refusal on the road has a sentence this crate declares.**
    #[test]
    fn every_refusal_is_a_sentence_this_crate_declares() {
        for key in every_refusal() {
            assert!(
                words::EVERY_WORD
                    .iter()
                    .any(|word| word.key().to_string() == key),
                "{key} is said by no word this crate declares"
            );
        }
    }

    /// **The chip's refusal has a sentence too**, made the way a person's
    /// machine really makes one: by a PIN being asked for where no chip can
    /// hold a key.
    #[test]
    fn the_chips_refusal_is_a_sentence_too() {
        let Ok(pin) = Pin::typed("197246", "197246") else {
            unreachable!("six characters twice is a PIN")
        };
        let Err(why) = HowItUnlocks::sealed_to_the_chip(TheChip::Absent, pin) else {
            unreachable!("a machine with no chip cannot seal a key to one")
        };
        assert_eq!(
            about_the_chip(&why).key().to_string(),
            words::THIS_MACHINE_HAS_NO_CHIP_FOR_THE_KEY
                .key()
                .to_string()
        );
    }

    /// **The refusal a person is most likely to meet names the way out.**
    /// Somebody at a machine that will not open needs to be told about the key
    /// they wrote down, in the same sentence, every time.
    #[test]
    fn the_refusal_a_person_meets_names_the_key_they_wrote_down() {
        let said = about_what_the_disk_refused(TheDiskRefused::WhatWasGivenDoesNotOpenIt).says();
        assert!(said.contains("key you wrote down"), "{said}");
    }

    /// **Two refusals that a person can do nothing about do not ask them to try
    /// again**, because asking somebody to retype a correct secret is how a
    /// person comes to believe they have lost their machine.
    #[test]
    fn a_refusal_that_is_ours_does_not_ask_the_person_to_try_again() {
        for why in [
            TheDiskRefused::TheRentedToolIsNotOnThisMachine,
            TheDiskRefused::TheRentedToolWouldNotDoIt,
        ] {
            let said = about_what_the_disk_refused(why).says();
            assert!(!said.contains("try again"), "{said}");
        }
    }
}
