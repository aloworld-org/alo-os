//! **A window put aside shows as a live miniature of itself**, with its
//! application's badge in the lower corner — the owner's approved design
//! `10 / Minimised windows, Mac-style` (`27:2`, 2026-11-12), whose owner chose
//! *live* over a still picture.
//!
//! # Drawn by the renderer from the window's own buffer, every frame
//!
//! The miniature is not a copy. Each frame the window's own surfaces are
//! imported exactly as a window on the canvas is (`crate::drawing::import_at`)
//! and drawn **scaled into the miniature's rectangle and clipped to it**, by
//! the one painter both renderers share (`crate::scene_drawing`). So it shows
//! whatever the application last drew: if the application draws again, the
//! miniature changes with it, and nothing here keeps a picture that could go
//! stale. How often an application is *asked* to draw while it is aside is
//! `crate::put_aside_windows_draw_again`'s, and bounded there.
//!
//! **Its surfaces are not counted as shown.** A window on the canvas enters the
//! display and is sent a frame each time it is drawn; a miniature's imports are
//! drawn and dropped, so a window put aside is not told it is on the screen at
//! the display's full rate, which would make a 44-pixel picture cost what the
//! whole window costs.
//!
//! # Where everything is, from the slot alone
//!
//! [`in_the_slot`] is the one answer to *where is the miniature and where is
//! the badge* for a slot the panel laid out: `crate::panel_cards` paints the
//! card and the badge from it and the painter draws the window into it, so the
//! two cannot disagree by a pixel. Every figure is a share of the slot
//! (`crate::miniature_measures`), and the slot is already through the
//! display's scale and the person's text size.
//!
//! # A private window gets no miniature
//!
//! [`alo_put_aside::Privacy::Private`] — a window the person marked — is drawn
//! as its application's icon on a blank card, and **the painter is never handed
//! its surfaces**: [`crate::Server::the_miniatures`] leaves it out, so there is
//! no picture of it to clip wrongly. Every other window was on the canvas,
//! visible, the moment before it was put aside; its miniature shows what the
//! window would show there, at a forty-fourth of the size.

use alo_put_aside::Panel;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Physical, Point, Rectangle};

use crate::miniature_measures::{
    BADGE, BADGE_AT, MINIATURE_BORDER, MINIATURE_LEFT, MINIATURE_TALL, MINIATURE_TOP,
    MINIATURE_WIDE,
};

/// Where the parts of one window's square are, on the display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TheSquaresParts {
    /// The miniature's card: its border, its ground, and the picture inside.
    pub(crate) card: Rectangle<i32, Physical>,
    /// The application's badge, over the card's lower corner.
    pub(crate) badge: Rectangle<i32, Physical>,
    /// The card's border, never thinner than one pixel.
    pub(crate) border: i32,
}

/// `logical` as pixels of a slot `side` pixels across, the slot being
/// `alo_dock::measures::ICON` logical.
#[must_use]
pub(crate) fn of_the_slot(side: i32, logical: u32) -> i32 {
    let icon = i64::from(alo_dock::measures::ICON);
    let wanted = i64::from(side) * i64::from(logical);
    // Rounded to the nearest pixel, half away from nought.
    i32::try_from((wanted * 2 + icon) / (icon * 2)).unwrap_or(0)
}

/// Where the card and the badge are in `slot`.
#[must_use]
pub(crate) fn in_the_slot(slot: Rectangle<i32, Physical>) -> TheSquaresParts {
    let side = slot.size.w;
    let px = |logical| of_the_slot(side, logical);
    TheSquaresParts {
        card: Rectangle::new(
            Point::from((
                slot.loc.x + px(MINIATURE_LEFT),
                slot.loc.y + px(MINIATURE_TOP),
            )),
            (px(MINIATURE_WIDE), px(MINIATURE_TALL)).into(),
        ),
        badge: Rectangle::new(
            Point::from((slot.loc.x + px(BADGE_AT), slot.loc.y + px(BADGE_AT))),
            (px(BADGE), px(BADGE)).into(),
        ),
        border: px(MINIATURE_BORDER).max(1),
    }
}

/// One window drawn small: which surfaces, where, how small, and the
/// rectangles it may touch.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AMiniature {
    /// The window's root surface; its children come with it.
    pub(crate) surface: WlSurface,
    /// Where the root's buffer origin lands, so that the window's own
    /// geometry — not its shadow — fills the picture.
    pub(crate) origin: Point<f64, Physical>,
    /// How small: the window's pixels times this are display pixels.
    pub(crate) scale: f64,
    /// The rectangles it is drawn into: the picture, less the badge, so the
    /// badge painted with the card stays over it.
    pub(crate) clip: Vec<Rectangle<i32, Physical>>,
}

/// Where a window `size` big is drawn inside a square's card: the largest
/// picture that fits inside the card's border without stretching, centred;
/// and how small that is.
///
/// [`None`] for a window or a card with no size, which has no picture.
#[must_use]
pub(crate) fn the_picture_in(
    parts: TheSquaresParts,
    size: (f64, f64),
) -> Option<(Rectangle<i32, Physical>, f64)> {
    let (card, border) = (parts.card, parts.border);
    let room = (
        f64::from(card.size.w - 2 * border),
        f64::from(card.size.h - 2 * border),
    );
    let (wide, tall) = size;
    if !(wide > 0.0 && tall > 0.0 && room.0 > 0.0 && room.1 > 0.0)
        || !wide.is_finite()
        || !tall.is_finite()
    {
        return None;
    }
    let scale = (room.0 / wide).min(room.1 / tall);
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a picture inside a card is at most the card, a few dozen pixels"
    )]
    let (w, h) = ((wide * scale).round() as i32, (tall * scale).round() as i32);
    let left = card.loc.x + border + (card.size.w - 2 * border - w) / 2;
    let top = card.loc.y + border + (card.size.h - 2 * border - h) / 2;
    Some((Rectangle::new((left, top).into(), (w, h).into()), scale))
}

/// `picture` less `badge`: the rectangles a miniature may be drawn into so
/// the badge over its corner is never painted over.
#[must_use]
pub(crate) fn round_the_badge(
    picture: Rectangle<i32, Physical>,
    badge: Rectangle<i32, Physical>,
) -> Vec<Rectangle<i32, Physical>> {
    let Some(overlap) = picture.intersection(badge) else {
        return vec![picture];
    };
    let (left, top) = (picture.loc.x, picture.loc.y);
    let (right, bottom) = (left + picture.size.w, top + picture.size.h);
    let (o_left, o_top) = (overlap.loc.x, overlap.loc.y);
    let (o_right, o_bottom) = (o_left + overlap.size.w, o_top + overlap.size.h);
    let pieces = [
        // Above the overlap, the whole width.
        (left, top, right, o_top),
        // Below it, the whole width.
        (left, o_bottom, right, bottom),
        // Beside it, left and right, the overlap's height.
        (left, o_top, o_left, o_bottom),
        (o_right, o_top, right, o_bottom),
    ];
    pieces
        .into_iter()
        .filter(|(l, t, r, b)| r > l && b > t)
        .map(|(l, t, r, b)| Rectangle::new((l, t).into(), (r - l, b - t).into()))
        .collect()
}

/// **Whether this window put aside may be drawn small at all**: not when the
/// person marked it private. The one answer the painter's list and the asking
/// to draw again (`crate::put_aside_windows_draw_again`) both read.
#[must_use]
pub(crate) const fn it_may_be_drawn_small(preview: &alo_put_aside::Preview) -> bool {
    !preview.privacy().is_private()
}

impl crate::Server {
    /// **The miniatures to draw this frame**: one for each application's window
    /// in the panel as it was last laid out, that is still put aside, is not
    /// private, and has a size.
    #[must_use]
    pub(crate) fn the_miniatures(&self, panel: &Panel) -> Vec<AMiniature> {
        let Some(drawn) = self.panel_as_drawn.as_ref() else {
            return Vec::new();
        };
        drawn
            .the_windows_slots()
            .filter_map(|(id, slot)| {
                // **Still in the panel, and not private** — asked of the panel
                // as it is now, so a window the person marked private, or one
                // brought back since the draw, is never drawn small.
                let preview = panel
                    .previews()
                    .iter()
                    .find(|preview| preview.window() == id)?;
                if !it_may_be_drawn_small(preview) {
                    return None;
                }
                // **Nor one whose miniature the renderer refused**: its square
                // shows its application's mark instead
                // (`crate::a_picture_the_renderer_refuses`).
                if self.refused_pictures.is_drawn_by_its_mark(id.number()) {
                    return None;
                }
                let surface = self.the_put_aside_window_numbered(id)?;
                let geometry = crate::scene::geometry(&surface);
                let parts = in_the_slot(slot);
                let (picture, scale) = the_picture_in(parts, (geometry.size.w, geometry.size.h))?;
                let origin = Point::from((
                    f64::from(picture.loc.x) - geometry.loc.x * scale,
                    f64::from(picture.loc.y) - geometry.loc.y * scale,
                ));
                Some(AMiniature {
                    surface,
                    origin,
                    scale,
                    clip: round_the_badge(picture, parts.badge),
                })
            })
            .collect()
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "a_window_in_miniature_tests.rs"]
mod tests;
