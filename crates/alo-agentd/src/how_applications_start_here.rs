//! **How an installed application starts on this machine**, read off its
//! desktop entry with `alo_launching` — the same decisions `alo-desktop`'s
//! sender makes when it carries the start out.
//!
//! The owner's decision of 2026-10-31 (ADR 0108): an approved `open` starts an
//! application exactly as a Dock press does, and the record says which road.
//! `alo_turn` touches no disk, so it asks this.
//!
//! **Read at the moment of the act**, not when the daemon started: the entry
//! that is there now is the one the desktop will start from.

use std::path::PathBuf;

use alo_launching::HowItStarts;
use alo_turn::{NoRoad, SaysHowAnApplicationStarts, TheRoad};

/// The places this machine's desktop entries live, most specific first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HowApplicationsStartHere {
    /// The `applications/` directories, as `alo_installed::where_entries_live`
    /// names them, flatpak's exports included.
    entries: Vec<PathBuf>,
}

impl HowApplicationsStartHere {
    /// Read from these places.
    #[must_use]
    pub const fn in_these(entries: Vec<PathBuf>) -> Self {
        Self { entries }
    }

    /// The places entries live, for the installed list to be read from the
    /// same ones.
    #[must_use]
    pub fn places(&self) -> &[PathBuf] {
        &self.entries
    }
}

impl SaysHowAnApplicationStarts for HowApplicationsStartHere {
    fn how_it_starts(&self, identifier: &str) -> Result<TheRoad, NoRoad> {
        let location =
            alo_launching::where_the_entry_is(identifier, &self.entries).ok_or(NoRoad::NoEntry)?;
        let entry = std::fs::read_to_string(&location)
            .ok()
            .as_deref()
            .and_then(alo_launching::TheEntry::read)
            .ok_or(NoRoad::NoEntry)?;
        match alo_launching::how_it_starts(identifier, &entry, location.to_str()) {
            Ok(HowItStarts::OverTheBus { .. }) => Ok(TheRoad::OverTheBus),
            Ok(HowItStarts::FromItsCommand(_)) => Ok(TheRoad::FromItsCommand),
            Err(_) => Err(NoRoad::NothingStartsIt),
        }
    }
}

#[cfg(test)]
#[path = "how_applications_start_here_tests.rs"]
mod tests;
