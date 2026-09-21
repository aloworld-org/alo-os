//! The four sentences about an update that `alo-keeping-up` had no reason to
//! have, and the note a translator works from.
//!
//! Everything else a person reads about an update was written where updates are
//! decided: *the update will apply the next time you restart*, *it could not be
//! prepared*, *this machine changed after the update was found*. Those are
//! `alo_keeping_up::words`, and [`crate::NotChanged`] says them rather than
//! writing them again — a second copy of a sentence is a second thing to
//! translate and a second thing to get out of step.
//!
//! What was missing is here: the three things only the **door** can answer.
//! An approval the broker would not take, a machine that has stopped writing
//! changes down, and nothing there to make them. They are the same three the
//! printers and the network each say in their own words, because they are facts
//! about the road rather than about what travelled it.
//!
//! # Nothing here names the machinery
//!
//! Not the broker, not a unit, not a socket, not a capability, not *root*, not
//! the base the machine is built on. A person applies an update or goes back;
//! what carries it out is alo OS's problem.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported so this
/// crate's files name it as `crate::words::Word`.
pub use alo_strings::Word;

/// The approval was not accepted.
pub const APPROVAL_NOT_ACCEPTED: Word = Word::saying(
    "changing-updates.refused.approval-not-accepted",
    "Nothing was changed, because the approval for it was not accepted: it was used already, \
     came too late, or was not given for this. Ask again, and approve it when you are asked",
)
.noting(
    "Said when a person's approval to apply an update, or to go back to the version before it, \
     reached the part of the machine that changes the system for everybody and was refused there. \
     An approval works once, for one change, and only for about a minute after it is given. The \
     machine is exactly as it was.",
);

/// Nothing is being written down, so nothing is changed.
pub const NOT_BEING_KEPT: Word = Word::saying(
    "changing-updates.refused.not-being-kept",
    "This machine has stopped keeping its record of changes to its system, so it makes none. \
     Nothing was changed. Restart the machine",
)
.noting(
    "Said when the machine could not write down a change to its own system before making it. It \
     makes no change it cannot write down, so it makes none until it is restarted. The next \
     restart starts this machine as it is now.",
);

/// Nothing is there to make the change.
pub const NOTHING_MAKES_CHANGES: Word = Word::saying(
    "changing-updates.refused.nothing-makes-changes",
    "The part of this machine that changes its system is not running, so nothing was changed. \
     Restart the machine",
)
.noting(
    "Said when a change a person approved could not be handed to the part of the machine that \
     changes the system for everybody who uses it. The machine is exactly as it was.",
);

/// What was handed over was not about an update at all.
pub const NOT_AN_UPDATE_CHANGE: Word = Word::saying(
    "changing-updates.refused.not-an-update-change",
    "This was not about an update, so nothing was changed",
)
.noting(
    "Said when something other than an approved update, or an approved return to the version \
     before one, was handed to the part of the machine that makes such changes. A person should \
     never see it; if they do, the machine is wrong rather than them.",
);

/// Every sentence this crate can say.
pub const EVERY_WORD: [Word; 4] = [
    APPROVAL_NOT_ACCEPTED,
    NOT_BEING_KEPT,
    NOTHING_MAKES_CHANGES,
    NOT_AN_UPDATE_CHANGE,
];

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
pub fn changing_updates_words() -> Result<Vocabulary, WordsError> {
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

    /// Every key is one of this crate's, and names one string.
    #[test]
    fn every_key_is_one_of_this_crates_and_names_one_string() {
        for word in EVERY_WORD {
            assert_eq!(alo_strings::Key::named(word.named()), Ok(word.key()));
            assert_eq!(word.key().area(), "changing-updates", "{}", word.named());
        }
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// The list declares, and a key already taken is not replaced.
    #[test]
    fn the_list_declares_and_nothing_is_replaced() {
        let mut vocabulary = changing_updates_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert!(matches!(
            declare_into(&mut vocabulary),
            Err(WordsError::List(_))
        ));
    }

    /// **Every word carries a note, and none of them has a gap.** Nothing here
    /// names a version, a digest or a build: a person reads about *an update*,
    /// and which one it is was said when they were offered it.
    #[test]
    fn every_word_carries_a_note_and_none_has_a_gap() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
            assert!(
                word.phrase().unwrap().source().gaps().is_empty(),
                "{} has a gap",
                word.named()
            );
        }
    }

    /// **Nothing here names the machinery, and nothing hedges.**
    #[test]
    fn nothing_here_names_the_machinery_or_hedges() {
        for word in EVERY_WORD {
            let said = word.says().to_lowercase();
            for forbidden in [
                "broker",
                "socket",
                "root",
                "token",
                "daemon",
                "unit",
                "capability",
                "bootc",
                "deployment",
                "image",
                "container",
                "digest",
                "luks",
                "tpm",
                "cups",
                "networkmanager",
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

    /// **Every sentence says nothing was changed, and what to do about it.**
    /// A person whose machine will not update needs to know, above all, that
    /// their machine is as it was.
    #[test]
    fn every_sentence_says_nothing_was_changed() {
        for word in EVERY_WORD {
            let said = word.says();
            assert!(
                said.contains("Nothing was changed") || said.contains("nothing was changed"),
                "{} does not say that nothing was changed",
                word.named()
            );
        }
    }
}
