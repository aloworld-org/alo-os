//! Why a person did not get one of their folders.
//!
//! Three, kept apart rather than folded into *it did not work*, because
//! whoever reads them does something different about each: a name that cannot
//! be a folder is a line to fix in a translation, a disk that would not is a
//! machine to look at, and a record that was not written leaves the folders
//! working and only the lookup broken.
//!
//! # None of these stops a person signing in
//!
//! A machine that would not make a Pictures folder is still a machine somebody
//! should be able to use. `crate::making` makes every folder it can and
//! reports what it could not, rather than giving up at the first refusal —
//! which is why these are carried in [`crate::Made`] alongside what worked
//! rather than returned in place of it.

use alo_strings::{Filling, Said, Strings};

use crate::words::{self, Word};

/// Why one folder, or the record, was not made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotMade {
    /// A translated name that cannot be a directory: empty, holding a slash,
    /// beginning with a dot, or nothing but punctuation.
    ///
    /// Carries the key so that whoever fixes the translation knows which line
    /// of which file to open.
    NotANameAFolderCanHave {
        /// The word whose translation cannot be a folder.
        key: String,
    },
    /// The machine refused to create the directory.
    TheDiskWouldNot {
        /// What the operating system said, kept for a log and never shown to
        /// a person — it is in the machine's language, not theirs.
        said: String,
    },
    /// The folders were made and the record naming them was not.
    TheRecordWasNotWritten {
        /// What the operating system said, kept for a log.
        said: String,
    },
}

impl NotMade {
    /// The string this crate declares for it: the key a translator's file is
    /// sorted by, and the English beside it.
    #[must_use]
    pub fn word(&self) -> Word {
        match self {
            Self::NotANameAFolderCanHave { .. } => words::NOT_A_NAME_A_FOLDER_CAN_HAVE,
            Self::TheDiskWouldNot { .. } => words::THE_DISK_WOULD_NOT,
            Self::TheRecordWasNotWritten { .. } => words::THE_RECORD_WAS_NOT_WRITTEN,
        }
    }

    /// What to tell whoever is looking after this machine, in their language.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not.
    /// A `Strings` that was never given [`crate::their_folders_words`] answers
    /// with the key, marked `Said::is_a_bug`.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }

    /// What the machine said, for a log — never for a person.
    ///
    /// **Separate from [`Self::said`] on purpose.** The operating system's own
    /// message is in the machine's language and names paths a person did not
    /// ask about; it belongs in a journal, beside the sentence rather than
    /// instead of it. `alo-picking` keeps the same division and
    /// `docs/misreadings/` has the entry for the day they were one thing.
    #[must_use]
    pub fn kept_for_a_maintainer(&self) -> &str {
        match self {
            Self::NotANameAFolderCanHave { key } => key,
            Self::TheDiskWouldNot { said } | Self::TheRecordWasNotWritten { said } => said,
        }
    }
}
