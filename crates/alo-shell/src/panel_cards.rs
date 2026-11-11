//! **What a put-aside window's card shows in the collapsed rail**, and the
//! expansion control above them, drawn as the design draws them.
//!
//! # What a walk found
//!
//! Measured on a booted machine on 2026-11-11 (the walk of `57b98c15`,
//! `w21-14`): a person minimised Dolphin and the rail at the right edge showed
//! **two plain accent squares** — no icon, no letter, nothing to say which
//! window either was. `crate::panel_raster` painted the expansion control and
//! every slot as the same flat `palette.accent` rectangle, so one window put
//! aside read as two, and neither said what it was.
//!
//! # What the design draws, measured
//!
//! The collapsed rail is `337:23664` (*Fixed icon rail*) in the design file,
//! and its node tree is in `docs/design/figma-snapshot/70-28.xml`:
//!
//! | node | what | measured |
//! |---|---|---|
//! | `358:25693` | the expansion control: a 48 target holding a chevron, **no fill** | the chevron about 4 × 8, in the primary ink, on the rail |
//! | `337:23667` | *Window icon n*: a 48 target | |
//! | `337:23668` | its *Hit area*: a rounded rectangle | 42 × 42, inset 3, corners about 10, `#F2F7F7` |
//! | `337:23669` | its *Symbol* | centred in the hit area |
//!
//! The colours and the corner were sampled from the design's own rendering of
//! that node on 2026-11-11, because its fills are bound to no variable (an
//! empty `get_variable_defs`, recorded in `docs/design/figma-snapshot/`).
//!
//! # The symbol is the Dock's application mark
//!
//! The design's *Symbol* is a placeholder glyph. ADR 0101 makes the Dock and
//! this panel one component family and ADR 0103 three connected surfaces, so an
//! application is marked here exactly as it is on the Dock: **its own icon at
//! `alo_dock::measures::GLYPH`**, or, where it drew none, **its first letter at
//! `AN_INITIAL_IS`** in the badge's ink. A window whose application nobody knows
//! is marked by the first letter of its heading — which for a private window is
//! its safe name, never its title (`alo_put_aside::Preview::headline`).
//!
//! # The ground is the recessed surface
//!
//! `#F2F7F7` is no role in `palette.toml`. The hit area is drawn in
//! `DesktopPalette::badge` — `Role::BgCool`, *a recessed or secondary surface*,
//! `#EEF2F4` — which is the role the Dock's own application ground already uses,
//! and which has a dark value and a high-contrast one where a sampled hex has
//! neither.
//!
//! # A proportion of the slot, never a pixel count
//!
//! Every figure here is a share of the slot it is drawn in — which is already
//! `ICON` through this display's scale and the person's text size — so the
//! card grows with the rail and can never disagree with it.

use alo_dock::artwork::TheArtwork;
use alo_dock::measures::{AN_INITIAL_IS, GLYPH, ICON};
use alo_put_aside::Panel;
use alo_put_aside::the_region_the_panel_claims::WhichEdge;
use cosmic_text::{FontSystem, Metrics};
use smithay::utils::{Physical, Rectangle};

use crate::desktop_look::DesktopPalette;
use crate::painted::Inked;
use crate::panel_raster::PanelPicture;

/// The hit area's side, logical, measured on `337:23668`.
pub(crate) const HIT_AREA: u32 = 42;

/// How round the hit area's corners are, logical, measured on `337:23668`.
pub(crate) const HIT_AREA_ROUNDING: u32 = 10;

/// The expansion control's chevron, logical: how far it reaches across.
pub(crate) const CHEVRON_ACROSS: u32 = 4;

/// The chevron's height, logical.
pub(crate) const CHEVRON_DOWN: u32 = 8;

/// The chevron's stroke, logical.
pub(crate) const CHEVRON_STROKE: u32 = 2;

/// `logical` as a share of a slot `side` pixels across, where the slot is
/// [`ICON`] logical.
fn of_the_slot(side: i32, logical: u32) -> f32 {
    #[expect(
        clippy::cast_precision_loss,
        reason = "a slot is at most a few hundred pixels, far inside an f32's exact range"
    )]
    let side = side as f32;
    #[expect(
        clippy::cast_precision_loss,
        reason = "the logical measures here are two-digit numbers"
    )]
    let share = logical as f32 / ICON as f32;
    side * share
}

/// The cards and the control, as tiles to paint over the rail.
///
/// One tile per slot `picture` laid out, in its order, and one for the control
/// when there is one. Nothing when the rail is not drawn.
pub(crate) fn drawn(
    panel: &Panel,
    picture: &PanelPicture,
    edge: WhichEdge,
    what_each_application_draws: &TheArtwork,
    palette: &DesktopPalette,
    fonts: &mut FontSystem,
) -> Vec<Inked> {
    let mut tiles = Vec::new();
    if let Some(control) = picture.control {
        tiles.push(the_control(control, edge, palette));
    }
    for (slot, preview) in picture.slots.iter().zip(panel.previews()) {
        let mut tile = a_ground(*slot, palette.dock);
        a_rounded_square_onto(&mut tile, palette.badge);
        let side = slot.size.w;
        let glyph = of_the_slot(side, GLYPH).round();
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a glyph inside a slot is a few dozen pixels"
        )]
        let glyph = glyph as i32;
        match preview
            .app()
            .and_then(|app| what_each_application_draws.of(app))
        {
            Some(icon) => an_icon_onto(&mut tile, icon, glyph),
            None => {
                let letter = preview.app().map_or_else(
                    || alo_dock::window::the_first_letter_of(preview.headline().text()),
                    alo_dock::AppId::first_letter,
                );
                a_letter_onto(&mut tile, letter, palette, fonts);
            }
        }
        tiles.push(tile);
    }
    tiles
}

/// A tile over `area`, all of it `ground`.
fn a_ground(area: Rectangle<i32, Physical>, ground: [u8; 3]) -> Inked {
    let count = usize::try_from(area.size.w.max(0) * area.size.h.max(0)).unwrap_or(0);
    Inked {
        area,
        pixels: vec![ground; count],
    }
}

/// `over` mixed onto `under` by `coverage`, nought to one.
fn mixed(under: [u8; 3], over: [u8; 3], coverage: f32) -> [u8; 3] {
    let coverage = coverage.clamp(0.0, 1.0);
    let one = |a: u8, b: u8| {
        let value = f32::from(a) + (f32::from(b) - f32::from(a)) * coverage;
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a mix of two bytes is a byte"
        )]
        let byte = value.round().clamp(0.0, 255.0) as u8;
        byte
    };
    let [under_red, under_green, under_blue] = under;
    let [over_red, over_green, over_blue] = over;
    [
        one(under_red, over_red),
        one(under_green, over_green),
        one(under_blue, over_blue),
    ]
}

/// The pixel at (`x`, `y`) in a tile, by its row and column.
fn at(tile: &mut Inked, x: i32, y: i32) -> Option<&mut [u8; 3]> {
    if x < 0 || y < 0 || x >= tile.area.size.w || y >= tile.area.size.h {
        return None;
    }
    let index = usize::try_from(y * tile.area.size.w + x).ok()?;
    tile.pixels.get_mut(index)
}

/// The hit area: [`HIT_AREA`] square with [`HIT_AREA_ROUNDING`] corners,
/// centred in the tile, with its edge covered partly where it is partly there.
fn a_rounded_square_onto(tile: &mut Inked, colour: [u8; 3]) {
    let side = tile.area.size.w;
    let half = of_the_slot(side, HIT_AREA) / 2.0;
    let radius = of_the_slot(side, HIT_AREA_ROUNDING).min(half);
    #[expect(
        clippy::cast_precision_loss,
        reason = "a slot is at most a few hundred pixels"
    )]
    let middle = side as f32 / 2.0;
    for y in 0..tile.area.size.h {
        for x in 0..side {
            #[expect(
                clippy::cast_precision_loss,
                reason = "a slot is at most a few hundred pixels"
            )]
            let (px, py) = (x as f32 + 0.5 - middle, y as f32 + 0.5 - middle);
            // Signed distance to a rounded square: negative inside.
            let (qx, qy) = (px.abs() - half + radius, py.abs() - half + radius);
            let outside = qx.max(0.0).hypot(qy.max(0.0));
            let distance = outside + qx.max(qy).min(0.0) - radius;
            let coverage = 0.5 - distance;
            if coverage <= 0.0 {
                continue;
            }
            if let Some(pixel) = at(tile, x, y) {
                *pixel = mixed(*pixel, colour, coverage);
            }
        }
    }
}

/// An application's own icon, `glyph` square, centred, over what is there.
fn an_icon_onto(tile: &mut Inked, icon: &alo_dock::artwork::APicture, glyph: i32) {
    let Ok(side) = u32::try_from(glyph) else {
        return;
    };
    if side == 0 {
        return;
    }
    let scaled = icon.scaled_to(side, side);
    let left = (tile.area.size.w - glyph) / 2;
    let top = (tile.area.size.h - glyph) / 2;
    for (index, &[red, green, blue, alpha]) in scaled.pixels().as_chunks::<4>().0.iter().enumerate() {
        let Ok(index) = i32::try_from(index) else {
            return;
        };
        let (x, y) = (left + index % glyph, top + index / glyph);
        if alpha == 0 {
            continue;
        }
        if let Some(pixel) = at(tile, x, y) {
            *pixel = mixed(*pixel, [red, green, blue], f32::from(alpha) / 255.0);
        }
    }
}

/// An application's first letter, at the Dock's initial size, centred on the
/// hit area in the badge's ink.
fn a_letter_onto(tile: &mut Inked, letter: &str, palette: &DesktopPalette, fonts: &mut FontSystem) {
    let side = tile.area.size.w;
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a letter's size is a few dozen pixels"
    )]
    let size = of_the_slot(side, AN_INITIAL_IS).round().max(1.0) as i32;
    #[expect(
        clippy::cast_precision_loss,
        reason = "a letter's size is a few dozen pixels"
    )]
    let metrics = Metrics::new(size as f32, size as f32);
    let shaped = crate::painted_text::centred(
        fonts,
        letter,
        side,
        metrics,
        palette.badge,
        palette.badge_ink,
    );
    let (wide, tall) = (shaped.width_of_box, shaped.height);
    let left = (side - wide) / 2;
    let top = (tile.area.size.h - tall) / 2;
    for (index, ink) in shaped.pixels.iter().enumerate() {
        let Ok(index) = i32::try_from(index) else {
            return;
        };
        if wide <= 0 {
            return;
        }
        if let Some(pixel) = at(tile, left + index % wide, top + index / wide) {
            *pixel = *ink;
        }
    }
}

/// The expansion control: a chevron pointing inward, from the edge the panel
/// is on, in the ink on the rail. **No fill**, as the design draws it — a
/// filled square there is what read as a second window.
fn the_control(area: Rectangle<i32, Physical>, edge: WhichEdge, palette: &DesktopPalette) -> Inked {
    let mut tile = a_ground(area, palette.dock);
    let side = area.size.w;
    let across = of_the_slot(side, CHEVRON_ACROSS);
    let down = of_the_slot(side, CHEVRON_DOWN);
    let stroke = of_the_slot(side, CHEVRON_STROKE).max(1.0);
    #[expect(
        clippy::cast_precision_loss,
        reason = "a slot is at most a few hundred pixels"
    )]
    let middle = side as f32 / 2.0;
    // Pointing inward: away from the edge the panel stands at.
    let (tip_x, back_x) = match edge {
        WhichEdge::Right => (middle - across / 2.0, middle + across / 2.0),
        WhichEdge::Left => (middle + across / 2.0, middle - across / 2.0),
    };
    let tip = (tip_x, middle);
    let top = (back_x, middle - down / 2.0);
    let bottom = (back_x, middle + down / 2.0);
    for y in 0..tile.area.size.h {
        for x in 0..side {
            #[expect(
                clippy::cast_precision_loss,
                reason = "a slot is at most a few hundred pixels"
            )]
            let point = (x as f32 + 0.5, y as f32 + 0.5);
            let distance = from_a_segment(point, top, tip).min(from_a_segment(point, tip, bottom));
            let coverage = stroke / 2.0 + 0.5 - distance;
            if coverage <= 0.0 {
                continue;
            }
            if let Some(pixel) = at(&mut tile, x, y) {
                *pixel = mixed(*pixel, palette.ink, coverage);
            }
        }
    }
    tile
}

/// How far `point` is from the segment `from`–`to`.
fn from_a_segment(point: (f32, f32), from: (f32, f32), to: (f32, f32)) -> f32 {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let length = dx.mul_add(dx, dy * dy);
    let along = if length <= f32::EPSILON {
        0.0
    } else {
        ((point.0 - from.0).mul_add(dx, (point.1 - from.1) * dy) / length).clamp(0.0, 1.0)
    };
    let (nearest_x, nearest_y) = (along.mul_add(dx, from.0), along.mul_add(dy, from.1));
    (point.0 - nearest_x).hypot(point.1 - nearest_y)
}

#[cfg(test)]
#[path = "panel_cards_tests.rs"]
mod tests;
