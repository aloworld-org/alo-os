//! What the keyboard can reach on the Dock's surfaces, and what Enter does
//! with it.
//!
//! The router next door — [`crate::a_key_reaches_the_docks_surfaces`] — decides
//! *which key*. This file answers the two questions that need the server:
//! **how many items are there to move through**, and **what happens to the one
//! the keyboard is on.** Both need the windows there are now, and only the
//! server has those.
//!
//! # Why a file of its own
//!
//! Escape needs nothing from the server but its own state, which is why the
//! router could be written without this. Arrows and Enter need the lists, and a
//! list is built from `Server::the_windows_the_dock_sees` — so a router that
//! answered them too would be a file about keys and about windows at once.
//!
//! # Selecting is the keyboard's only way in today, and that is a gap
//!
//! A press on the popover does nothing: the band's hit test is
//! `crate::where_the_dock_is::TheDocksPlaces` and it covers the bar, not the
//! surface above it. So Enter here is the **first** way to choose anything from
//! either surface, and the pointer's way is owed. It is named here rather than
//! left to be discovered, because a drawn list whose rows do nothing on a click
//! is a promise `docs/decisions/0103-*` already made.

use crate::Server;

impl Server {
    /// How many items the open surface holds, and how many sit on one row.
    ///
    /// `(0, _)` with nothing open, and with an open surface that holds nothing.
    /// The arrow rule treats both the same way — it returns the key — so the
    /// caller does not have to tell them apart.
    ///
    /// **The same lists the draw uses, through the same methods.** The overview
    /// is `Self::what_more_apps_holds` and the rows are the picker
    /// `Self::what_the_dock_has_open` built; a count worked out here from the
    /// windows directly would be a second answer to a question that already has
    /// one, and the keyboard would move through a list nobody had drawn.
    pub(crate) fn what_the_keyboard_moves_through(&self) -> (usize, usize) {
        match self.what_the_dock_has_open() {
            crate::WhatTheDockHasOpen::Nothing => (0, 1),
            crate::WhatTheDockHasOpen::MoreApps { .. } => (
                // **This person's list, which is the one the draw asks for.**
                // Both read `Server::what_this_persons_more_apps_holds`, so the
                // arrows cannot come to move through a grid nobody drew. They
                // each built a `Holding::nothing()` until the pins became real,
                // and the comment that stood here predicted this commit.
                self.what_this_persons_more_apps_holds().len(),
                alo_dock::more_apps::TheOverview::how_many_across(),
            ),
            // **A list, so one across.** Up and Down then move by one and Left
            // and Right by one as well, which is the right answer for a column
            // of rows: a person pressing Right on a list expects the next
            // thing, not nothing.
            crate::WhatTheDockHasOpen::TheWindowsOf { picker, .. } => (picker.rows().len(), 1),
        }
    }

    /// Do to the focused item what pressing it would do, and close up.
    ///
    /// Nothing happens when nothing is focused, which is the state a person is
    /// in until they press an arrow.
    ///
    /// # What each surface's Enter means
    ///
    /// - **the overview**: the same as pressing that application's icon, by
    ///   going through `Self::go_to_this_application`. So an application with
    ///   several windows opens its window list rather than guessing — one rule,
    ///   whichever way a person reached the icon.
    /// - **a window list**: that window, brought back if it was put aside and
    ///   then focused. This is `ADR 0103`'s *the person says which*, and it is
    ///   the end of the road: the surfaces close.
    pub(crate) fn the_keyboard_takes_what_is_focused(&mut self) {
        let Some(which) = self.the_dock_was_asked_for.focused() else {
            return;
        };
        self.choose_from_the_open_surface(which);
    }

    /// Do to the `which`th item of whichever surface is open what choosing it
    /// means.
    ///
    /// **One performer for the keyboard and the pointer.** Enter and a press on
    /// a row are the same act and this is the only place that carries it out.
    /// `crate::window_control_input` records what the other choice costs: when
    /// only the keyboard was migrated to a new mechanism, the button stayed
    /// broken while a keyboard test passed — *a fix down the path nobody uses*.
    ///
    /// `which` indexes the whole open list, counting the overview's pinned
    /// group first. Out of range does nothing, which is an ordinary race: an
    /// application can close between the draw a person aimed at and the press
    /// or key that arrives.
    pub(crate) fn choose_from_the_open_surface(&mut self, which: usize) {
        match self.what_the_dock_has_open() {
            crate::WhatTheDockHasOpen::Nothing => {}
            crate::WhatTheDockHasOpen::MoreApps { .. } => {
                // **The row says which kind it is**, so Enter on a native
                // entry does what a press on it does and not what a press on an
                // application does. Keyboard and pointer reaching two different
                // handlers for one row is the fault this whole file exists to
                // prevent.
                let Some(chosen) = self
                    .what_this_persons_more_apps_holds()
                    .get(which)
                    .map(|on| on.what().clone())
                else {
                    return;
                };
                match chosen {
                    alo_dock::ADockEntry::Application(app) => self.go_to_this_application(&app),
                    alo_dock::ADockEntry::Native(action) => self.do_this_native_action(action),
                }
            }
            crate::WhatTheDockHasOpen::TheWindowsOf { picker, .. } => {
                let Some(row) = picker.rows().get(which) else {
                    return;
                };
                let (id, put_aside) = (row.id(), row.sits().was_put_aside());
                // Closed before the window is reached, not after, for
                // `Self::go_to_this_application`'s reason: a refusal below must
                // not leave a list open over the window a person just asked
                // for.
                self.the_overflow_closes();
                // **Looked up now.** Ids are the window list's own indices and
                // a window closing renumbers the rest, so the surface the
                // picker named at the last draw may be a different window by
                // the time a key lands.
                let Some(surface) = self.the_window_numbered(id).cloned() else {
                    return;
                };
                if put_aside && self.set_window_minimized(&surface, false).is_err() {
                    // A window that will not come back is not then focused:
                    // the keyboard would be somewhere a person cannot see.
                    return;
                }
                let _ = self.activate_window(&surface);
            }
        }
    }
}

#[cfg(test)]
#[path = "the_keyboard_on_the_docks_surfaces_tests.rs"]
mod tests;
