//! The line round an application's window and the shadow under it, as the
//! rasters a frame draws **before** the window.
//!
//! # Round the content, never on it
//!
//! The board the owner approved on 2026-11-01 (`557:456`) says it in as many
//! words: *the application's own content is never dimmed, cropped or
//! recoloured.* So the one-unit line is drawn **outside** the window's own
//! rectangle, in the pixel row and column next to it, and the shadow is drawn
//! round that. Nothing here is drawn where the application draws.
//!
//! **And so the body's corners are square.** The design draws every window's
//! body at `radius/lg`, 18 (`557:213`), on a mock-up of the application's
//! content. Rounding a real application's corners means cutting off the pixels
//! it drew there, which is the cropping the same board forbids — so the body
//! keeps the corners the application gave it, and the rounding is the strip's,
//! above it (`crate::the_title_strip_drawn`). Recorded in
//! `docs/design/the-external-window-edge.md` as a reading, for the owner.
//!
//! # Only the bands round the window
//!
//! The window covers its own middle, so a raster of it would be pixels nobody
//! sees, computed every frame. This draws the four bands between the window and
//! the shadow's reach — top, bottom, left and right — and nothing inside.

use smithay::utils::{Physical, Rectangle};

use crate::painted::Washed;
use crate::window_edge_look::{A_LINE_IS, WhichLook};

/// The shadow's colour, `#07131f`, as every raised surface in this crate has it.
const THE_SHADOWS_COLOUR: [u8; 3] = [0x07, 0x13, 0x1F];

/// The line and the shadow round one window, as up to four rasters.
///
/// `window` is the application's own rectangle in this display's pixels,
/// `scale` the display's as a fraction, `edge` the line's colour and `output`
/// the display's size: nothing is drawn off it.
///
/// Empty for a window with no area, which is a window with nothing to frame.
#[must_use]
pub(crate) fn round_the_window(
    window: Rectangle<i32, Physical>,
    scale: f64,
    look: WhichLook,
    output: (i32, i32),
) -> Vec<Washed> {
    if window.size.w <= 0 || window.size.h <= 0 {
        return Vec::new();
    }
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    let at = |logical: i32| -> f32 {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a few dozen units at a display's scale is well inside an f32"
        )]
        let it = (f64::from(logical) * scale) as f32;
        it
    };
    let line = at(A_LINE_IS).round().max(1.0);
    let shadow = look.shadow();
    let (falls, blurs, spreads) = (
        at(shadow.falls),
        at(shadow.blurs).max(1.0),
        at(shadow.spreads),
    );
    let edge = look.colours().edge;

    let (left, top) = (window.loc.x as f32, window.loc.y as f32);
    let (right, bottom) = (left + window.size.w as f32, top + window.size.h as f32);
    // The ring: one line outside the window on every side.
    let ring = (left - line, top - line, right + line, bottom + line);
    // The shadow's own box: the ring grown by the spread and pushed down.
    let shade = (
        ring.0 - spreads,
        ring.1 - spreads + falls,
        ring.2 + spreads,
        ring.3 + spreads + falls,
    );
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a reach of a few dozen pixels is well inside an i32"
    )]
    let reach = (falls.abs() + blurs + spreads.max(0.0) + line).ceil() as i32;

    let outer = Rectangle::<i32, Physical>::new(
        (window.loc.x - reach, window.loc.y - reach).into(),
        (window.size.w + reach * 2, window.size.h + reach * 2).into(),
    );
    let Some(outer) = outer.intersection(Rectangle::from_size(output.into())) else {
        return Vec::new();
    };
    let (ox, oy) = (outer.loc.x, outer.loc.y);
    let (or, ob) = (ox + outer.size.w, oy + outer.size.h);
    let (wx, wy) = (window.loc.x.max(ox), window.loc.y.max(oy));
    let (wr, wb) = (
        (window.loc.x + window.size.w).min(or),
        (window.loc.y + window.size.h).min(ob),
    );
    let bands = [
        (ox, oy, or, wy),
        (ox, wb, or, ob),
        (ox, wy, wx, wb),
        (wr, wy, or, wb),
    ];
    bands
        .into_iter()
        .filter_map(|(x0, y0, x1, y1)| {
            let band = Rectangle::<i32, Physical>::new((x0, y0).into(), (x1 - x0, y1 - y0).into());
            raster(band, |px, py| {
                if inside(px, py, ring) && !inside(px, py, (left, top, right, bottom)) {
                    return (edge, 1.0);
                }
                let d = away_from(px, py, shade);
                let across = (0.5 - d / blurs).clamp(0.0, 1.0);
                (
                    THE_SHADOWS_COLOUR,
                    across * across * (3.0 - 2.0 * across) * shadow.darkness,
                )
            })
        })
        .collect()
}

/// Whether a pixel's centre is inside a box `(left, top, right, bottom)`.
fn inside(px: f32, py: f32, (l, t, r, b): (f32, f32, f32, f32)) -> bool {
    px >= l && px < r && py >= t && py < b
}

/// How far a point is outside a box, negative inside: a square box's distance.
fn away_from(px: f32, py: f32, (l, t, r, b): (f32, f32, f32, f32)) -> f32 {
    let dx = (l - px).max(px - r);
    let dy = (t - py).max(py - b);
    if dx > 0.0 && dy > 0.0 {
        dx.hypot(dy)
    } else {
        dx.max(dy)
    }
}

/// One raster of `area`, each pixel a colour and how much of it there is.
///
/// [`None`] for an area with no pixels. Shared with
/// `crate::the_title_strip_drawn`, so straight alpha is written one way.
pub(crate) fn raster(
    area: Rectangle<i32, Physical>,
    pixel: impl Fn(f32, f32) -> ([u8; 3], f32),
) -> Option<Washed> {
    if area.size.w <= 0 || area.size.h <= 0 {
        return None;
    }
    let (w, h) = (area.size.w, area.size.h);
    let mut pixels = Vec::with_capacity(usize::try_from(w * h * 4).unwrap_or(0));
    for y in 0..h {
        for x in 0..w {
            let ([red, green, blue], alpha) = pixel(
                (area.loc.x + x) as f32 + 0.5,
                (area.loc.y + y) as f32 + 0.5,
            );
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "clamped to a byte's range before the cast"
            )]
            let alpha = (alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
            pixels.extend_from_slice(&[red, green, blue, alpha]);
        }
    }
    Some(Washed { area, pixels })
}

#[cfg(test)]
#[path = "a_window_raised_tests.rs"]
mod tests;
