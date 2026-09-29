//! **Painting a compositor-owned cursor**, whichever one it is.
//!
//! There are two now — `crate::default_cursor`'s arrow and
//! `crate::resize_cursor`'s four double-headed ones — drawn the same way from
//! the same three characters, clipped to the same output, above everything else.
//! The loop that does it lives here rather than once in each, because two
//! painters held to one behaviour by nothing but their similarity is how a cursor
//! ends up clipping differently at an output's edge depending on which one it is.
//!
//! What each of those files keeps is what is actually its own: the shape, and
//! where the hotspot sits inside it. The arrow's is its tip and a resize arrow's
//! is its middle (ADR 0071), which is a difference in what the shape *means* and
//! not in how it is drawn.

use crate::RenderError;
use smithay::{
    backend::renderer::Color32F,
    utils::{Logical, Physical, Point, Rectangle},
};

/// One opaque output pixel and its contrast colour.
pub(crate) type Pixel = (Rectangle<i32, Physical>, Color32F);

/// The mask's pixels, placed with its top-left corner here and clipped to bounds.
///
/// `B` is outline, `W` is interior and a space is nothing, which is the form both
/// masks are written in. Fractional positions are floored, as they were before
/// this was shared, so a cursor never lands between pixels.
///
/// Refuses a non-finite origin rather than drawing nothing: a position that is
/// not a number is a fault upstream of the cursor, and a painter that silently
/// drew an empty frame for it would hide that fault behind a missing pointer.
pub(crate) fn painted(
    rows: &[&str],
    origin: Point<f64, Logical>,
    bounds: Rectangle<i32, Physical>,
) -> Result<Vec<Pixel>, RenderError> {
    if !origin.x.is_finite() || !origin.y.is_finite() {
        return Err(RenderError::Submission("non-finite cursor position".into()));
    }
    let mut pixels = Vec::new();
    for (y, row) in (0..).zip(rows) {
        for (x, value) in (0..).zip(row.bytes()) {
            if value == b' ' {
                continue;
            }
            let px = origin.x.floor() + f64::from(x);
            let py = origin.y.floor() + f64::from(y);
            if px < f64::from(bounds.loc.x)
                || py < f64::from(bounds.loc.y)
                || px >= f64::from(bounds.loc.x) + f64::from(bounds.size.w)
                || py >= f64::from(bounds.loc.y) + f64::from(bounds.size.h)
            {
                continue;
            }
            let channel = if value == b'W' { 1.0 } else { 0.0 };
            pixels.push((
                Rectangle::new((px as i32, py as i32).into(), (1, 1).into()),
                Color32F::new(channel, channel, channel, 1.0),
            ));
        }
    }
    Ok(pixels)
}
