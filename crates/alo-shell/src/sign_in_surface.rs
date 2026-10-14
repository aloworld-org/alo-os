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
/// Every colour this screen draws, for one scheme and one contrast.
///
/// **Taken from the palette, not written down.** Each of these was a fixed
/// light-mode hex until 2026-10-13, while the text was drawn in
/// `Contrast::ink`, which does follow the scheme — so in dark mode the ink
/// turned light and landed on a hardcoded light ground. Measured before it was
/// fixed: 591 inked pixels against 3826 in light, and at least one label at a
/// contrast of **zero** against its own background. The two-field screen this
/// design replaced took every colour from the palette and was readable in both
/// schemes, so this was a regression, and high contrast — the setting that
/// exists for people who need it — was broken the same way.
///
/// # Why mixes and not the design file's dark table
///
/// `variables.toml` records a dark set, and says of it: *dark-theme parity
/// needs its own measured evidence before being marked complete … still
/// adopted by nothing*. Adopting it here would be this lane deciding a
/// question the design lane has left open, and it has no dark value for the
/// screen's ground at all.
///
/// So the three roles the palette already answers are taken from it, and the
/// four it does not are mixed from those. **In light, at the designed
/// contrast, the mixes land within three units of the design's own hexes** —
/// `#EEF2F4`, `#E7EBEF` and `#596B78` — which is what
/// `the_light_colours_are_still_the_designs` holds them to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TheColours {
    /// The screen behind everything, which the wallpaper is drawn onto.
    pub(crate) ground: [u8; 3],
    /// The entry field's own surface.
    pub(crate) entry: [u8; 3],
    /// The monogram's rounded square.
    pub(crate) monogram: [u8; 3],
    /// Words, marks and the arrow.
    pub(crate) ink: [u8; 3],
    /// The placeholder inside an empty entry.
    pub(crate) muted: [u8; 3],
    /// The entry's border at rest.
    pub(crate) border: [u8; 3],
    /// The entry's border after a refusal.
    pub(crate) refused: [u8; 3],
}

impl TheColours {
    /// This screen's colours for the scheme and contrast a person chose.
    ///
    /// **At the designed contrast these are the design's own recorded values,
    /// per scheme.** `variables.toml` carries both tables, and taking the dark
    /// one is reading what was measured rather than inventing it — that file
    /// asks for more evidence before dark parity is called *complete*, which
    /// is a different thing from a screen being readable at all.
    ///
    /// **At high contrast the design is overruled**, which is the whole point
    /// of the setting: the palette answers, and the tints are derived from it
    /// so nothing is softer than the person asked for.
    pub(crate) fn of(contrast: crate::Contrast, scheme: alo_appearance::Scheme) -> Self {
        let ground = contrast.ground(scheme);
        let ink = contrast.ink(scheme);
        if matches!(contrast, crate::Contrast::High) {
            return Self {
                ground,
                entry: contrast.dock(scheme),
                monogram: contrast.dock(scheme),
                ink,
                // No softening at high contrast: a muted placeholder is the
                // thing this setting exists to stop.
                muted: ink,
                border: ink,
                refused: THE_DANGER_IS,
            };
        }
        match scheme {
            alo_appearance::Scheme::Light => Self {
                ground,
                entry: [0xFF, 0xFF, 0xFF],
                monogram: [0xEE, 0xF2, 0xF4],
                ink,
                muted: [0x59, 0x6B, 0x78],
                border: [0xE7, 0xEB, 0xEF],
                refused: THE_DANGER_IS,
            },
            // Every one of these is in `variables.toml`'s own dark table:
            // `navy/950`, `bg/surface`, `bg/cool`, `text/primary`,
            // `text/muted` and `border/default`.
            alo_appearance::Scheme::Dark => Self {
                ground: [0x07, 0x13, 0x1F],
                entry: [0x10, 0x2A, 0x43],
                monogram: [0x18, 0x3B, 0x56],
                ink: [0xFF, 0xFC, 0xF8],
                muted: [0xA4, 0xB4, 0xC2],
                border: [0x27, 0x4C, 0x68],
                refused: THE_DANGER_IS,
            },
        }
    }
}

/// A refusal's colour, `status/danger`.
///
/// **A status colour, not a scheme colour.** Danger reads as danger in both,
/// and a refusal that changed hue with the appearance would be saying
/// something different about the same event.
const THE_DANGER_IS: [u8; 3] = [0xC0, 0x39, 0x2B];

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
    colours: TheColours,
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
    let power = at.power.to_physical_precise_round::<f64, i32>(scale);
    // The power mark.s stroke, which is the entry.s border a little heavier so
    // a 20-unit ring reads at arm.s length.
    let stroke = (f64::from(THE_BORDER_IS) * scale.x * 1.3).max(1.5);
    let ink = colours.ink;
    let edge = match entry {
        TheEntryIs::Waiting => colours.border,
        TheEntryIs::Refused => colours.refused,
    };

    // **The design's own ground**, which is the five folds of light on
    // `THE_GROUND_IS` rather than the flat colour this drew until the
    // wallpaper existed. One raster for the whole screen, made once, read
    // below as each pixel's starting colour.
    let behind = crate::sign_in_wallpaper::the_wallpaper(w, h, colours.ground);

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
                .unwrap_or(colours.ground);

            // The monogram's square, under everything else it overlaps with
            // nothing — it is above the ground and below no other shape.
            let d = how_far(px, py, monogram, monogram_corner);
            let inside = coverage(d);
            if inside > 0.0 {
                colour = mixed(colour, colours.monogram, inside);
            }

            // The entry: its surface, then its border drawn as the ring
            // between the outer edge and the same shape inset by the border's
            // thickness. One distance field answers both, which is what keeps
            // the corner's border the same thickness as the flat's.
            let outer = how_far(px, py, field, corner);
            let in_field = coverage(outer);
            if in_field > 0.0 {
                colour = mixed(colour, colours.entry, in_field);
                let inner = how_far(px, py, inset(field, border), (corner - border).max(0.0));
                let ring = in_field - coverage(inner);
                if ring > 0.0 {
                    colour = mixed(colour, edge, ring);
                }
            }

            // **The power mark**, drawn rather than lettered. Neither bundled
            // face carries U+23FB, and a glyph a font does not have is drawn
            // as a tofu box — which is what this screen showed before this
            // existed, a hex rectangle where the design has a power control.
            // A ring with a gap at the top and a bar through it, from the
            // same distance fields the entry.s corners use.
            let mark = the_power_mark(px, py, power, stroke);
            if mark > 0.0 {
                colour = mixed(colour, ink, mark);
            }

            pixels.extend_from_slice(&[colour[0], colour[1], colour[2], 0xFF]);
        }
    }
    Some(Washed { area, pixels })
}

/// How much of this pixel the power mark covers.
///
/// A ring with a gap at the top and a bar standing in the gap, which is the
/// power symbol every machine draws. Built from the same distance fields as
/// the rounded rectangles above, because neither bundled face carries
/// U+23FB and a glyph a font lacks is drawn as a tofu box.
fn the_power_mark(px: f64, py: f64, within: Rectangle<i32, Physical>, stroke: f64) -> f64 {
    if within.size.w <= 0 || within.size.h <= 0 {
        return 0.0;
    }
    let side = f64::from(within.size.w.min(within.size.h));
    let cx = f64::from(within.loc.x) + f64::from(within.size.w) / 2.0;
    let cy = f64::from(within.loc.y) + f64::from(within.size.h) / 2.0;
    // The ring takes a little under half the target, which is what leaves the
    // 48-unit control reading as a mark rather than a filled circle.
    let radius = side * 0.21;
    let half = stroke / 2.0;
    let from_centre = (px - cx).hypot(py - cy);
    // The ring itself: the distance to the circle, signed, so `coverage`
    // antialiases it by the one rule the rest of this file uses.
    let on_the_ring = coverage((from_centre - radius).abs() - half);
    // **Only the notch the bar stands in.** Cutting the whole upper middle
    // left two side arcs and a bar, which reads as brackets rather than as a
    // power control.
    let notch = py < cy - radius * 0.55 && (px - cx).abs() < half * 2.2;
    let ring = if notch { 0.0 } else { on_the_ring };
    // The bar standing in the notch, from above the ring down to its middle.
    let within_the_bar = py >= cy - radius * 1.5 && py <= cy - radius * 0.15;
    let bar = if within_the_bar {
        coverage((px - cx).abs() - half)
    } else {
        0.0
    };
    ring.max(bar).clamp(0.0, 1.0)
}

/// The signed distance from a point to a rounded rectangle: negative inside,
/// positive outside.
///
/// `crate::dock_surface::rounded`'s formulation, in this file's own units. It
/// is exact for every point, which is what lets one `0.5 - d` antialias the
/// flats and the corners by the same rule.
pub(crate) fn how_far(px: f64, py: f64, of: Rectangle<i32, Physical>, radius: f64) -> f64 {
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
pub(crate) fn coverage(distance: f64) -> f64 {
    (0.5 - distance).clamp(0.0, 1.0)
}

/// The same rectangle, pulled in on every side.
pub(crate) fn inset(of: Rectangle<i32, Physical>, by: f64) -> Rectangle<i32, Physical> {
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
pub(crate) fn mixed(under: [u8; 3], over: [u8; 3], how_much: f64) -> [u8; 3] {
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
