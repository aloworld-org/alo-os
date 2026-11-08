//! **Every window on the Place being looked at**, on the plane: what a new
//! window of alo's is placed beside and never exactly on top of
//! (`docs/design/where-a-new-window-opens.md`).

use smithay::utils::{Logical, Rectangle};

impl crate::Server {
    /// The applications' windows on the Place being looked at, front first,
    /// then alo's terminal, Settings, the record and the task windows where
    /// they are on it.
    ///
    /// **The task windows since 2026-11-08.** Until then this list stopped at
    /// the record, so the terminal, Settings or the record, opened over a view
    /// a task window filled, found no window in the way and opened on top of
    /// it — and the design promises a window the person opens goes to empty
    /// space *without covering any window*. A task window placing itself does
    /// not mind meeting itself here: the one being placed has no position yet.
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
            .chain(
                self.the_task_windows_on_the_plane()
                    .into_iter()
                    .map(|(_, window)| {
                        Rectangle::new(
                            (window.loc.x, window.loc.y).into(),
                            (window.size.w, window.size.h).into(),
                        )
                    }),
            )
            .collect()
    }
}

#[cfg(test)]
#[path = "the_windows_on_this_place_tests.rs"]
mod tests;
