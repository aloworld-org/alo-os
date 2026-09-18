//! Refusals from keeping the person's gesture preferences.

use std::path::PathBuf;

use alo_kept::{Unread, Unwritten};
use alo_strings::{Filling, Said, Strings};

/// A settings read refused whole; the original file remains available to repair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotRead {
    /// File supplied by the session.
    pub at: PathBuf,
    /// Structural or filesystem refusal.
    pub why: Unread,
}

impl NotRead {
    /// Explain the refusal through the desktop vocabulary.
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&crate::words::GESTURES_NOT_READ.key(), &Filling::nothing())
    }
}

/// A settings write refused without replacing the previous file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotWritten {
    /// File supplied by the session.
    pub at: PathBuf,
    /// Structural or filesystem refusal.
    pub why: Unwritten,
}

impl NotWritten {
    /// Explain the refusal through the desktop vocabulary.
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(
            &crate::words::GESTURES_NOT_WRITTEN.key(),
            &Filling::nothing(),
        )
    }
}
