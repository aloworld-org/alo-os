//! **What an application's window put aside shows in its square**: the card
//! its live miniature is drawn into, and its application's badge over the
//! card's lower corner — or, for a private window, the application's icon on a
//! blank card. The approved design `27:2`; the measures are
//! `crate::miniature_measures` and the places `crate::a_window_in_miniature`.
//!
//! **The picture itself is not painted here.** This is the raster under and
//! over it: the card's border and ground, and the badge. The painter draws the
//! window into the card between the two, clipped round the badge, so the badge
//! stays on top (`crate::scene_drawing`).
//!
//! # The badge is the application's own icon, or its letter
//!
//! The design draws a coloured tile with the application's letter as a stand-in
//! for its icon. An application that has an icon of its own is badged with it,
//! inside the badge's white border; one that has none is badged with its
//! letter on the recessed ground every fallback badge on this machine uses
//! (`DesktopPalette::badge`, the owner's ruling of 2026-10-11) — never a colour
//! taken from the application, which the same ruling refuses.

use alo_dock::artwork::{APicture, TheArtwork};
use alo_put_aside::Preview;
use cosmic_text::{FontSystem, Metrics};
use smithay::utils::{Physical, Rectangle};

use crate::a_window_in_miniature::{in_the_slot, of_the_slot};
use crate::desktop_look::DesktopPalette;
use crate::miniature_measures::{
    BADGE_BORDER_HALVES, BADGE_LETTER, BADGE_ROUNDING, MINIATURE_ROUNDING,
};
use crate::painted::Inked;
use crate::panel_cards::{a_ground, at, mixed};

/// A rectangle inside a tile, in the tile's own pixels, as floats.
#[derive(Debug, Clone, Copy)]
struct Inside {
    /// Its left edge, from the tile's.
    left: f32,
    /// Its top edge, from the tile's.
    top: f32,
    /// How wide it is.
    wide: f32,
    /// How tall it is.
    tall: f32,
}

impl Inside {
    /// `area`, a rectangle on the display, inside `tile`.
    #[expect(
        clippy::cast_precision_loss,
        reason = "a square in the panel is a few dozen pixels"
    )]
    fn of(area: Rectangle<i32, Physical>, tile: &Inked) -> Self {
        Self {
            left: (area.loc.x - tile.area.loc.x) as f32,
            top: (area.loc.y - tile.area.loc.y) as f32,
            wide: area.size.w as f32,
            tall: area.size.h as f32,
        }
    }

    /// The same rectangle, `by` pixels smaller on every side.
    fn inset(self, by: f32) -> Self {
        Self {
            left: self.left + by,
            top: self.top + by,
            wide: (self.wide - 2.0 * by).max(0.0),
            tall: (self.tall - 2.0 * by).max(0.0),
        }
    }
}

/// Fill `inside` with `colour`, its corners rounded by `radius`, its edge
/// covered partly where it is partly there.
fn a_rounded_rectangle_onto(tile: &mut Inked, inside: Inside, radius: f32, colour: [u8; 3]) {
    let (half_w, half_h) = (inside.wide / 2.0, inside.tall / 2.0);
    if half_w <= 0.0 || half_h <= 0.0 {
        return;
    }
    let radius = radius.min(half_w).min(half_h).max(0.0);
    let (middle_x, middle_y) = (inside.left + half_w, inside.top + half_h);
    for y in 0..tile.area.size.h {
        for x in 0..tile.area.size.w {
            #[expect(
                clippy::cast_precision_loss,
                reason = "a square in the panel is a few dozen pixels"
            )]
            let (px, py) = (x as f32 + 0.5 - middle_x, y as f32 + 0.5 - middle_y);
            let (qx, qy) = (px.abs() - half_w + radius, py.abs() - half_h + radius);
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

/// An application's icon scaled into `inside`, over what is there.
fn an_icon_into(tile: &mut Inked, icon: &APicture, inside: Inside) {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "an icon in a square is a few dozen pixels, never negative"
    )]
    let side = inside.wide.min(inside.tall).round().max(0.0) as u32;
    if side == 0 {
        return;
    }
    let scaled = icon.scaled_to(side, side);
    let Ok(across) = i32::try_from(side) else {
        return;
    };
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a position inside a square is a few dozen pixels"
    )]
    let (left, top) = (
        (inside.left + (inside.wide - inside.wide.min(inside.tall)) / 2.0).round() as i32,
        (inside.top + (inside.tall - inside.wide.min(inside.tall)) / 2.0).round() as i32,
    );
    for (index, &[red, green, blue, alpha]) in scaled.pixels().as_chunks::<4>().0.iter().enumerate()
    {
        let Ok(index) = i32::try_from(index) else {
            return;
        };
        if alpha == 0 {
            continue;
        }
        if let Some(pixel) = at(tile, left + index % across, top + index / across) {
            *pixel = mixed(*pixel, [red, green, blue], f32::from(alpha) / 255.0);
        }
    }
}

/// `letter` centred in `inside`, `size` pixels tall, `ink` on `ground`.
fn a_letter_into(
    tile: &mut Inked,
    letter: &str,
    inside: Inside,
    size: i32,
    (ground, ink): ([u8; 3], [u8; 3]),
    fonts: &mut FontSystem,
) {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a badge is a few dozen pixels"
    )]
    let box_wide = inside.wide.round() as i32;
    if box_wide <= 0 || size <= 0 || letter.trim().is_empty() {
        return;
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "a letter's size is a few dozen pixels"
    )]
    let metrics = Metrics::new(size as f32, size as f32);
    let shaped = crate::painted_text::centred(fonts, letter, box_wide, metrics, ground, ink);
    let (wide, tall) = (shaped.width_of_box, shaped.height);
    if wide <= 0 {
        return;
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a position inside a square is a few dozen pixels"
    )]
    let (left, top) = (
        (inside.left + (inside.wide - wide as f32) / 2.0).round() as i32,
        (inside.top + (inside.tall - tall as f32) / 2.0).round() as i32,
    );
    for (index, ink) in shaped.pixels.iter().enumerate() {
        let Ok(index) = i32::try_from(index) else {
            return;
        };
        if let Some(pixel) = at(tile, left + index % wide, top + index / wide) {
            *pixel = *ink;
        }
    }
}

/// The square of an application's window put aside, laid out in `slot`, on
/// the rail's ground.
pub(crate) fn a_windows_square(
    slot: Rectangle<i32, Physical>,
    preview: &Preview,
    what_each_application_draws: &TheArtwork,
    palette: &DesktopPalette,
    fonts: &mut FontSystem,
) -> Inked {
    let mut tile = a_ground(slot, palette.dock);
    let parts = in_the_slot(slot);
    let side = slot.size.w;
    #[expect(
        clippy::cast_precision_loss,
        reason = "a square in the panel is a few dozen pixels"
    )]
    let px = |logical: u32| of_the_slot(side, logical) as f32;
    #[expect(
        clippy::cast_precision_loss,
        reason = "a border is a pixel or two"
    )]
    let border = parts.border as f32;
    // **The card**: the hairline, then the window's ground inside it. The
    // picture is drawn over the ground by the painter; a private window's
    // card stays blank and carries the icon instead.
    let card = Inside::of(parts.card, &tile);
    a_rounded_rectangle_onto(&mut tile, card, px(MINIATURE_ROUNDING), palette.edge);
    a_rounded_rectangle_onto(
        &mut tile,
        card.inset(border),
        (px(MINIATURE_ROUNDING) - border).max(0.0),
        palette.ground,
    );
    let icon = preview
        .app()
        .and_then(|app| what_each_application_draws.of(app));
    let letter = preview.app().map_or_else(
        || alo_dock::window::the_first_letter_of(preview.headline().text()),
        alo_dock::AppId::first_letter,
    );
    if preview.privacy().is_private() {
        // **A private window: the application's mark on a blank card**, the
        // size the rail draws an icon at, and no badge — there is no picture
        // for one to sit over.
        let mark = card.inset(border * 2.0);
        match icon {
            Some(icon) => an_icon_into(&mut tile, icon, mark),
            None => a_letter_into(
                &mut tile,
                letter,
                mark,
                of_the_slot(side, alo_dock::measures::AN_INITIAL_IS) / 2,
                (palette.ground, palette.badge_ink),
                fonts,
            ),
        }
        return tile;
    }
    // **The badge**: a white border round the application's own icon, or its
    // letter on the fallback badge's ground.
    let badge = Inside::of(parts.badge, &tile);
    let white = px(BADGE_BORDER_HALVES) / 2.0;
    a_rounded_rectangle_onto(&mut tile, badge, px(BADGE_ROUNDING), palette.ground);
    let inner = badge.inset(white);
    let inner_rounding = (px(BADGE_ROUNDING) - white).max(0.0);
    match icon {
        Some(icon) => {
            a_rounded_rectangle_onto(&mut tile, inner, inner_rounding, palette.ground);
            an_icon_into(&mut tile, icon, inner);
        }
        None => {
            a_rounded_rectangle_onto(&mut tile, inner, inner_rounding, palette.badge);
            a_letter_into(
                &mut tile,
                letter,
                inner,
                of_the_slot(side, BADGE_LETTER),
                (palette.badge, palette.badge_ink),
                fonts,
            );
        }
    }
    tile
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "a_miniatures_card_tests.rs"]
mod tests;
