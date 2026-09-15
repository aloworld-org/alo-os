//! Every string this crate can say, and the English beside each one.
//!
//! Where the machine stands, when an update applies, the promise an update
//! keeps, applying one, and going back to the version before. Every sentence is about the person's machine and
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

// ---------------------------------------------------------------------------
// Applying it — `crate::Staging`, `crate::NotStaged`, `crate::Deployments`.
// ---------------------------------------------------------------------------

/// The update is waiting for the person's next restart.
pub const WAITING_FOR_THE_RESTART: Word = Word::saying(
    "keeping-up.waiting-for-the-restart",
    "The update will apply the next time you restart. Your files and settings stay as they are",
)
.noting(
    "Said once the person has chosen to apply an update at their next restart and the machine has \
     prepared it. Nothing happens until the person restarts. The second sentence is a promise the \
     machine keeps: an update never changes the person's own files, settings or history.",
);

/// The update was not prepared because the machine changed since it was found.
pub const CHANGED_SINCE_IT_WAS_FOUND: Word = Word::saying(
    "keeping-up.changed-since-it-was-found",
    "This machine changed after the update was found, so nothing was changed. Check for the \
     update again",
)
.noting(
    "Said when the person chose to apply an update, but the system running on the machine is no \
     longer the one the update was found for — for example because another update was applied in \
     the meantime. The machine is exactly as it was, and checking again finds the right update.",
);

/// The update the person chose is already waiting.
pub const ALREADY_WAITING: Word = Word::saying(
    "keeping-up.already-waiting",
    "This update is already waiting for your next restart",
)
.noting(
    "Said when the person chooses to apply an update that the machine has already prepared. \
     Nothing is done twice.",
);

/// Which version of its system the machine runs could not be read.
pub const RUNNING_NOT_KNOWN: Word = Word::saying(
    "keeping-up.running-not-known",
    "Which version of its system this machine is running could not be read, so nothing was \
     changed",
)
.noting(
    "Said when the machine could not find out which version of alo OS it is running, so it could \
     not safely prepare or record an update. The machine is exactly as it was.",
);

/// The update could not be prepared.
pub const NOT_PREPARED: Word = Word::saying(
    "keeping-up.not-prepared",
    "The update could not be prepared, so nothing was changed. The next restart starts this \
     machine as it is now",
)
.noting(
    "Said when the machine tried to prepare an update the person chose and could not — the \
     download failed, or what arrived was not a genuine alo OS. The important half is that the \
     machine is unchanged and will start normally.",
);

/// Whether the machine updated could not be written down.
pub const NOT_WRITTEN_DOWN: Word = Word::saying(
    "keeping-up.not-written-down",
    "This machine could not write down whether it started on an updated version of its system. \
     Nothing else was changed",
)
.noting(
    "Said when the machine starts and cannot keep, in its own history, the fact that it has just \
     been updated — for example because the place that fact is kept could not be read or written. \
     The machine itself is working; what is missing is the line in its history.",
);

// ---------------------------------------------------------------------------
// Going back — `crate::GoingBack`, `crate::CannotGoBack`, `crate::Returning`.
// ---------------------------------------------------------------------------

/// Going back is offered.
pub const GOING_BACK_OFFERED: Word = Word::saying(
    "keeping-up.going-back.offered",
    "This machine can go back to the version of its system it ran before its last update. Your \
     files and your own settings stay as they are. Accounts, passwords and settings for the whole \
     machine that changed since that update go back to how they were",
)
.noting(
    "The sentence a person approves before their machine returns to the earlier version of alo OS. \
     Every part of it is a promise the machine keeps and must survive translation: the person's own \
     files and personal settings are not touched, but changes made since the update to user \
     accounts, passwords and settings that apply to everyone on the machine do not come back with \
     it.",
);

/// Going back is offered, and sets aside an update that is waiting.
pub const GOING_BACK_SETS_ASIDE_AN_UPDATE: Word = Word::saying(
    "keeping-up.going-back.sets-aside-an-update",
    "This machine can go back to the version of its system it ran before its last update. Your \
     files and your own settings stay as they are. Accounts, passwords and settings for the whole \
     machine that changed since that update go back to how they were. The update waiting for \
     your next restart will not apply",
)
.noting(
    "The same sentence as the offer to go back, said when the person had also chosen a newer update \
     that is waiting for their next restart. Going back cancels that waiting update, and the last \
     sentence tells them so before they approve.",
);

/// The choice to go back at the next restart.
pub const GO_BACK_AT_THE_NEXT_RESTART: Word = Word::saying(
    "keeping-up.going-back.at-the-next-restart",
    "Go back the next time I restart",
)
.noting(
    "One of two choices once going back to the earlier version is offered, written as the person \
     speaking. Nothing happens until the person restarts the machine themselves.",
);

/// The choice to restart now and go back.
pub const RESTART_AND_GO_BACK_NOW: Word = Word::saying(
    "keeping-up.going-back.now",
    "Restart now and go back, which closes the applications that are open",
)
.noting(
    "The other of the two choices, written as an instruction the person gives. It says that open \
     applications close, because restarting closes them.",
);

/// Going back is waiting for the person's next restart.
pub const GOING_BACK_AT_THE_RESTART: Word = Word::saying(
    "keeping-up.going-back.waiting-for-the-restart",
    "This machine will go back to the version it ran before the next time you restart. Your files \
     stay as they are",
)
.noting(
    "Said once the person has chosen to go back and the machine has prepared it. Nothing happens \
     until the person restarts.",
);

/// There is nothing to go back to.
pub const NOTHING_TO_GO_BACK_TO: Word = Word::saying(
    "keeping-up.going-back.nothing-before",
    "This machine has no earlier version of its system to go back to",
)
.noting(
    "Said instead of offering to go back, when this machine has only ever run the version it runs \
     now — for example before its first update.",
);

/// The version before is no longer kept.
pub const NO_LONGER_KEPT: Word = Word::saying(
    "keeping-up.going-back.no-longer-kept",
    "The version of its system this machine ran before is no longer kept on it, so it cannot go \
     back to it",
)
.noting(
    "Said instead of offering to go back, when the machine knows which version it ran before but no \
     longer has a copy of it. It is said before anything is offered, so nothing is started that \
     could fail halfway.",
);

/// Going back is already waiting.
pub const ALREADY_GOING_BACK: Word = Word::saying(
    "keeping-up.going-back.already-waiting",
    "This machine will already go back to the version it ran before the next time you restart",
)
.noting(
    "Said when the person chooses to go back and has already chosen it. Nothing is done twice.",
);

/// The machine changed after going back was offered.
pub const CHANGED_SINCE_GOING_BACK_WAS_OFFERED: Word = Word::saying(
    "keeping-up.going-back.changed-since-it-was-offered",
    "This machine changed after going back was offered, so nothing was changed. Look again at \
     what it can go back to",
)
.noting(
    "Said when the person approved going back, but the machine is no longer as it was when that was \
     offered — for example an update was prepared in the meantime. What they approved would not be \
     what happens, so nothing is done.",
);

/// Going back could not be prepared.
pub const GOING_BACK_NOT_PREPARED: Word = Word::saying(
    "keeping-up.going-back.not-prepared",
    "Going back could not be prepared, so nothing was changed. The next restart starts this \
     machine as it is now",
)
.noting(
    "Said when the person chose to go back and the machine could not prepare it. The important half \
     is that the machine is unchanged and will start normally.",
);

/// Every string this crate can say.
pub const EVERY_WORD: [Word; 24] = [
    READY,
    UP_TO_DATE,
    ANSWER_NOT_UNDERSTOOD,
    APPLY_AT_THE_NEXT_RESTART,
    RESTART_AND_APPLY_NOW,
    NEVER_RESTARTS,
    NEVER_CLOSES_AN_APPLICATION,
    NEVER_INTERRUPTS,
    WAITING_FOR_THE_RESTART,
    CHANGED_SINCE_IT_WAS_FOUND,
    ALREADY_WAITING,
    RUNNING_NOT_KNOWN,
    NOT_PREPARED,
    NOT_WRITTEN_DOWN,
    GOING_BACK_OFFERED,
    GOING_BACK_SETS_ASIDE_AN_UPDATE,
    GO_BACK_AT_THE_NEXT_RESTART,
    RESTART_AND_GO_BACK_NOW,
    GOING_BACK_AT_THE_RESTART,
    NOTHING_TO_GO_BACK_TO,
    NO_LONGER_KEPT,
    ALREADY_GOING_BACK,
    CHANGED_SINCE_GOING_BACK_WAS_OFFERED,
    GOING_BACK_NOT_PREPARED,
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
                "rollback",
                "roll back",
                "rolled back",
                "snapshot",
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
