//! What a pointer is on, when it is on a window's edge.
//!
//! `crate::window_edge` says where everything is, `crate::window_edge_paint`
//! says what colour it is in each state, and `crate::window_edge_reading`
//! says what each control does. This answers the one question between them:
//! **a pointer is at this position — what is under it?**
//!
//! # Why it is a file of its own
//!
//! Until now nothing asked. `EdgeControl::target` has carried the sentence
//! *the 44 × 44 it answers in* since the layout was written, and
//! `window_edge_reading::what_it_does` has mapped every control to an
//! [`alo_shortcuts::Action`] — but no code turned a position into a control,
//! so the only caller in this repository passed
//! `Pointing::default()` and every window's edge was a picture. It drew
//! correctly, announced itself correctly to a reader, and could not be
//! clicked.
//!
//! So this is the hit test, and it is its own file because it is its own
//! reason to change: the design recolouring a control is
//! `window_edge_paint`'s business, the design moving one is
//! `window_edge`'s, and the design changing **what a press lands on** is
//! this one's.
//!
//! # The target answers, not the highlight
//!
//! A control's target is 44 × 44 and the pill drawn behind it while hovered is
//! 32 × 28 at `left 6, top 14`. Those are different rectangles on purpose
//! (`crate::window_edge::A_HIGHLIGHT_IS` says so in as many words), and this
//! asks the **target**. A person aiming at the lower-left of Close's icon is
//! inside the target and outside the pill, and a hit test on what is drawn
//! would quietly ignore them — it is the kind of fault that feels like a bad
//! mouse rather than like a bug.
//!
//! # Where nothing is
//!
//! Three places on an edge are deliberately not a control and not a drag:
//!
//! - the four units between one target and the next
//!   (`crate::window_edge::BETWEEN_TARGETS`), and the four after the last;
//! - the first eight units of the leading edge, before the drag region starts
//!   (`crate::window_edge::THE_DRAG_STARTS_AT`);
//! - at rest, everything above the shallow band the grip sits in — the region
//!   is the whole 44 so the edge reveals, but the band that answers a press is
//!   12 tall.
//!
//! Each of those answers [`UnderThePointer::TheEdgeAndNothingOnIt`] rather
//! than [`None`], because the two mean different things to a host: inside the
//! region the edge stays revealed, outside it the edge conceals.

use smithay::utils::{Logical, Point};

use crate::window_edge::{OnTheEdge, WindowEdge};
use crate::window_edge_paint::Pointing;

/// What a pointer is over, on an edge it is over at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnderThePointer {
    /// One of the edge's controls, which a press would act on.
    Control(OnTheEdge),
    /// The drag region, where a press begins moving the window.
    Drag,
    /// On the edge's region, but on nothing that answers a press.
    ///
    /// Still on the edge: this is what keeps it revealed.
    TheEdgeAndNothingOnIt,
}

/// What is under `pointer` on this edge, or [`None`] if it is not on the edge.
///
/// [`None`] is the answer a host conceals a revealed edge on, and every other
/// answer keeps it revealed — including
/// [`UnderThePointer::TheEdgeAndNothingOnIt`].
///
/// **A control wins over the drag region**, and that order is not a tiebreak
/// so much as a guard: `crate::window_edge` places the drag so it never
/// reaches under a control, and the test `a_control_is_never_also_the_drag`
/// holds it to that. If the layout ever changed, this order means a person
/// gets the button they aimed at rather than a window that jumps.
#[must_use]
pub fn what_is_under(edge: &WindowEdge, pointer: Point<i32, Logical>) -> Option<UnderThePointer> {
    if !edge.region.contains(pointer) {
        return None;
    }
    if let Some(control) = edge
        .controls
        .iter()
        .find(|one| one.target.contains(pointer))
    {
        return Some(UnderThePointer::Control(control.does));
    }
    if edge.drag.contains(pointer) {
        return Some(UnderThePointer::Drag);
    }
    Some(UnderThePointer::TheEdgeAndNothingOnIt)
}

/// The [`Pointing`] this edge should be drawn with, for a pointer here.
///
/// The value [`crate::EdgePicture::of`] takes, assembled rather than left to
/// each caller — there is one correct way to turn a position and a keyboard
/// focus into it, and a second copy of that reasoning in a host is how one
/// machine ends up highlighting a different control from another.
///
/// `focused` is passed through untouched. Keyboard focus is not a thing a
/// pointer position can answer, and guessing it from one would make a focus
/// outline follow the mouse.
#[must_use]
pub fn pointing_at(
    edge: &WindowEdge,
    pointer: Option<Point<i32, Logical>>,
    focused: Option<OnTheEdge>,
) -> Pointing {
    let hovered = pointer
        .and_then(|at| what_is_under(edge, at))
        .and_then(|under| match under {
            UnderThePointer::Control(does) => Some(does),
            UnderThePointer::Drag | UnderThePointer::TheEdgeAndNothingOnIt => None,
        });
    Pointing { hovered, focused }
}

#[cfg(test)]
#[path = "window_edge_pointing_tests.rs"]
mod tests;
