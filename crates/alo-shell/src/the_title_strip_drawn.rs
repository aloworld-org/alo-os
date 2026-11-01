//! The strip a window's title sits on, drawn with its top corners rounded.
//!
//! `554:25`, *32 visible edge / 44 interaction*: a strip 32 tall, filled in the
//! look's ground, with a one-unit line all round and its **top** corners
//! rounded at 10 — the bottom two meet the window and stay square. A flat
//! rectangle cannot be that: a rounded corner is partial coverage, so the strip
//! is a raster, drawn under the window's controls and title and over nothing
//! the application drew.
//!
//! **One unit wider on each side than the strip's own rectangle**, so its
//! sides run straight down into the line `crate::a_window_raised` draws round
//! the window, and the two read as one edge.

use smithay::utils::{Physical, Rectangle};

use crate::painted::Washed;
use crate::window_edge_look::{A_LINE_IS, THE_STRIPS_CORNER, WhichLook};
use crate::window_rasters_kept::{TheKey, Which, kept_or_made, placed};

/// The strip, at `strip` in this display's pixels, in this look.
///
/// `strip` is the edge's own strip rectangle already in pixels; `scale` the
/// display's as a fraction; nothing is drawn off `output`. [`None`] for a strip
/// with no area or wholly off the display.
#[must_use]
pub(crate) fn the_strip(
    strip: Rectangle<i32, Physical>,
    scale: f64,
    look: WhichLook,
    output: (i32, i32),
) -> Option<Washed> {
    if strip.size.w <= 0 || strip.size.h <= 0 {
        return None;
    }
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a line and a corner at a display's scale are well inside an f32"
    )]
    let (line, corner) = (
        (f64::from(A_LINE_IS) * scale).round().max(1.0) as f32,
        (f64::from(THE_STRIPS_CORNER) * scale) as f32,
    );
    #[expect(
        clippy::cast_possible_truncation,
        reason = "one line's width in pixels is well inside an i32"
    )]
    let wider = line as i32;
    // **Made once for a size and a look, at the origin**, and moved and
    // clipped each frame (`crate::window_rasters_kept`).
    let size = (strip.size.w + wider * 2, strip.size.h);
    let key = TheKey::of(Which::Strip, size, scale, look);
    let kept = kept_or_made(key, || {
        made_at_the_origin(size, (line, corner), look)
            .into_iter()
            .collect()
    });
    placed(&kept, (strip.loc.x - wider, strip.loc.y), output)
        .into_iter()
        .next()
}

/// The strip `size` wide and tall with its top-left at the origin, its line
/// `line` thick and its top corners `corner` round.
pub(crate) fn made_at_the_origin(
    size: (i32, i32),
    (line, corner): (f32, f32),
    look: WhichLook,
) -> Option<Washed> {
    let whole = Rectangle::<i32, Physical>::from_size(size.into());
    let colours = look.colours();
    let (l, t) = (whole.loc.x as f32, whole.loc.y as f32);
    let (r, b) = (l + whole.size.w as f32, t + whole.size.h as f32);
    let corner = corner.min((r - l) / 2.0).min(b - t);
    crate::a_window_raised::raster(whole, |px, py| {
        let d = away_from_a_top_rounded(px, py, (l, t, r, b), corner);
        let inside = (0.5 - d).clamp(0.0, 1.0);
        if inside <= 0.0 {
            return (colours.strip, 0.0);
        }
        let within = (0.5 - (d + line)).clamp(0.0, 1.0);
        // The line where the ground stops: the edge colour over the fill.
        if within >= 1.0 {
            (colours.strip, 1.0)
        } else {
            let mix = |edge: u8, fill: u8| {
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "a blend of two bytes stays a byte"
                )]
                let it =
                    (f32::from(edge) * (1.0 - within) + f32::from(fill) * within).round() as u8;
                it
            };
            let [er, eg, eb] = colours.edge;
            let [fr, fg, fb] = colours.strip;
            ([mix(er, fr), mix(eg, fg), mix(eb, fb)], inside)
        }
    })
}

/// How far a point is outside a box whose top two corners are rounded at
/// `corner`, negative inside.
fn away_from_a_top_rounded(
    px: f32,
    py: f32,
    (l, t, r, b): (f32, f32, f32, f32),
    corner: f32,
) -> f32 {
    if py < t + corner {
        if px < l + corner {
            return (px - (l + corner)).hypot(py - (t + corner)) - corner;
        }
        if px > r - corner {
            return (px - (r - corner)).hypot(py - (t + corner)) - corner;
        }
    }
    let dx = (l - px).max(px - r);
    let dy = (t - py).max(py - b);
    if dx > 0.0 && dy > 0.0 {
        dx.hypot(dy)
    } else {
        dx.max(dy)
    }
}

#[cfg(test)]
#[path = "the_title_strip_drawn_tests.rs"]
mod tests;
