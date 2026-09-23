//! Every sentence this crate can say, and the note a translator works from.
//!
//! A person with two systems on one machine wants three things: to see both
//! when the machine starts, to say which one it starts at when nobody chooses,
//! and to go across to the other without restarting twice. These are the
//! sentences around those three.
//!
//! # Nothing here names the machinery
//!
//! Not a loader, not a menu entry, not a partition, not a variable, not the
//! part of the machine with authority over all of it, not a socket and not
//! *root*. A person starts their computer and it starts something. The other
//! system is named the way its maker names it, because that is the word on the
//! machine a person bought.
//!
//! # The menu's own title is here, and that is not an accident
//!
//! [`THE_WINDOWS_ENTRY_TITLE`] is the first thing a person reads on a machine
//! that has just been switched on, and it goes into a file a loader parses. It
//! is a sentence somebody translates like any other, and `crate::menu` refuses
//! a translation carrying anything that file would read as punctuation of its
//! own — a menu that will not parse is a machine nobody can start.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported so this
/// crate's files name it as `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// What a person reads on the machine itself.
// ---------------------------------------------------------------------------

/// What the other system is called in the list a machine starts at.
pub const THE_WINDOWS_ENTRY_TITLE: Word = Word::saying("starting.menu.windows", "Windows").noting(
    "The name of the other operating system on this computer, shown in the short list a person \
     chooses from when the computer is switched on. It is the name that system's maker uses, and \
     in most languages it is left exactly as it is; translate it only where that maker's own \
     name for it in your language is what people read on their screens. It is drawn in a plain \
     console of eighty columns, so it must be short, and it cannot hold a quote, a backslash, a \
     brace or a dollar sign.",
);

// ---------------------------------------------------------------------------
// What became of the change.
// ---------------------------------------------------------------------------

/// Windows starts next time, once.
pub const WINDOWS_NEXT_TIME: Word = Word::saying(
    "starting.windows-next",
    "Windows will start the next time you switch this computer on. Nothing else changed: the \
     time after that, it starts the way it always does",
)
.noting(
    "Said once this computer has been set to start the other operating system the next time it \
     is switched on. It is for one start only, which is what the second sentence says: a person \
     who wants that system every time chooses it in Settings instead. Nothing has happened yet — \
     the person restarts when they are ready.",
);

/// alo OS now starts when nobody chooses.
pub const NOW_STARTS_ALO_OS: Word = Word::saying(
    "starting.now-starts-alo-os",
    "alo OS now starts on this computer whenever nobody chooses. You can change that here at any \
     time",
)
.noting(
    "Said once a person has changed which of the two operating systems on this computer starts on \
     its own after the short wait. It is for every time from now on, which is what separates it \
     from starting.windows-next; \"here\" is the same place in Settings they are reading it. \
     \"alo OS\" is this system's name and is never translated.",
);

/// Windows now starts when nobody chooses.
pub const NOW_STARTS_WINDOWS: Word = Word::saying(
    "starting.now-starts-windows",
    "Windows now starts on this computer whenever nobody chooses. You can change that here at any \
     time",
)
.noting(
    "The counterpart of starting.now-starts-alo-os, for the other operating system. \"Windows\" \
     is that system's maker's name for it and is left as it is in most languages.",
);

/// This computer starts alo OS when nobody chooses.
pub const STARTS_AT_ALO_OS: Word = Word::saying(
    "starting.starts-at-alo-os",
    "This computer starts alo OS when nobody chooses",
)
.noting(
    "Said in Settings, about a computer with two operating systems on it, to say which one it \
     starts on its own after the short wait. \"alo OS\" is this system's name and is never \
     translated.",
);

/// This computer starts Windows when nobody chooses.
pub const STARTS_AT_WINDOWS: Word = Word::saying(
    "starting.starts-at-windows",
    "This computer starts Windows when nobody chooses",
)
.noting(
    "Said in Settings, about a computer with two operating systems on it, to say which one it \
     starts on its own after the short wait. The counterpart of starting.starts-at-alo-os.",
);

// ---------------------------------------------------------------------------
// Why nothing happened.
// ---------------------------------------------------------------------------

/// There is no Windows on this computer.
pub const NO_WINDOWS_HERE: Word = Word::saying(
    "starting.refused.no-windows-here",
    "There is no Windows on this computer to start, so nothing was changed. A computer alo OS \
     replaced has only alo OS on it",
)
.noting(
    "Said when nothing this computer can start is the other operating system — usually because \
     alo OS replaced it rather than being installed beside it. Nothing has gone wrong.",
);

/// More than one thing on this computer starts Windows.
pub const MORE_THAN_ONE_WINDOWS: Word = Word::saying(
    "starting.refused.more-than-one-windows",
    "More than one thing on this computer starts Windows, so nothing was changed. Switch the \
     computer off and on, and choose Windows from the list it starts at",
)
.noting(
    "Said when two of the computer's own start-up choices are both the other operating system, \
     which happens on a computer that has been reinstalled. It never guesses which one was \
     meant, and the second sentence is the way through that always works.",
);

/// What was chosen does not start Windows.
pub const DOES_NOT_START_WINDOWS: Word = Word::saying(
    "starting.refused.does-not-start-windows",
    "What was chosen does not start Windows, so nothing was changed. Switch the computer off and \
     on, and choose Windows from the list it starts at",
)
.noting(
    "Said when what was approved turned out not to be the other operating system after all — \
     the names a computer keeps for what it can start are whatever was typed when they were \
     made, and on a reinstalled computer they are regularly wrong. This is the check that reads \
     what a choice really starts rather than what it is called.",
);

/// This computer will not say what it can start.
pub const NOT_ANSWERING: Word = Word::saying(
    "starting.refused.not-answering",
    "This computer will not say what it can start, so nothing was changed. Restart it, then try \
     again",
)
.noting(
    "Said when the part of the computer that remembers what it can start could not be asked at \
     all. It is the computer's own fault rather than the person's.",
);

/// This computer would not be told.
pub const WOULD_NOT_BE_TOLD: Word = Word::saying(
    "starting.refused.would-not-be-told",
    "This computer would not be told to start Windows next time, so nothing was changed. Switch \
     it off and on, and choose Windows from the list it starts at",
)
.noting(
    "Said when the computer refused to remember the choice. Nothing about how the computer \
     starts has changed, and the second sentence is the way across that needs nothing \
     remembered.",
);

/// This computer would not keep the answer.
pub const NOT_REMEMBERED: Word = Word::saying(
    "starting.refused.not-remembered",
    "This computer would not keep which system it should start, so nothing was changed. It still \
     starts the one it started before, and you can choose the other from the list it shows when \
     it is switched on",
)
.noting(
    "Said when the answer to \"which of the two operating systems starts on its own\" could not \
     be kept. Nothing about the computer has changed, and the second sentence is the way across \
     that needs nothing kept. The counterpart of starting.refused.would-not-be-told, for the \
     setting that lasts rather than the one-off.",
);

/// The approval was not accepted.
pub const APPROVAL_NOT_ACCEPTED: Word = Word::saying(
    "starting.refused.approval-not-accepted",
    "This was not done, because its approval was not accepted: it was used already, came too \
     late, or was not given for this. Nothing was changed. Ask again, and approve it when you \
     are asked",
)
.noting(
    "Said when a person's approval reached the part of the machine that makes changes for \
     everybody who uses it and was refused there. An approval works once, for one change, and \
     only for about a minute after it is given.",
);

/// Nothing is being written down, so nothing is changed.
pub const NOT_BEING_KEPT: Word = Word::saying(
    "starting.refused.not-being-kept",
    "This machine has stopped keeping its record of changes to its settings, so it makes none. \
     Nothing was changed. Restart the machine",
)
.noting(
    "Said when the machine could not write down a change before making it. It makes no change it \
     cannot write down, so it makes none until it is restarted.",
);

/// Nothing is there to make the change.
pub const NOTHING_MAKES_CHANGES: Word = Word::saying(
    "starting.refused.nothing-makes-changes",
    "The part of this machine that changes its settings is not running, so nothing was changed. \
     Restart the machine",
)
.noting(
    "Said when a change a person approved could not be handed to the part of the machine that \
     makes changes to its settings for everybody who uses it.",
);

/// What was handed over was not about starting at all.
pub const NOT_A_STARTING_CHANGE: Word = Word::saying(
    "starting.refused.not-a-starting-change",
    "This was not about which system starts, so nothing was changed",
)
.noting(
    "Said when something other than an approved change to which system starts was handed to the \
     part of the machine that makes such changes. A person should never see it; if they do, the \
     machine is wrong rather than them.",
);

/// Every word that is a sentence a person reads about a change.
pub const THE_SENTENCES: [Word; 12] = [
    WINDOWS_NEXT_TIME,
    NOW_STARTS_ALO_OS,
    NOW_STARTS_WINDOWS,
    STARTS_AT_ALO_OS,
    STARTS_AT_WINDOWS,
    NO_WINDOWS_HERE,
    MORE_THAN_ONE_WINDOWS,
    DOES_NOT_START_WINDOWS,
    NOT_ANSWERING,
    WOULD_NOT_BE_TOLD,
    APPROVAL_NOT_ACCEPTED,
    NOT_BEING_KEPT,
];

/// Every refusal a person can read, which is what the road answers with when it
/// answers with anything but *done*.
pub const EVERY_REFUSAL: [Word; 10] = [
    NO_WINDOWS_HERE,
    MORE_THAN_ONE_WINDOWS,
    DOES_NOT_START_WINDOWS,
    NOT_ANSWERING,
    WOULD_NOT_BE_TOLD,
    NOT_REMEMBERED,
    APPROVAL_NOT_ACCEPTED,
    NOT_BEING_KEPT,
    NOTHING_MAKES_CHANGES,
    NOT_A_STARTING_CHANGE,
];

/// Every sentence this crate can say.
pub const EVERY_WORD: [Word; 16] = [
    THE_WINDOWS_ENTRY_TITLE,
    WINDOWS_NEXT_TIME,
    NOW_STARTS_ALO_OS,
    NOW_STARTS_WINDOWS,
    STARTS_AT_ALO_OS,
    STARTS_AT_WINDOWS,
    NO_WINDOWS_HERE,
    MORE_THAN_ONE_WINDOWS,
    DOES_NOT_START_WINDOWS,
    NOT_ANSWERING,
    WOULD_NOT_BE_TOLD,
    NOT_REMEMBERED,
    APPROVAL_NOT_ACCEPTED,
    NOT_BEING_KEPT,
    NOTHING_MAKES_CHANGES,
    NOT_A_STARTING_CHANGE,
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
pub fn starting_words() -> Result<Vocabulary, WordsError> {
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
            assert_eq!(word.key().area(), "starting", "{}", word.named());
        }
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// The groups and the whole list are one list.
    #[test]
    fn the_groups_are_the_whole_list() {
        let grouped: BTreeSet<&str> = THE_SENTENCES
            .iter()
            .chain(EVERY_REFUSAL.iter())
            .map(Word::named)
            .collect();
        let every: BTreeSet<&str> = EVERY_WORD
            .iter()
            .map(Word::named)
            .filter(|named| *named != THE_WINDOWS_ENTRY_TITLE.named())
            .collect();
        assert_eq!(grouped, every);
    }

    /// The list declares, and a key already taken is not replaced.
    #[test]
    fn the_list_declares_and_nothing_is_replaced() {
        let mut vocabulary = starting_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert!(matches!(
            declare_into(&mut vocabulary),
            Err(WordsError::List(_))
        ));
    }

    /// **Every word carries a note, and none of them has a gap.** Nothing here
    /// is filled in from anywhere: a machine's two systems are named by their
    /// makers, not by data.
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
                "firmware",
                "uefi",
                "efi",
                "bios",
                "grub",
                "loader",
                "bootloader",
                "chainload",
                "partition",
                "volume",
                "variable",
                "mount",
                "broker",
                "socket",
                "root",
                "token",
                "daemon",
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

    /// **Every refusal says nothing was changed, and says what to do about
    /// it** — a second clause after what happened.
    ///
    /// All but one: `starting.refused.not-a-starting-change` is the refusal a
    /// person should never see, because it means the machine handed the wrong
    /// thing to the wrong place. There is nothing for them to do about it, and
    /// inventing a suggestion would be advice about somebody else's bug.
    #[test]
    fn every_refusal_says_nothing_changed_and_what_to_do() {
        for word in EVERY_REFUSAL {
            let said = word.says();
            assert!(
                said.to_lowercase().contains("nothing was changed"),
                "{} does not say nothing was changed",
                word.named()
            );
            if word.named() == NOT_A_STARTING_CHANGE.named() {
                continue;
            }
            assert!(
                said.contains(". ") || said.contains(", then "),
                "{} names what happened and not what to do",
                word.named()
            );
        }
    }

    /// **The one sentence said after it worked says that it is for one start
    /// only.** A person who read it as *from now on* would restart later and
    /// find alo OS, which is the machine appearing to undo what they asked.
    #[test]
    fn what_is_read_afterwards_says_it_is_for_one_start() {
        let said = WINDOWS_NEXT_TIME.says().to_lowercase();
        assert!(said.contains("next time"), "{said}");
        assert!(said.contains("the time after that"), "{said}");
    }

    /// **The two sentences said once the default changed say that it lasts**,
    /// which is what separates them from the one-start sentence. A person who
    /// read *Windows will start next time* as *from now on* would restart later
    /// and find alo OS; a person who read this as *once* would wait for a
    /// change back that never comes.
    #[test]
    fn what_is_read_after_the_default_changed_says_it_lasts() {
        for word in [NOW_STARTS_ALO_OS, NOW_STARTS_WINDOWS] {
            let said = word.says().to_lowercase();
            assert!(said.contains("whenever nobody chooses"), "{said}");
            assert!(said.contains("change that"), "{said}");
            assert!(!said.contains("next time"), "{said}");
        }
        assert_ne!(NOW_STARTS_ALO_OS.says(), NOW_STARTS_WINDOWS.says());
    }

    /// **The menu's title will go into the file the menu is written in.** It is
    /// held here to what `crate::menu` accepts, so a title that could not be
    /// written is caught where it is written rather than on a machine that will
    /// not start.
    #[test]
    fn the_menus_title_is_one_the_menu_can_be_written_with() {
        assert!(crate::menu::Menu::offering(THE_WINDOWS_ENTRY_TITLE.says(), 5).is_ok());
    }
}
