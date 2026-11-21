//! **What a file opens with on this machine, and which applications open
//! folders**, read off the desktop entries and the person's own choices at the
//! moment a turn asks — so a file opened *in* a file manager opens where a
//! double-click there would (the owner's decision of 2026-11-17;
//! `alo_turn::SaysWhatAFileOpensWith`).
//!
//! Every answer is the one the rest of the machine gives:
//!
//! - **what opens folders** is `alo_installed::what_opens_folders`, the one
//!   place that fact is declared — an entry that opens `inode/directory`;
//! - **what a file opens with** is `alo_applications::WhatOpensWhat`, the
//!   open-with portal's own answer: the person's choice for the file's kind, or
//!   the first installed application that declares it, from the file's own
//!   bytes, and never a fallback.
//!
//! **A person's choices that will not read refuse**, in their own words, rather
//! than opening the file in something they chose against; a file that is not
//! there is a person who has chosen nothing, which is an answer.

use std::path::{Path, PathBuf};

use alo_applications::{Chosen, WhatOpensWhat};
use alo_strings::Strings;
use alo_turn::{NothingOpensIt, SaysWhatAFileOpensWith};

/// The places this machine's entries live, and where the person keeps their
/// choices of what opens what.
pub struct WhatFilesOpenWithHere<'s> {
    /// The `applications/` directories, as `alo_installed` names them.
    entries: Vec<PathBuf>,
    /// The person's `what-opens-what.toml`, where their session says where it
    /// is; [`None`] for a session with no home, where nothing was chosen.
    chosen_at: Option<PathBuf>,
    /// The words a refusal is said in.
    strings: &'s Strings,
}

impl<'s> WhatFilesOpenWithHere<'s> {
    /// Read from these places, with the person's choices at `chosen_at`.
    #[must_use]
    pub const fn in_these(
        entries: Vec<PathBuf>,
        chosen_at: Option<PathBuf>,
        strings: &'s Strings,
    ) -> Self {
        Self {
            entries,
            chosen_at,
            strings,
        }
    }
}

impl SaysWhatAFileOpensWith for WhatFilesOpenWithHere<'_> {
    fn opens_folders(&self, identifier: &str) -> bool {
        alo_installed::what_opens_folders(&self.entries).contains(identifier)
    }

    fn is_a_folder(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn called(&self, identifier: &str) -> Option<String> {
        // **The one answer to what a person calls an application**, which the
        // egress corner names it by too (`alo_installed::what_a_person_calls_it`).
        alo_installed::what_a_person_calls_it(&self.entries, self.strings, identifier)
    }

    fn opens_with(&self, file: &Path) -> Result<String, NothingOpensIt> {
        let (installed, declared) = alo_installed::read_with_what_they_open(&self.entries);
        let chosen = match &self.chosen_at {
            Some(at) => alo_applications::keeping::read(at)
                .map_err(|why| NothingOpensIt::because(why.said(self.strings)))?,
            None => Chosen::untouched(),
        };
        let mut opened = std::fs::File::open(file).map_err(|why| {
            NothingOpensIt::because(
                alo_files::RealError::Unreadable {
                    path: file.display().to_string(),
                    why: why.to_string(),
                }
                .said(self.strings),
            )
        })?;
        WhatOpensWhat::on(&installed, &declared, &chosen)
            .what_opens(&mut opened)
            .map(|opener| opener.application().identifier().to_owned())
            .map_err(|why| NothingOpensIt::because(why.said(self.strings)))
    }
}

#[cfg(test)]
#[path = "what_files_open_with_here_tests.rs"]
mod tests;
