//! Every sentence this crate can say, and the note a translator works from.
//!
//! A person plugs a drive in and wants to open what is on it; later they want
//! to take it away without losing anything. Those two moments are the broker's
//! `storage.mount` and `storage.eject`, and these are the sentences around
//! them.
//!
//! # Nothing here names the machinery
//!
//! Not *mount*, not *unmount*, not a filesystem, not a device, not a partition,
//! not the disk service, not the part of the machine with authority over all of
//! it, not a socket and not *root*. A person opens a drive and finishes with
//! it. The drive's own name is data, filled into `{drive}` exactly as the drive
//! gave it, and never written into a sentence.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported so this
/// crate's files name it as `crate::words::Word`.
pub use alo_strings::Word;

/// The gap holding the drive's name.
pub const DRIVE: &str = "drive";

// ---------------------------------------------------------------------------
// What became of the change.
// ---------------------------------------------------------------------------

/// The drive is open.
pub const READY: Word = Word::saying(
    "changing-drives.ready",
    "{drive} is ready, and what is on it is yours to open",
)
.noting(
    "Said once a drive a person plugged in has been opened for them. {drive} is the name the \
     drive gave itself, usually its maker and model, and is not translated. Nothing on it has \
     been given to an assistant by opening it.",
);

/// The drive is finished with.
pub const SAFE_TO_UNPLUG: Word = Word::saying(
    "changing-drives.safe-to-unplug",
    "{drive} is finished with, and safe to unplug",
)
.noting(
    "Said once everything this machine was doing with a drive has been finished and the drive \
     switched off. It is the sentence a person waits for before pulling a drive out, so it must \
     never be said early. {drive} is the drive's name.",
);

// ---------------------------------------------------------------------------
// Why nothing happened.
// ---------------------------------------------------------------------------

/// No drive plugged in has that name.
pub const NONE_PLUGGED_IN_CALLED: Word = Word::saying(
    "changing-drives.refused.none-plugged-in-called",
    "Nothing called {drive} is plugged into this machine, so nothing was opened. Plug it in \
     again, then try once more",
)
.noting(
    "Said when a drive was named and no drive plugged in now has that name — usually because it \
     was taken out between being offered and being chosen. {drive} is the name that was chosen.",
);

/// More than one drive has that name.
pub const MORE_THAN_ONE_CALLED: Word = Word::saying(
    "changing-drives.refused.more-than-one-called",
    "More than one thing plugged into this machine is called {drive}, so nothing was opened. \
     Take one of them out, then try again",
)
.noting(
    "Said when two drives report the same name, which happens with two sticks of the same make \
     and model. {drive} is the name. It never guesses which one was meant.",
);

/// The drive is part of the machine.
pub const PART_OF_THIS_MACHINE: Word = Word::saying(
    "changing-drives.refused.part-of-this-machine",
    "{drive} is part of this machine rather than something plugged into it, so nothing was \
     changed",
)
.noting(
    "Said when what was named is one of the machine's own disks. Only a drive a person plugged \
     in is opened or finished with this way, and a person can do nothing about this one: it is \
     said so that nobody is left waiting. {drive} is the name.",
);

/// There is nothing on the drive this machine can open.
pub const NOTHING_TO_OPEN: Word = Word::saying(
    "changing-drives.refused.nothing-to-open",
    "There is nothing on {drive} this machine can open. It may be empty, or prepared for a \
     machine of another kind",
)
.noting(
    "Said when a drive was plugged in and this machine found nothing on it that it recognises. \
     Neither the drive nor the person has done anything wrong, and nothing on the drive has been \
     touched. {drive} is the drive's name.",
);

/// It is already open.
pub const ALREADY_OPEN: Word = Word::saying(
    "changing-drives.refused.already-open",
    "{drive} is already open, so nothing was changed. What is on it is yours to open",
)
.noting(
    "Said when a drive that had already been opened was chosen again. Nothing has gone wrong and \
     nothing is lost. {drive} is the drive's name.",
);

/// This machine could not be asked what is plugged into it.
pub const DRIVES_NOT_ANSWERING: Word = Word::saying(
    "changing-drives.refused.drives-not-answering",
    "This machine is not answering about what is plugged into it, so nothing was changed. \
     Restart the machine, then try again",
)
.noting(
    "Said when the part of this machine that keeps track of drives could not be asked at all. It \
     is this machine's fault rather than the drive's.",
);

/// The approval was not accepted.
pub const APPROVAL_NOT_ACCEPTED: Word = Word::saying(
    "changing-drives.refused.approval-not-accepted",
    "This was not done, because its approval was not accepted: it was used already, came too \
     late, or was not given for this. Nothing was changed. Ask again, and approve it when you \
     are asked",
)
.noting(
    "Said when a person's approval reached the part of the machine that makes changes for \
     everybody who uses it and was refused there. An approval works once, for one change, and \
     only for about a minute after it is given.",
);

/// It was accepted and could not be finished.
pub const COULD_NOT_FINISH: Word = Word::saying(
    "changing-drives.refused.could-not-finish",
    "This machine could not finish what was asked of {drive}. Do not unplug it yet: try again, \
     and close anything you have open on it first",
)
.noting(
    "Said when an approved change to a drive was started and could not be completed — most often \
     because something on the machine is still using it. The warning not to unplug it is the \
     important half: work can be lost. {drive} is the drive's name.",
);

/// Nothing is being written down, so nothing is changed.
pub const NOT_BEING_KEPT: Word = Word::saying(
    "changing-drives.refused.not-being-kept",
    "This machine has stopped keeping its record of changes to its settings, so it makes none. \
     Nothing was changed. Restart the machine",
)
.noting(
    "Said when the machine could not write down a change before making it. It makes no change it \
     cannot write down, so it makes none until it is restarted.",
);

/// Nothing is there to make the change.
pub const NOTHING_MAKES_CHANGES: Word = Word::saying(
    "changing-drives.refused.nothing-makes-changes",
    "The part of this machine that changes its settings is not running, so nothing was changed. \
     Restart the machine",
)
.noting(
    "Said when a change a person approved could not be handed to the part of the machine that \
     makes changes to its settings for everybody who uses it.",
);

/// What was handed over was not about a drive at all.
pub const NOT_A_DRIVE_CHANGE: Word = Word::saying(
    "changing-drives.refused.not-a-drive-change",
    "This was not about a drive, so nothing was changed",
)
.noting(
    "Said when something other than an approved change to a drive was handed to the part of the \
     machine that makes such changes. A person should never see it; if they do, the machine is \
     wrong rather than them.",
);

/// Every word that is a sentence of its own.
pub const THE_SENTENCES: [Word; 12] = [
    READY,
    SAFE_TO_UNPLUG,
    NONE_PLUGGED_IN_CALLED,
    MORE_THAN_ONE_CALLED,
    PART_OF_THIS_MACHINE,
    NOTHING_TO_OPEN,
    ALREADY_OPEN,
    DRIVES_NOT_ANSWERING,
    APPROVAL_NOT_ACCEPTED,
    COULD_NOT_FINISH,
    NOT_BEING_KEPT,
    NOTHING_MAKES_CHANGES,
];

/// Every sentence this crate can say.
pub const EVERY_WORD: [Word; 13] = [
    READY,
    SAFE_TO_UNPLUG,
    NONE_PLUGGED_IN_CALLED,
    MORE_THAN_ONE_CALLED,
    PART_OF_THIS_MACHINE,
    NOTHING_TO_OPEN,
    ALREADY_OPEN,
    DRIVES_NOT_ANSWERING,
    APPROVAL_NOT_ACCEPTED,
    COULD_NOT_FINISH,
    NOT_BEING_KEPT,
    NOTHING_MAKES_CHANGES,
    NOT_A_DRIVE_CHANGE,
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
pub fn changing_drives_words() -> Result<Vocabulary, WordsError> {
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
            assert_eq!(word.key().area(), "changing-drives", "{}", word.named());
        }
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// The sentences and the one refusal nobody should see are the whole list.
    #[test]
    fn the_groups_are_the_whole_list() {
        let mut all: Vec<&str> = THE_SENTENCES.iter().map(Word::named).collect();
        all.push(NOT_A_DRIVE_CHANGE.named());
        let every: Vec<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(all, every);
    }

    /// The list declares, and a key already taken is not replaced.
    #[test]
    fn the_list_declares_and_nothing_is_replaced() {
        let mut vocabulary = changing_drives_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert!(matches!(
            declare_into(&mut vocabulary),
            Err(WordsError::List(_))
        ));
    }

    /// **Every word carries a note, and every gap is the drive's name.**
    #[test]
    fn every_word_carries_a_note_and_only_the_drives_name_fills_a_gap() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
            for gap in word.phrase().unwrap().source().gaps() {
                assert_eq!(gap, DRIVE, "{} has a gap called {gap}", word.named());
            }
        }
    }

    /// **Nothing here names the machinery, and nothing hedges.**
    #[test]
    fn nothing_here_names_the_machinery_or_hedges() {
        for word in EVERY_WORD {
            let said = word.says().to_lowercase();
            for forbidden in [
                "mount",
                "unmount",
                "filesystem",
                "file system",
                "partition",
                "volume",
                "udisks",
                "broker",
                "socket",
                "root",
                "token",
                "daemon",
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

    /// **Every refusal says what to do about it, or that there is nothing to
    /// do** — a second clause after what happened.
    #[test]
    fn every_refusal_says_what_to_do_about_it() {
        for word in [
            NONE_PLUGGED_IN_CALLED,
            MORE_THAN_ONE_CALLED,
            NOTHING_TO_OPEN,
            ALREADY_OPEN,
            DRIVES_NOT_ANSWERING,
            APPROVAL_NOT_ACCEPTED,
            COULD_NOT_FINISH,
            NOT_BEING_KEPT,
            NOTHING_MAKES_CHANGES,
        ] {
            let said = word.says();
            assert!(
                said.contains(". ") || said.contains(", then "),
                "{} names what happened and not what to do",
                word.named()
            );
        }
    }
}
