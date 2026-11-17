//! Asking the desktop to show the person alo's file chooser, and handing what
//! they chose to the application that asked — the two ends the file chooser
//! portal stands between.
//!
//! The owner, 2026-11-17: *"Yes, build it"* — one file chooser, the same for
//! every application. The sheet is the desktop's (`docs/contracts/application-door.md`,
//! `choose-file`); this is the backend's side of it as two traits, so the
//! backend is testable with neither a desktop nor a documents portal.
//!
//! # The file is handed over through the documents portal
//!
//! A path means nothing inside a sandbox that was never given it. So the file
//! the person chose is **exported** to the asking application alone, through
//! upstream's documents portal ([ADR 0159](../../../docs/decisions/0159-a-chosen-file-reaches-an-application-through-the-documents-portal.md)),
//! and the application is handed the exported path. **Only the file the person
//! chose is exported**, and nothing is exported when they chose nothing.
//!
//! # A backend with neither says so
//!
//! [`NoDesktopToAsk`](crate::NoDesktopToAsk) answers every sheet
//! [`FileSheetAnswer::NoDesktop`], and [`NoDocumentsPortal`] every export
//! [`NotExported::NoDocumentsPortal`]: the response is a refusal and the record
//! says why. A default that answered *chosen* would hand an application a file
//! nobody chose.

use std::path::{Path, PathBuf};

/// What the application asked the person for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToChoose {
    /// A file to open.
    Open,
    /// A place to save a file, under a name the application suggested.
    Save {
        /// The name suggested, which the person may change.
        name: String,
    },
}

/// What the person did with the file chooser, as the desktop answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileSheetAnswer {
    /// The person chose this file — to open it, or to save there.
    Picked(PathBuf),
    /// The person pressed *Cancel* or Escape.
    Cancelled,
    /// Nobody answered while the sheet was up.
    Unanswered,
    /// The desktop could not show the sheet now.
    CannotShow,
    /// No desktop is there to ask.
    NoDesktop,
    /// The desktop was reached and could not be asked, or answered something
    /// that is not an answer.
    CouldNotAsk,
}

/// Whatever asks the person, on the desktop, to choose a file.
pub trait ChoosesFiles: Send + Sync {
    /// Ask the person to choose a file for `asking`, starting in `starting_in`
    /// — or in their home folder when the application named none. **Waits for
    /// the person**, so it is never called on the bus's own thread.
    fn chosen(&self, asking: &str, to: &ToChoose, starting_in: Option<&Path>) -> FileSheetAnswer;
}

/// How an exported file may be used by the application it is exported to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportedFor {
    /// Opening a file that is there: read, and written when `writable`.
    Opening {
        /// Whether the application may write the file too.
        writable: bool,
    },
    /// Saving: the file may not exist yet; read, written and replaced.
    Saving,
}

/// Why a chosen file could not be handed over.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NotExported {
    /// No documents portal answered on the person's bus.
    NoDocumentsPortal,
    /// The file to open is not a file any more, or the folder to save in is
    /// not a folder.
    NotThere,
    /// The documents portal was asked and refused, or answered something that
    /// is not an exported file.
    Refused,
}

/// Whatever hands a chosen file to one application.
pub trait ExportsDocuments: Send + Sync {
    /// Hand `file` to `application` for `what`, and the path the application
    /// opens it at — inside the documents portal's folder, or `file` itself
    /// where the application's own sandbox already reaches it.
    ///
    /// # Errors
    /// [`NotExported`], whatever stopped it. Nothing was handed over.
    fn exported(
        &self,
        application: &str,
        file: &Path,
        what: ExportedFor,
    ) -> Result<PathBuf, NotExported>;
}

impl ChoosesFiles for crate::opening_links::NoDesktopToAsk {
    fn chosen(&self, _: &str, _: &ToChoose, _: Option<&Path>) -> FileSheetAnswer {
        FileSheetAnswer::NoDesktop
    }
}

/// A backend that was never told how to reach a documents portal.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoDocumentsPortal;

impl ExportsDocuments for NoDocumentsPortal {
    fn exported(&self, _: &str, _: &Path, _: ExportedFor) -> Result<PathBuf, NotExported> {
        Err(NotExported::NoDocumentsPortal)
    }
}
