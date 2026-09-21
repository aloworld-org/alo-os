//! Every sentence a person reads while their disk is being encrypted, and the
//! note a translator works from.
//!
//! They are read in two places a person can be in: **during an install**, when
//! the machine is asking them for a secret and handing them a recovery key, and
//! **in front of a machine that will not open**, when the only thing between
//! them and their work is a key on a piece of paper. The second is the harder
//! audience, and the sentences are written for it: short, in the present tense,
//! and saying what to do next rather than what went wrong.
//!
//! # What none of them says
//!
//! No sentence names LUKS, a TPM, `cryptsetup`, a keyslot, a device or *root*.
//! A person is told about *this machine's security chip*, *the key you wrote
//! down* and *the disk*, because those are things they have; the rest are
//! things alo OS has, and naming them in a sentence somebody reads at six in the
//! morning helps nobody. `tests/what_this_crate_says.rs` holds it.

use alo_strings::Vocabulary;

pub use alo_strings::Word;

/// Asking for a PIN, on a machine whose chip will hold the key.
pub const CHOOSE_A_PIN: Word = Word::saying(
    "enrolling.choose-a-pin",
    "choose a PIN you will type when this machine starts",
)
.noting(
    "Asked once, while the machine is being set up. The PIN is short because this machine's \
     security chip counts wrong answers; do not translate it as \"password\". How many characters \
     at least is a number shown beside this line.",
);

/// Asking for a passphrase, on a machine with no chip to hold a key.
pub const CHOOSE_A_PASSPHRASE: Word = Word::saying(
    "enrolling.choose-a-passphrase",
    "choose a passphrase you will type when this machine starts",
)
.noting(
    "Asked once, while the machine is being set up, on a machine that has no security chip to \
     hold the key. It is longer than a PIN for that reason. How many characters at least is a \
     number shown beside this line.",
);

/// The recovery key, on the screen, once.
pub const WRITE_THIS_DOWN: Word = Word::saying(
    "enrolling.write-this-down",
    "write this key down and keep it away from this machine",
)
.noting(
    "Shown once, above a key of sixty-four characters. It is the only thing that opens this \
     machine if the person forgets what they chose. \"Away from this machine\" means on paper or \
     somewhere else entirely — a photograph on the same machine is no use when the machine is \
     the thing that will not open.",
);

/// Asking for it back.
pub const TYPE_IT_BACK: Word = Word::saying(
    "enrolling.type-it-back",
    "type the key back, so we know you have it",
)
.noting(
    "Asked immediately after the key is shown. Spacing, dashes and capitals do not have to match; \
     the letters do. This is the only confirmation there is, and the setting up does not go on \
     until it is given.",
);

/// It is done.
pub const THE_DISK_IS_ENCRYPTED: Word = Word::saying(
    "enrolling.the-disk-is-encrypted",
    "this machine's disk is encrypted",
)
.noting(
    "Said once the setting up has finished. A statement of fact a person can repeat to somebody \
     who asks whether their laptop is safe if it is stolen; not a congratulation.",
);

/// No PIN was typed.
pub const NO_PIN_WAS_TYPED: Word =
    Word::saying("enrolling.no-pin-was-typed", "type a PIN to go on").noting(
        "Shown when the box was left empty. It asks for the thing rather than reporting that it \
         was missing.",
    );

/// The PIN is too short.
pub const THE_PIN_IS_TOO_SHORT: Word =
    Word::saying("enrolling.the-pin-is-too-short", "that PIN is too short").noting(
        "Shown when fewer characters were typed than a PIN takes. How many are needed is a number \
     shown beside this line. It never says how many were typed — that is a fact about somebody's \
     secret.",
    );

/// The two typings of the PIN were not the same.
pub const THE_TWO_PINS_DIFFER: Word = Word::saying(
    "enrolling.the-two-pins-differ",
    "the two PINs are not the same",
)
.noting(
    "Shown when a PIN was typed twice and the two did not match. Typing it twice is deliberate: a \
     machine sealed to a PIN nobody typed correctly the first time is a machine opened with the \
     recovery key on its first start.",
);

/// No passphrase was typed.
pub const NO_PASSPHRASE_WAS_TYPED: Word = Word::saying(
    "enrolling.no-passphrase-was-typed",
    "type a passphrase to go on",
)
.noting(
    "Shown when the box was left empty. Like the PIN's, it asks for the thing rather than \
     reporting that it was missing.",
);

/// The passphrase is too short.
pub const THE_PASSPHRASE_IS_TOO_SHORT: Word = Word::saying(
    "enrolling.the-passphrase-is-too-short",
    "that passphrase is too short",
)
.noting(
    "Shown when fewer characters were typed than a passphrase takes. How many are needed is a \
     number shown beside this line. A passphrase is longer than a PIN because nothing counts the \
     wrong answers to it.",
);

/// The two typings of the passphrase were not the same.
pub const THE_TWO_PASSPHRASES_DIFFER: Word = Word::saying(
    "enrolling.the-two-passphrases-differ",
    "the two passphrases are not the same",
)
.noting(
    "Shown when a passphrase was typed twice and the two did not match. Typing it twice is \
     deliberate: nothing counts the wrong answers to a passphrase, so a machine whose passphrase \
     was mistyped once is a machine opened with the recovery key on its first start.",
);

/// Nothing was typed back where the recovery key was asked for.
pub const THE_KEY_WAS_NOT_TYPED_BACK: Word = Word::saying(
    "enrolling.the-key-was-not-typed-back",
    "type the key back to go on",
)
.noting(
    "Shown when the box was left empty, or held nothing but dashes and spaces. The key is still \
     on the screen above it.",
);

/// What was typed back was not the key.
pub const THAT_IS_NOT_THE_KEY_SHOWN: Word = Word::saying(
    "enrolling.that-is-not-the-key-shown",
    "that is not the key on the screen — check it and try again",
)
.noting(
    "Shown when something was typed back and it was not the key. The key is shown again beside \
     it: being shown it twice is far better than a machine nobody can open.",
);

/// The machine's chip cannot hold the key.
pub const THIS_MACHINE_HAS_NO_CHIP_FOR_THE_KEY: Word = Word::saying(
    "enrolling.no-chip-for-the-key",
    "this machine has no security chip that can hold the key, so you will type a passphrase every \
     time it starts",
)
.noting(
    "Said once, while the machine is being set up, on a machine whose chip is missing, not ready, \
     or could not be asked about. It is not a failure and nothing has gone wrong: the disk is \
     still encrypted, and what changes is what the person types.",
);

/// Nothing was made where a recovery key was expected.
pub const NO_RECOVERY_KEY_WAS_MADE: Word = Word::saying(
    "enrolling.no-recovery-key-was-made",
    "this machine could not make a recovery key, so its disk has not been encrypted",
)
.noting(
    "Said when the part of the machine that makes the key answered with nothing. Encryption is \
     not gone on with, deliberately: a disk nobody can recover is worse than a disk nobody \
     encrypted.",
);

/// What was made was not the shape of a recovery key.
pub const THE_RECOVERY_KEY_IS_THE_WRONG_SHAPE: Word = Word::saying(
    "enrolling.the-recovery-key-is-the-wrong-shape",
    "the recovery key this machine made is not one it can show you, so its disk has not been \
     encrypted",
)
.noting(
    "Said when the key that came back was not sixty-four characters in eight groups, or held a \
     character that is not one of the sixteen a recovery key uses. It is alo OS's own fault and \
     the person can do nothing about it; it exists so that they are never shown a key that is \
     not the key.",
);

/// The disk that was chosen is not one this machine can name.
pub const THAT_IS_NOT_A_DISK: Word = Word::saying(
    "enrolling.that-is-not-a-disk",
    "this machine cannot use the disk that was chosen",
)
.noting(
    "Said when what was named is not a disk this machine knows by its own identity. The person \
     chooses again from the list.",
);

/// Part of a disk was named where a disk was meant.
pub const THAT_IS_PART_OF_A_DISK: Word = Word::saying(
    "enrolling.that-is-part-of-a-disk",
    "that is part of a disk rather than a whole one",
)
.noting(
    "Said when what was named is one piece of a disk. \"Part of a disk\" is something a person can \
     act on where \"not understood\" is not.",
);

/// There is no such part of that disk.
pub const THERE_IS_NO_SUCH_PART_OF_THE_DISK: Word = Word::saying(
    "enrolling.no-such-part-of-the-disk",
    "there is no such part of that disk",
)
.noting(
    "Said when a piece of a disk was asked for by a number the disk does not have. Nobody types \
     that number: it means alo OS asked for something this disk has not got, and the person \
     chooses a disk again.",
);

/// What was given does not open the volume.
pub const THAT_DOES_NOT_OPEN_THIS_MACHINE: Word = Word::saying(
    "enrolling.that-does-not-open-this-machine",
    "that does not open this machine — try again, or use the key you wrote down",
)
.noting(
    "The one a person is most likely to read, and the most important to get right. It is shown \
     when a passphrase or a recovery key was typed and it was not this machine's. It always names \
     the way out: the key they wrote down when the machine was set up.",
);

/// What was named is not an encrypted volume.
pub const THIS_DISK_IS_NOT_ENCRYPTED: Word = Word::saying(
    "enrolling.this-disk-is-not-encrypted",
    "this disk is not an encrypted one",
)
.noting(
    "Said when the disk that was opened turned out never to have been encrypted. Nothing is wrong \
     with the person's key; they are at the wrong disk.",
);

/// It is already open.
pub const IT_IS_ALREADY_OPEN: Word =
    Word::saying("enrolling.it-is-already-open", "this disk is already open").noting(
        "Said when the disk was opened a second time, or something else is still using it. Nothing \
     has gone wrong and nothing is lost.",
    );

/// The rented tool refused and said nothing more.
pub const THIS_MACHINE_COULD_NOT_DO_IT: Word = Word::saying(
    "enrolling.this-machine-could-not-do-it",
    "this machine could not do that to its disk",
)
.noting(
    "Said when the part of the machine that changes the disk refused and did not say why. It is \
     deliberately vague because the machine genuinely does not know more; what it must never do \
     is guess and tell the person their key was wrong.",
);

/// Part of the machine is missing.
pub const PART_OF_THIS_MACHINE_IS_MISSING: Word = Word::saying(
    "enrolling.part-of-this-machine-is-missing",
    "part of this machine is missing, so its disk cannot be encrypted",
)
.noting(
    "Said when what changes the disk is not on the machine at all. It is alo OS's own fault, and \
     the sentence says so rather than asking the person to try their secret again.",
);

/// Every sentence this crate can say.
pub const EVERY_WORD: [Word; 24] = [
    CHOOSE_A_PIN,
    CHOOSE_A_PASSPHRASE,
    WRITE_THIS_DOWN,
    TYPE_IT_BACK,
    THE_DISK_IS_ENCRYPTED,
    NO_PIN_WAS_TYPED,
    THE_PIN_IS_TOO_SHORT,
    THE_TWO_PINS_DIFFER,
    NO_PASSPHRASE_WAS_TYPED,
    THE_PASSPHRASE_IS_TOO_SHORT,
    THE_TWO_PASSPHRASES_DIFFER,
    THE_KEY_WAS_NOT_TYPED_BACK,
    THAT_IS_NOT_THE_KEY_SHOWN,
    THIS_MACHINE_HAS_NO_CHIP_FOR_THE_KEY,
    NO_RECOVERY_KEY_WAS_MADE,
    THE_RECOVERY_KEY_IS_THE_WRONG_SHAPE,
    THAT_IS_NOT_A_DISK,
    THAT_IS_PART_OF_A_DISK,
    THERE_IS_NO_SUCH_PART_OF_THE_DISK,
    THAT_DOES_NOT_OPEN_THIS_MACHINE,
    THIS_DISK_IS_NOT_ENCRYPTED,
    IT_IS_ALREADY_OPEN,
    THIS_MACHINE_COULD_NOT_DO_IT,
    PART_OF_THIS_MACHINE_IS_MISSING,
];

/// Why this crate's own list could not be declared.
///
/// Not a refusal anybody reads in their own language: what has gone wrong is
/// the list, so whoever reads it is whoever is fixing it.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] alo_strings::WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] alo_strings::VocabularyError),
}

/// This crate's words, as a vocabulary of their own.
///
/// For a test, and for anything holding only this list. A machine loads one
/// vocabulary that every crate declares into, which is [`declare_into`].
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn enrolling_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Declare this crate's words into a vocabulary.
///
/// # Errors
/// [`WordsError`], where a sentence is not a phrase or a key is already taken.
pub fn declare_into(vocabulary: &mut Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD {
        vocabulary.says(word.phrase()?)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **No two sentences share a key**, which is what would make one of them
    /// unreachable in every language at once.
    #[test]
    fn no_two_sentences_share_a_key() {
        let mut keys: Vec<String> = EVERY_WORD
            .iter()
            .map(|word| word.key().to_string())
            .collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), EVERY_WORD.len(), "two sentences share a key");
    }

    /// **Every sentence has a note**, because a translator with no note writes
    /// a guess.
    #[test]
    fn every_sentence_has_a_note_a_translator_can_work_from() {
        for word in EVERY_WORD {
            let note = word.note().unwrap_or_default();
            assert!(note.len() > 40, "{}: {note}", word.key());
            assert!(!word.says().is_empty(), "{}", word.key());
        }
    }

    /// The list declares.
    #[test]
    fn the_list_declares() {
        let mut vocabulary = Vocabulary::empty();
        assert_eq!(declare_into(&mut vocabulary), Ok(()));
        for word in EVERY_WORD {
            assert!(vocabulary.phrase(&word.key()).is_some(), "{}", word.key());
        }
    }
}
