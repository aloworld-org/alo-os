//! **alo's own windows open in the room the Dock and the put-aside panel
//! leave**, as an application's window does: Settings, the record and alo's
//! terminal.
//!
//! `docs/design/where-a-new-window-opens.md`, amended 2026-11-12: *a window alo
//! places lands wholly in the room the Dock leaves*; amended 2026-11-01:
//! *alo's own windows count as windows*; 2026-11-07, the owner on whether
//! they follow the same rule: *"Yes, the same"*; and 2026-11-13, on the
//! panel: *"Windows open clear of it"*.
//!
//! # What was wrong
//!
//! Seen on a booted machine on 2026-11-13 (`w26-43`, 1 280 × 800): Settings,
//! opened from the Dock, was placed reaching down behind the Dock. Confirmed by
//! reading the code: #1206 gave applications' windows the room the Dock leaves
//! (`crate::the_room_alo_places_a_window_in`), and Settings, the record and the
//! terminal each still placed against the whole view, with no fixed controls
//! and nothing keeping them inside.
//!
//! # The rule now
//!
//! Each of the three is placed in the same room an application's window is —
//! the view down to the Dock's top and clear of the panel's column — and kept
//! wholly inside it, moved the least it must be, before the full view's rule
//! (`crate::room_in_a_full_view`) is asked, exactly as
//! `Server::where_this_window_goes` does for an application.
//!
//! **Settings is also sized for that room** since the owner's *"Yes, fit above
//! the Dock"* of 2026-11-13 (`crate::settings_fits_the_room`), so on a small
//! screen it opens shorter rather than with its foot under the Dock. The record
//! and the terminal keep their own size rules, which are not Settings'.

use smithay::utils::{Logical, Rectangle};

impl crate::Server {
    /// **The room one of alo's own windows opens in**: `view` — what the
    /// person will be looking at once any glide ends — down to the Dock's top
    /// and clear of the put-aside panel's column, each where it was last drawn
    /// reaching into it.
    pub(crate) fn the_room_an_alo_window_opens_in(
        &self,
        view: Rectangle<i32, Logical>,
    ) -> Rectangle<i32, Logical> {
        let camera = self.the_camera_after_the_glide();
        let zoom = crate::scene::drawn_at(camera);
        let drawn = self
            .the_controls_of_the_display_for(None)
            .and_then(crate::canvas_fixed_controls::FixedControls::labelled);
        let on_the_plane = |control: Option<Rectangle<i32, smithay::utils::Physical>>| {
            control
                .filter(|control| control.size.w > 0 && control.size.h > 0)
                .and_then(|control| {
                    crate::a_new_window_is_placed::onto_the_plane(control, camera.at(), zoom)
                })
        };
        let dock = on_the_plane(drawn.and_then(|drawn| drawn.dock_band));
        let panel = on_the_plane(drawn.map(|drawn| drawn.panel_reserved));
        crate::the_room_alo_places_a_window_in::the_room_the_controls_leave(view, dock, panel)
    }
}
