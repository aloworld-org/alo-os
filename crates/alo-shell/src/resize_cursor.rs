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
//! than the tip**, unlike `crate::default_cursor`'s arrow. An arrow that points
//! both ways has no tip to put it at, and a person aims the crossing point of one
//! at the edge they mean.
//!
//! # Drawn here, in the same shape as the arrow beside it
//!
//! A mask of rows, `B` for the outline, `W` for the interior, a space for
//! nothing — `crate::default_cursor`'s own form, so the two are read the same way
//! by whoever meets them next. Compositor-owned, no theme read from disk and no
//! client resource: the cursor over a frame's edge is the shell's answer about
//! the shell's own band, and asking a theme for it would make what the border
//! says depend on what somebody installed.

use crate::FrameEdge;

/// A left-and-right arrow, for the left and right edges.
const ACROSS: [&str; 7] = [
    "   B     B   ",
    "  BB     BB  ",
    " BWBBBBBBBWB ",
    "BWWWWWWWWWWWB",
    " BWBBBBBBBWB ",
    "  BB     BB  ",
    "   B     B   ",
];

/// An up-and-down arrow, for the top and bottom edges.
///
/// [`ACROSS`] turned a quarter, written out rather than transposed at runtime so
/// that what is drawn can be read here.
const DOWN: [&str; 13] = [
    "   B   ", "  BWB  ", " BWWWB ", "BBWWWBB", "  BWB  ", "  BWB  ", "  BWB  ", "  BWB  ",
    "  BWB  ", "BBWWWBB", " BWWWB ", "  BWB  ", "   B   ",
];

/// A top-left-to-bottom-right arrow, for those two corners.
///
/// **Symmetric by construction rather than by eye.** The first draft of this was
/// drawn by hand and `every_arrow_points_both_ways` caught it: one end was heavier
/// than the other, which tells somebody an edge moves one way. One half is drawn
/// and the other is that half turned half a turn, so the two ends cannot differ.
const FALLING: [&str; 13] = [
    "BBBBBBB",
    "BWWWWWB",
    "BWWWWBB",
    "BWWWBBB",
    "BWWBWWBB",
    "BWBBBWWBB",
    "BBB BWWWB BBB",
    "    BBWWBBBWB",
    "     BBWWBWWB",
    "      BBBWWWB",
    "      BBWWWWB",
    "      BWWWWWB",
    "      BBBBBBB",
];

/// A top-right-to-bottom-left arrow, for the other two.
///
/// [`FALLING`] mirrored, which keeps the two diagonals the same weight as each
/// other as well as at both of their own ends.
const RISING: [&str; 13] = [
    "      BBBBBBB",
    "      BWWWWWB",
    "      BBWWWWB",
    "      BBBWWWB",
    "     BBWWBWWB",
    "    BBWWBBBWB",
    "BBB BWWWB BBB",
    "BWBBBWWBB",
    "BWWBWWBB",
    "BWWWBBB",
    "BWWWWBB",
    "BWWWWWB",
    "BBBBBBB",
];

/// What a pointer shows over this edge or corner.
///
/// Four answers for eight questions, which is what a double-headed arrow is for:
/// the shape says which way the thing under it moves, and both ends of it are the
/// same claim.
#[must_use]
pub fn the_cursor_over(edge: FrameEdge) -> &'static [&'static str] {
    match edge {
        FrameEdge::Left | FrameEdge::Right => &ACROSS,
        FrameEdge::Top | FrameEdge::Bottom => &DOWN,
        FrameEdge::TopLeft | FrameEdge::BottomRight => &FALLING,
        FrameEdge::TopRight | FrameEdge::BottomLeft => &RISING,
    }
}

/// Where the pointer actually is within that shape.
///
/// **The middle, not the tip**, which ADR 0071 names as the difference from the
/// arrow: an arrow pointing both ways has no tip to put a hotspot at, and the
/// crossing point is what a person aims at an edge.
///
/// [`None`] for a shape with no rows, which cannot happen for the four above and
/// is answered rather than unwrapped because a cursor is drawn every frame.
#[must_use]
pub fn the_hotspot_of(shape: &[&str]) -> Option<(i32, i32)> {
    let height = i32::try_from(shape.len()).ok()?;
    let width = i32::try_from(shape.iter().map(|row| row.chars().count()).max()?).ok()?;
    Some((width / 2, height / 2))
}

#[cfg(test)]
#[path = "resize_cursor_tests.rs"]
mod tests;
