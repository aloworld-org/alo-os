//! Every string this crate can say, and the English beside each one.
//!
//! **Two**, and the second is not a sentence about this session. A lock screen
//! may say that the machine is locked ([`LOCKED`]), and it may say the words on
//! the one road off it that is not the way back in ([`SOMEBODY_ELSE`]). Nothing
//! else a person reads is this crate's: the time is formatted by whoever formats
//! times, the battery is read by whoever reads it, the egress light is
//! `alo-indicator`'s own sentence, and a refused password is `alo-accounts`'
//! one refusal carried through the greeting unchanged.
//!
//! That is also why a refusal here has no sentence of its own. The agent's key
//! pressed at a locked machine, an approval answered at one, and a machine with
//! no account for anybody to sign in to, are all told [`LOCKED`] — the one thing
//! the lock screen already says. A second sentence naming what was refused
//! would be a second thing on a screen whose whole rule is that it shows four.
//!
//! [`SOMEBODY_ELSE`] is held to the same rule from the other side: it is what a
//! person reads on the way to the sign-in, and it names nobody — not who locked
//! the machine and not who might arrive
//! ([ADR 0061](../../../docs/decisions/0061-a-locked-screen-offers-a-road-to-the-greeter.md)).

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files and the tests that read its list name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

/// The machine is locked.
pub const LOCKED: Word = Word::saying("locking.locked", "This machine is locked").noting(
    "Shown on the lock screen, beside the time, and read aloud by a screen reader when the \
     screen locks. It is also the whole answer when somebody at a locked machine presses the key \
     that opens the agent or tries to answer a waiting change: the lock screen deliberately says \
     nothing else, so do not add who the machine belongs to or what is waiting. Say it the way a \
     sign on a door would: short, and not an error.",
);

/// Let somebody else use this machine.
pub const SOMEBODY_ELSE: Word = Word::saying("locking.somebody-else", "Somebody else").noting(
    "The one road off a lock screen that is not the way back in: it hands the screen to the \
     sign-in so another person can use this machine, and whoever locked it stays signed in with \
     everything they had open still running. Two or three words where the language has them, the \
     way a label on a button is written rather than a question or an instruction. It names \
     nobody, and a translation must not name anybody either — not the person who locked the \
     machine and not the person arriving. Avoid a word that suggests the first person is being \
     put out or signed out: nothing is ended by taking this road.",
);

/// Every string this crate can say, in the order this file declares them.
pub const EVERY_WORD: [Word; 2] = [LOCKED, SOMEBODY_ELSE];

/// Why this crate's own words could not be declared.
///
/// Neither can happen to the list above — the tests at the bottom of this file
/// are what say so. It is a `Result` rather than an unwrap because a library
/// that panics on its own string table takes the surface with it, and because
/// [`declare_into`] can genuinely fail against a vocabulary that already holds
/// the key.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn locking_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// The machine has one vocabulary and every crate adds its own to it —
/// `alo-saying` is the one place that calls all of these.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced.
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

    /// What we ship is held to the rule everybody else is held to.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(
                alo_strings::Key::named(word.named()),
                Ok(word.key()),
                "{}",
                word.named()
            );
        }
    }

    /// Every one of them sorts under this crate's area.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), "locking", "{}", word.named());
        }
    }

    /// The list declares, and a second declaration replaces nothing.
    #[test]
    fn the_whole_list_declares_once() {
        let mut vocabulary = locking_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert!(matches!(
            declare_into(&mut vocabulary),
            Err(WordsError::List(_))
        ));
    }

    /// **Nothing here has a gap in it.** A gap is the one road a name, a
    /// document or a sender could take onto the lock screen, and the lock
    /// screen is where none of them may appear.
    #[test]
    fn nothing_here_has_a_gap_anything_private_could_enter_by() {
        for word in EVERY_WORD {
            assert!(
                word.phrase().unwrap().source().gaps().is_empty(),
                "{} has a gap in it",
                word.named()
            );
        }
    }

    /// Every word carries a note: the translator has to be able to picture a
    /// screen with almost nothing on it.
    #[test]
    fn every_word_carries_a_note_for_the_translator() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }
}
