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
use smithay::utils::{Physical, Rectangle};

use crate::RenderError;

/// The largest display side, in pixels, the dock is laid out for.
pub(crate) const LARGEST_SIDE: i32 = 16_384;

/// Where the Dock is on one display, and what is in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TheDocksPlaces {
    /// `alo-dock`'s layout for this display.
    pub(crate) layout: Layout,
    /// The whole band, on the person's chosen edge.
    pub(crate) band: Rectangle<i32, Physical>,
    /// How thick the band is, already clamped to the display.
    pub(crate) thickness: i32,
    /// Every slot along the bar: the applications, then the overflow control if
    /// there is one.
    pub(crate) places: Places,
    /// What there was no room for, in order, behind that control.
    pub(crate) over: Vec<OnTheDock>,
}

// **`how_far_along` is deliberately not here yet.**
//
// Turning a point on a screen into a distance along the bar is the other half
// of this file, and it was written, tested on four edges, and then taken out
// again before landing — because **nothing presses the Dock**. Measured
// 2026-10-10 across the 2531 `.rs` files outside `alo-dock`:
// `alo_dock::clicking` has no caller, `alo_dock::menu` has none, and
// `alo_dock::Places` is named twice, both in `crate::dock_raster` and both for
// drawing, so `Places::at` is never called by anything.
//
// A correct hit test nothing calls is the fault class `CLAUDE.md` names first —
// *a check that stands in for the thing is not the thing* — and this crate has
// just spent two changes removing two of them. It comes back in the change that
// routes a press, with its caller, and `clippy` refusing an unused `pub(crate)`
// method is what will keep that promise rather than this comment.

/// Where `dock` sits on a display of `size`, holding `on_the_dock`.
///
/// # Errors
/// [`RenderError::DesktopScene`] for a display `alo-dock` refuses to lay a dock
/// out on, or one larger than any this dock is laid out for.
pub(crate) fn where_the_dock_is(
    dock: &Dock,
    size: (i32, i32),
    on_the_dock: &[OnTheDock],
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
    let thickness = i32::try_from(layout.thickness().as_pixels())
        .map_err(|_| RenderError::DesktopScene)?
        .clamp(1, width.min(height));

    let margin = i32::try_from(alo_dock::measures::MARGIN).unwrap_or(i32::MAX);
    let floating = i32::try_from(alo_dock::measures::FLOATING_ABOVE_THE_EDGE).unwrap_or(i32::MAX);
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
    let at_most = Room::pixels(u32::try_from(widest).unwrap_or(u32::MAX)).how_many_fit();
    let fitted = alo_dock::fit(on_the_dock.to_vec(), at_most);
    let places = Places::of(&fitted);
    let wanted = i32::try_from(Room::a_bar_holding(places.how_many()).as_pixels())
        .map_err(|_| RenderError::DesktopScene)?;
    // Clamped as a last resort that cannot fire: `at_most` came from `widest`,
    // so `wanted` cannot exceed it except on a display too small for one slot,
    // where there is nothing to choose.
    let bar = wanted.clamp(1, widest);

    Ok(TheDocksPlaces {
        layout,
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
