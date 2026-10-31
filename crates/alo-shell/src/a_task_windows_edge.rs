//! A task window's edge: the design's external window edge above it, carrying
//! **Close** and nothing else.
//!
//! The design draws every task window with the same `External window edge`
//! client windows have — 44 tall, above the window, its controls 44 × 44 at the
//! trailing end (`docs/design/the-ask-alo-task-window.md`). **The owner decided
//! on 2026-10-28 to draw Close now**, and minimise and maximise later with the
//! right-hand panel. So this lays the edge out through `crate::window_edge`
//! itself, with `Decorations::ATaskWindow`, and paints it through
//! `crate::window_edge_picture` — one edge, one set of measures, scaled at the
//! one boundary every other edge is.
//!
//! # Always revealed
//!
//! A client window's edge rests concealed and appears under the pointer. This
//! one is laid out revealed: the design's task frames draw it so, and a Close
//! that appears only under a pointer is one a person must hunt for on the one
//! window that cannot otherwise be closed with the pointer. There is nothing
//! to drag on it, because a task window does not move.
//!
//! # A press on Close follows Alt+F4's rules
//!
//! It is `Server::the_person_closes_a_task`, the same door the keyboard uses:
//! a finished task closes, a running one does not yet, and a change waiting is
//! never answered.
//!
//! # Scale
//!
//! A task window is placed and drawn with logical and physical as one number
//! (`crate::where_a_task_window_opens`). The edge is laid out from the window's
//! pixels divided by the display's scale, then scaled back by
//! `EdgePicture::of`, so it sits on the window at any scale and its own figures
//! are the design's, scaled. A press and the pointer's highlight are read the
//! same way: the pointer divided by the scale and tested against the edge laid
//! out from the divided window, which is `crate::an_edge_in_its_displays_units`.
//! Until 2026-10-28 a press was read against the edge laid out at 100, so on a
//! 200 per cent display Close was drawn where a press did not find it.

use smithay::utils::{Logical, Physical, Point, Rectangle, Size};

use crate::Server;
use crate::an_edge_in_its_displays_units::{
    a_point_in_the_edges_units, a_window_in_the_edges_units,
};
use crate::the_tasks_a_person_opened::WhichTask;
use crate::window_edge::{Decorations, OnTheEdge, WindowEdge, edge_of};
use crate::window_edge_picture::{EdgePicture, OverTheEdge};
use crate::window_edge_pointing::{UnderThePointer, pointing_at, what_is_under};

/// The edge above one task window, revealed, in `room`.
#[must_use]
pub(crate) fn the_edge_of_a_task_window(
    window: Rectangle<i32, Logical>,
    room: Size<i32, Logical>,
) -> WindowEdge {
    edge_of(window, Decorations::ATaskWindow, true, room)
}

/// Every open task window's edge, as pictures for this display.
///
/// `output` is the display's size in pixels and `scale` its scale in per
/// cent, exactly as `crate::every_windows_edge` takes them.
pub(crate) fn every_task_windows_edge(
    server: &Server,
    fonts: &mut cosmic_text::FontSystem,
    output: (i32, i32),
    scale: u32,
) -> Vec<EdgePicture> {
    let room = crate::window_edge::the_usable_output(Size::from(output), scale);
    let pointer = server.where_the_pointer_is_in_logical_pixels();
    server
        .tasks()
        .each_open_window()
        .into_iter()
        .filter_map(|(_, window)| {
            let edge = the_edge_of_a_task_window(a_window_in_the_edges_units(window, scale), room);
            let pointing = pointing_at(
                &edge,
                pointer.map(|at| a_point_in_the_edges_units(at, scale)),
                None,
            );
            EdgePicture::of(
                &edge,
                pointing,
                OverTheEdge {
                    menu: None,
                    showing: None,
                },
                None,
                fonts,
                output,
                scale,
            )
        })
        .collect()
}

impl Server {
    /// Whether any task window's edge is under `at`, in the units
    /// [`Self::a_press_on_a_task_windows_edge`] takes, without pressing it.
    pub(crate) fn a_task_windows_edge_is_under(&self, at: Point<i32, Logical>) -> bool {
        let room = self.the_room_an_edge_is_clamped_in();
        let scale = self.the_scale_an_edge_is_measured_at();
        let at = a_point_in_the_edges_units(at, scale);
        self.tasks()
            .each_open_window()
            .into_iter()
            .any(|(_, window)| {
                let edge =
                    the_edge_of_a_task_window(a_window_in_the_edges_units(window, scale), room);
                what_is_under(&edge, at).is_some()
            })
    }

    /// A press at `at`, offered to the task windows' edges, newest first.
    ///
    /// Answers whether one took it. **Anywhere on a task window's edge takes
    /// the press**, Close or not: the edge is drawn over whatever is behind
    /// it, so a press on it was never a press on that. Only Close does
    /// anything.
    pub(crate) fn a_press_on_a_task_windows_edge(&mut self, at: Point<i32, Logical>) -> bool {
        let room = self.the_room_an_edge_is_clamped_in();
        let scale = self.the_scale_an_edge_is_measured_at();
        let at = a_point_in_the_edges_units(at, scale);
        let open: Vec<(WhichTask, Rectangle<i32, Physical>)> = self.tasks().each_open_window();
        for (which, window) in open.into_iter().rev() {
            let edge = the_edge_of_a_task_window(a_window_in_the_edges_units(window, scale), room);
            match what_is_under(&edge, at) {
                None => {}
                Some(UnderThePointer::Control(OnTheEdge::Close)) => {
                    let _ = self.the_person_closes_a_task(which);
                    return true;
                }
                Some(
                    UnderThePointer::Control(_)
                    | UnderThePointer::Drag
                    | UnderThePointer::TheEdgeAndNothingOnIt,
                ) => return true,
            }
        }
        false
    }
}

#[cfg(test)]
#[path = "a_task_windows_edge_tests.rs"]
mod tests;
