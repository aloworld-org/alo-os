//! Where a task window goes when it opens, and how it stays there.
//!
//! [`crate::where_a_window_opens`] is the owner's specification made into
//! arithmetic — rules 3 to 7 of
//! `docs/design/where-a-new-window-opens.md`. **This does not re-decide any of
//! it.** It answers the two things that specification leaves to a caller: what
//! size a task window asks for, and which rectangles to hand over as the ones
//! already open.
//!
//! # Placed once, and then left alone
//!
//! > *Apply default placement once. Restoring a window uses saved geometry;
//! > later redraws or remapping must not unexpectedly reposition it.*
//!
//! So a task window's position is **kept** once it has one, and a frame drawn a
//! second later asks nothing. [`crate::the_tasks_a_person_opened::TheTasks`]
//! holds it, which is why placement is stored beside the task rather than
//! recomputed: a position recomputed every frame is a window that moves when
//! somebody else's window opens.
//!
//! # In screen space, following the two shell windows that came before
//!
//! A task window is drawn by the shell, not by a client, and the two that
//! already are — what is running and what is filling the disk — are laid out
//! and painted against the **display**, not against the plane a person pans.
//! This follows them, because `crate::task_raster` draws into a frame at
//! physical pixels and a surface on the plane would need the camera applied to
//! its texture.
//!
//! **Whether a task window ought to live on the canvas instead is a real
//! question and it is not answered here.** The design draws it with an external
//! window edge, which every canvas window has; it also draws the two desktop
//! windows that way, and those are screen-space. Naming it rather than
//! deciding it: a task window that panned away under a person who dragged the
//! canvas would be a different product decision from one that stayed put, and
//! the owner has not made it.
//!
//! # Logical and physical are one number here, as everywhere else in the shell
//!
//! `where_a_window_opens` answers in `Logical` and the raster draws in
//! `Physical`, and this passes one to the other unchanged. That is not a
//! conversion being skipped — it is `crate::dock_raster`'s documented state:
//! **nothing in this shell applies a display scale yet**, and making it
//! scale-aware is one change that moves every measure together. A conversion
//! invented here would be the *one scaled figure in an unscaled geometry* that
//! file warns against.

use alo_strings::Direction;
use alo_tasks::layout::{the_least_height, the_least_width};
use smithay::utils::{Logical, Physical, Point, Rectangle, Size};

/// How large a task window asks to be, before the display has its say.
///
/// The size the design draws it at. **A request, not a guarantee** — rule 6:
/// *use the application's requested initial size, constrained by the usable
/// display area and its supported minimum size.*
pub(crate) const AS_LARGE_AS_THE_DESIGN_DRAWS_IT: (u32, u32) = (1040, 800);

/// What size a task window asks for on a display this big.
///
/// The design's size, cut to the view, and never below what
/// `alo_tasks::layout` will lay out — because a window narrower than its own
/// floor is one the rasteriser refuses, and a refused window is one a person
/// cannot see at all.
///
/// Answers [`None`] for a view too small to hold even the floor. That is a
/// true answer about a very small display rather than a failure: there is no
/// size at which this window works there, and saying so is better than placing
/// one that cannot be drawn.
#[must_use]
pub(crate) fn what_size_it_asks_for(view: Size<i32, Logical>) -> Option<Size<i32, Logical>> {
    let (least_w, least_h) = (
        i32::try_from(the_least_width()).unwrap_or(i32::MAX),
        i32::try_from(the_least_height()).unwrap_or(i32::MAX),
    );
    if view.w < least_w || view.h < least_h {
        return None;
    }
    let (wanted_w, wanted_h) = (
        i32::try_from(AS_LARGE_AS_THE_DESIGN_DRAWS_IT.0).unwrap_or(i32::MAX),
        i32::try_from(AS_LARGE_AS_THE_DESIGN_DRAWS_IT.1).unwrap_or(i32::MAX),
    );
    Some(Size::from((
        wanted_w.min(view.w).max(least_w),
        wanted_h.min(view.h).max(least_h),
    )))
}

/// Where one more task window goes, beside the ones already open.
///
/// `already` is every task window that has a position, front-to-back — the
/// most recently placed first, which is what makes the newest one *the window
/// a person was working in* for rule 3's purposes and gives rule 7's repeated
/// openings their offsets.
///
/// Answers [`None`] when the view cannot hold the window at all.
#[must_use]
pub(crate) fn where_one_more_goes(
    view: Rectangle<i32, Logical>,
    already: &[Rectangle<i32, Logical>],
    reading: Direction,
    fixed: &[Rectangle<i32, Logical>],
) -> Option<Rectangle<i32, Logical>> {
    // **Below its own edge.** A task window carries the design's external
    // edge above it, and Close is on that edge — so a window placed at the
    // very top of the view would have its Close off the screen, and the one
    // pointer road to closing it would be gone. The edge's own height comes
    // off the top of the view before anything is sized or placed.
    let edge = crate::window_edge::THE_REGION_IS_TALL;
    let view = Rectangle::new(
        Point::from((view.loc.x, view.loc.y + edge)),
        Size::from((view.size.w, (view.size.h - edge).max(0))),
    );
    let wanted = what_size_it_asks_for(view.size)?;
    // **The newest is the active one.** `already` is front-to-back, so the
    // first is the window a person most recently had opened for them — and
    // *beside the active window* means beside that one. Taking the focused
    // window instead would be a second idea of *active* in a shell that has
    // one.
    let active = already.first().copied();
    let at = crate::where_a_window_opens(view, already, active, wanted, reading, fixed);
    Some(Rectangle::new(at, wanted))
}

/// The same rectangle, as the pixels a frame draws into.
///
/// One number to one number, for the reason in this file's header: nothing in
/// this shell applies a display scale yet, and inventing a conversion here
/// would be the single scaled figure in an unscaled geometry that
/// `crate::dock_raster` records as worse than scaling nothing.
#[must_use]
pub(crate) fn as_a_frame_draws_it(at: Rectangle<i32, Logical>) -> Rectangle<i32, Physical> {
    Rectangle::new(
        Point::from((at.loc.x, at.loc.y)),
        Size::from((at.size.w, at.size.h)),
    )
}

#[cfg(test)]
#[path = "where_a_task_window_opens_tests.rs"]
mod tests;
