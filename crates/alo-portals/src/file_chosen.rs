//! What the file chooser portal answered with: a file the person chose and the
//! application was given, or nothing, and why.
//!
//! **Chosen or not, it is in the record**, naming the application that asked
//! and **the file the person chose**, so a person reading their record sees
//! which application was given which of their files. It is kept **before** the
//! file is handed over: when the handing then fails, a [`FileUnchosen`] with
//! [`NoFile::NotHandedOver`] follows it, and the response is `2`.
//!
//! **Only the person cancelling is response `1`.** Everything else that hands
//! nothing over is `2`, because the person did not say no.

use std::path::{Path, PathBuf};

use alo_capability::Applicant;
use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::choosing_files::NotExported;
use crate::words;

/// A file the person chose for an application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChosen {
    /// The application that asked.
    from: Applicant,
    /// The file the person chose.
    chosen: PathBuf,
    /// Whether it was chosen to save, rather than to open.
    saving: bool,
}

impl FileChosen {
    /// `from` asked, and the person chose `chosen` — to save there when
    /// `saving`.
    #[must_use]
    pub const fn of(from: Applicant, chosen: PathBuf, saving: bool) -> Self {
        Self {
            from,
            chosen,
            saving,
        }
    }

    /// The application that asked.
    #[must_use]
    pub const fn from(&self) -> &Applicant {
        &self.from
    }

    /// The file the person chose.
    #[must_use]
    pub fn chosen(&self) -> &Path {
        &self.chosen
    }

    /// Whether it was chosen to save.
    #[must_use]
    pub const fn saving(&self) -> bool {
        self.saving
    }

    /// What this says, in the language the person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        said_as_chosen(
            Filling::of("application", self.from.as_str()),
            &self.chosen,
            self.saving,
            strings,
        )
    }
}

/// The sentence for a file chosen, from what was decided or from what the
/// record kept — one function, so the two cannot be worded apart.
pub(crate) fn said_as_chosen(
    asked: Filling,
    chosen: &Path,
    saving: bool,
    strings: &Strings,
) -> Said {
    let word = if saving {
        words::FILE_CHOSEN_TO_SAVE
    } else {
        words::FILE_CHOSEN_TO_OPEN
    };
    strings.say(
        &word.key(),
        &asked.and("file", chosen.to_string_lossy().as_ref()),
    )
}

/// Why no file was handed over.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NoFile {
    /// The person pressed *Cancel* or Escape.
    Cancelled,
    /// Nobody answered while the sheet was up, and the desktop took it down.
    Unanswered,
    /// The desktop could not show the sheet now.
    CannotShow,
    /// No desktop was there to ask.
    NoDesktop,
    /// The desktop was reached and could not be asked.
    CouldNotAsk,
    /// The desktop answered with something that cannot have been chosen — a
    /// file that is not there, a folder to open, a path that is not whole —
    /// and it was not believed.
    NotBelieved,
    /// The person chose a file and it could not be handed to the application.
    NotHandedOver(NotExported),
    /// A folder, or several files, which this chooser does not offer yet.
    NotOfferedYet,
}

impl NoFile {
    /// The response it is sent as: `1` for the person's own cancel, `2`
    /// otherwise.
    #[must_use]
    pub const fn response(self) -> u32 {
        match self {
            Self::Cancelled => 1,
            Self::Unanswered
            | Self::CannotShow
            | Self::NoDesktop
            | Self::CouldNotAsk
            | Self::NotBelieved
            | Self::NotHandedOver(_)
            | Self::NotOfferedYet => 2,
        }
    }

    /// The sentence for it.
    const fn word(self) -> alo_strings::Word {
        match self {
            Self::Cancelled => words::NO_FILE_CANCELLED,
            Self::Unanswered => words::NO_FILE_UNANSWERED,
            Self::CannotShow => words::NO_FILE_CANNOT_SHOW,
            Self::NoDesktop => words::NO_FILE_NO_DESKTOP,
            Self::CouldNotAsk => words::NO_FILE_COULD_NOT_ASK,
            Self::NotBelieved => words::NO_FILE_NOT_BELIEVED,
            Self::NotHandedOver(NotExported::NoDocumentsPortal) => {
                words::NO_FILE_NO_DOCUMENTS_PORTAL
            }
            Self::NotHandedOver(NotExported::NotThere) => words::NO_FILE_NOT_THERE,
            Self::NotHandedOver(NotExported::Refused) => words::NO_FILE_REFUSED,
            Self::NotOfferedYet => words::NO_FILE_NOT_OFFERED_YET,
        }
    }
}

/// An application that asked for a file and was given none, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileUnchosen {
    /// The application that asked.
    from: Applicant,
    /// Why it was given none.
    why: NoFile,
}

impl FileUnchosen {
    /// `from` asked for a file and was given none, for `why`.
    #[must_use]
    pub const fn of(from: Applicant, why: NoFile) -> Self {
        Self { from, why }
    }

    /// The application that asked.
    #[must_use]
    pub const fn from(&self) -> &Applicant {
        &self.from
    }

    /// Why it was given none.
    #[must_use]
    pub const fn why(&self) -> NoFile {
        self.why
    }

    /// What this says, in the language the person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        said_as_unchosen(Filling::of("application", self.from.as_str()), self.why, strings)
    }
}

/// The sentence for no file handed over, from what was decided or from what
/// the record kept.
pub(crate) fn said_as_unchosen(asked: Filling, why: NoFile, strings: &Strings) -> Said {
    strings.say(&why.word().key(), &asked)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "file_chosen_tests.rs"]
mod tests;
