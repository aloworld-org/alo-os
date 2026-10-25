//! **A person gives alo a folder inside Settings**, by walking their own
//! folders and choosing one.
//!
//! This is the link that made the rest real. `alo-picking` has held the rules
//! since it was written — where a person is standing, the three moves they can
//! make, and the grant a pick becomes — and `alo_changing::Changing::granted`
//! has been able to carry one to the disk. **Nothing called either.** Settings
//! could list grants and revoke them and had no way to make one, so the list
//! could only ever say *nothing granted*, every file verb was refused forever,
//! and the refusal a person met — *grants are made by picking a folder* —
//! named an act that did not exist.
//!
//! # Every move is a row
//!
//! Settings has four keys: previous, next, choose, close. Rather than teach
//! the picker a fifth for *pick this one* and a sixth for *go up*, both are
//! rows. A person moves the ring and presses Enter, which is what they already
//! do for an accent or a shortcut, and the picker needs no explaining.
//!
//! The order is deliberate. **Giving the folder you are standing in is the
//! first row**, so the folder somebody just opened is the one row away rather
//! than the last; *up one* comes next and is absent at the top of the disk,
//! because a row that cannot do anything is a row that teaches somebody the
//! list lies; then the folders themselves.
//!
//! # Two steps, and the second is the person's to answer
//!
//! Picking a folder does not grant it. A grant has a length, ADR 0001 says
//! there is no grant that outlives the reason it was made, and the owner's
//! decision of 2026-10-24 was that **the person is asked every time** rather
//! than a default being chosen for them. So a pick moves the list on to
//! `crate::how_long_a_grant_lasts::HowLong`, and the grant is made when they
//! answer that.
//!
//! # This file makes no grant
//!
//! It answers [`WhatPickingDid::GiveItThisFolder`] and the caller does it, for
//! `crate::a_provider_is_added_in_settings`' reason: making a grant is a write
//! to the machine and a knock at the daemon, and this crate shows rather than
//! writes. It also means every rule here is tested against a disk written down
//! in a test rather than a real one.

use std::path::Path;

use alo_picking::{Chosen, NotPicked, Picker};
use alo_strings::{Filling, Said, Strings};

/// The one road to a disk, re-exported so that the window can name a `Folders`
/// without naming `alo-picking`.
///
/// **That is a rule rather than a tidiness.** `tests/settings_source.rs`
/// refuses `alo_picking` in any Settings file but the one that may carry a
/// pick to the disk, so that there is exactly one road to a grant. The window
/// needs the trait to hand one in and makes no grant, so it names this module
/// — which is what it is actually wired to — and the rule stays whole.
pub use alo_picking::Folders;

use crate::how_long_a_grant_lasts::HowLong;
use crate::the_pickers_words::{HOW_LONG, PICK_THIS_FOLDER, UP_ONE_FOLDER};

/// One row of the picker, as a person meets it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickingRow {
    /// Give alo the folder the person is standing in.
    PickThisFolder,
    /// Leave this folder for the one that holds it.
    UpOneFolder,
    /// Open this folder, by the name it is shown under.
    IntoFolder(String),
    /// Give it for this long, which makes the grant.
    For(HowLong),
}

/// What came of acting on one row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatPickingDid {
    /// The list changed and should be drawn again.
    ItChanged,
    /// Nothing happened — a row that is not in the list in front of them.
    Nothing,
    /// They chose a folder and said how long. **The caller makes the grant.**
    GiveItThisFolder {
        /// The folder, sealed by the picker so nothing can invent one.
        chosen: Box<Chosen>,
        /// How long they said.
        how_long: HowLong,
    },
}

/// A person part way through giving alo a folder.
#[derive(Debug)]
pub struct AFolderBeingPicked {
    /// Where they are standing, and what is in front of them.
    picker: Picker,
    /// The folder they chose, once they have — which is also what says the
    /// list has moved on to *how long*.
    ///
    /// **One field for both**, rather than a step enum beside a folder that
    /// may or may not be there: two fields could disagree about which step
    /// this is, and the one that goes wrong is a list showing lengths with
    /// nothing to grant them over.
    chosen: Option<Chosen>,
    /// The last refusal, in the person's own language, until they move.
    told: Option<String>,
}

impl AFolderBeingPicked {
    /// Open the picker in this folder.
    ///
    /// # Errors
    /// [`NotPicked`], as `alo-picking` refuses it — a folder that is not a
    /// full path, one whose name steps upwards, one that is not there, and one
    /// this machine will not open.
    pub fn standing_in(at: &Path, folders: &dyn Folders) -> Result<Self, NotPicked> {
        Ok(Self {
            picker: Picker::standing_in(at, folders)?,
            chosen: None,
            told: None,
        })
    }

    /// Where the person is standing.
    #[must_use]
    pub fn at(&self) -> &Path {
        self.picker.at()
    }

    /// Whether they have chosen a folder and are saying how long.
    #[must_use]
    pub const fn is_saying_how_long(&self) -> bool {
        self.chosen.is_some()
    }

    /// The last refusal, until they move.
    #[must_use]
    pub fn told(&self) -> Option<&str> {
        self.told.as_deref()
    }

    /// The heading over the list: what this step is asking.
    #[must_use]
    pub fn asking(&self, strings: &Strings) -> Said {
        let word = if self.is_saying_how_long() {
            HOW_LONG
        } else {
            alo_picking::words::ASK
        };
        strings.say(&word.key(), &Filling::nothing())
    }

    /// The rows in front of the person, in the order they are drawn.
    #[must_use]
    pub fn rows(&self) -> Vec<PickingRow> {
        if self.is_saying_how_long() {
            return HowLong::ALL.iter().copied().map(PickingRow::For).collect();
        }
        let mut rows = vec![PickingRow::PickThisFolder];
        // **Absent at the top rather than refusing there.** `Picker::go_up`
        // answers `NothingAbove`, and a row whose only outcome is a sentence
        // explaining why it did nothing is a row that teaches somebody the
        // list is not to be trusted.
        if !self.picker.is_at_the_top() {
            rows.push(PickingRow::UpOneFolder);
        }
        rows.extend(
            self.picker
                .showing()
                .iter()
                .map(|name| PickingRow::IntoFolder(name.clone())),
        );
        rows
    }

    /// What this row says, for drawing it.
    #[must_use]
    pub fn said(&self, row: &PickingRow, strings: &Strings) -> String {
        match row {
            PickingRow::PickThisFolder => strings
                .say(&PICK_THIS_FOLDER.key(), &Filling::nothing())
                .into_text(),
            PickingRow::UpOneFolder => strings
                .say(&UP_ONE_FOLDER.key(), &Filling::nothing())
                .into_text(),
            // **The folder's own name and not a sentence.** It is the person's
            // word, not ours, and putting it through the vocabulary would be
            // asking a translator about somebody's filing.
            PickingRow::IntoFolder(name) => name.clone(),
            PickingRow::For(how_long) => strings
                .say(&how_long.word().key(), &Filling::nothing())
                .into_text(),
        }
    }

    /// The person pressed Enter on this row.
    pub fn act(
        &mut self,
        row: &PickingRow,
        folders: &dyn Folders,
        strings: &Strings,
    ) -> WhatPickingDid {
        // **A row that is not in the list does nothing.** The ring is drawn
        // from `rows`, so this cannot happen from a keyboard — but a caller
        // holding a row from before a move could ask for *up one* at the top
        // of the disk, or for a folder that has since gone.
        if !self.rows().contains(row) {
            return WhatPickingDid::Nothing;
        }
        self.told = None;
        match row {
            PickingRow::PickThisFolder => match self.picker.pick() {
                // `Chosen::Nothing` is the picker being closed, which is not
                // what this row is, so it is refused rather than carried into
                // a grant that has no folder.
                Ok(chosen @ Chosen::Folder(_)) => {
                    self.chosen = Some(chosen);
                    WhatPickingDid::ItChanged
                }
                Ok(Chosen::Nothing) => WhatPickingDid::Nothing,
                Err(why) => self.refused(why, strings),
            },
            PickingRow::UpOneFolder => match self.picker.go_up(folders) {
                Ok(()) => WhatPickingDid::ItChanged,
                Err(why) => self.refused(why, strings),
            },
            PickingRow::IntoFolder(name) => match self.picker.go_into(name, folders) {
                Ok(()) => WhatPickingDid::ItChanged,
                Err(why) => self.refused(why, strings),
            },
            PickingRow::For(how_long) => match self.chosen.clone() {
                Some(chosen) => WhatPickingDid::GiveItThisFolder {
                    chosen: Box::new(chosen),
                    how_long: *how_long,
                },
                // Unreachable: `rows` only offers a length once something is
                // chosen. An arm rather than an unwrap, because a window that
                // panicked on a key press is a machine nobody can use.
                None => WhatPickingDid::Nothing,
            },
        }
    }

    /// Keep a refusal to show, in the person's own language.
    fn refused(&mut self, why: NotPicked, strings: &Strings) -> WhatPickingDid {
        self.told = Some(why.said(strings).into_text());
        WhatPickingDid::ItChanged
    }
}

#[cfg(test)]
#[path = "a_folder_is_picked_in_settings_tests.rs"]
mod tests;
