//! Every string this crate can say, and the English beside each one.
//!
//! Three groups: where the machine stands, when an update applies, and the
//! promise an update keeps. Every sentence is about the person's machine and
//! what they choose, never about what is underneath it.
//!
//! # Nothing a person reads names the machinery
//!
//! Not the base's tooling, not a deployment, not a digest, not an image, not a
//! registry (`docs/features.md`: *the system updates; it does not "pull an
//! image"*). And nothing says *urgent*, *critical* or *required*, because this
//! machine has no update that is any of those. A test at the bottom of this file
//! reads every sentence for all of them.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported so this
/// crate's files name it as `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// Where the machine stands — `crate::Standing`.
// ---------------------------------------------------------------------------

/// An update is ready.
pub const READY: Word = Word::saying(
    "keeping-up.ready",
    "An update is ready. It will apply when you choose, and nothing you are doing will be \
     interrupted until then",
)
.noting(
    "Said when this machine has found a newer version of its own system. The update never \
     restarts the machine or closes anything by itself: the person decides when it applies, \
     usually the next time they restart. The sentence must keep both promises — that it waits for \
     the person, and that nothing is interrupted.",
);

/// Nothing to update.
pub const UP_TO_DATE: Word = Word::saying("keeping-up.up-to-date", "This machine is up to date")
    .noting("Said when the person checks for an update and there is none to apply.");

/// The answer about updates could not be understood.
pub const ANSWER_NOT_UNDERSTOOD: Word = Word::saying(
    "keeping-up.answer-not-understood",
    "The answer about whether there is an update could not be understood, so nothing on this \
     machine has changed",
)
.noting(
    "Said when this machine asked whether there is an update and the reply did not make sense to \
     it. The important half is the second: the person's machine is exactly as it was.",
);

// ---------------------------------------------------------------------------
// When it applies — `crate::WhenItApplies`.
// ---------------------------------------------------------------------------

/// The choice to apply it at the next restart.
pub const APPLY_AT_THE_NEXT_RESTART: Word = Word::saying(
    "keeping-up.when.at-the-next-restart",
    "Apply it the next time I restart",
)
.noting(
    "One of two choices a person has once an update is ready, written as the person speaking. \
     Chosen when nothing else has been: the update waits, and applies the next time the person \
     restarts this machine themselves.",
);

/// The choice to restart now and apply it.
pub const RESTART_AND_APPLY_NOW: Word = Word::saying(
    "keeping-up.when.now",
    "Restart now and apply it, which closes the applications that are open",
)
.noting(
    "The other of the two choices, written as an instruction the person gives. It says that open \
     applications close, because restarting closes them — the person is choosing that, and must \
     know it before they do.",
);

// ---------------------------------------------------------------------------
// The promise — `crate::THE_RULE`.
// ---------------------------------------------------------------------------

/// An update never restarts the machine.
pub const NEVER_RESTARTS: Word = Word::saying(
    "keeping-up.never.restarts",
    "An update never restarts this machine. It applies when you restart",
)
.noting(
    "The first of three promises shown together where updates are described in Settings. They are \
     promises, not settings: none of them can be turned off, and nothing overrides them.",
);

/// An update never closes an application.
pub const NEVER_CLOSES_AN_APPLICATION: Word = Word::saying(
    "keeping-up.never.closes-an-application",
    "An update never closes an application you have open",
)
.noting("The second of the three promises about updates.");

/// An update never interrupts the person.
pub const NEVER_INTERRUPTS: Word = Word::saying(
    "keeping-up.never.interrupts",
    "An update never interrupts what you are doing",
)
.noting(
    "The third of the three promises about updates. Interrupting means taking the person away \
     from their work: covering it, taking the keyboard, or asking something they did not start.",
);

/// Every string this crate can say.
pub const EVERY_WORD: [Word; 8] = [
    READY,
    UP_TO_DATE,
    ANSWER_NOT_UNDERSTOOD,
    APPLY_AT_THE_NEXT_RESTART,
    RESTART_AND_APPLY_NOW,
    NEVER_RESTARTS,
    NEVER_CLOSES_AN_APPLICATION,
    NEVER_INTERRUPTS,
];

/// Why this crate's own words could not be declared.
///
/// Neither can happen to the list above — the tests at the bottom of this file
/// say so — and [`declare_into`] can genuinely fail against a vocabulary that
/// already holds one of these keys.
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
pub fn keeping_up_words() -> Result<Vocabulary, WordsError> {
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

    /// Every key is a key, in this crate's area, and names one string.
    #[test]
    fn every_key_is_one_of_this_crates_and_names_one_string() {
        for word in EVERY_WORD {
            assert_eq!(alo_strings::Key::named(word.named()), Ok(word.key()));
            assert_eq!(word.key().area(), "keeping-up", "{}", word.named());
        }
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// The list declares, and a key already taken is not replaced.
    #[test]
    fn the_list_declares_and_nothing_is_replaced() {
        let mut vocabulary = keeping_up_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert!(matches!(
            declare_into(&mut vocabulary),
            Err(WordsError::List(_))
        ));
    }

    /// **Every word carries a note, and none has a gap** — nothing this crate
    /// says names a build, a place or a time, so there is nothing to fill.
    #[test]
    fn every_word_carries_a_note_and_has_no_gap() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
            assert!(
                word.phrase().unwrap().source().gaps().is_empty(),
                "{} has a gap",
                word.named()
            );
        }
    }

    /// **Nothing here names the machinery, hedges, or calls an update urgent.**
    #[test]
    fn nothing_here_names_the_machinery_hedges_or_calls_an_update_urgent() {
        for word in EVERY_WORD {
            let said = word.says().to_lowercase();
            for forbidden in [
                "bootc",
                "ostree",
                "deployment",
                "digest",
                "image",
                "registry",
                "container",
                "pull",
                "sha256",
                "reboot",
                "urgent",
                "critical",
                "required",
                "must ",
                "forced",
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
}
