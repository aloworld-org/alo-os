//! The Dock's own surface: a rounded white panel with a hairline and a shadow.
//!
//! **[ADR 0101](../../../docs/decisions/0101-one-dock-component-family-and-the-renderer-that-can-draw-it.md)**,
//! normalised from the design file's newer edge family:
//!
//! | | |
//! |---|---|
//! | corner radius | `alo_dock::measures::THE_DOCKS_CORNER`, 18 |
//! | border | `THE_DOCKS_EDGE`, 1, in `border/default` |
//! | shadow | `0 8px 24px rgba(7,19,31,0.08)` — `THE_SHADOW_FALLS` and `THE_SHADOW_SPREADS` |
//!
//! # Why this is a raster and not more rectangles
//!
//! `crate::painted` draws opaque axis-aligned rectangles, which is right for
//! what it draws and cannot make a rounded corner or a shadow: both are
//! **partial coverage**, and a rectangle is all or nothing. A stair-stepped
//! corner is not an acceptable substitute for a radius on a surface a person
//! looks at all day.
//!
//! So the surface is rasterised here, once per layout, into straight `RGBA` and
//! handed to the renderer as a texture. That is not a new road: the lock screen
//! has imported an `Abgr8888` buffer through `ImportMem` and drawn it with
//! `Frame::render_texture_from_to` since long before this file —
//! `crate::lock_texture` is thirty lines and does exactly that.
//!
//! **This lane reported the opposite and was wrong.** The report said the
//! compositor could only draw opaque rectangles, generalising from
//! `crate::painted`'s `alpha: 1.0` to the whole pipeline without looking at the
//! other paths in the same crate. ADR 0101 records the correction. The renderer
//! is `GlesRenderer` and has had per-pixel alpha all along.
//!
//! # The arithmetic
//!
//! One signed distance function, sampled per pixel. The distance to a rounded
//! rectangle is cheap and exact, and **coverage is `0.5 - d` clamped** — which
//! anti-aliases every edge, straight or curved, by the same rule and without a
//! special case for the corners.
//!
//! The shadow is the same distance taken against a rectangle offset downward,
//! faded across the blur's width. A true Gaussian would cost a convolution for
//! a difference nobody can see at this radius and this opacity.

use alo_dock::measures::{THE_DOCKS_CORNER, THE_DOCKS_EDGE, THE_SHADOW_FALLS, THE_SHADOW_SPREADS};
use smithay::utils::{Physical, Rectangle};

use crate::painted::Washed;

/// How dark the shadow is at its darkest, out of 255.
///
/// `rgba(7,19,31,0.08)` — the `Overlay / Glass` effect style, measured on
/// `348:23874`. The **colour** is `text/primary`'s own near-black rather than a
/// fourth number: a shadow is that colour at eight percent, which is what the
/// design file says and what keeps this from being a hex nobody can trace.
const THE_SHADOW_IS_THIS_DARK: f32 = 0.08;

/// The colour a shadow is cast in: `navy/950`, `#07131F`.
///
/// **Not from the palette, and that is deliberate.** A shadow is not a surface
/// colour a person chose or a contrast setting changes — it is the absence of
/// light behind a raised object, and it is the same in every scheme. The
/// palette's roles are things a person's choices reach; this is not one.
const THE_SHADOWS_COLOUR: [u8; 3] = [0x07, 0x13, 0x1F];

/// How much room the shadow needs beyond the panel's own edge.
///
/// The blur reaches `THE_SHADOW_SPREADS` and the whole thing is pushed down by
/// `THE_SHADOW_FALLS`, so the raster has to be bigger than the panel by their
/// sum on the side it falls toward — and by the spread alone on the others.
/// Taken on every side, because one number is easier to be right about than
/// four and the cost is a few transparent pixels.
const fn how_far_the_shadow_reaches() -> i32 {
    (THE_SHADOW_SPREADS + THE_SHADOW_FALLS) as i32
}

/// What the surface is drawn out of.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ADocksSurface {
    /// Where the panel itself is, in this display's pixels.
    pub(crate) panel: Rectangle<i32, Physical>,
    /// The panel's fill.
    pub(crate) fill: [u8; 3],
    /// The hairline around it.
    pub(crate) edge: [u8; 3],
    /// The person's accent, drawn as a rule along the face looking at the
    /// canvas.
    pub(crate) accent: [u8; 3],
    /// Which edge of the screen the Dock is on, so the accent knows which face
    /// looks at the canvas.
    pub(crate) against: alo_dock::Edge,
    /// How thick the accent rule is.
    pub(crate) accent_is: i32,
    /// The display, so the raster can be kept on it.
    ///
    /// **The shadow's reach puts the raster past the screen's edge** — the Dock
    /// floats eight clear of its edge and the shadow spreads thirty-two — so
    /// without this the texture's destination would hang off the framebuffer.
    /// Found by `the_dock_reaches_the_bytes_a_card_is_handed`, which draws
    /// through the software renderer and got a flat frame.
    pub(crate) room: (i32, i32),
    /// How many physical pixels this display draws for one logical one, in
    /// hundredths.
    ///
    /// **The radius and the hairline are logical measures** and every other
    /// figure here is in this display's own pixels, so this is where the one
    /// becomes the other. A radius that skipped it would be half the size it
    /// should be on a dense screen, which is the standing rule in `CLAUDE.md`
    /// about a figure reaching a display without passing through its scale.
    pub(crate) scale: i32,
}

/// Rasterise the Dock's surface.
///
/// Returns [`None`] for a panel with no area, which is a Dock that gave way
/// rather than a failure.
pub(crate) fn surface(of: ADocksSurface) -> Option<Washed> {
    let panel = of.panel;
    if panel.size.w <= 0 || panel.size.h <= 0 {
        return None;
    }
    let reach = how_far_the_shadow_reaches();
    // Inflated for the shadow, then **clipped to the display**: a texture whose
    // destination hangs off the framebuffer is pixels nobody sees at best, and
    // a frame the software renderer declines to draw at worst.
    let (screen_w, screen_h) = of.room;
    let left = (panel.loc.x - reach).max(0);
    let top = (panel.loc.y - reach).max(0);
    let right = (panel.loc.x + panel.size.w + reach).min(screen_w.max(0));
    let bottom = (panel.loc.y + panel.size.h + reach).min(screen_h.max(0));
    if right <= left || bottom <= top {
        return None;
    }
    let area = Rectangle::new((left, top).into(), (right - left, bottom - top).into());
    let (width, height) = (area.size.w, area.size.h);
    let at = |logical: u32| -> f32 { (logical as f32) * (of.scale.max(1) as f32) / 100.0 };
    // The panel's own geometry, relative to the raster's corner — which is not
    // `reach` any more once the clip above has moved it.
    let (left, top) = (
        (panel.loc.x - area.loc.x) as f32,
        (panel.loc.y - area.loc.y) as f32,
    );
    let (right, bottom) = (left + panel.size.w as f32, top + panel.size.h as f32);
    let (cx, cy) = ((left + right) / 2.0, (top + bottom) / 2.0);
    let (hw, hh) = ((right - left) / 2.0, (bottom - top) / 2.0);
    let radius = at(THE_DOCKS_CORNER).min(hw).min(hh);
    let line = at(THE_DOCKS_EDGE).max(1.0);
    let falls = at(THE_SHADOW_FALLS);
    let spreads = at(THE_SHADOW_SPREADS).max(1.0);

    let mut pixels = vec![0_u8; (width as usize) * (height as usize) * 4];
    for y in 0..height {
        for x in 0..width {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            // The shadow first: the same panel, pushed toward the screen's edge
            // and faded across the blur's width.
            let shadow = {
                let d = rounded(px, py, cx, cy + falls, hw, hh, radius);
                let across = (0.5 - d / spreads).clamp(0.0, 1.0);
                // Smoothed, so the fade has no visible band in it. The cubic is
                // the usual one and is here rather than in a helper because it
                // is three operations and naming it would not make it clearer.
                across * across * (3.0 - 2.0 * across) * THE_SHADOW_IS_THIS_DARK
            };
            let d = rounded(px, py, cx, cy, hw, hh, radius);
            let inside = (0.5 - d).clamp(0.0, 1.0);
            // The hairline is the ring between the panel's edge and one line in.
            let within = (0.5 - (d + line)).clamp(0.0, 1.0);
            let hairline = (inside - within).clamp(0.0, 1.0);
            // **Clipped by the panel's own shape, not by the panel less its
            // hairline.** The accent *is* the edge treatment on the face that
            // looks at the canvas, so it covers the hairline there rather than
            // sitting inside it — and clipping it with `within` left its
            // outermost row blended with the fill, which read as the wrong
            // colour to anything sampling it.
            let accent = how_much_accent(&of, px, py, left, top, right, bottom) * inside;

            // Composited back to front: shadow, fill, hairline, accent.
            let mut colour = [0.0_f32; 3];
            let mut alpha = 0.0_f32;
            for (over, its) in [
                (shadow, THE_SHADOWS_COLOUR),
                (within, of.fill),
                (hairline, of.edge),
                (accent, of.accent),
            ] {
                if over <= 0.0 {
                    continue;
                }
                for channel in 0..3 {
                    let Some(have) = colour.get_mut(channel) else {
                        continue;
                    };
                    let it = f32::from(*its.get(channel).unwrap_or(&0)) / 255.0;
                    *have = it * over + *have * (1.0 - over);
                }
                alpha = over + alpha * (1.0 - over);
            }
            let at = ((y as usize) * (width as usize) + (x as usize)) * 4;
            if let Some(out) = pixels.get_mut(at..at + 4) {
                for channel in 0..3 {
                    if let (Some(byte), Some(have)) = (out.get_mut(channel), colour.get(channel)) {
                        *byte = (have * 255.0).round().clamp(0.0, 255.0) as u8;
                    }
                }
                if let Some(byte) = out.get_mut(3) {
                    *byte = (alpha * 255.0).round().clamp(0.0, 255.0) as u8;
                }
            }
        }
    }
    Some(Washed { area, pixels })
}

/// The signed distance from a point to a rounded rectangle: negative inside,
/// positive outside, and the straight-line distance to the nearest edge either
/// way.
///
/// The standard formulation. It is exact for every point, which is what lets one
/// `0.5 - d` anti-alias the flats and the corners by the same rule.
fn rounded(px: f32, py: f32, cx: f32, cy: f32, hw: f32, hh: f32, r: f32) -> f32 {
    let qx = (px - cx).abs() - (hw - r);
    let qy = (py - cy).abs() - (hh - r);
    let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
    outside + qx.max(qy).min(0.0) - r
}

/// How much of the accent rule covers this pixel.
///
/// **Not `accent_at`, which is `alo_appearance`'s own accessor for the colour a
/// person chose at a moment.** `tests/desktop_source.rs` holds that an accent in
/// the drawing path comes from `appearance.accent_at(now)` and nowhere else, and
/// it reads the text — so a local function wearing that name is a name collision
/// with a rule, found by the rule.
///
/// **Along the face that looks at the canvas**, which is the opposite end of the
/// thickness from the screen edge the Dock is lifted off — the same rule
/// `crate::dock_raster::the_accent_on` holds, and it is here as well because
/// this is now where that face is drawn.
fn how_much_accent(
    of: &ADocksSurface,
    px: f32,
    py: f32,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
) -> f32 {
    let thick = of.accent_is.max(0) as f32;
    if thick <= 0.0 {
        return 0.0;
    }
    let within = match of.against {
        alo_dock::Edge::Bottom => py < top + thick,
        alo_dock::Edge::Top => py > bottom - thick,
        alo_dock::Edge::Left => px > right - thick,
        alo_dock::Edge::Right => px < left + thick,
    };
    if within { 1.0 } else { 0.0 }
}

#[cfg(test)]
#[path = "dock_surface_tests.rs"]
mod tests;
