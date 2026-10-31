//! **A desktop entry that was there and could not be read**, said rather than
//! dropped.
//!
//! The walk has always skipped a file it could not read, so that one broken
//! entry cannot cost a person every other application — and that stays. What
//! changes is that the skip is **carried out of the walk**: until this file, an
//! unreadable entry and an application that was never installed were the same
//! absence, and a person asking *why is Firefox not in the launcher* could not
//! be told which.
//!
//! # What is a refusal, and what is not
//!
//! An entry that says `NoDisplay=true`, `Hidden=true` or a `Type` other than
//! `Application` is **not** here. Those are an entry's author saying *do not
//! list me*, which is the entry working; it is left out without a word, as it
//! always was. What is here is a file that could not be opened, or that does
//! not say what a desktop entry has to.

use std::path::{Path, PathBuf};

/// One entry that could not be read, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotRead {
    /// The file, or the directory that would not list.
    at: PathBuf,
    /// What was wrong with it.
    why: WhyNotRead,
}

/// Why an entry could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhyNotRead {
    /// The file or directory would not open — no permission, not text, or a
    /// link to something that is gone, which is what a flatpak export left by
    /// a half-removed application looks like.
    WouldNotOpen(std::io::ErrorKind),
    /// It opened and has no `[Desktop Entry]` group, so it is not an entry.
    NoEntryInIt,
    /// It has the group and no `Name`, which the specification requires.
    NoName,
}

impl NotRead {
    /// One refusal.
    pub(crate) const fn at(at: PathBuf, why: WhyNotRead) -> Self {
        Self { at, why }
    }

    /// The file or directory it was about.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.at
    }

    /// What was wrong with it.
    #[must_use]
    pub const fn why(&self) -> WhyNotRead {
        self.why
    }
}
