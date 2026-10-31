//! The region the top controls reserve, and where it stops.
//!
//! Canvas task 6a's second bullet: **the top controls are the fourth fixed control**,
//! and a frame keeps a usable part of its name clear of this one too. The region can
//! exist before the controls are painted, and the panel is the precedent rather than
//! the excuse — `panel_raster`'s reserved column is taken unconditionally because *the
//! panel owns its edge whether or not it is showing*, and an empty rail draws nothing
//! while the column still asks. The same is true here: the band is room the canvas
//! does not get, whatever is drawn in it.
//!
//! What is **not** here is what the band holds. Task 6a's outcome names the active
//! window's controls and the way back to the canvas, and neither is laid out yet. So
//! this reserves the room and says so, rather than painting a guess.
//!
//! # Where the number comes from, and why it is not derived
//!
//! `docs/design/the-regions-a-pointer-can-be-in.md` gives the top controls as
//! **1440 × 84** in a 1440 × 960 frame, so the height is 84 logical pixels. It is a
//! constant here and held to that document by a test, which is the discipline
//! `tests/the_frame_in_numbers.rs` was written for after a name band was **derived**
//! as 32 while the document had been giving 48 since #191. *A number a document
//! already gives is not a number to derive.*
//!
//! The width is derived, and that is not an inconsistency: the document's 1440 is the
//! whole frame, and the owner's ruling below makes the band's right edge the panel's
//! left one — a position no document can state because it moves with the panel.
//!
//! # Where it stops, decided by the owner rather than inferred
//!
//! The design as drawn gave the top controls the full width and the panel's region the
//! full height, which overlap over the top-right corner, and the frames could not say
//! which surface owned it. The owner settled it on 2026-09-30:
//!
//! > - **Top controls:** span the screen up to the reserved right-panel area.
//! > - **Right panel:** owns that area, including the top-right corner.
//! > - **Bottom Dock:** stops before the same area, as it already does.
//! >
//! > The reserved width follows the panel's current expanded or collapsed width.
//! > Activation strips and pointer paths must follow those bounds too, **so one
//! > pointer position cannot reveal two surfaces.**
//!
//! That last clause is `alo_put_aside::the_region_the_panel_claims`'s invariant in the
//! owner's own words, eleven days before it was written — and it is why this takes the
//! panel's column as an argument rather than reading a width of its own. The band's
//! right edge **is** the panel's left edge, so the two cannot overlap by construction
//! rather than by two numbers agreeing.
//!
//! # And where it starts, decided by the owner on 2026-10-31
//!
//! **The egress indicator owns the top-left corner, and the band starts just to
//! its right.** The owner moved the indicator out of the Dock's far end on
//! 2026-10-31, where it covered a task window's Send button; the approved frames
//! put it top right, and that corner is the panel's by the ruling above, so the
//! owner gave it the top-left instead and had this band make room. The band
//! still ends at the panel's column; it now begins where the indicator's circle
//! or pills end, by [`after_the_indicator`], so one pointer position still
//! cannot reveal two surfaces.

use smithay::utils::{Physical, Point, Rectangle, Size};

/// How tall the top controls are, in logical pixels.
///
/// From `docs/design/the-regions-a-pointer-can-be-in.md`'s own table, and held to it by
/// `tests/the_frame_in_numbers.rs` rather than trusted to stay in step.
const THE_TOP_CONTROLS: f64 = 84.0;

/// How tall the top controls are, for a test to hold against the design file.
#[must_use]
pub fn the_top_controls() -> f64 {
    THE_TOP_CONTROLS
}

/// The room the top controls reserve, or [`None`] when they are not there to.
///
/// [`None`] when a window fills the screen, which is task 6a's *gives way to a
/// full-screen window*. It is the same answer `dock_band` gives for a Dock that gave
/// way, and it means the canvas gets the room rather than a band nothing occupies
/// holding a frame away from it.
///
/// [`None`] too when the panel's column leaves no room at all — a display narrow
/// enough that the band would have no width is a band that does not exist, and a
/// rectangle of no extent would claim no point while still reading as a surface.
pub(crate) fn reserved(
    room: (i32, i32),
    panel_reserved: Rectangle<i32, Physical>,
    filling_the_screen: bool,
) -> Option<Rectangle<i32, Physical>> {
    if filling_the_screen {
        return None;
    }
    // The band's right edge is the panel's left edge, so the two cannot overlap
    // whatever the panel's width turns out to be. Taken from the column rather than
    // from a width of our own, which is the owner's *the reserved width follows the
    // panel's current expanded or collapsed width*.
    let stops_at = panel_reserved.loc.x.max(0);
    let width = if panel_reserved.size.w > 0 {
        stops_at
    } else {
        // No column means nothing to stop before, so the band spans the room. An
        // absent panel is not a panel at the left edge.
        room.0
    };
    let height = i32::try_from(THE_TOP_CONTROLS.round() as i64).unwrap_or(i32::MAX);
    (width > 0 && height > 0 && room.1 >= height)
        .then(|| Rectangle::new(Point::from((0, 0)), Size::from((width, height))))
}

/// **The band, starting just to the right of the egress indicator** — the
/// owner's decision of 2026-10-31 (this file's header). `indicator` is where
/// the indicator was drawn on this frame, its quiet circle or its pills.
///
/// [`None`] when the indicator leaves the band no room. A band the indicator
/// does not reach is returned as it was.
pub(crate) fn after_the_indicator(
    band: Option<Rectangle<i32, Physical>>,
    indicator: Option<Rectangle<i32, Physical>>,
) -> Option<Rectangle<i32, Physical>> {
    let band = band?;
    let Some(corner) = indicator.filter(|corner| corner.overlaps(band)) else {
        return Some(band);
    };
    let starts = (corner.loc.x + corner.size.w).max(band.loc.x);
    let ends = band.loc.x + band.size.w;
    (starts < ends).then(|| {
        Rectangle::new(
            Point::from((starts, band.loc.y)),
            Size::from((ends - starts, band.size.h)),
        )
    })
}
