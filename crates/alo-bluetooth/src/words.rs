//! Every sentence this crate can say.
//!
//! None of them names BlueZ, an adapter, a profile or a protocol. A person
//! pairing headphones is pairing headphones.

use alo_strings::Vocabulary;

pub use alo_strings::Word;

/// The radio is on.
pub const RADIO_ON: Word = Word::saying("bluetooth.radio-on", "Bluetooth is on").noting(
    "Beside the switch that turns it on and off. Bluetooth is a trademark and stays as it is in \
     every language.",
);

/// The radio is off, and off is complete.
pub const RADIO_OFF: Word = Word::saying(
    "bluetooth.radio-off",
    "Bluetooth is off. Nothing is connected and nothing can connect until you turn it on",
)
.noting(
    "Beside the switch. The second sentence is the promise and must not be softened to something \
     like not currently connected: off means every device, including ones that were connected a \
     moment ago and ones that ask.",
);

/// This machine has no Bluetooth at all.
pub const NO_BLUETOOTH_HERE: Word = Word::saying(
    "bluetooth.none-here",
    "This machine has no Bluetooth. Nothing found is not the same as nothing to find",
)
.noting(
    "Shown where the list of devices would be, on a machine with no radio in it. The second \
     sentence is why this is a separate message rather than an empty list: a person who sees an \
     empty list goes looking for their headphones.",
);

/// Nothing has been found yet.
pub const NOTHING_FOUND_YET: Word = Word::saying(
    "bluetooth.nothing-found-yet",
    "Nothing found yet. Put your device into pairing mode and it will appear here",
)
.noting(
    "Shown while this machine is looking. Pairing mode is the phrase device makers print in their \
     own instructions; use whatever that phrase is in your language.",
);

/// A device with nothing to show and nothing to type.
pub const PAIR_WITH_THIS: Word = Word::saying("bluetooth.pair-with-this", "Pair with this device?")
    .noting(
        "Asked about a device that has no way to show or take a code, such as most headphones.",
    );

/// Six digits to be typed on the device.
pub const TYPE_THIS_ON_IT: Word = Word::saying(
    "bluetooth.type-this-on-it",
    "Type this on the device, then press enter on it",
)
.noting(
    "Shown above six digits, for a keyboard. The digits are shown in full beside this sentence \
     and are never part of it.",
);

/// The same six digits at both ends.
pub const THE_SAME_ON_BOTH: Word = Word::saying(
    "bluetooth.the-same-on-both",
    "Check that the device shows these same digits, then say yes on both",
)
.noting(
    "Shown above six digits. The person is being asked to compare, which is the whole security of \
     this shape of pairing: translate it as a comparison, never as a confirmation of this machine \
     alone.",
);

/// A code the maker printed.
pub const ITS_OWN_CODE: Word = Word::saying(
    "bluetooth.its-own-code",
    "Type the code that came with the device",
)
.noting(
    "For older devices with a fixed code, printed on them or in their instructions — often 0000 \
     or 1234, which is why this machine never guesses it.",
);

/// A device is not another machine.
pub const NOT_A_MACHINE: Word = Word::saying(
    "bluetooth.not-a-machine",
    "This is a device, not another computer. Pairing it gives it nothing on this machine",
)
.noting(
    "Shown once where somebody pairs their first device. alo OS uses the word pairing for two \
     things — a device, and another computer this machine may work with — and only the second is \
     permission. If your language has two words, use the plainer one here.",
);

/// Forgetting a device, and what it takes with it.
pub const FORGET_THIS_DEVICE: Word = Word::saying(
    "bluetooth.forget-this-device",
    "Forget this device. Its keys are removed from this machine and it will have to be paired \
     again",
)
.noting(
    "On the button that removes a device. The second sentence is what makes it different from \
     disconnecting, which is why both halves are needed.",
);

/// Every string this crate can say, in the order this file declares them.
pub const EVERY_WORD: [Word; 10] = [
    RADIO_ON,
    RADIO_OFF,
    NO_BLUETOOTH_HERE,
    NOTHING_FOUND_YET,
    PAIR_WITH_THIS,
    TYPE_THIS_ON_IT,
    THE_SAME_ON_BOTH,
    ITS_OWN_CODE,
    NOT_A_MACHINE,
    FORGET_THIS_DEVICE,
];

/// Why this crate's own list could not be declared.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] alo_strings::WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] alo_strings::VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn bluetooth_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put every sentence this crate says into a vocabulary somebody else holds.
///
/// # Errors
/// [`WordsError`] where the vocabulary already has one of these keys.
pub fn declare_into(vocabulary: &mut Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD {
        vocabulary.says(word.phrase()?)?;
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **Every sentence has a note, and none of them names the stack.**
    #[test]
    fn nothing_here_names_what_a_person_does_not_have_to_know() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
            let said = word.says().to_lowercase();
            for never in ["bluez", "adapter", "a2dp", "hci", "profile", "d-bus"] {
                assert!(!said.contains(never), "{}: {never}", word.named());
            }
        }
        assert!(
            bluetooth_words()
                .unwrap()
                .phrase(&RADIO_OFF.key())
                .is_some()
        );
    }

    /// **The digits are never inside a sentence**, because a translator moving
    /// them would move what a person types.
    #[test]
    fn no_sentence_carries_the_digits_it_is_shown_beside() {
        for word in EVERY_WORD {
            assert!(
                !word.says().contains(|letter: char| letter.is_ascii_digit()),
                "{} has digits in it",
                word.named()
            );
        }
    }
}
