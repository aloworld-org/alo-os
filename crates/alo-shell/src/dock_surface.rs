//! A rounded panel with a hairline and a shadow — the Dock's own surface, and
//! any other.
//!
//! **The name is owed a change.** [`a_panel`] already made this general, and the
//! Ask alo task window is a second caller that is not a Dock. The file is left
//! named for the Dock in this change because renaming it touches every caller
//! and the signature change here has to land with the three it already breaks;
//! `CLAUDE.md`'s law 4 is the reason to do it next rather than later.
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

use alo_dock::measures::{
    THE_COMPOSERS_CORNER, THE_DOCKS_CORNER, THE_DOCKS_EDGE, THE_FOCUS_RING_IS_THICK,
    THE_FOCUS_RING_STANDS_OFF, THE_SHADOW_FALLS, THE_SHADOW_SPREADS,
};
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

/// One rounded well, with the corner it is drawn at.
///
/// **The radius is per well, not per surface.** It was one constant —
/// `THE_COMPOSERS_CORNER`, 12 — which was right while every well was a Dock
/// field or a window-list row. The Ask alo task window needs three radii on one
/// panel: `radius/sm` 8 for a shared-file chip, `radius/md` 12 for a message
/// bubble and a composer control, and 18 for the reply field. A single constant
/// cannot draw them, and drawing them all at 12 would be the design
/// approximated rather than followed.
///
/// **Not a second list of wells-with-radii.** This file records that a
/// `composer` and a `well` were two fields differing only in how many there
/// could be, and that collapsing them was the fix; adding a parallel list now
/// would rebuild that fault with the radius as the excuse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AWell {
    /// Where it goes.
    pub(crate) at: Rectangle<i32, Physical>,
    /// Its corner radius, clamped to half its shorter side when it is drawn, so
    /// a radius larger than the shape becomes a capsule rather than nothing.
    pub(crate) corner: u32,
}

/// Shapes drawn in the design's `text/primary`, which on an approval card is two
/// things at once: the ground of the *Primary* answer, and the keyboard's
/// outline — *focus uses a visible navy outline*
/// (`docs/design/the-ask-alo-task-window.md`).
///
/// **A set of its own rather than more `wells` and `rings`.** A panel has one
/// colour for each, and the task window already spends its rings on the reply
/// field's grey border — so a navy focus outline drawn as a ring there would
/// be grey, and indistinguishable from a border. Every panel that draws
/// nothing in it says [`Self::NOTHING`], one line, so adding this did not add
/// three fields to every surface that never needed them.
#[derive(Debug, Clone, Copy)]
pub(crate) struct InPrimary<'a> {
    /// Wells filled in it — the *Primary* answer.
    pub(crate) wells: &'a [AWell],
    /// Outlines in it, each the **outer** rectangle, already stood off from
    /// what it marks: the keyboard's place.
    pub(crate) outlines: &'a [Rectangle<i32, Physical>],
    /// The colour, `text/primary`.
    pub(crate) colour: [u8; 3],
}

impl InPrimary<'static> {
    /// Nothing drawn in it, which is every panel except one carrying a
    /// question.
    pub(crate) const NOTHING: Self = Self {
        wells: &[],
        outlines: &[],
        colour: [0, 0, 0],
    };
}

/// What the surface is drawn out of.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ADocksSurface<'a> {
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
    /// Where a fallback badge goes, for each application with no usable icon.
    ///
    /// **The owner's ruling of 2026-10-11**: *for applications without a usable
    /// icon, display the application's first letter inside its own circular
    /// badge. Do not show bare letters.* Each is a 28 × 28 box in this
    /// display's pixels, centred in its 48 × 48 target — so a real icon
    /// replaces one without the slot moving or resizing.
    ///
    /// **Drawn here and not as rectangles**, for the reason the panel itself is
    /// here: a circle is partial coverage at every pixel of its edge.
    pub(crate) badges: &'a [Rectangle<i32, Physical>],
    /// Rounded bars drawn on the panel: running and focused marks.
    pub(crate) marks: &'a [Rectangle<i32, Physical>],
    /// What colour they are.
    pub(crate) mark: [u8; 3],
    /// The ground of a fallback badge.
    pub(crate) badge: [u8; 3],
    /// Rounded wells drawn on the panel.
    ///
    /// **One shape with three uses**: the composer's field on the Dock, the
    /// search field at the top of *More apps*, and every row of a window list.
    /// They were two things — a `composer` and a `well` — until 2026-10-11,
    /// and the only difference was how many there could be.
    ///
    /// **Rounded, so they are here rather than `Solid`s** — the same reason the
    /// panel is. `radius/md` = `THE_COMPOSERS_CORNER` = 12, measured on both
    /// Dock families.
    pub(crate) wells: &'a [AWell],
    /// Their ground, which is cooler than the Dock's.
    pub(crate) well_is: [u8; 3],
    /// Keyboard focus rings: the **outer** rectangle of each, already stood off
    /// from what it surrounds.
    ///
    /// Outlines and not fills, which is why they are here and not in `solids`
    /// twice over: a rectangle cannot say *this edge and not the middle*, and
    /// the corners are partial coverage like everything else on a panel.
    ///
    /// **Drawn last of all**, so nothing on the panel covers the one mark that
    /// tells a person where their next key press will land.
    pub(crate) rings: &'a [Rectangle<i32, Physical>],
    /// What colour they are. Navy in the light appearance; see
    /// `alo_dock::measures::THE_FOCUS_RING_IS_THICK`.
    pub(crate) ring_is: [u8; 3],
    /// Shapes in `text/primary`: the Primary answer and the keyboard's
    /// outline on an approval card. [`InPrimary::NOTHING`] everywhere else.
    pub(crate) in_primary: InPrimary<'a>,
    /// The display, so the raster can be kept on it.
    ///
    /// **The shadow's reach puts the raster past the screen's edge** — the Dock
    /// floats eight clear of its edge and the shadow spreads thirty-two — so
    /// without this the texture's destination would hang off the framebuffer.
    /// Found by `the_dock_reaches_the_bytes_a_card_is_handed`, which draws
    /// through the software renderer and got a flat frame.
    pub(crate) room: (i32, i32),
    // **No scale, and that is the correction rather than the omission.**
    //
    // This took one and multiplied the radius and the hairline by it, which
    // made a dense display draw an 18 radius as 36 **on a band still 76 tall**,
    // because `crate::dock_raster` hands `alo_dock`'s logical measures straight
    // through as pixels and scales none of them. Scaling one measure of a
    // geometry that is otherwise unscaled is worse than scaling none: it is the
    // *applied twice* fault wearing the *applied once* rule's clothes.
    //
    // So every figure reaching this file is in the one space the rest of the
    // Dock is in. When the Dock becomes scale-aware, the band, the slots, the
    // radius and the hairline move together — and the rule in `CLAUDE.md` is
    // kept by them sharing a conversion, not by this one having its own.
}

/// Rasterise the Dock's surface.
///
/// Returns [`None`] for a panel with no area, which is a Dock that gave way
/// rather than a failure.
pub(crate) fn surface(of: ADocksSurface<'_>) -> Option<Washed> {
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
    let at = |logical: u32| -> f32 { logical as f32 };
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
            // **The badges, clipped to the panel like everything else on it.**
            // One pass over however many there are: a Dock holds a few dozen
            // slots at most, and a per-pixel loop over them is cheaper to read
            // than an index and is run once per layout rather than per frame.
            // A rounded bar: the panel's own distance function with the radius
            // set to half the short side, which makes both ends semicircles.
            let mark = of
                .marks
                .iter()
                .map(|at| {
                    let (mx, my) = (
                        (at.loc.x - area.loc.x) as f32,
                        (at.loc.y - area.loc.y) as f32,
                    );
                    let (w, h) = (at.size.w as f32 / 2.0, at.size.h as f32 / 2.0);
                    let d = rounded(px, py, mx + w, my + h, w, h, w.min(h));
                    (0.5 - d).clamp(0.0, 1.0)
                })
                .fold(0.0_f32, f32::max)
                * inside;
            // Every well, by the same rounded form the composer's field uses.
            let well = under_a_well(of.wells, px, py, area) * inside;
            let well_in_primary = under_a_well(of.in_primary.wells, px, py, area) * inside;
            let badge = of
                .badges
                .iter()
                .map(|at| {
                    let (bx, by) = (
                        (at.loc.x - area.loc.x) as f32 + at.size.w as f32 / 2.0,
                        (at.loc.y - area.loc.y) as f32 + at.size.h as f32 / 2.0,
                    );
                    let radius = at.size.w.min(at.size.h) as f32 / 2.0;
                    let d = ((px - bx).powi(2) + (py - by).powi(2)).sqrt() - radius;
                    (0.5 - d).clamp(0.0, 1.0)
                })
                .fold(0.0_f32, f32::max)
                * inside;

            // **Each ring, by the hairline's own rule.** Two distances a line
            // apart and the difference between them is the stroke: the same
            // arithmetic `within`/`hairline` does for the panel's own edge
            // twenty lines up, which is why there is no second formulation of
            // *an outline* in this file.
            //
            // The radius is the item's corner plus the stand-off, because the
            // ring is outside the item by that much on every side and a ring
            // with the item's own radius would run closer at the corners than
            // along the flats. The design's node measured 16 against a card of
            // 12, which is this sum.
            let ring = on_an_outline(of.rings, px, py, area) * inside;
            let outline_in_primary = on_an_outline(of.in_primary.outlines, px, py, area) * inside;

            // Composited back to front: shadow, fill, hairline, accent.
            let mut colour = [0.0_f32; 3];
            let mut alpha = 0.0_f32;
            for (over, its) in [
                (shadow, THE_SHADOWS_COLOUR),
                (within, of.fill),
                (hairline, of.edge),
                (accent, of.accent),
                // **The wells under the badges**, so a circle drawn on a row
                // sits on it rather than under it.
                (well, of.well_is),
                // **The Primary answer over the plain wells**, so a card's
                // own ground never covers it.
                (well_in_primary, of.in_primary.colour),
                (badge, of.badge),
                // **The mark after the badge**, so a running dot sits on the
                // circle rather than under it.
                (mark, of.mark),
                // **And the focus ring after everything.** It stands outside
                // the item it marks, so it rarely meets anything — but a cell
                // at the edge of the list has a neighbour four away, and the
                // ring is the one thing on this panel that must not be the
                // shape that loses.
                (ring, of.ring_is),
                // **And the keyboard's outline after even the rings**, for
                // the ring's own reason: it is the one mark that tells a
                // person where their next key press will land.
                (outline_in_primary, of.in_primary.colour),
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

/// Rasterise a plain rounded panel: a fill, a hairline and a shadow.
///
/// **The Dock's own surface without the things only a Dock has** — no badges,
/// no composer, no accent. The overflow list is a panel that sits on the canvas
/// exactly as the Dock does, so it gets the same radius, the same hairline and
/// the same shadow, and it had none of them until 2026-10-11: it was one
/// `Solid`, with square corners and nothing under it.
///
/// Both go through [`surface`] rather than a second rasteriser, so a change to
/// the radius or the shadow cannot reach one and miss the other.
pub(crate) fn a_panel(
    panel: Rectangle<i32, Physical>,
    fill: [u8; 3],
    edge: [u8; 3],
    room: (i32, i32),
    on_it: APanelsShapes<'_>,
) -> Option<Washed> {
    surface(ADocksSurface {
        panel,
        fill,
        edge,
        // No accent on a list, and nothing to put one on: `accent_is: 0` is what
        // makes `how_much_accent` return nothing at all.
        accent: fill,
        against: alo_dock::Edge::Bottom,
        accent_is: 0,
        room,
        badges: on_it.circles,
        badge: on_it.circle,
        marks: on_it.marks,
        mark: on_it.mark,
        wells: on_it.wells,
        well_is: on_it.well_is,
        rings: on_it.rings,
        ring_is: on_it.ring_is,
        in_primary: on_it.in_primary,
    })
}

/// The rounded shapes drawn **on** a panel, in its own raster.
///
/// **Not separate rectangles pushed into `solids`.** A circle and a rounded
/// well are partial coverage at their edges, and a `Solid` is one opaque colour
/// over an axis-aligned rectangle — which is why the Dock's own band stopped
/// being two `Solid`s. The popover drew its badges as squares and its search
/// field with square corners for exactly that reason until 2026-10-11, and the
/// fix is not a second rasteriser: it is handing these to the one that already
/// draws the Dock's.
#[derive(Debug, Clone, Copy)]
pub(crate) struct APanelsShapes<'a> {
    /// Circles, each drawn inside the square it is given.
    pub(crate) circles: &'a [Rectangle<i32, Physical>],
    /// What colour they are.
    pub(crate) circle: [u8; 3],
    /// Running and focused marks: rounded bars, each fully round at its ends.
    ///
    /// **In the raster and not in `solids`, for two reasons.** A 4-by-3 bar
    /// with a 1.5 radius is round at both ends, which is partial coverage a
    /// rectangle cannot express. And `crate::desktop_paint` paints every flat
    /// shape *before* any panel, so a mark pushed into `solids` is painted and
    /// then covered by the panel it belongs to — which is exactly what happened
    /// on 2026-10-11: the marks were drawn, the code was right, and a pixel
    /// sample under every badge found nothing at all.
    pub(crate) marks: &'a [Rectangle<i32, Physical>],
    /// What colour they are.
    pub(crate) mark: [u8; 3],
    /// Rounded wells: the search field, or every row of a window list.
    ///
    /// **A list and not one.** It was one, which was right while the only well
    /// was the search field. A window list is a column of them, and drawing
    /// only the first is the kind of fault that looks like a layout problem —
    /// one row with a surface and the rest without.
    pub(crate) wells: &'a [AWell],
    /// What colour it is.
    pub(crate) well_is: [u8; 3],
    /// Keyboard focus rings, each the outer rectangle of one ring.
    pub(crate) rings: &'a [Rectangle<i32, Physical>],
    /// What colour they are.
    pub(crate) ring_is: [u8; 3],
    /// Shapes in `text/primary`: the Primary answer and the keyboard's
    /// outline on an approval card. [`InPrimary::NOTHING`] everywhere else.
    pub(crate) in_primary: InPrimary<'a>,
}

/// How much of the pixel at `(px, py)` is under any of these wells.
///
/// **The one formulation of a well**, shared by every colour of them. Clamped
/// to half the shorter side, so a radius larger than the shape draws a capsule
/// rather than collapsing.
fn under_a_well(wells: &[AWell], px: f32, py: f32, area: Rectangle<i32, Physical>) -> f32 {
    wells
        .iter()
        .map(|well| {
            let at = well.at;
            let (wx, wy) = (
                (at.loc.x - area.loc.x) as f32,
                (at.loc.y - area.loc.y) as f32,
            );
            let (w, h) = (at.size.w as f32 / 2.0, at.size.h as f32 / 2.0);
            let r = (well.corner as f32).min(w).min(h);
            let d = rounded(px, py, wx + w, wy + h, w, h, r);
            (0.5 - d).clamp(0.0, 1.0)
        })
        .fold(0.0_f32, f32::max)
}

/// How much of the pixel at `(px, py)` is on any of these outlines.
///
/// **The one formulation of an outline**, by the hairline's own rule: two
/// distances a line apart, and the difference between them is the stroke. The
/// radius is the item's corner plus the stand-off, because the ring is outside
/// the item by that much on every side — the design's node measured 16 against
/// a card of 12, which is this sum.
fn on_an_outline(
    outlines: &[Rectangle<i32, Physical>],
    px: f32,
    py: f32,
    area: Rectangle<i32, Physical>,
) -> f32 {
    outlines
        .iter()
        .map(|at| {
            let (rx, ry) = (
                (at.loc.x - area.loc.x) as f32,
                (at.loc.y - area.loc.y) as f32,
            );
            let (w, h) = (at.size.w as f32 / 2.0, at.size.h as f32 / 2.0);
            let r = ((THE_COMPOSERS_CORNER + THE_FOCUS_RING_STANDS_OFF) as f32)
                .min(w)
                .min(h);
            let d = rounded(px, py, rx + w, ry + h, w, h, r);
            let outer = (0.5 - d).clamp(0.0, 1.0);
            let inner = (0.5 - (d + THE_FOCUS_RING_IS_THICK as f32)).clamp(0.0, 1.0);
            (outer - inner).clamp(0.0, 1.0)
        })
        .fold(0.0_f32, f32::max)
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
