//! A press while the launcher is open: on Close, on a row, on the panel, or on
//! the dimmed screen around it — and which rows the keyboard walks.
//!
//! # Two faces, since 2026-11-11
//!
//! The owner's decision of that day keeps the keyboard **in the Dock's own
//! field** for a click on it, Super+Space and Super+A, with the matches rising
//! from it in a compact, non-modal list. A press on that list's row does what
//! Enter does on it; a press on the field is the field's; and a press anywhere
//! else **closes the list and goes on to what it landed on**, which is what
//! every list opened from the Dock does (`Server::the_open_surface_takes_this_press`).
//! What was typed stays in the field, as it does when a person clicks a window.
//!
//! The centred panel over a dimmed screen is still the face *Help with this
//! window* opens, and there a press anywhere is the panel's own.
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
use crate::where_the_bars_matches_rise::TheMatchesPlaces;

impl crate::Server {
    /// Where the launcher was drawn on the last frame, or that it was not.
    pub(crate) fn the_launcher_was_drawn(&mut self, places: Option<TheBarsPlaces>) {
        self.the_launchers_places = places;
    }

    /// Where the matches rising from the Dock's field were drawn on the last
    /// frame, or that they were not.
    pub(crate) fn the_bars_matches_were_drawn(&mut self, places: Option<TheMatchesPlaces>) {
        self.the_bars_matches_places = places;
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
            || self
                .the_bars_matches_places
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
            return self.a_press_while_typing_in_the_docks_field(at);
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

    /// A press while the keyboard is in the Dock's field, with or without the
    /// matches risen from it. Answers whether the list took it.
    fn a_press_while_typing_in_the_docks_field(&mut self, at: Point<i32, Physical>) -> bool {
        if let Some(places) = self.the_bars_matches_places.clone()
            && places.panel.contains(at)
        {
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
            // **Taken either way**: the list lies over the canvas, so a press
            // between its rows must not reach what is behind it.
            return true;
        }
        // **A press on the field is the field's**, which keeps the keyboard
        // and what is typed (`WhatAPressOnTheDockDoes::TypeIntoTheComposer`).
        if self.a_press_lands_on_the_docks_field(at) {
            return false;
        }
        // **Anywhere else closes the list and gives the keyboard back**, and the
        // press goes on to whatever it landed on. What was typed stays, as it
        // does when a person clicks a window: somebody who looked away and
        // comes back is writing one request.
        self.the_composer.it_gave_the_keyboard_up();
        self.the_bars_matches_places = None;
        self.which_row_in_the_bar = None;
        false
    }

    /// Whether `at` is on the Dock's field, on any display.
    fn a_press_lands_on_the_docks_field(&self, at: Point<i32, Physical>) -> bool {
        self.the_bars_matches_places
            .as_ref()
            .is_some_and(|places| places.field.contains(at))
            || self.dock_as_drawn.values().any(|dock| {
                crate::a_press_on_the_dock::what_a_press_on_the_dock_does(dock, at)
                    == crate::a_press_on_the_dock::WhatAPressOnTheDockDoes::TypeIntoTheComposer
            })
    }
}

#[cfg(test)]
#[path = "a_press_on_the_launcher_tests.rs"]
mod tests;
