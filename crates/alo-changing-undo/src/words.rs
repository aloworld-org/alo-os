//! The three sentences this pane has of its own, and the note a translator
//! works from.
//!
//! Three, because everything else a person reads here was written where it was
//! decided and is *said* rather than written again:
//!
//! | what a person reads | whose sentence it is |
//! |---|---|
//! | the one act they approve | `alo_keeping_up::WhatWasKept::forgetting` |
//! | a window that is not a window | `alo_keeping_up::HowFarBack`'s refusal |
//! | this machine keeps nothing of what your files were | `alo_keeping_up::words::NOT_UNDONE_NOTHING_KEEPS_WHAT_WAS_THERE` |
//! | why a size is not the whole truth | `alo_measuring::Counted::said` |
//!
//! A second wording of any of those would be a second thing to translate and a
//! second thing to get out of step, and a person who met both would have no way
//! to tell which was the machine's real answer.
//!
//! What was missing is what only a pane can say: **how far back this machine
//! keeps**, which nothing said because nothing showed it; **how much room that
//! is holding**, for the same reason; and **that nothing is being kept yet**,
//! which is a true and unalarming thing to find, and which a pane showing a bare
//! zero would leave a person guessing about.
//!
//! # Nothing here names the machinery
//!
//! Not a snapshot, not a subvolume, not `btrfs`, not a folder, not a unit, not
//! *root*. A person reads about changes that can be put back and the space that
//! costs. What holds it is alo OS's problem, and naming it would send somebody
//! looking at a directory they cannot read.
//!
//! # Nor does anything here say *undo*
//!
//! That word is this repository's, not a person's. What a person met was an
//! agent changing their files, and what they are offered is putting those
//! changes back — so these sentences say *changes the agent made* and *put
//! back*, which are the words the rest of `alo_keeping_up::words` already uses
//! to a person.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported so this
/// crate's files name it as `crate::words::Word`.
pub use alo_strings::Word;

/// The name the number of days is filled in under.
pub const DAYS: &str = "days";

/// The name the number of changes is filled in under.
pub const CHANGES: &str = "changes";

/// The name the size is filled in under.
pub const SIZE: &str = "size";

/// How far back this machine keeps what the agent changed.
pub const HOW_FAR_BACK: Word = Word::saying(
    "changing-undo.how-far-back",
    "Changes the agent makes can be put back for {days} days, or for the last {changes} changes — \
     whichever ends first",
)
.noting(
    "Shown where a person reads what their machine is keeping — the agent being the assistant \
     built into alo OS and not a person. Both numbers are limits and the smaller one wins: a \
     machine used hard reaches the count of changes long before the days run out, and one used \
     lightly runs out of days first. {days} is a whole number of days and {changes} a whole \
     number of changes, both at least one.",
);

/// How much room what can be put back is holding.
///
/// The size arrives already worded, because how to write a number of bytes for a
/// person is a question about drawing and this crate draws nothing — and because
/// a second opinion in this repository about what `5183545344` should read as is
/// the kind of second answer the rest of this crate exists to avoid.
pub const HOLDING: Word = Word::saying(
    "changing-undo.holding",
    "Keeping these changes so they can be put back is using {size}",
)
.noting(
    "Shown beside how far back a machine keeps changes the agent made. {size} is an amount of disk \
     space, already written the way this language writes one — do not translate the number or its \
     unit, only the sentence around it. It is what the space costs today and it goes up and down \
     as a person works.",
);

/// Nothing has been kept yet, so it is holding nothing.
pub const HOLDING_NOTHING: Word = Word::saying(
    "changing-undo.holding-nothing",
    "Nothing has needed keeping yet, so this is using no space",
)
.noting(
    "Shown instead of an amount, on a machine that can put the agent's changes back but has not \
     had to keep anything yet — because the agent has changed nothing, or because everything it \
     changed has passed out of the window above. It is an ordinary state and not a fault, and the \
     wording should not sound like a warning or like something is missing.",
);

/// Every sentence this crate can say.
pub const EVERY_WORD: [Word; 3] = [HOW_FAR_BACK, HOLDING, HOLDING_NOTHING];

/// Why this crate's words could not be declared.
#[derive(Debug, thiserror::Error)]
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
pub fn changing_undo_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
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
    use std::collections::BTreeSet;

    /// **Every sentence is a phrase, and no two share a key.**
    #[test]
    fn every_sentence_is_a_phrase_with_a_key_of_its_own() {
        for word in EVERY_WORD {
            word.phrase().unwrap();
        }
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// **The vocabulary takes them once**, and says so the second time rather
    /// than replacing one quietly.
    #[test]
    fn a_key_is_declared_once() {
        let mut vocabulary = changing_undo_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert!(declare_into(&mut vocabulary).is_err());
    }

    /// **Every key belongs to this crate**, so that a pane's sentence cannot be
    /// mistaken for the sentence of the crate that decides the thing.
    #[test]
    fn every_key_is_this_crates_own() {
        for word in EVERY_WORD {
            assert!(
                word.named().starts_with("changing-undo."),
                "{} is not this crate's key",
                word.named()
            );
        }
    }

    /// **Every sentence has a note for whoever translates it**, and the note
    /// says more than the sentence does.
    #[test]
    fn every_sentence_carries_a_note_for_a_translator() {
        for word in EVERY_WORD {
            let note = word.note().unwrap_or_default();
            assert!(
                note.len() > word.says().len(),
                "{} tells a translator less than it says",
                word.named()
            );
        }
    }

    /// **Nothing here names the machinery.** A person reads about their changes
    /// and their disk space; what holds it is never their problem.
    #[test]
    fn nothing_here_names_the_machinery() {
        for word in EVERY_WORD {
            let said = word.says().to_lowercase();
            for forbidden in [
                "snapshot",
                "subvolume",
                "btrfs",
                "ext4",
                "filesystem",
                "undo",
                "folder",
                "directory",
                "root",
                "unit",
                "daemon",
                "socket",
                "capability",
                "toml",
                "bytes",
                "probably",
                "might",
                "perhaps",
                "possibly",
            ] {
                assert!(
                    !said.contains(forbidden),
                    "{} says \"{forbidden}\"",
                    word.named()
                );
            }
        }
    }

    /// **No sentence here is a second wording of one that already exists.**
    /// Held against the two crates this pane says the rest of its sentences
    /// out of, by the sentence itself rather than by its key: a copy somebody
    /// made would arrive with a key of its own and this is what would catch it.
    #[test]
    fn no_sentence_here_is_a_second_wording_of_one_that_exists() {
        let elsewhere: Vec<&str> = alo_keeping_up::words::EVERY_WORD
            .iter()
            .chain(alo_measuring::words::EVERY_WORD.iter())
            .map(Word::says)
            .collect();
        for word in EVERY_WORD {
            assert!(
                !elsewhere.contains(&word.says()),
                "{} is word for word a sentence that already exists",
                word.named()
            );
        }
    }
}
