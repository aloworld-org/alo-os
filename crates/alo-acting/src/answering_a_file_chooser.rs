//! The words on the compositor's door for **alo answering the file chooser**:
//! the person approved *choose this file on the file chooser*, and alo
//! answers the sheet that is up as the person would have
//! (`choose_on_the_file_chooser`, the coordinator's approval of 2026-11-17).
//!
//! ```text
//! → alo-chooses-file /var/home/alo/Documents notes.txt
//! ← chosen org.gnome.TextEditor     (or: no-sheet, not-believed)
//! ```
//!
//! `docs/contracts/application-door.md`, *alo answers the file chooser*, is
//! the published surface; this is it as working code. The folder and the name
//! are percent-encoded as `choose-file`'s are (`crate::choosing_a_file`).
//!
//! # Who may write it
//!
//! `alo-agentd`, **after** the turn has judged the folder and the file against
//! the grants and the person has approved the sentence. The desktop answers it
//! only when a file chooser is up, and only with a file that sheet could have
//! been answered with; it judges no grant, because the grants are the turn's.

use std::path::{Path, PathBuf};

use alo_applications::Application;

use crate::choosing_a_file::{decoded, encoded, is_a_name, is_a_whole_path};

/// The word that begins this act.
pub const ALO_CHOOSES_FILE: &str = "alo-chooses-file";

/// alo answering the file chooser that is up with this file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToAnswerAFileChooser {
    /// The folder the file is in.
    folder: PathBuf,
    /// The file's name in it.
    name: String,
}

impl ToAnswerAFileChooser {
    /// Answer with `name` in `folder`.
    ///
    /// # Errors
    /// [`NotAnAnswer::NotAFolder`] for a folder that is not a whole path, and
    /// [`NotAnAnswer::NotAName`] for a name that is not one name.
    pub fn of(folder: PathBuf, name: &str) -> Result<Self, NotAnAnswer> {
        if !is_a_whole_path(&folder) {
            return Err(NotAnAnswer::NotAFolder);
        }
        if !is_a_name(name) {
            return Err(NotAnAnswer::NotAName);
        }
        Ok(Self {
            folder,
            name: name.to_owned(),
        })
    }

    /// The folder the file is in.
    #[must_use]
    pub fn folder(&self) -> &Path {
        &self.folder
    }

    /// The file's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The file: the name in the folder.
    #[must_use]
    pub fn file(&self) -> PathBuf {
        self.folder.join(&self.name)
    }

    /// The line, without its newline.
    #[must_use]
    pub fn written(&self) -> String {
        format!(
            "{ALO_CHOOSES_FILE} {} {}",
            encoded(&self.folder.to_string_lossy(), true),
            encoded(&self.name, false)
        )
    }

    /// One line, read as this act — [`None`] when it is not this act at all,
    /// and `Some(Err(…))` for a line that began as it and is not exactly it.
    #[must_use]
    pub fn read(line: &str) -> Option<Result<Self, NotAnAnswer>> {
        let line = line.trim_end_matches(['\r', '\n']);
        let mut words = line.split(' ');
        if words.next() != Some(ALO_CHOOSES_FILE) {
            return None;
        }
        let (Some(folder), Some(name), None) = (words.next(), words.next(), words.next()) else {
            return Some(Err(NotAnAnswer::NotEnoughWords));
        };
        let Some(folder) = decoded(folder) else {
            return Some(Err(NotAnAnswer::NotAFolder));
        };
        let Some(name) = decoded(name) else {
            return Some(Err(NotAnAnswer::NotAName));
        };
        Some(Self::of(PathBuf::from(folder), &name))
    }
}

/// Why a line that began as this act is not one. English, for whoever is
/// fixing the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotAnAnswer {
    /// Not exactly a folder and a name.
    #[error("alo-chooses-file is followed by a folder and a name")]
    NotEnoughWords,
    /// The folder is not a whole, ordinary path.
    #[error("alo-chooses-file names a whole folder from the root, percent-encoded")]
    NotAFolder,
    /// The name is not one name.
    #[error("alo-chooses-file names one file, percent-encoded, with no slash in it")]
    NotAName,
}

/// What the desktop did with alo's answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnsweredTheChooser {
    /// The sheet this application had up was answered with the file.
    Chosen(Application),
    /// No file chooser was up.
    NoSheet,
    /// The file could not have been chosen on the sheet that is up: not there
    /// to open, or a folder's name to save under.
    NotBelieved,
}

/// The word for [`AnsweredTheChooser::Chosen`].
const CHOSEN: &str = "chosen";
/// The word for [`AnsweredTheChooser::NoSheet`].
const NO_SHEET: &str = "no-sheet";
/// The word for [`AnsweredTheChooser::NotBelieved`].
const NOT_BELIEVED: &str = "not-believed";

impl AnsweredTheChooser {
    /// The answer, without its newline.
    #[must_use]
    pub fn written(&self) -> String {
        match self {
            Self::Chosen(application) => format!("{CHOSEN} {}", application.identifier()),
            Self::NoSheet => NO_SHEET.to_owned(),
            Self::NotBelieved => NOT_BELIEVED.to_owned(),
        }
    }

    /// An answer heard, or [`None`]. **An unknown answer is never read as a
    /// choice.**
    #[must_use]
    pub fn read(heard: &str) -> Option<Self> {
        let heard = heard.trim_end_matches(['\r', '\n']);
        match heard {
            NO_SHEET => return Some(Self::NoSheet),
            NOT_BELIEVED => return Some(Self::NotBelieved),
            _ => {}
        }
        let named = heard.strip_prefix(CHOSEN)?.strip_prefix(' ')?;
        let application = Application::identified(named).ok()?;
        (application.identifier() == named).then_some(Self::Chosen(application))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "answering_a_file_chooser_tests.rs"]
mod tests;
