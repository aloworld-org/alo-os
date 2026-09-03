//! Every string this crate can say, and the English beside each one.
//!
//! Until this file existed, `alo-agentd` said nothing in anybody's language,
//! and [`crate::refusing`] still holds the reason: a directory that belongs to
//! somebody else and a socket another daemon is listening on are read out of a
//! service log by whoever is standing the machine up. Those keep their English.
//!
//! What is new is that there is now somebody at the other end of a connection.
//! Three things this service refuses are refusals **of a request**, and a
//! request comes from an agent that will show its person what it was told, or
//! from the person's own shell. `docs/contracts/daemon-protocol.md` says a
//! message that is not acted on is refused in words and never dropped, and
//! these are the three ways this crate can be the one refusing.
//!
//! # There are only three, because a turn words its own answers
//!
//! Everything that happens *inside* a turn — a verb that is not on the list,
//! the grants at the moment of execution, a full disk, a change nobody
//! answered — is `alo_turn::NotDone::said`, and this crate carries it rather
//! than rewording it. That is item 9e's rule at the last boundary before a
//! person reads the sentence, and it is why holding a turn adds three strings
//! and not thirty.
//!
//! # None of them quotes anything a client sent
//!
//! `alo-protocol`'s rule, kept here for its reason: what arrives is bytes
//! somebody else wrote, and a sentence with a gap in it is the only road they
//! could take into something a person reads.

use alo_strings::Vocabulary;

/// One string a crate can say.
///
/// Re-exported as every other declaring crate does, so this crate's own files
/// name it as `crate::words::Word`.
pub use alo_strings::Word;

/// A second agent, arriving while one turn is already under way.
pub const A_TURN_IS_UNDER_WAY: Word = Word::saying(
    "agentd.a-turn-is-under-way",
    "this machine is already in a turn, so nothing has been started for you — try again when it is over",
)
.noting(
    "Said to an agent whose connection arrived while another agent's turn was running, and shown \
     to the person by whatever they were talking to. A \"turn\" is one stretch of work an agent \
     does after a person asked it something; alo OS runs one at a time.",
);

/// A second shell, arriving while one is already connected.
pub const SOMEBODY_IS_ALREADY_ANSWERING: Word = Word::saying(
    "agentd.somebody-is-already-answering",
    "something else on this machine is already answering for you — close it and try again",
)
.noting(
    "Said to a second window or program that connected to alo OS on the person's own side while \
     one was already connected. It is addressed to the person, who is being told which of the two \
     things in front of them to close.",
);

/// A question for a model, on a machine where nothing has been chosen to answer
/// one.
pub const NOTHING_ANSWERS_QUESTIONS: Word = Word::saying(
    "agentd.nothing-answers-questions",
    "nothing on this machine has been chosen to answer questions — choose a model or a provider in Settings",
)
.noting(
    "Said when an agent puts a question to a model and the person has picked neither a model that \
     runs on this machine nor a provider that answers elsewhere. \"Settings\" is the name of the \
     panel in alo OS and should be translated as that panel's name is.",
);

/// Everything this crate can say.
pub const EVERY_WORD: [Word; 3] = [
    A_TURN_IS_UNDER_WAY,
    SOMEBODY_IS_ALREADY_ANSWERING,
    NOTHING_ANSWERS_QUESTIONS,
];

/// Why this crate's own list could not be declared.
///
/// Not a refusal a person reads: it keeps its English and its `Display` for the
/// reason every other crate's does, which is that whoever reads it is whoever
/// is fixing the list.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase: a sentence that is not one, or a note that
    /// could not be attached.
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
pub fn agentd_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put this crate's words into a vocabulary something else is building.
///
/// # Errors
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
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_strings::Key;
    use std::collections::BTreeSet;

    /// Every key here is a key, which is what `Word::key` going through
    /// `Key::unchecked` owes each declaring crate.
    #[test]
    fn every_key_this_crate_writes_is_a_key() {
        for word in EVERY_WORD {
            assert!(Key::named(word.named()).is_ok(), "{}", word.named());
        }
    }

    /// A key means one string, so the list may not name one twice.
    #[test]
    fn no_key_is_declared_twice() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// **Every one of these carries a note**, because none of the three can be
    /// translated from its own words: a "turn" is this product's idea, and the
    /// other two name something in front of the person that a translator has to
    /// be told about.
    #[test]
    fn every_word_tells_a_translator_what_it_is_about() {
        for word in EVERY_WORD {
            assert!(
                word.note().is_some_and(|note| !note.trim().is_empty()),
                "{} has nothing to say to a translator",
                word.named()
            );
        }
    }

    /// **Nothing here has a gap in it.** A gap is the only road text a client
    /// wrote could take into a sentence a person reads, and everything that
    /// arrives at this service is written by somebody else.
    #[test]
    fn no_sentence_here_can_be_handed_something_a_client_wrote() {
        for word in EVERY_WORD {
            assert!(
                !word.says().contains('{'),
                "{} has a gap in it",
                word.named()
            );
        }
    }

    /// The list can be declared, which is what anything loading it does.
    #[test]
    fn this_crate_can_declare_its_own_words() {
        assert_eq!(agentd_words().unwrap().how_many(), EVERY_WORD.len());
    }

    /// **None of the three names anything alo OS rents.** `alo-saying` asks
    /// this of the machine's whole vocabulary, and this crate is the one list
    /// that is not in it: it is Linux, so it declares its own three on top.
    /// Asking here is what stops that exemption from being a hole — and this
    /// crate is where it would show first, because these are the strings
    /// written closest to a service log.
    #[test]
    fn nothing_this_crate_says_names_anything_we_rent() {
        let overheard = alo_saying::what_a_person_would_have_to_learn(&agentd_words().unwrap());
        assert!(
            overheard.is_empty(),
            "{}",
            overheard
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}
