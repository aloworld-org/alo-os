//! **Settings is sized for the room it opens in**, not for the whole screen:
//! the display less the Dock's band and the put-aside panel's column, and less
//! its own band above it — so the whole window, band included, is on the
//! screen above the Dock and clear of the panel.
//!
//! # The owner's decision, 2026-11-13
//!
//! Asked whether Settings should fit above the Dock, given that the design's
//! size rule (`crate::settings_frame`) was the rule *of the display*, the owner
//! answered **"Yes, fit above the Dock"**: its size is derived from the usable
//! space above the Dock, and on a small screen it is shorter and its page
//! scrolls (`crate::settings_raster` already scrolls the page to the row the
//! keyboard is on). And asked about the panel, **"Windows open clear of it"**.
//!
//! # What was wrong
//!
//! On the design's own 1 280 × 800 frame the rule gave 1 152 × 688, under a
//! 54 band: 742 tall with the band, where the walk's Dock (`w26-43`) left 717.
//! #1217 placed it with its band at the top of the room and its foot still
//! under the Dock, and said making it fit was the owner's call. It is now made.
//!
//! # The rule
//!
//! **The same rule, asked of the room**: at most 1 200 × 816, never nearer the
//! room's edge than 64 across and 56 down, through the person's text size and
//! the display's scale. The room is the display's `the_work_area` with only the
//! Dock and the panel counted — the two controls a window alo places opens
//! clear of (`crate::the_room_alo_places_a_window_in`) — **as last reserved**,
//! so the Dock giving way does not make Settings grow and shrink
//! (`docs/design/when-the-dock-gives-way.md`), less the band, which is part of
//! the window as a person sees it.

use smithay::utils::{Physical, Rectangle, Size};

use crate::the_room_a_window_fills::{WhatIsReserved, the_work_area};

/// **The size Settings' rule is asked of** on a display of `size` pixels: the
/// display less `dock` and `panel`, as last reserved, and less `band` — its
/// own band's height in the same pixels — off the height.
///
/// With neither control known it is the display less the band; a control that
/// covers nothing takes nothing.
#[must_use]
pub(crate) fn the_room_settings_is_sized_for(
    size: (i32, i32),
    dock: Option<Rectangle<i32, Physical>>,
    panel: Option<Rectangle<i32, Physical>>,
    band: i32,
) -> (i32, i32) {
    let room = the_work_area(
        size,
        WhatIsReserved {
            dock,
            panel,
            top: None,
            leaving: None,
        },
    );
    (room.size.w, room.size.h.saturating_sub(band.max(0)).max(0))
}

impl crate::Server {
    /// **The size Settings' rule is asked of on the display named**, which is
    /// `size` pixels: [`the_room_settings_is_sized_for`] with this display's
    /// Dock and panel as last reserved, and the band at the scale windows'
    /// strips are placed at.
    #[must_use]
    pub(crate) fn the_room_settings_is_sized_for(
        &self,
        named: &str,
        size: (i32, i32),
    ) -> (i32, i32) {
        let reserved = self
            .surfaces
            .rooms
            .iter()
            .find(|(name, _, _)| name == named)
            .map(|(_, _, reserved)| *reserved)
            .unwrap_or_default();
        let strip = crate::a_window_and_its_strip::ItsStrip::at(
            self.the_display_scale_edges_are_measured_at(),
        );
        let band = strip.round_a_size(Size::from((1, 1))).h.saturating_sub(1);
        the_room_settings_is_sized_for(size, reserved.dock, reserved.panel, band)
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
#[path = "settings_fits_the_room_tests.rs"]
mod tests;
