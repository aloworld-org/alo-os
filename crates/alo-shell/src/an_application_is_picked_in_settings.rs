//! **A person gives alo one application inside Settings**: which one, by
//! name, and then how long.
//!
//! ADR 0143 Q10 says where: *in Settings … one application, by name*. The
//! owner decided on 2026-11-15 how it looks — **the terminal settings' own
//! picker**, frames `494:29660` and `501:35565` — and
//! `docs/design/the-settings-window.md` records what was taken off them and
//! every reading that had to be made.
//!
//! Before this nothing on the machine could make the grant. The daemon
//! honoured one, every application verb asked for one, and Settings could only
//! give a folder, so at the default level *open VSCodium* was refused before
//! any approval card could appear.
//!
//! # The folder picker's shape, on purpose
//!
//! Every move is a row, worked with the four keys Settings already has
//! (`crate::a_folder_is_picked_in_settings` says why), and choosing is two
//! answers: an application, then one of the same four lengths a folder is
//! given for (`crate::how_long_a_grant_lasts`), asked every time. Escape leaves
//! without granting anything, which the line under the list says.
//!
//! # This file makes no grant
//!
//! It answers [`WhatChoosingDid::GiveItThisApplication`] and
//! `crate::settings_granted` makes it, through the same call a folder goes
//! through — so there is still one road to a grant, and it is the person's.

use alo_picking::{Applications, Chosen};
use alo_strings::{Filling, Said, Strings};

use crate::how_long_a_grant_lasts::HowLong;
use crate::the_pickers_words::{
    CHOOSE_ONE_APPLICATION, HOW_LONG_FOR_AN_APPLICATION, ONLY_THE_SELECTED_APPLICATION,
    SELECT_AN_APPLICATION, THE_APPLICATION,
};

/// One row, as a person meets it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationRow {
    /// Give alo this application, by the identifier the machine knows it by.
    This(String),
    /// Give it for this long, which makes the grant.
    For(HowLong),
}

/// What came of acting on one row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatChoosingDid {
    /// The list changed and should be drawn again.
    ItChanged,
    /// Nothing happened — a row that is not in the list in front of them.
    Nothing,
    /// They chose an application and said how long. **The caller makes the
    /// grant.**
    GiveItThisApplication {
        /// The application, sealed by `alo-picking` so nothing can invent one.
        chosen: Box<Chosen>,
        /// How long they said.
        how_long: HowLong,
    },
}

/// A person part way through giving alo an application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnApplicationBeingPicked {
    /// What they are choosing among.
    applications: Applications,
    /// The application they chose, once they have — which is also what says
    /// the list has moved on to *how long*, for the folder picker's reason:
    /// one field cannot disagree with itself about which step this is.
    chosen: Option<Chosen>,
}

impl AnApplicationBeingPicked {
    /// Open the list over these applications.
    #[must_use]
    pub const fn among(applications: Applications) -> Self {
        Self {
            applications,
            chosen: None,
        }
    }

    /// Whether they have chosen an application and are saying how long.
    #[must_use]
    pub const fn is_saying_how_long(&self) -> bool {
        self.chosen.is_some()
    }

    /// The heading: *Select an application*, then the question of how long,
    /// naming the one they chose.
    #[must_use]
    pub fn asking(&self, strings: &Strings) -> Said {
        match self.chosen.as_ref().and_then(Chosen::application) {
            Some(picked) => strings.say(
                &HOW_LONG_FOR_AN_APPLICATION.key(),
                &Filling::of(THE_APPLICATION, picked.called().to_owned()),
            ),
            None => strings.say(&SELECT_AN_APPLICATION.key(), &Filling::nothing()),
        }
    }

    /// The line under the heading, while they are choosing which one.
    #[must_use]
    pub fn beneath(&self, strings: &Strings) -> Option<String> {
        (!self.is_saying_how_long()).then(|| {
            strings
                .say(&CHOOSE_ONE_APPLICATION.key(), &Filling::nothing())
                .into_text()
        })
    }

    /// The line under the list: only the one chosen, and how to leave.
    #[must_use]
    pub fn at_the_end(&self, strings: &Strings) -> String {
        strings
            .say(&ONLY_THE_SELECTED_APPLICATION.key(), &Filling::nothing())
            .into_text()
    }

    /// The rows in front of the person, in the order they are drawn.
    #[must_use]
    pub fn rows(&self) -> Vec<ApplicationRow> {
        if self.is_saying_how_long() {
            return HowLong::ALL.iter().copied().map(ApplicationRow::For).collect();
        }
        self.applications
            .listed()
            .iter()
            .map(|listed| ApplicationRow::This(listed.identifier().to_owned()))
            .collect()
    }

    /// What this row says, for drawing it: an application's own name, which
    /// is its entry's word and not ours, or a length.
    #[must_use]
    pub fn said(&self, row: &ApplicationRow, strings: &Strings) -> String {
        match row {
            ApplicationRow::This(identifier) => self
                .applications
                .listed()
                .iter()
                .find(|listed| listed.identifier() == identifier)
                .map_or_else(|| identifier.clone(), |listed| listed.called().to_owned()),
            ApplicationRow::For(how_long) => strings
                .say(&how_long.word().key(), &Filling::nothing())
                .into_text(),
        }
    }

    /// The person pressed Enter on this row.
    pub fn act(&mut self, row: &ApplicationRow) -> WhatChoosingDid {
        // **A row that is not in the list does nothing**, for the folder
        // picker's reason: the ring is drawn from `rows`, and a row held from
        // before a move is not a choice the person made now.
        if !self.rows().contains(row) {
            return WhatChoosingDid::Nothing;
        }
        match row {
            ApplicationRow::This(identifier) => match self.applications.pick(identifier) {
                Ok(chosen) => {
                    self.chosen = Some(chosen);
                    WhatChoosingDid::ItChanged
                }
                // Unreachable: `rows` lists only what `pick` takes. An arm
                // rather than an unwrap, because a window that panicked on a
                // key press is a machine nobody can use.
                Err(_) => WhatChoosingDid::Nothing,
            },
            ApplicationRow::For(how_long) => match self.chosen.clone() {
                Some(chosen) => WhatChoosingDid::GiveItThisApplication {
                    chosen: Box::new(chosen),
                    how_long: *how_long,
                },
                None => WhatChoosingDid::Nothing,
            },
        }
    }
}

#[cfg(test)]
#[path = "an_application_is_picked_in_settings_tests.rs"]
mod tests;
