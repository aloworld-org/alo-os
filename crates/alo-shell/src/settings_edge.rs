//! **Settings' window edge**: *Settings* and Close above it, in the focused or
//! unfocused look, following the camera — `crate::an_alo_windows_edge`, the
//! edge every one of alo's own windows wears.
//!
//! # What the design draws, and what is a reading
//!
//! The Settings frames (`550:98`, `550:420`) draw the window alone, with no
//! edge; ADR 0145 gives every one of alo's windows the window look canvas
//! windows have, and the owner asked for this edge by name. So:
//!
//! - **the title** is `settings.title`, the window's own name, which its side
//!   column already heads;
//! - **no icon**: the design draws none for Settings, and the edge draws the
//!   grip's mark where an application has no icon this machine knows, which
//!   keeps the target from being an empty square. *A reading, for the owner to
//!   overrule*, in `docs/design/the-settings-window.md`;
//! - **Close**, which shuts Settings as Escape does — giving up a provider
//!   form that is open, as giving up on the form does. Nothing is written by
//!   closing it.
//!
//! The edge is laid out over the window **where the last frame drew it**
//! (`crate::settings_as_drawn`), so the two cannot come apart at any zoom.

use smithay::utils::{Logical, Point};

use crate::an_alo_windows_edge::{OnItsEdge, WhatItShows, its_picture, what_a_press_is_on};
use crate::which_window_is_in_front::AKindOfWindow;

impl crate::Server {
    /// Where Settings' window is seen on the display, while it is open here
    /// and has been drawn.
    fn where_settings_is_seen(
        &self,
    ) -> Option<smithay::utils::Rectangle<i32, smithay::utils::Physical>> {
        if !self.settings_is_open_here() {
            return None;
        }
        self.settings_as_drawn
            .as_ref()
            .map(crate::settings_as_drawn::SettingsAsDrawn::the_window_on_the_screen)
    }

    /// Settings' edge as a picture for this display: `output` its size in
    /// pixels and `scale` its scale in per cent.
    pub(crate) fn the_settings_edge_picture(
        &self,
        fonts: &mut cosmic_text::FontSystem,
        says: &alo_strings::Strings,
        output: (i32, i32),
        scale: u32,
    ) -> Option<crate::window_edge_picture::EdgePicture> {
        let window = self.where_settings_is_seen()?;
        let title = says.say(
            &crate::the_settings_words::SETTINGS.key(),
            &alo_strings::Filling::nothing(),
        );
        its_picture(
            window,
            WhatItShows {
                title: title.text(),
                icon: None,
                focused: self.settings_is_taking_the_keys(),
            },
            self.where_the_pointer_is_in_logical_pixels(),
            fonts,
            (output, scale),
        )
    }

    /// Whether Settings' edge is under `at`, in the display's pixels.
    pub(crate) fn the_settings_edge_is_under(&self, at: Point<i32, Logical>) -> bool {
        self.where_settings_is_seen().is_some_and(|window| {
            what_a_press_is_on(
                window,
                at,
                (
                    self.the_room_an_edge_is_clamped_in(),
                    self.the_scale_an_edge_is_measured_at(),
                ),
            )
            .is_some()
        })
    }

    /// A press at `at`, in the display's pixels, offered to Settings' edge.
    /// Answers whether the edge took it: Close shuts Settings, anywhere else
    /// on it brings Settings in front.
    pub(crate) fn a_press_on_the_settings_edge(&mut self, at: Point<i32, Logical>) -> bool {
        let Some(window) = self.where_settings_is_seen() else {
            return false;
        };
        match what_a_press_is_on(
            window,
            at,
            (
                self.the_room_an_edge_is_clamped_in(),
                self.the_scale_an_edge_is_measured_at(),
            ),
        ) {
            None => false,
            Some(OnItsEdge::Close) => {
                self.that_provider_is_not_being_added();
                self.settings.close();
                self.settings_on_the_plane = None;
                self.settings_as_drawn = None;
                true
            }
            Some(OnItsEdge::TheRest) => {
                self.the_composer.it_gave_the_keyboard_up();
                self.the_person_went_to(AKindOfWindow::Settings);
                true
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "settings_edge_tests.rs"]
mod tests;
