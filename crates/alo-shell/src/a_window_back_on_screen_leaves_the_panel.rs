//! **A window that is back on the screen leaves the panel**, whichever road
//! brought it back.
//!
//! Measured on a booted machine on 2026-11-11 (the walk of `57b98c15`,
//! `w21-15`): a person minimised Files, chose Files again, and its window was
//! on the screen while its card was still in the panel. A card is the panel's
//! promise that a window is put aside (`docs/design/the-windows-put-aside.md`:
//! selecting a card *takes that item out of the panel*), and one standing for a
//! window that is not aside is a card for nothing.
//!
//! # Why every road, and not each road remembering
//!
//! Only a click on a card went through `Server::bring_this_window_back`, the
//! one method that empties the panel. The Dock, its keyboard, both window
//! chords and the alo Bar restore with `Server::set_window_minimized` through
//! `Server::go_to_this_application`, and a client may un-minimise its own
//! window. Each could be taught to empty the panel — and the next road would
//! be the one that forgot. So the panel is reconciled with what is hidden,
//! once a frame, where the desktop already meets it: **a card stays exactly
//! while its window is among the hidden ones**. A window that closed while it
//! was aside leaves too, for the same reason.

use alo_dock::window::WindowId;
use alo_put_aside::Panel;

impl crate::Server {
    /// Take out of `panel` every card whose window is no longer put aside —
    /// back on the screen by some other road, or gone — and answer how many.
    ///
    /// `None`, a desktop with no panel, has nothing to take.
    pub fn the_panel_lets_go_of_windows_no_longer_aside(&self, panel: Option<&mut Panel>) -> usize {
        let Some(panel) = panel else {
            return 0;
        };
        let back: Vec<WindowId> = panel
            .previews()
            .iter()
            .map(alo_put_aside::Preview::window)
            .filter(|id| self.the_put_aside_window_numbered(*id).is_none())
            .collect();
        back.iter()
            .filter(|id| panel.bring_back(**id).is_ok())
            .count()
    }
}
