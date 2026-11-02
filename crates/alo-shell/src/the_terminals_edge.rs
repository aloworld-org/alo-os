//! **alo's terminal's window edge**: the strip above it with its icon, its
//! title and Close, in the focused or unfocused look — the edge every
//! application's window has.
//!
//! # What the design draws, and what is drawn
//!
//! The approved frame `557:148`, *Window · Terminal · not focused*, puts the
//! same edge over the terminal as over Dolphin (`554:56`): a 32-tall strip in
//! a 44 band, the icon at (14, 20), *Terminal* at (38, 19), and Minimise,
//! Maximise and Close on the trailing walk. A walk on a booted machine
//! (`8263ddcb`, 2026-11-01) showed the terminal on the canvas with none of it.
//!
//! **Drawn:** the strip, the icon (`crate::the_terminals_icon`), the title, and
//! Close, in the focused look while the terminal is the window the keys go to
//! and the unfocused one otherwise (`crate::window_edge_look`).
//!
//! **Not drawn, and why:** Minimise and Maximise. Nothing can minimise or
//! maximise alo's terminal yet, and a control that reaches nothing looks live
//! and is not — the rule the task windows' edge already keeps
//! (`window_edge::Decorations::AlosOwnWindow`).
//!
//! # The application edge, reused
//!
//! Laid out by `crate::window_edge::edge_of`, painted by
//! `crate::window_edge_picture::EdgePicture`, pressed through
//! `crate::window_edge_pointing` — the very code every application's edge goes
//! through. **The window it is laid out from is where the camera draws the
//! terminal** (`Server::where_the_terminal_is`, worked out each frame by
//! `crate::the_terminal_on_the_plane::on_the_screen`), so the edge pans and
//! zooms with it, as #1084 made every application's do. Its title and
//! controls keep their size round a window of whatever size the zoom makes it.
//!
//! # Always revealed
//!
//! The owner's answer of 2026-11-01 shows every window's title all the time.
//! The person's *show the title only when I point at it* hides an
//! application's strip until it is pointed at; this edge is laid out revealed
//! whatever that setting says, because it carries the only Close this window
//! has, as a task window's does. *A reading, recorded so it can be disagreed
//! with* in `docs/design/the-terminal-window.md`.
//!
//! # A press on Close
//!
//! `Server::the_person_closes_the_terminal` (`crate::closing_the_terminal`):
//! at once when the shell is idle, after a question when a command is running.

use smithay::utils::{Logical, Point, Size};

use crate::Server;
use crate::an_edge_in_its_displays_units::{
    a_point_in_the_edges_units, a_window_in_the_edges_units,
};
use crate::window_edge::{Decorations, OnTheEdge, WindowEdge, edge_of};
use crate::window_edge_picture::{EdgePicture, OverTheEdge};
use crate::window_edge_pointing::{UnderThePointer, pointing_at, what_is_under};

impl Server {
    /// The terminal's edge, laid out at `scale` per cent in `room`, where the
    /// terminal is drawn now. [`None`] with no terminal drawn.
    fn the_terminals_edge(&self, room: Size<i32, Logical>, scale: u32) -> Option<WindowEdge> {
        self.alos_own_terminal.as_ref()?;
        let window = self.where_the_terminal_is?;
        Some(edge_of(
            a_window_in_the_edges_units(window, scale),
            Decorations::AlosOwnWindow,
            true,
            room,
        ))
    }

    /// The terminal's edge as a picture for this display: `output` its size in
    /// pixels and `scale` its scale in per cent, as
    /// `crate::every_windows_edge` takes them.
    pub(crate) fn the_terminals_edge_picture(
        &self,
        fonts: &mut cosmic_text::FontSystem,
        says: &alo_strings::Strings,
        output: (i32, i32),
        scale: u32,
    ) -> Option<EdgePicture> {
        let room = crate::window_edge::the_usable_output(Size::from(output), scale);
        let edge = self.the_terminals_edge(room, scale)?;
        let pointer = self.where_the_pointer_is_in_logical_pixels();
        let pointing = pointing_at(
            &edge,
            pointer.map(|at| a_point_in_the_edges_units(at, scale)),
            None,
        );
        let title = says.say(
            &alo_terminal::words::TERMINAL.key(),
            &alo_strings::Filling::nothing(),
        );
        let icon = crate::the_terminals_icon::the_terminals_icon(
            crate::the_terminals_icon::across_at(scale),
        );
        EdgePicture::of(
            &edge,
            pointing,
            OverTheEdge {
                menu: None,
                showing: None,
            },
            // **No body from the edge**: the terminal draws its own panel,
            // hairline and shadow (`crate::the_terminals_frame`), in the same
            // look as this strip.
            crate::window_edge_picture::TheWindow {
                title: Some(title.text()),
                look: crate::window_edge_look::WhichLook::of(
                    self.the_terminal_is_taking_the_keys(),
                ),
                icon: icon.as_ref(),
                body: None,
            },
            fonts,
            output,
            scale,
        )
    }

    /// Whether the terminal's edge is under `at`, in the display's pixels,
    /// without pressing it.
    pub(crate) fn the_terminals_edge_is_under(&self, at: Point<i32, Logical>) -> bool {
        let scale = self.the_scale_an_edge_is_measured_at();
        self.the_terminals_edge(self.the_room_an_edge_is_clamped_in(), scale)
            .is_some_and(|edge| {
                what_is_under(&edge, a_point_in_the_edges_units(at, scale)).is_some()
            })
    }

    /// A press at `at`, in the display's pixels, offered to the terminal's
    /// edge. Answers whether the edge took it.
    ///
    /// **Anywhere on the edge takes the press**, as on a task window's: the
    /// strip is drawn over whatever is behind it. Only Close does anything, and
    /// a press anywhere on the edge brings the terminal in front.
    pub(crate) fn a_press_on_the_terminals_edge(&mut self, at: Point<i32, Logical>) -> bool {
        let scale = self.the_scale_an_edge_is_measured_at();
        let Some(edge) = self.the_terminals_edge(self.the_room_an_edge_is_clamped_in(), scale)
        else {
            return false;
        };
        match what_is_under(&edge, a_point_in_the_edges_units(at, scale)) {
            None => false,
            Some(UnderThePointer::Control(OnTheEdge::Close)) => {
                let _ = self.the_person_closes_the_terminal();
                true
            }
            Some(
                UnderThePointer::Control(_)
                | UnderThePointer::Drag
                | UnderThePointer::TheEdgeAndNothingOnIt,
            ) => {
                self.the_composer.it_gave_the_keyboard_up();
                self.the_keyboard_is_now_in(None);
                self.the_person_went_to(
                    crate::which_window_is_in_front::AKindOfWindow::TheTerminal,
                );
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
#[path = "the_terminals_edge_tests.rs"]
mod tests;
