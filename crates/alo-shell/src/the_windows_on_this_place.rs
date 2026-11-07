//! **Every window on the Place being looked at**, on the plane: what a new
//! window of alo's is placed beside and never exactly on top of
//! (`docs/design/where-a-new-window-opens.md`).

use smithay::utils::{Logical, Rectangle};

impl crate::Server {
    /// The applications' windows on the Place being looked at, front first,
    /// then alo's terminal and Settings where they are on it.
    pub(crate) fn the_windows_on_this_place(&self) -> Vec<Rectangle<i32, Logical>> {
        let place = self.the_place_now();
        self.mapped_surfaces()
            .filter(|surface| crate::canvas_place::the_place_of(surface) == Some(place))
            .map(crate::where_a_window_is)
            .chain(self.where_the_terminal_is_on_the_plane())
            .chain(self.where_settings_is_on_the_plane())
            // **And the record**, a window on the canvas like Settings since
            // ADR 0145's fourth stage, so nothing opens over it unasked.
            .chain(self.where_the_record_is_on_the_plane())
            .collect()
    }
}
