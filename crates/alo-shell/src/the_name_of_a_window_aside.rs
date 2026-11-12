//! **The name of a window put aside, beside its square while a person points
//! at it** — the approved design `27:2` (`27:39`): a navy pill, corner 8,
//! padding 10 × 6, the name in 12 SemiBold white, 10 inward of the rail and
//! centred on the square. *Home · Dolphin*: the window's own title, which for a
//! private window is its safe name and never its title
//! (`alo_put_aside::Preview::headline`).
//!
//! **It appears straight away**, with no timer, as the Dock's names do
//! (`crate::dock_tooltip`): reaching a name must not depend on how fast
//! somebody moves. **Its colours are the Dock's name's**, `Role::TextPrimary`
//! behind `Role::BgSurface` words, in either scheme.
//!
//! Every figure is a share of the slot it names (`crate::miniature_measures`),
//! so it grows with the rail through the display's scale and the person's text
//! size. Toward the canvas: left of a right-hand panel, right of a left-hand
//! one.

use alo_put_aside::the_region_the_panel_claims::WhichEdge;
use cosmic_text::{FontSystem, Metrics};
use smithay::utils::{Physical, Rectangle};

use crate::a_window_in_miniature::of_the_slot;
use crate::miniature_measures::{
    NAME_ACROSS, NAME_DOWN, NAME_FROM_THE_RAIL, NAME_ROUNDING, NAME_TEXT,
};
use crate::painted::Inked;
use crate::panel_cards::{at, mixed};

/// The pill naming the window in `slot`, on a display `width` wide.
///
/// [`None`] for a name with nothing to draw, or a pill that would not fit on
/// the display beside the rail.
#[must_use]
pub(crate) fn the_name_beside(
    slot: Rectangle<i32, Physical>,
    edge: WhichEdge,
    name: &str,
    width: i32,
    fonts: &mut FontSystem,
) -> Option<Inked> {
    if name.trim().is_empty() {
        return None;
    }
    let side = slot.size.w;
    let px = |logical| of_the_slot(side, logical);
    let fill = alo_appearance::Role::TextPrimary.colour();
    let fill = [fill.red(), fill.green(), fill.blue()];
    let ink = alo_appearance::Role::BgSurface.colour();
    let ink = [ink.red(), ink.green(), ink.blue()];
    let size = px(NAME_TEXT).max(1);
    #[expect(
        clippy::cast_precision_loss,
        reason = "a name's size is a few dozen pixels"
    )]
    let metrics = Metrics::new(size as f32, size as f32);
    // **At most the room toward the canvas**, less its padding: a long title
    // is cut by the shaping, never run off the display.
    let beside = slot.loc.x - px(alo_dock::measures::MARGIN) - px(NAME_FROM_THE_RAIL);
    let room = match edge {
        WhichEdge::Right => beside,
        WhichEdge::Left => width - (slot.loc.x + side + px(alo_dock::measures::MARGIN)),
    } - 2 * px(NAME_ACROSS);
    if room <= 0 {
        return None;
    }
    let words =
        crate::painted_text::sentence(fonts, name, room, metrics, fill, ink).cropped_to_its_ink();
    let (wide, tall) = (
        words.width_of_box + 2 * px(NAME_ACROSS),
        words.height + 2 * px(NAME_DOWN),
    );
    let left = match edge {
        WhichEdge::Right => beside - wide,
        WhichEdge::Left => {
            slot.loc.x + side + px(alo_dock::measures::MARGIN) + px(NAME_FROM_THE_RAIL)
        }
    };
    let top = slot.loc.y + (slot.size.h - tall) / 2;
    if left < 0 || left + wide > width || wide <= 0 || tall <= 0 {
        return None;
    }
    // The ground behind the pill is not known here, so the pill's own corners
    // are blended onto the colour of the canvas it sits over, a light ground.
    let behind = {
        let surface = alo_appearance::Role::BgSurface.colour();
        [surface.red(), surface.green(), surface.blue()]
    };
    let count = usize::try_from(wide * tall).ok()?;
    let mut pill = Inked {
        area: crate::painted::drawn_at(Rectangle::new((left, top).into(), (wide, tall).into())),
        pixels: vec![behind; count],
    };
    #[expect(
        clippy::cast_precision_loss,
        reason = "a pill is a few hundred pixels at most"
    )]
    let (half_w, half_h, radius) = (
        wide as f32 / 2.0,
        tall as f32 / 2.0,
        (px(NAME_ROUNDING) as f32).min(tall as f32 / 2.0),
    );
    for y in 0..tall {
        for x in 0..wide {
            #[expect(
                clippy::cast_precision_loss,
                reason = "a pill is a few hundred pixels at most"
            )]
            let (dx, dy) = (x as f32 + 0.5 - half_w, y as f32 + 0.5 - half_h);
            let (qx, qy) = (dx.abs() - half_w + radius, dy.abs() - half_h + radius);
            let distance = qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - radius;
            if let Some(pixel) = at(&mut pill, x, y) {
                *pixel = mixed(*pixel, fill, 0.5 - distance);
            }
        }
    }
    let (from_x, from_y) = (px(NAME_ACROSS), px(NAME_DOWN));
    for (index, colour) in words.pixels.iter().enumerate() {
        let index = i32::try_from(index).ok()?;
        if words.width_of_box <= 0 {
            break;
        }
        let (x, y) = (
            from_x + index % words.width_of_box,
            from_y + index / words.width_of_box,
        );
        if let Some(pixel) = at(&mut pill, x, y) {
            *pixel = *colour;
        }
    }
    Some(pill)
}

impl crate::Server {
    /// **The name of the window put aside under the pointer**, drawn beside
    /// its square, from the panel as it is now and as it was last laid out.
    /// [`None`] with the pointer on no window's square.
    pub(crate) fn the_name_pointed_at_in_the_panel(
        &self,
        panel: &alo_put_aside::Panel,
        (edge, width): (WhichEdge, i32),
        fonts: &mut FontSystem,
    ) -> Option<Inked> {
        let crate::which_preview_the_pointer_is_on::OnThePanel::APreview(id) =
            self.where_the_pointer_is_on_the_panel(panel)?
        else {
            return None;
        };
        let (_, slot) = self
            .panel_as_drawn
            .as_ref()?
            .the_windows_slots()
            .find(|(drawn, _)| *drawn == id)?;
        let preview = panel
            .previews()
            .iter()
            .find(|preview| preview.window() == id)?;
        the_name_beside(slot, edge, preview.headline().text(), width, fonts)
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_name_of_a_window_aside_tests.rs"]
mod tests;
