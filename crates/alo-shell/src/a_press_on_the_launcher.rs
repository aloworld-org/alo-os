//! A press while the launcher is open: on Close, on a row, on the panel, or on
//! the dimmed screen around it — and which rows the keyboard walks.
//!
//! **Where everything is comes from the draw**, recorded by
//! [`crate::Server::the_launcher_was_drawn`], because the launcher is laid out
//! where the display's scale is known and nowhere else — the rule the Dock's
//! slots already keep (`Server::the_docks_slots_were_drawn`).
//!
//! # One performer for the pointer and the keyboard
//!
//! A press on a row chooses it and then does what Enter does, through
//! `Server::a_key_reached_the_composer` — so a pointer and a key cannot come to
//! do different things to one row. Close does what Escape does.

use smithay::utils::{Physical, Point};

use crate::composer_keys::ComposerKey;
use crate::where_the_bar_opens::TheBarsPlaces;

impl crate::Server {
    /// Where the launcher was drawn on the last frame, or that it was not.
    pub(crate) fn the_launcher_was_drawn(&mut self, places: Option<TheBarsPlaces>) {
        self.the_launchers_places = places;
    }

    /// How many rows the keyboard walks: every application found, and the row
    /// that asks alo where the last draw offered it.
    pub(crate) fn how_many_rows_the_launcher_has(&self) -> usize {
        let found = crate::the_bar_finds::what_matches(
            self.the_composer.typed(),
            &self.what_this_machine_has,
            &self.what_each_is_called,
            &self.what_each_is,
        )
        .len();
        found + usize::from(self.the_launcher_offers_to_ask())
    }

    /// Whether the last draw offered the row that asks alo.
    pub(crate) fn the_launcher_offers_to_ask(&self) -> bool {
        self.the_launchers_places
            .as_ref()
            .is_some_and(|places| places.ask.is_some())
    }

    /// Whether the keyboard is on the row that asks alo, which is one past the
    /// last application found.
    pub(crate) fn the_ask_row_is_chosen(&self) -> bool {
        self.the_launcher_offers_to_ask()
            && self.which_row_in_the_bar
                == Some(
                    crate::the_bar_finds::what_matches(
                        self.the_composer.typed(),
                        &self.what_this_machine_has,
                        &self.what_each_is_called,
                        &self.what_each_is,
                    )
                    .len(),
                )
    }

    /// A press at `at`, in the display's pixels, while the launcher may be
    /// open. Answers whether the launcher took it.
    ///
    /// **Every press while it is open is the launcher's**: the screen behind
    /// is dimmed to say it is not there to be pressed, so a press on it closes
    /// the launcher, as Escape does, and reaches nothing behind.
    pub(crate) fn a_press_on_the_launcher(&mut self, at: Point<i32, Physical>) -> bool {
        if !self.the_composer.has_the_keyboard() {
            return false;
        }
        let Some(places) = self.the_launchers_places.clone() else {
            return false;
        };
        if places.close.contains(at) || !places.panel.contains(at) {
            drop(self.a_key_reached_the_composer(ComposerKey::Clear));
            self.the_launchers_places = None;
            return true;
        }
        let row = places
            .rows
            .iter()
            .find(|(_, row)| row.contains(at))
            .map(|(which, _)| *which)
            .or_else(|| {
                places
                    .ask
                    .filter(|ask| ask.contains(at))
                    .map(|_| self.how_many_rows_the_launcher_has().saturating_sub(1))
            });
        if let Some(which) = row {
            self.which_row_in_the_bar = Some(which);
            drop(self.a_key_reached_the_composer(ComposerKey::Ask));
        }
        true
    }
}

#[cfg(test)]
#[path = "a_press_on_the_launcher_tests.rs"]
mod tests;
