//! Every string this crate can say, and the English beside each one.
//!
//! Four, and every one of them is a check that came back with no answer. The
//! answers themselves are already worded and are not worded again here: *an
//! update is ready* and *this machine is up to date* are
//! `alo_keeping_up::Standing::said`, and *the answer could not be understood*
//! is `alo_keeping_up::words::ANSWER_NOT_UNDERSTOOD`, which
//! [`crate::NoAnswer::NotUnderstood`] renders rather than saying a second time.
//!
//! # Every one of them ends the same way, and that is the point
//!
//! A check changes nothing on the machine — it asks a question — so a person
//! who reads one of these has lost nothing and has nothing to repair. Each
//! sentence says so, because a refusal that leaves somebody wondering whether
//! their machine is half-updated is worse than no sentence at all.
//!
//! # Nothing a person reads names the machinery
//!
//! Not the place's software, not a registry, not a tag, not a digest, not a
//! manifest (`docs/features.md`: *a person never learns the name of anything we
//! rented*). *The place this machine's updates come from* is as close as any of
//! them goes, and a test at the bottom of this file holds every sentence to the
//! same list `alo-keeping-up` holds its own to.
//!
//! # And none of them asks anybody to do anything
//!
//! No *try again later*, no *check your connection*, no *contact your
//! administrator*. A machine that cannot reach the place its updates come from
//! will ask again the next time somebody asks it to or the next time it starts,
//! and saying so once is [`crate::SaidOnce`]'s whole job — a line telling a
//! person to keep trying would be the nagging that promise exists to prevent.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported so this
/// crate's files and its tests name it as `crate::words::Word`.
pub use alo_strings::Word;

/// There is no road out of this machine at all.
pub const NO_WAY_OUT: Word = Word::saying(
    "looking.no-way-out",
    "This machine cannot reach the place its updates come from, so it cannot tell whether there \
     is a newer version. Nothing on this machine has changed",
)
.noting(
    "Said when the machine looked for an update and could not reach anything at all — no network, \
     or nothing answering where this machine's updates come from. It is said once rather than at \
     every attempt. The second sentence is the important one: looking changes nothing, so the \
     person has lost nothing and has nothing to repair.",
);

/// It was reached, and nothing came back.
pub const NOTHING_CAME_BACK: Word = Word::saying(
    "looking.nothing-came-back",
    "The place this machine's updates come from did not answer, so nothing on this machine has \
     changed",
)
.noting(
    "Said when the machine reached the place its updates come from and no answer arrived before \
     it gave up waiting. Different from being unable to reach it at all: something is there and \
     it is not answering. The second half says that looking changes nothing.",
);

/// It answered, and the answer was a refusal.
pub const IT_REFUSED: Word = Word::saying(
    "looking.it-refused",
    "The place this machine's updates come from would not answer this machine, so nothing on this \
     machine has changed",
)
.noting(
    "Said when the place this machine's updates come from answered, and what it answered was a \
     refusal rather than an answer — for example because it does not know this machine, or is \
     turning it away for the moment. The person is not asked to do anything about it.",
);

/// It answered, and it is offering no version of this system.
pub const NOTHING_IS_OFFERED: Word = Word::saying(
    "looking.nothing-is-offered",
    "The place this machine's updates come from is offering no version of this system, so nothing \
     on this machine has changed",
)
.noting(
    "Said when the place this machine's updates come from answered properly and had no version of \
     alo OS in it to offer. It is not a failure of the machine and not a failure of the \
     connection; there is simply nothing there. \"alo OS\" is the product's name and is never \
     translated.",
);

/// Every string this crate can say, in the order this file declares them.
pub const EVERY_WORD: [Word; 4] = [
    NO_WAY_OUT,
    NOTHING_CAME_BACK,
    IT_REFUSED,
    NOTHING_IS_OFFERED,
];

/// Why this crate's own words could not be declared.
///
/// Neither can happen to the list above — the tests at the bottom of this file
/// are what say so. It is a `Result` rather than an unwrap because a library
/// that panics on its own string table takes the shell with it.
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
pub fn looking_words() -> Result<Vocabulary, WordsError> {
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
/// [`WordsError::List`] if the vocabulary already holds one of these keys.
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
    use std::collections::BTreeSet;

    /// Every key written here is a key, which `Word::key` does not check.
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

    /// A key names one string.
    #[test]
    fn no_two_words_are_named_the_same() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// Every one of them is in this crate's own area of the one vocabulary.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), "looking", "{}", word.named());
        }
    }

    /// The list declares, and nothing in it is refused.
    #[test]
    fn the_whole_list_declares() {
        assert_eq!(looking_words().unwrap().how_many(), EVERY_WORD.len());
    }

    /// A vocabulary that already holds one of these keeps its own.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = looking_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **Every sentence says the machine is as it was.** A check changes
    /// nothing, and a refusal that left that unsaid would leave somebody
    /// wondering whether their machine is half-updated.
    #[test]
    fn every_sentence_says_nothing_on_this_machine_changed() {
        for word in EVERY_WORD {
            let said = word.says().to_lowercase();
            assert!(
                said.contains("nothing on this machine has changed"),
                "{} does not say the machine is as it was",
                word.named()
            );
        }
    }

    /// **Nothing here names the machinery**, held to the list
    /// `alo-keeping-up` holds its own sentences to and to the four words this
    /// crate could have leaked that that list does not carry.
    #[test]
    fn no_sentence_here_names_the_machinery() {
        for word in EVERY_WORD {
            let read =
                format!("{} {}", word.says(), word.note().unwrap_or_default()).to_ascii_lowercase();
            for machinery in [
                "bootc",
                "ostree",
                "deployment",
                "digest",
                "registry",
                "container",
                "sha256",
                "manifest",
                "http",
                "proxy",
            ] {
                assert!(
                    !read.contains(machinery),
                    "{} says \"{machinery}\"",
                    word.named()
                );
            }
        }
    }

    /// **Nothing here asks anybody to do anything.** A machine that cannot
    /// reach the place its updates come from asks again when somebody asks it
    /// to or when it next starts, and a line telling a person to keep trying
    /// would be the nagging [`crate::SaidOnce`] exists to prevent.
    #[test]
    fn nothing_here_asks_the_person_to_do_anything() {
        for word in EVERY_WORD {
            let said = word.says().to_lowercase();
            for asking in [
                "try again",
                "check your",
                "contact",
                "administrator",
                "please",
                "make sure",
                "later",
            ] {
                assert!(
                    !said.contains(asking),
                    "{} asks the person to \"{asking}\"",
                    word.named()
                );
            }
        }
    }

    /// **No sentence hedges.** Each says what happened, once.
    #[test]
    fn nothing_here_hedges() {
        for word in EVERY_WORD {
            let said = word.says().to_lowercase();
            for hedge in ["probably", "might", "perhaps", "possibly", "may be"] {
                assert!(!said.contains(hedge), "{} hedges", word.named());
            }
        }
    }

    /// Every word carries a note a translator can work from, and every one of
    /// them is a whole sentence with nothing to fill in: these are read at a
    /// moment something already did not work, and a gap left in one would put
    /// braces in front of a person at the worst moment.
    #[test]
    fn every_word_is_whole_and_carries_a_note() {
        for word in EVERY_WORD {
            assert!(
                word.note().is_some_and(|note| note.len() > 30),
                "{} has no note a translator could work from",
                word.named()
            );
            assert!(
                word.phrase().unwrap().source().gaps().is_empty(),
                "{} has a gap in it",
                word.named()
            );
            assert!(
                word.says().chars().next().is_some_and(char::is_uppercase),
                "{} does not begin a sentence",
                word.named()
            );
        }
    }
}
