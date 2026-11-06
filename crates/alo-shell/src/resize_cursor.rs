//! **The double-headed arrow**, which is what makes ADR 0071's decision real.
//!
//! That ADR settled that the edges and corners resize a frame and the name moves
//! it, and then said the part that matters: *the pointer is the affordance*.
//! Without it, *the edge resizes and the name moves* is a rule somebody has to be
//! told and then remember, on a frame that deliberately shows them nothing — a
//! rule nobody can see is one they discover by resizing a window they meant to
//! move. With it, a frame keeps ADR 0065's promise of no furniture at all and
//! still answers *what does this part do* the moment a pointer arrives.
//!
//! # Four shapes, eight directions, and the hotspot in the middle
//!
//! A double-headed arrow means the same thing at both ends, so the top edge and
//! the bottom edge share one shape and the two diagonals cover four corners. ADR
//! 0071 says so and says the other half too: **the hotspot is the middle rather
//! than the tip**, because an arrow that points both ways has no tip to put it
//! at.
//!
//! # Named here, drawn where every other cursor is
//!
//! Until 2026-11-06 these were four masks of black and white pixels written in
//! this file. They are now the cursor names any application would ask for —
//! `ew-resize`, `ns-resize`, `nwse-resize`, `nesw-resize` — drawn by
//! `alo_cursors` from the same vector source as the arrow, so the shell's own
//! edge and an application's resize handle show the same pointer.

use crate::FrameEdge;
use alo_cursors::CursorIcon;

/// What a pointer shows over this edge or corner.
///
/// Four answers for eight questions, which is what a double-headed arrow is for:
/// the shape says which way the thing under it moves, and both ends of it are the
/// same claim. Its hotspot is its middle, which `alo_cursors` holds for each.
#[must_use]
pub const fn the_cursor_over(edge: FrameEdge) -> CursorIcon {
    match edge {
        FrameEdge::Left | FrameEdge::Right => CursorIcon::EwResize,
        FrameEdge::Top | FrameEdge::Bottom => CursorIcon::NsResize,
        FrameEdge::TopLeft | FrameEdge::BottomRight => CursorIcon::NwseResize,
        FrameEdge::TopRight | FrameEdge::BottomLeft => CursorIcon::NeswResize,
    }
}

#[cfg(test)]
#[path = "resize_cursor_tests.rs"]
mod tests;
