//! Every string this crate can say, and the English beside each one.
//!
//! `CLAUDE.md` says hardcoded English is a bug. This is the list that stops it
//! being one here: a key, the sentence in the language the code is written in,
//! and the note a translator needs.
//!
//! # What is on this list, and what is deliberately not
//!
//! **What a pairing permits is here**, one sentence an arm. ADR 0003 says a
//! pairing is *enumerated*, and a list somebody can read to the end only counts
//! as enumerated in a language they read. These are the strings a person looks
//! at while deciding whether to let the machine down the corridor ask this one
//! anything, which makes them the strings in this crate it would be least
//! acceptable to leave in English.
//!
//! **The refusals are here too**, which is why [`crate::NotPaired`] has no
//! `Display`. Somebody reads these having just been told they cannot pair with
//! something, not having just read this file.
//!
//! **Nothing about discovery is here at all.** A machine found on the network
//! has no sentence, because there is nothing to tell anybody: presence is not
//! news, and a notification saying *a machine appeared* would turn an open
//! protocol into a thing that interrupts people in cafés.
//!
//! # The other machine's name is never translated
//!
//! `{machine}` is what a person called the machine when they paired with it, or
//! its identity if they called it nothing. It is somebody's own data, like a
//! filename in `alo-files` and a model's name in `alo-models`, and a
//! translation of it would be an invention.

use alo_strings::Vocabulary;

/// One string a crate can say.
///
/// Re-exported because this crate's own files, and the tests that read this
/// list, name it as `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// What a pairing permits — [`crate::MayAskIts`].
//
// Each of these goes in a list a person reads before agreeing to anything, and
// again in the list of pairings they can see afterwards. They are phrases
// rather than sentences, because they appear under a heading naming the other
// machine.
// ---------------------------------------------------------------------------

/// A paired machine may put questions to this one's models.
pub const MAY_ASK_ITS_MODELS: Word = Word::saying(
    "nearby.may.ask-its-models",
    "Ask questions of the models on this machine",
)
.noting(
    "Read while deciding whether to pair with another machine, and again in the list of machines \
     already paired with. \"Ask questions of\" is deliberately the whole of it: a paired machine \
     may ask, and may never act. The models are this machine's own — the reader is the person \
     being asked to share them.",
);

/// A paired machine may reach a workspace this one serves.
pub const MAY_REACH_ITS_WORKSPACE: Word = Word::saying(
    "nearby.may.reach-its-workspace",
    "Reach the workspace this machine serves",
)
.noting(
    "Read in the same list as \"nearby.may.ask-its-models\". A workspace here is the self-hosted \
     one an office runs — mail, documents, calendar — not a window or a desktop. The reader is \
     the person being asked to share it.",
);

// ---------------------------------------------------------------------------
// Why there is no pairing — [`crate::NotPaired`].
//
// Somebody reads these having just been told they cannot pair with something.
// Each one says what is true rather than what went wrong, because in three of
// the four cases nothing did. The fifth is read on the other side of the
// corridor, by whoever a machine without a pairing asked for something.
// ---------------------------------------------------------------------------

/// One machine's person has agreed and the other's has not.
pub const BOTH_MACHINES_HAVE_NOT_AGREED: Word = Word::saying(
    "nearby.not-paired.both-have-not-agreed",
    "Both machines have to agree. Confirm this on the other machine as well.",
)
.noting(
    "The commonest thing that happens here, and usually not a refusal at all: somebody has agreed \
     on one machine and has not yet walked over to the other. It deliberately does not say that \
     anybody refused — the machine cannot tell the difference between a refusal and an answer \
     that has not come yet, and guessing wrong would accuse somebody. Two sentences; the second \
     is what to do.",
);

/// A machine proposing to pair with itself.
pub const A_MACHINE_CANNOT_PAIR_WITH_ITSELF: Word = Word::saying(
    "nearby.not-paired.with-itself",
    "This is the machine you are using.",
)
.noting(
    "Said when somebody picks their own machine out of a list of machines on the network. It is a \
     statement of fact rather than a refusal, because that is what it is: there is nothing to \
     pair, and nothing has gone wrong.",
);

/// A pairing that would permit nothing.
pub const A_PAIRING_HAS_TO_PERMIT_SOMETHING: Word = Word::saying(
    "nearby.not-paired.permits-nothing",
    "Choose at least one thing the other machine may ask for.",
)
.noting(
    "Said when somebody has cleared every item in the list of what a pairing would permit. It \
     asks for what is missing rather than describing the state, because the reader is in front of \
     that list and can fix it.",
);

/// A pairing that does not end.
pub const A_PAIRING_HAS_TO_END: Word = Word::saying(
    "nearby.not-paired.has-to-end",
    "Choose how long this pairing lasts. Pairings always end, and can be renewed.",
)
.noting(
    "Covers a pairing asked for with no duration and one asked for with too long a duration — one \
     sentence for both, because what the reader has to do is the same and the second sentence is \
     the reason. \"Always\" is doing real work: it says this is how the product is rather than a \
     limit somebody set, which is what stops the reader looking for a setting that turns it off. \
     Nothing here can turn it off.",
);

/// This machine is not paired with the one that asked it for something.
pub const NOT_PAIRED_WITH_THE_ONE_THAT_ASKED: Word = Word::saying(
    "nearby.not-paired.not-with-the-one-that-asked",
    "This machine is not paired with the one that asked, so nothing it asked for was considered.",
)
.noting(
    "Said on the machine that was asked, about a machine that asked it to do something without a \
     pairing standing between them: one that was only seen on the network, one whose pairing has \
     run out, or one whose pairing was undone. It deliberately does not say which, because saying \
     which would tell the asker how to become paired. \"Was considered\" is the whole of it: no \
     grant was looked at and nobody on this machine was asked anything.",
);

/// Everything this crate can say.
pub const EVERY_WORD: [Word; 7] = [
    MAY_ASK_ITS_MODELS,
    MAY_REACH_ITS_WORKSPACE,
    BOTH_MACHINES_HAVE_NOT_AGREED,
    A_MACHINE_CANNOT_PAIR_WITH_ITSELF,
    A_PAIRING_HAS_TO_PERMIT_SOMETHING,
    A_PAIRING_HAS_TO_END,
    NOT_PAIRED_WITH_THE_ONE_THAT_ASKED,
];

/// Why this crate's list could not be declared.
#[derive(Debug, thiserror::Error)]
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
///
/// [`WordsError`], which the list above cannot cause.
pub fn nearby_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// # Errors
///
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced, because a key means one string and whoever declared it
/// first said what that string is.
pub fn declare_into(vocabulary: &mut Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD {
        vocabulary.says(word.phrase()?)?;
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::{EVERY_WORD, WordsError, declare_into, nearby_words};
    use alo_strings::{Key, Vocabulary};

    /// Every key in this file is a key, checked here because a key written in
    /// this file cannot arrive from anywhere else.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(Key::named(word.named()), Ok(word.key()), "{}", word.named());
        }
    }

    /// A key names one string.
    #[test]
    fn the_list_declares_into_a_vocabulary_once() {
        assert!(nearby_words().is_ok());
        let mut vocabulary = Vocabulary::empty();
        declare_into(&mut vocabulary).unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **Every string here carries a note.** A translator with no product in
    /// front of them cannot tell that *ask* is the whole of what a pairing
    /// permits, or that *both machines have to agree* is usually not a refusal.
    #[test]
    fn every_string_carries_a_note_for_whoever_translates_it() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// Every key is under this crate's own area, so nothing here can collide
    /// with another crate's list by accident.
    #[test]
    fn every_key_is_this_crates_own() {
        for word in EVERY_WORD {
            assert!(
                word.named().starts_with("nearby."),
                "{} is not under this crate's area",
                word.named()
            );
        }
    }
}
