//! The rounded shapes on the sign-in screen, rasterised with their corners.
//!
//! The monogram's square and the entry field have an 18-unit corner, and the
//! shell's flat [`Solid`](crate::painted::Solid) is an axis-aligned rectangle
//! with no corner at all. `crate::dock_surface` met this first and answered it
//! with a signed distance field antialiased by one `0.5 - d` rule; this is
//! that answer for a second surface, and it uses the same
//! [`Washed`](crate::painted::Washed) the compositor already composites.
//!
//! # Why a raster rather than more rectangles
//!
//! A corner drawn as a staircase of rectangles is visible at any size and
//! unbearable at 18 units. `dock_surface`'s own header says the same of the
//! Dock's radius, and the two surfaces now share the rule rather than each
//! approximating it.
//!
//! # What this draws, and what it does not
//!
//! Drawn: the ground, the monogram's rounded square, and the entry field — its
//! surface, its border, and the border in the failure colour when a person has
//! just been refused.
//!
//! The ground is **the wallpaper**, which `crate::sign_in_wallpaper` draws:
//! five cubic paths with linear gradients, from
//! `docs/design/figma-snapshot/176-10686-wallpaper.svg`. This file said *not
//! drawn: the wallpaper* until 2026-10-12, and the screen was flat `#FAF7F2`
//! behind the group — recognisably not the design, and the first thing an eye
//! went to in a photograph of a running machine. A rounded-rectangle distance
//! field cannot express a cubic, so that file is a scanline rasteriser rather
//! than more of this one.
//!
//! **Not drawn: text.** That is `crate::painted_text`'s, inked over this.

use smithay::utils::{Physical, Rectangle, Scale};

use crate::painted::Washed;
use crate::sign_in_layout::{SignInLayout, THE_ENTRY_CORNER_IS};

/// The ground the design draws the screen on.
///
/// `#FAF7F2`, a warm off-white, read from the wallpaper export's own first
/// fill. **It is not one of the bound colour tokens** — it is a fill on the
/// frame — which is why it is a constant here with its provenance beside it
/// rather than a lookup in `alo_appearance`.
pub(crate) const THE_GROUND_IS: [u8; 3] = [0xFA, 0xF7, 0xF2];

/// The monogram's square, `bg/cool`.
pub(crate) const THE_MONOGRAM_GROUND_IS: [u8; 3] = [0xEE, 0xF2, 0xF4];

/// The entry's surface, `bg/surface`.
const THE_ENTRY_GROUND_IS: [u8; 3] = [0xFF, 0xFF, 0xFF];

/// The entry's border at rest, `border/default`.
const THE_ENTRY_BORDER_IS: [u8; 3] = [0xE7, 0xEB, 0xEF];

/// The entry's border after a refusal.
///
/// Read from the render of `367:25584`, not from the tree: a stroke on a frame
/// is not a child layer, which is the blind spot `MANIFEST.md` records twice.
const THE_REFUSED_BORDER_IS: [u8; 3] = [0xC0, 0x39, 0x2B];

/// How thick the entry's border is.
const THE_BORDER_IS: f32 = 1.5;

/// The monogram's corner, `radius/md`.
const THE_MONOGRAM_CORNER_IS: i32 = 12;

/// Whether the person has just been refused, which is the only thing that
/// changes what this draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TheEntryIs {
    /// At rest, or being typed into.
    Waiting,
    /// The last attempt was refused. The border takes the failure colour and
    /// the sentence is inked beneath it by the caller.
    Refused,
}

/// Rasterise the screen's rounded shapes for this layout, at this scale.
///
/// [`None`] when the screen has no area, which is the one case with nothing to
/// draw rather than a refusal worth reporting.
#[must_use]
pub(crate) fn sign_in_surface(
    at: &SignInLayout,
    entry: TheEntryIs,
    scale: Scale<f64>,
) -> Option<Washed> {
    let area: Rectangle<i32, Physical> = at.screen.to_physical_precise_round(scale);
    if area.size.w <= 0 || area.size.h <= 0 {
        return None;
    }
    let (w, h) = (area.size.w, area.size.h);
    let monogram = at.monogram.to_physical_precise_round::<f64, i32>(scale);
    let field = at.entry.to_physical_precise_round::<f64, i32>(scale);
    let corner = f64::from(THE_ENTRY_CORNER_IS) * scale.x;
    let monogram_corner = f64::from(THE_MONOGRAM_CORNER_IS) * scale.x;
    let border = f64::from(THE_BORDER_IS) * scale.x;
    let edge = match entry {
        TheEntryIs::Waiting => THE_ENTRY_BORDER_IS,
        TheEntryIs::Refused => THE_REFUSED_BORDER_IS,
    };

    // **The design's own ground**, which is the five folds of light on
    // `THE_GROUND_IS` rather than the flat colour this drew until the
    // wallpaper existed. One raster for the whole screen, made once, read
    // below as each pixel's starting colour.
    let behind = crate::sign_in_wallpaper::the_wallpaper(w, h, THE_GROUND_IS);

    let mut pixels = Vec::with_capacity((w as usize) * (h as usize) * 4);
    for y in 0..h {
        for x in 0..w {
            // The pixel's middle, which is what a distance is measured to.
            let px = f64::from(x) + 0.5;
            let py = f64::from(y) + 0.5;

            // The wallpaper under this pixel. A screen it could not draw for
            // falls back to the flat ground rather than to nothing, so the
            // screen is never a hole.
            let mut colour = behind
                .get(((y * w + x) as usize).min(behind.len().saturating_sub(1)))
                .copied()
                .unwrap_or(THE_GROUND_IS);

            // The monogram's square, under everything else it overlaps with
            // nothing — it is above the ground and below no other shape.
            let d = how_far(px, py, monogram, monogram_corner);
            let inside = coverage(d);
            if inside > 0.0 {
                colour = mixed(colour, THE_MONOGRAM_GROUND_IS, inside);
            }

            // The entry: its surface, then its border drawn as the ring
            // between the outer edge and the same shape inset by the border's
            // thickness. One distance field answers both, which is what keeps
            // the corner's border the same thickness as the flat's.
            let outer = how_far(px, py, field, corner);
            let in_field = coverage(outer);
            if in_field > 0.0 {
                colour = mixed(colour, THE_ENTRY_GROUND_IS, in_field);
                let inner = how_far(px, py, inset(field, border), (corner - border).max(0.0));
                let ring = in_field - coverage(inner);
                if ring > 0.0 {
                    colour = mixed(colour, edge, ring);
                }
            }

            pixels.extend_from_slice(&[colour[0], colour[1], colour[2], 0xFF]);
        }
    }
    Some(Washed { area, pixels })
}

/// The signed distance from a point to a rounded rectangle: negative inside,
/// positive outside.
///
/// `crate::dock_surface::rounded`'s formulation, in this file's own units. It
/// is exact for every point, which is what lets one `0.5 - d` antialias the
/// flats and the corners by the same rule.
fn how_far(px: f64, py: f64, of: Rectangle<i32, Physical>, radius: f64) -> f64 {
    let hw = f64::from(of.size.w) / 2.0;
    let hh = f64::from(of.size.h) / 2.0;
    let cx = f64::from(of.loc.x) + hw;
    let cy = f64::from(of.loc.y) + hh;
    let r = radius.min(hw).min(hh).max(0.0);
    let qx = (px - cx).abs() - (hw - r);
    let qy = (py - cy).abs() - (hh - r);
    let outside = qx.max(0.0).hypot(qy.max(0.0));
    outside + qx.max(qy).min(0.0) - r
}

/// How much of this pixel the shape covers, from its signed distance.
///
/// **One rule for flats and corners alike**, which is the whole reason for the
/// distance field: a corner antialiased differently from an edge reads as a
/// different shape at the join.
fn coverage(distance: f64) -> f64 {
    (0.5 - distance).clamp(0.0, 1.0)
}

/// The same rectangle, pulled in on every side.
fn inset(of: Rectangle<i32, Physical>, by: f64) -> Rectangle<i32, Physical> {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a border is a few pixels and the rectangle it insets is an output's worth"
    )]
    let by = by.round() as i32;
    Rectangle::new(
        (of.loc.x + by, of.loc.y + by).into(),
        ((of.size.w - 2 * by).max(0), (of.size.h - 2 * by).max(0)).into(),
    )
}

/// `over` laid on `under` at this coverage.
fn mixed(under: [u8; 3], over: [u8; 3], how_much: f64) -> [u8; 3] {
    let how_much = how_much.clamp(0.0, 1.0);
    let one = |u: u8, o: u8| {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "both ends are bytes and the mix is clamped between them"
        )]
        {
            (f64::from(u) + (f64::from(o) - f64::from(u)) * how_much).round() as u8
        }
    };
    [
        one(under[0], over[0]),
        one(under[1], over[1]),
        one(under[2], over[2]),
    ]
}

#[cfg(test)]
#[path = "sign_in_surface_tests.rs"]
mod tests;
