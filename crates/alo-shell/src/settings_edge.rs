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

use crate::an_alo_windows_edge::{WhatItShows, its_picture};

impl crate::Server {
    /// Where Settings' window is seen on the display, while it is open here
    /// and has been drawn.
    pub(crate) fn where_settings_is_seen(
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
            self,
            (alo_put_aside::AlosOwnWindow::Settings, window),
            WhatItShows {
                title: title.text(),
                focused: self.settings_is_taking_the_keys(),
            },
            fonts,
            (output, scale),
        )
    }

    /// **The person closed Settings**, by its edge's Close or by Alt+F4
    /// (`crate::a_chord_closes_alos_own_window`): one road, so the two cannot
    /// disagree about what closing gives up.
    ///
    /// A provider form that is open is given up, as giving up on the form
    /// does, and nothing is written.
    pub(crate) fn the_person_closes_settings(&mut self) {
        self.that_provider_is_not_being_added();
        self.settings.close();
        self.settings_on_the_plane = None;
        self.settings_as_drawn = None;
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "settings_edge_tests.rs"]
mod tests;
