//! Why an answer could not be kept, or a kept record could not be read.
//!
//! **An answer that could not be kept is not sent.** Every door of the backend
//! asks [`crate::Recording::keep`] before the application hears anything, and a
//! [`NotRecorded`] turns the answer into the portal's refusal: an application
//! is never told something the record does not hold, and a file is never
//! opened, nor a secret handed over, on a request nobody could read about
//! later.
//!
//! English, like `alo_remembering::NotRemembered`: these are read out of a
//! service log by whoever administers the machine and is looking at the file,
//! never shown to a person as the reason an application was refused.

use std::path::PathBuf;

/// Why an answer was not kept, or the answers file was not read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotRecorded {
    /// There is no answers file at this path, so nothing has been answered
    /// there yet. Only reading says this; keeping makes the file.
    #[error("{at} is not there: no application has been answered on this machine yet")]
    NotThere {
        /// Where it was looked for.
        at: PathBuf,
    },

    /// The path is a symbolic link, which is never followed.
    #[error("{at} is a symbolic link, and the answers file is never reached through one")]
    ALink {
        /// The link.
        at: PathBuf,
    },

    /// The file belongs to somebody other than root or the login keeping it.
    #[error("{at} belongs to user {owner}, who is neither root nor this login")]
    SomebodyElses {
        /// The file.
        at: PathBuf,
        /// Its owner.
        owner: u32,
    },

    /// The group or the world can write the file.
    #[error("{at} has mode {mode:o}, so somebody other than its owner could have written it")]
    WritableByOthers {
        /// The file.
        at: PathBuf,
        /// Its permission bits.
        mode: u32,
    },

    /// What is at the path is not an answers file: not a regular file, or one
    /// whose first line does not say which format it is in.
    #[error("{at} is not an answers file, so nothing is added to it")]
    NotAnAnswersFile {
        /// The file.
        at: PathBuf,
    },

    /// The file is in a format newer than this backend knows, so nothing is
    /// added to it — a line in a shape the file's writer does not know would
    /// leave a file neither version reads.
    #[error("{at} is in format {format}, newer than this machine reads, so nothing is added to it")]
    ANewerFormat {
        /// The file.
        at: PathBuf,
        /// The format it says it is in.
        format: u64,
    },

    /// The file at the path is no longer the file this backend has open — it
    /// was moved, removed or replaced underneath — so it is not shortened:
    /// renaming over it would remove a file nobody asked this backend to
    /// shorten.
    #[error("{at} is no longer the file this backend keeps answers in, so it is not shortened")]
    Replaced {
        /// The path.
        at: PathBuf,
    },

    /// The machine would not read the file.
    #[error("{at} could not be read: {why}")]
    NotRead {
        /// The file.
        at: PathBuf,
        /// What the machine said.
        why: String,
    },

    /// The machine would not write the answer — including a folder that is
    /// not there, which is refused rather than made.
    #[error("the answer could not be written to {at}: {why}")]
    NotWritten {
        /// The file.
        at: PathBuf,
        /// What the machine said.
        why: String,
    },
}
