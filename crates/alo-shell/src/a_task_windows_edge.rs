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
//! are the design's, scaled. A press is read against the edge laid out at the
//! ordinary scale, which is what `crate::a_press_on_an_edge` does for every
//! client window's edge.

use smithay::utils::{Logical, Physical, Point, Rectangle, Size};

use crate::Server;
use crate::the_tasks_a_person_opened::WhichTask;
use crate::window_edge::{Decorations, OnTheEdge, WindowEdge, edge_of};
use crate::window_edge_picture::{EdgePicture, OverTheEdge};
use crate::window_edge_pointing::{UnderThePointer, pointing_at, what_is_under};

/// The ordinary scale, in per cent.
const ORDINARY: u32 = 100;

/// A task window's pixels in the units the edge is laid out in at this scale.
fn in_the_edges_units(at: Rectangle<i32, Physical>, scale: u32) -> Rectangle<i32, Logical> {
    let down = |length: i32| {
        i32::try_from(i64::from(length) * i64::from(ORDINARY) / i64::from(scale.max(1)))
            .unwrap_or(length)
    };
    Rectangle::new(
        (down(at.loc.x), down(at.loc.y)).into(),
        (down(at.size.w), down(at.size.h)).into(),
    )
}

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
            let edge = the_edge_of_a_task_window(in_the_edges_units(window, scale), room);
            let pointing = pointing_at(&edge, pointer, None);
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
    /// A press at `at`, offered to the task windows' edges, newest first.
    ///
    /// Answers whether one took it. **Anywhere on a task window's edge takes
    /// the press**, Close or not: the edge is drawn over whatever is behind
    /// it, so a press on it was never a press on that. Only Close does
    /// anything.
    pub(crate) fn a_press_on_a_task_windows_edge(&mut self, at: Point<i32, Logical>) -> bool {
        let room = self.the_room_an_edge_is_clamped_in();
        let open: Vec<(WhichTask, Rectangle<i32, Physical>)> = self.tasks().each_open_window();
        for (which, window) in open.into_iter().rev() {
            let edge = the_edge_of_a_task_window(in_the_edges_units(window, ORDINARY), room);
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
