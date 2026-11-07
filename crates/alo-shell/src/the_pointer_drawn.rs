//! **The pointer alo draws**, placed on one display at that display's scale.
//!
//! The picture is `alo_cursors`': one vector source for every shape, a navy
//! shape with a white keyline and a soft shadow (`docs/features.md`, *The alo
//! Cursor*, *one cursor, drawn by alo*). What this file adds is only where the
//! picture goes and how large it is on *this* display:
//!
//! - **Its size is a logical measure through the display's scale** —
//!   `alo_cursors::measures::pixels_at` — so a screen at 200 per cent draws the
//!   same pointer sharper rather than half as big (`CLAUDE.md`, *nothing is
//!   built to one screen size*). The 18-row mask this replaced was one size on
//!   every display.
//! - **Its hotspot lands on the pointer's position**: the arrow's tip, or a
//!   resize arrow's middle (ADR 0071), whichever the shape says.
//!
//! A client's own cursor surface is not drawn here; it is that client's
//! business, imported by `crate::scene_drawing` as it always was.

use crate::display_scale::DisplayScale;
use crate::{Cursor, RenderError};
use alo_cursors::{CursorIcon, Picture};
use smithay::utils::{Physical, Rectangle};

/// A pointer drawn and placed on one display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ThePointer {
    /// Where its square goes, in this display's pixels. It may reach past the
    /// display's edge, which the renderer clips.
    pub(crate) area: Rectangle<i32, Physical>,
    /// The picture: premultiplied RGBA, `area`'s size.
    pub(crate) picture: Picture,
}

/// Which of alo's shapes this cursor state is, or [`None`] for a state alo
/// draws nothing for: hidden, unpositioned, or a client's own surface.
pub(crate) const fn the_shape_of(cursor: &Cursor) -> Option<CursorIcon> {
    match cursor {
        Cursor::Arrow { .. } => Some(CursorIcon::Default),
        Cursor::Resize { edge, .. } => Some(crate::resize_cursor::the_cursor_over(*edge)),
        Cursor::Named { shape, .. } => Some(*shape),
        Cursor::Default | Cursor::Hidden | Cursor::Surface { .. } => None,
    }
}

/// The pointer for `cursor` on a display of `bounds` drawn at `scale`, in the
/// person's `look`, or
/// [`None`] when alo draws nothing: a state that is not alo's to draw, or a
/// pointer wholly off this display.
///
/// # Errors
/// Refuses a position that is not a number rather than drawing nothing: a
/// position like that is a fault upstream of the cursor, and a frame silently
/// missing its pointer would hide it. Also refuses a picture that could not be
/// drawn, which no shipped shape is.
pub(crate) fn the_pointer(
    cursor: &Cursor,
    scale: DisplayScale,
    look: crate::ThePointersLook,
    bounds: Rectangle<i32, Physical>,
) -> Result<Option<ThePointer>, RenderError> {
    let (Some(shape), Some(location)) = (the_shape_of(cursor), where_it_is(cursor)) else {
        return Ok(None);
    };
    if !location.0.is_finite() || !location.1.is_finite() {
        return Err(RenderError::Submission("non-finite cursor position".into()));
    }
    // **The display's scale times the size the person chose**, so a pointer made
    // twice as large on a screen at 200 per cent is four times the pixels.
    let per_cent = u32::from(scale.per_cent()) * u32::from(look.size.per_cent()) / 100;
    let size = alo_cursors::measures::pixels_at(u16::try_from(per_cent).unwrap_or(u16::MAX));
    // **The arrow the person chose**: the kite the owner picked on 2026-11-06,
    // unless they chose the familiar one in Settings (`crate::the_pointers_look`).
    let picture = alo_cursors::draw_in(shape, look.silhouette(), look.colour, size)
        .map_err(|why| RenderError::Submission(why.to_string()))?;
    let left = location.0.floor() - f64::from(picture.hotspot.0);
    let top = location.1.floor() - f64::from(picture.hotspot.1);
    let across = f64::from(size);
    // **Off this display altogether is nothing to draw**, and asked in floating
    // point so a position far outside `i32` never reaches an integer.
    if left + across <= f64::from(bounds.loc.x)
        || top + across <= f64::from(bounds.loc.y)
        || left >= f64::from(bounds.loc.x) + f64::from(bounds.size.w)
        || top >= f64::from(bounds.loc.y) + f64::from(bounds.size.h)
    {
        return Ok(None);
    }
    let (Ok(side), Some(x), Some(y)) = (i32::try_from(size), whole(left), whole(top)) else {
        return Ok(None);
    };
    Ok(Some(ThePointer {
        area: Rectangle::new((x, y).into(), (side, side).into()),
        picture,
    }))
}

/// Where the pointer is, for the states alo draws.
const fn where_it_is(cursor: &Cursor) -> Option<(f64, f64)> {
    match cursor {
        Cursor::Arrow { location }
        | Cursor::Resize { location, .. }
        | Cursor::Named { location, .. } => Some((location.x, location.y)),
        Cursor::Default | Cursor::Hidden | Cursor::Surface { .. } => None,
    }
}

/// A whole coordinate that is already known to be near the display, as an
/// `i32`.
fn whole(at: f64) -> Option<i32> {
    if at < f64::from(i32::MIN) || at > f64::from(i32::MAX) {
        return None;
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "already whole and inside i32's range, so the conversion is exact"
    )]
    let at = at as i32;
    Some(at)
}

#[cfg(test)]
#[path = "the_pointer_drawn_tests.rs"]
mod tests;
