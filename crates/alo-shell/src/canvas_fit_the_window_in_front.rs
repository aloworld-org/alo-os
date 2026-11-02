//! **Fit the selected window in view** — `⊞`+2.
//!
//! `docs/design/the-shortcuts-and-the-edges.md`, *Moving around the canvas*:
//! *Fit the selected window in view — double-click its name — `⊞` + 2*, and
//! `docs/design/the-canvas-as-a-workspace.md` §4 names *Fit selection: frame the
//! selected window or windows* among the essential zoom commands. Nothing did it
//! until 2026-11-01, while a status note recorded it as done. This makes an
//! existing promise true rather than adding one.
//!
//! # Which window is *the selected one*
//!
//! **The window in front**, because this canvas has no selection yet: selecting
//! several windows is its own promise, unbuilt. The window in front is the one
//! the person is working in — the one the keyboard reaches — and it is the
//! selection of one that every road here already agrees on
//! (`crate::which_window_is_in_front`). When selecting several lands, this
//! fits the selection instead.
//!
//! A task window is on the canvas since ADR 0145's second stage, so with one in
//! front it is the one fitted.
//!
//! # The same fit as *Show all*, of one window
//!
//! Into the room the fixed controls leave clear, with the same margin, through
//! `Server::look_at_these_frames`. So *fit the window* and *show all* agree
//! about what *in view* means, and one window fitted is the same picture as
//! *Show all* with only that window open.
//!
//! The double-click on the window's name is the design's pointer road. It needs
//! the name band the window-look lane is building, and is not here.

use alo_canvas::{Camera, Frame};

use crate::which_window_is_in_front::AKindOfWindow;

impl crate::Server {
    /// The window in front, as a frame on the canvas, if it is on one.
    #[must_use]
    pub(crate) fn the_window_in_front_as_a_frame(&self) -> Option<Frame> {
        match self.the_kind_in_front() {
            AKindOfWindow::TheTerminal => self.the_terminal_as_a_frame(),
            AKindOfWindow::Applications => {
                let place = self.the_place_now();
                let front = self
                    .mapped_surfaces()
                    .find(|surface| crate::canvas_place::the_place_of(surface) == Some(place))?;
                let number = crate::window_number::Numbers::of(front);
                self.the_frames_on_the_plane()
                    .into_iter()
                    .find(|frame| frame.id() == number)
            }
            AKindOfWindow::TaskWindows => self.the_task_window_in_front_as_a_frame(),
        }
    }

    /// Fit the window in front in view, and look at it.
    ///
    /// # Errors
    /// [`None`] with no window in front on the canvas, or no output to fit it
    /// into — and the camera is left where it was.
    pub fn fit_the_window_in_front_in_view(&mut self) -> Option<Camera> {
        let frame = self.the_window_in_front_as_a_frame()?;
        self.look_at_these_frames(&[frame])
    }
}
