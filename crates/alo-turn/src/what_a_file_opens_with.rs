//! **What a double-click would open a file in**, as this machine answers it.
//!
//! The owner's decision of 2026-11-17: a person asking alo to open a file *in*
//! a file manager means what a double-click there means — the file opens in
//! the application its kind opens with. `alo_turn` touches no disk, so it asks
//! this, as it asks [`crate::SaysHowAnApplicationStarts`] how an application
//! starts; the daemon answers it from the same desktop entries and from the
//! person's own choices (`alo_applications::WhatOpensWhat`), read at the moment
//! it is asked.
//!
//! **A machine that was not told this cannot tell a file manager**, and opens a
//! file in the application named, as before
//! (`crate::Machine::knowing_what_files_open_with`).

use std::path::Path;

use alo_strings::Said;

/// What this machine says about which application opens what.
pub trait SaysWhatAFileOpensWith {
    /// Whether the application with this identifier is one this machine opens
    /// folders with — a file manager, by its own entry
    /// (`alo_installed::what_opens_folders`).
    fn opens_folders(&self, identifier: &str) -> bool;

    /// Whether this real path is a folder, which a file manager opens itself.
    fn is_a_folder(&self, path: &Path) -> bool;

    /// **The name a person reads for this application** — its entry's `Name`,
    /// in their language where the entry says it in that language — or
    /// [`None`] where its entry names it nothing. A card names applications by
    /// this, never by an identifier, wherever there is one.
    fn called(&self, identifier: &str) -> Option<String>;

    /// The identifier of the application this file opens with, as a
    /// double-click would open it: the person's choice for its kind, or the
    /// first installed application that declares it.
    ///
    /// # Errors
    /// [`NothingOpensIt`], in the words the person is told.
    fn opens_with(&self, file: &Path) -> Result<String, NothingOpensIt>;
}

/// Why nothing opens a file, in the words the person is told.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NothingOpensIt {
    /// What the machine said.
    said: Said,
}

impl NothingOpensIt {
    /// Nothing opens it, for the reason `said` gives.
    #[must_use]
    pub const fn because(said: Said) -> Self {
        Self { said }
    }

    /// What the person is told.
    #[must_use]
    pub const fn said(&self) -> &Said {
        &self.said
    }
}
