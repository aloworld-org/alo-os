//! **alo's own windows open in the room the Dock leaves**, as an application's
//! window does: Settings, the record and alo's terminal.
//!
//! `docs/design/where-a-new-window-opens.md`, amended 2026-11-12: *a window alo
//! places lands wholly in the room the Dock leaves*; amended 2026-11-01:
//! *alo's own windows count as windows*; and 2026-11-07, the owner on whether
//! they follow the same rule: *"Yes, the same"*.
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
//! the view down to the Dock's top — and kept wholly inside it, moved the least
//! it must be, before the full view's rule (`crate::room_in_a_full_view`) is
//! asked, exactly as `Server::where_this_window_goes` does for an application.
//!
//! **What this does not do is make Settings smaller.** Its size is the design's
//! rule of the display (`docs/design/the-settings-window.md`: at most
//! 1200 × 816, never nearer the screen's edge than 64 across and 56 down), and
//! on the design's own 1 280 × 800 frame that is 688 tall under a 54 band —
//! more than the 717 above the walk's Dock, with the gap. There it is kept with
//! its band at the top of the room and its foot still under the Dock; making it
//! fit is a change to the design's size rule, and the owner's to make.

use smithay::utils::{Logical, Rectangle};

impl crate::Server {
    /// **The room one of alo's own windows opens in**: `view` — what the
    /// person will be looking at once any glide ends — down to the Dock's top,
    /// where the Dock was last drawn reaching into it.
    pub(crate) fn the_room_an_alo_window_opens_in(
        &self,
        view: Rectangle<i32, Logical>,
    ) -> Rectangle<i32, Logical> {
        let camera = self.the_camera_after_the_glide();
        let zoom = crate::scene::drawn_at(camera);
        let dock = self
            .the_controls_of_the_display_for(None)
            .and_then(crate::canvas_fixed_controls::FixedControls::labelled)
            .and_then(|drawn| drawn.dock_band)
            .filter(|band| band.size.w > 0 && band.size.h > 0)
            .and_then(|band| {
                crate::a_new_window_is_placed::onto_the_plane(band, camera.at(), zoom)
            });
        crate::the_room_alo_places_a_window_in::the_room_the_dock_leaves(view, dock)
    }
}
