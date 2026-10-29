//! Where the Dock is on one display, worked out once for everybody who asks.
//!
//! **One answer, two readers**, which is `alo_dock::places`'s own rule one level
//! up: *a drawing that put an icon somewhere and a click that looked for it
//! somewhere else would be two answers to one question, and the second would be
//! wrong in a way nobody could see — a person pressing a picture and reaching
//! the application beside it.*
//!
//! That crate already holds the arithmetic along the bar. What it refuses to
//! know is where the bar itself is, because it refuses to know about a screen.
//! This file is the other half: the band on this display's edge, and the slots
//! inside it, in the display's own pixels.
//!
//! # Where the logical measures meet the display
//!
//! `alo_dock` lays the bar out in logical measures — a slot is 48, the band 76
//! — and this is the boundary where those meet a display. Every one of them
//! goes through that display's `crate::display_scale::DisplayScale`, so a
//! two-times screen draws the same Dock twice as large in pixels and sharper,
//! rather than half the size. That was the owner's decision of 2026-10-28; until
//! then the band, the slots and the gaps reached every display as raw pixels.
//!
//! **`places` stays logical**, because it is `alo_dock`'s answer and a press is
//! asked of it: [`TheDocksPlaces::how_far_along`] turns a pixel back into a
//! logical distance, and the raster turns each slot forward through the same
//! scale. The band is in pixels, because it is a rectangle on a screen.
//!
//! # Why it is here and not inside the raster
//!
//! `crate::dock_raster` computed all of this and kept none of it, so the only
//! way to ask *what is under the pointer* would have been to rasterise a frame —
//! which needs a font system and a palette to answer a question about a
//! rectangle. Worse, the obvious shortcut is for the hit test to work the band
//! out **again**, and then there are two placements that must agree and nothing
//! that makes them.
//!
//! So the geometry moved here, the raster reads it, and a press reads the same
//! thing. Neither can be right while the other is wrong.
//!
//! # It is computed, never stored
//!
//! There is no `Dock` geometry on the `Server`. A band is a function of the
//! display's size and the person's chosen edge and what the Dock is holding, all
//! of which the caller already has, and a copy kept beside them is a second home
//! that can go stale — which `tests/the_camera_has_one_home.rs` is this crate's
//! long note about.

use alo_dock::places::Places;
use alo_dock::{Dock, Layout, OnTheDock, Room, Screen};
use smithay::utils::{Physical, Point, Rectangle};

use crate::RenderError;

/// The largest display side, in pixels, the dock is laid out for.
pub(crate) const LARGEST_SIDE: i32 = 16_384;

/// Where the Dock is on one display, and what is in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TheDocksPlaces {
    /// `alo-dock`'s layout for this display.
    pub(crate) layout: Layout,
    /// The display's scale, which every logical figure below went through.
    ///
    /// Kept with the answer so a press reads the slots at the scale they were
    /// drawn at, rather than asking the display a second time.
    pub(crate) scale: crate::display_scale::DisplayScale,
    /// The whole band, on the person's chosen edge.
    pub(crate) band: Rectangle<i32, Physical>,
    /// How thick the band is in pixels, already clamped to the display.
    pub(crate) thickness: i32,
    /// Every slot along the bar: the applications, then the overflow control if
    /// there is one — in **logical** distances along it, as `alo_dock` gives
    /// them. [`Self::scale`] turns one into pixels.
    pub(crate) places: Places,
    /// What there was no room for, in order, behind that control.
    pub(crate) over: Vec<OnTheDock>,
}

impl TheDocksPlaces {
    /// How far along the bar this point is, in logical units, or [`None`] if it
    /// is not on the bar at all.
    ///
    /// **Logical, because `places` is.** The point is in the display's pixels and
    /// the slots are in `alo_dock`'s measures, so the distance is divided by the
    /// display's scale here, once, before any slot is asked about it.
    ///
    /// **Along, not `x`.** `alo_dock::Places` lays the slots out as distances
    /// from the bar's own start without knowing which edge the bar is on, so
    /// this is the one place that turns a point on a screen into that distance.
    /// A dock down the left of the screen measures down; one along the bottom
    /// measures across, and getting that wrong on the two edges nobody draws by
    /// default is exactly how it would go unnoticed.
    ///
    /// **Half-open at both ends**, like `alo_dock::APlace::holds`: the first
    /// pixel of the band is on it and the first pixel past it is not, so a press
    /// on the seam between the Dock and the canvas belongs to one of them.
    ///
    /// *This was written, tested on four edges and taken out again before
    /// landing on 2026-10-10, because nothing pressed the Dock. It is back with
    /// its caller.*
    #[must_use]
    pub(crate) fn how_far_along(&self, at: Point<i32, Physical>) -> Option<u32> {
        let inside = at.x >= self.band.loc.x
            && at.y >= self.band.loc.y
            && at.x < self.band.loc.x + self.band.size.w
            && at.y < self.band.loc.y + self.band.size.h;
        if !inside {
            return None;
        }
        let along = if self.layout.edge().runs_across() {
            at.x - self.band.loc.x
        } else {
            at.y - self.band.loc.y
        };
        (along >= 0).then(|| self.scale.logical(along))
    }
}

/// Where `dock` sits on a display of `size` drawn at `scale`, holding
/// `on_the_dock`.
///
/// # Errors
/// [`RenderError::DesktopScene`] for a display `alo-dock` refuses to lay a dock
/// out on, or one larger than any this dock is laid out for.
pub(crate) fn where_the_dock_is(
    dock: &Dock,
    size: (i32, i32),
    on_the_dock: &[OnTheDock],
    scale: crate::display_scale::DisplayScale,
) -> Result<TheDocksPlaces, RenderError> {
    let (width, height) = size;
    if width > LARGEST_SIDE || height > LARGEST_SIDE {
        return Err(RenderError::DesktopScene);
    }
    let screen = Screen::of(
        u32::try_from(width).map_err(|_| RenderError::DesktopScene)?,
        u32::try_from(height).map_err(|_| RenderError::DesktopScene)?,
    )
    .map_err(|_| RenderError::DesktopScene)?;
    let layout = dock.layout_on(screen);
    let thickness = scale.room(layout.thickness()).clamp(1, width.min(height));

    let margin = scale.px(alo_dock::measures::MARGIN);
    let floating = scale.px(alo_dock::measures::FLOATING_ABOVE_THE_EDGE);
    // **The edge the bar runs along, not always the width.** A dock down a side
    // is as long as the screen is tall, and measuring it against the width would
    // make a portrait screen's side dock short for a reason that has nothing to
    // do with where it is.
    let along = if layout.edge().runs_across() {
        width
    } else {
        height
    };
    let widest = (along - 2 * margin).max(1);

    // The slot count comes from the edge, which is why `alo_dock::fit` can be
    // called here and nowhere above: `alo-dock` knows how wide a bar holding `n`
    // is and refuses to know about a screen, and the caller knows about windows
    // and not about edges.
    // **The room the composer and the commands take comes off first.** The bar
    // is not only icons: since 2026-10-11 it carries the field a person types
    // into and the one or two controls after it, measured on the design file's
    // own Docks. So what is left for applications is the edge less that group,
    // and `how_many_fit` is asked about the remainder.
    let commands =
        scale.room(Places::of(&alo_dock::fit(Vec::new(), 0), layout.edge()).how_far_across());
    // `commands` carries both of the bar's own margins and the gap *before* the
    // composer is not in it, so both come back: `how_many_fit` adds the margins
    // itself, and the gap between the last application and the composer is the
    // one piece neither of them accounts for.
    let gap = scale.px(alo_dock::measures::GAP);
    let for_applications = (widest - commands + 2 * margin - gap).max(1);
    // **Back to logical before `alo_dock` is asked**, because it counts in its
    // own measures: the room is in pixels and a slot is 48 logical.
    let at_most = Room::pixels(scale.logical(for_applications)).how_many_fit();
    let fitted = alo_dock::fit(on_the_dock.to_vec(), at_most);
    let places = Places::of(&fitted, layout.edge());
    let wanted = scale.room(places.how_far_across());
    // Clamped as a last resort: `at_most` came from `widest`, so `wanted` cannot
    // exceed it except on a display too small for one slot, where there is
    // nothing to choose — or by a pixel of rounding at a fractional scale.
    let bar = wanted.clamp(1, widest);

    Ok(TheDocksPlaces {
        layout,
        scale,
        band: the_band_on(layout.edge(), size, bar, thickness, floating),
        thickness,
        places,
        over: fitted.over().to_vec(),
    })
}

/// Where the bar sits on this edge: `along` units down its edge, `thickness`
/// across, lifted `floating` clear of the screen's own edge.
///
/// **One function for four edges rather than four placements.** The bar is the
/// same rectangle each time — as long as what it holds, as thick as its lane —
/// and only two things vary: which axis the length runs along, and which end of
/// the other axis it is lifted from. Written as four `if`s in the caller, the two
/// that are never drawn in this release would be the two nobody notices going
/// wrong.
///
/// **`floating` is the gap, not the position.** The bar is inset from its edge
/// rather than flush to it, which is the design file's rule for every edge and
/// the reason the egress corner has to subtract it too.
pub(crate) fn the_band_on(
    edge: alo_dock::Edge,
    size: (i32, i32),
    along: i32,
    thickness: i32,
    floating: i32,
) -> Rectangle<i32, Physical> {
    let (width, height) = size;
    let origin = match edge {
        alo_dock::Edge::Bottom => ((width - along) / 2, height - thickness - floating),
        alo_dock::Edge::Top => ((width - along) / 2, floating),
        alo_dock::Edge::Left => (floating, (height - along) / 2),
        alo_dock::Edge::Right => (width - thickness - floating, (height - along) / 2),
    };
    let extent = if edge.runs_across() {
        (along, thickness)
    } else {
        (thickness, along)
    };
    Rectangle::new(origin.into(), extent.into())
}

#[cfg(test)]
#[path = "where_the_dock_is_tests.rs"]
mod tests;
