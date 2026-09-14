//! The window of what is filling the disk: `docs/features.md`'s *sizes you can
//! open up and click through, not a number in Settings*.
//!
//! # What it counts, and when
//!
//! A person names a folder, and `alo_measuring::Holding::of` counts it: the
//! whole tree, each node's size the sum of what is inside it plus its own
//! bytes, and every way a size falls short of the truth written on the node it
//! happens to. The window counts when it opens and when the person asks it to
//! count again, and at no other time — the whole disk is never walked unasked.
//!
//! # Opening up
//!
//! The folder asked about is open when the window opens, so what is directly
//! inside it is in front of the person at once. Opening a folder shows what is
//! inside it; closing it hides that again. What is open is remembered by path,
//! so counting again leaves the same folders open. Opening changes what is in
//! view and never what was counted: nothing is counted again, sorted or left
//! out by opening or closing.
//!
//! # It counts and does nothing else
//!
//! Nothing here deletes, moves or empties anything, and nothing reaches an
//! agent. A folder that could not be counted at all is `alo-measuring`'s
//! refusal drawn in the window — never an empty tree.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use alo_measuring::{Holding, NotMeasured};

use crate::FillingKey;
use crate::filling_rows::{can_open, in_view};

/// What the window of what is filling the disk draws now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillingShows<'a> {
    /// The window is closed.
    Nothing,
    /// The tree of sizes.
    Holding {
        /// What `alo-measuring` counted.
        holding: &'a Holding,
        /// The folders a person has open, by path.
        opened: &'a BTreeSet<PathBuf>,
        /// The selected row, counted from the folder asked about.
        selected: usize,
    },
    /// Why nothing could be counted, in `alo-measuring`'s words.
    Refusal(&'a NotMeasured),
}

/// What a key press did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillingPressed {
    /// Nothing at all.
    Nothing,
    /// Another row is selected.
    Moved,
    /// The selected folder was opened.
    Opened,
    /// The selected folder was closed.
    Shut,
    /// The folder was counted again.
    Counted,
    /// The window closed.
    Closed,
}

/// The window of what is filling the disk.
#[derive(Debug, Default)]
pub struct FillingWindow {
    /// The folder a person named, while the window is open.
    folder: Option<PathBuf>,
    /// What was counted, while the window shows it.
    holding: Option<Holding>,
    /// Why nothing could be counted, while the window says so.
    refusal: Option<NotMeasured>,
    /// The folders open, by path.
    opened: BTreeSet<PathBuf>,
    /// The selected row.
    selected: usize,
}

impl FillingWindow {
    /// The window, closed.
    #[must_use]
    pub fn closed() -> Self {
        Self::default()
    }

    /// A person opened the window on `folder`: it is counted now, and opened.
    pub fn opened(&mut self, folder: &Path) {
        *self = Self::closed();
        self.folder = Some(folder.to_path_buf());
        self.count();
        if let Some(holding) = &self.holding {
            self.opened.insert(holding.tree.at.clone());
        }
    }

    /// One key press. Every key does nothing while the window is closed.
    pub fn pressed(&mut self, key: FillingKey) -> FillingPressed {
        if self.folder.is_none() {
            return FillingPressed::Nothing;
        }
        match key {
            FillingKey::CountAgain => {
                self.count();
                return FillingPressed::Counted;
            }
            FillingKey::Close => {
                *self = Self::closed();
                return FillingPressed::Closed;
            }
            _ => {}
        }
        let Some(holding) = &self.holding else {
            return FillingPressed::Nothing;
        };
        let rows = in_view(holding, &self.opened);
        let last = rows.len().saturating_sub(1);
        let before = self.selected;
        let chosen = rows
            .get(self.selected)
            .filter(|seen| can_open(seen.node))
            .map(|seen| seen.node.at.clone());
        match (key, chosen) {
            (FillingKey::Previous, _) => self.selected = self.selected.saturating_sub(1),
            (FillingKey::Next, _) => self.selected = (self.selected + 1).min(last),
            (FillingKey::First, _) => self.selected = 0,
            (FillingKey::Last, _) => self.selected = last,
            (FillingKey::Open, Some(at)) => {
                return if self.opened.insert(at) {
                    FillingPressed::Opened
                } else {
                    FillingPressed::Nothing
                };
            }
            (FillingKey::Shut, Some(at)) => {
                return if self.opened.remove(&at) {
                    FillingPressed::Shut
                } else {
                    FillingPressed::Nothing
                };
            }
            (FillingKey::Toggle, Some(at)) => {
                return if self.opened.remove(&at) {
                    FillingPressed::Shut
                } else {
                    self.opened.insert(at);
                    FillingPressed::Opened
                };
            }
            _ => return FillingPressed::Nothing,
        }
        if before == self.selected {
            FillingPressed::Nothing
        } else {
            FillingPressed::Moved
        }
    }

    /// Close the window, letting go of what was counted.
    pub fn close(&mut self) {
        *self = Self::closed();
    }

    /// What the window draws now.
    #[must_use]
    pub fn shows(&self) -> FillingShows<'_> {
        if let Some(why) = &self.refusal {
            return FillingShows::Refusal(why);
        }
        match &self.holding {
            Some(holding) => FillingShows::Holding {
                holding,
                opened: &self.opened,
                selected: self.selected,
            },
            None => FillingShows::Nothing,
        }
    }

    /// Whether the window is open, so the host routes keys here.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.folder.is_some()
    }

    /// Count the folder a person named, keeping what they had open and as near
    /// the row they had selected as the new count allows.
    fn count(&mut self) {
        let Some(folder) = &self.folder else {
            return;
        };
        match Holding::of(folder) {
            Ok(holding) => {
                self.refusal = None;
                let last = in_view(&holding, &self.opened).len().saturating_sub(1);
                self.selected = self.selected.min(last);
                self.holding = Some(holding);
            }
            Err(why) => {
                self.holding = None;
                self.refusal = Some(why);
            }
        }
    }
}

#[cfg(test)]
#[path = "filling_window_tests.rs"]
mod tests;
