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
//! **A task window belongs on the canvas, and this file does not put it there
//! yet.** This paragraph used to say the question was the owner's and unmade.
//! The owner made it on 2026-11-01 ([ADR
//! 0145](../../../docs/decisions/0145-alos-own-windows-live-on-the-canvas.md)):
//! alo's own windows pan and zoom with the canvas like every application's. What
//! is described above and below is the screen-space placement that stands until
//! the task-window stage of that ADR lands.
//!
//! # In the display's pixels, sized through its scale
//!
//! `where_a_window_opens` answers in `Logical` and the raster draws in
//! `Physical`, and this passes one to the other unchanged: clients are told
//! scale one (`crate::their_displays`), so the seat's coordinates **are** the
//! display's pixels, and a press is tested against the rectangle kept here
//! with no conversion. What goes through the display's scale is every
//! **measure** this reads — the size the design asks for, the least the
//! layout will hold, and the room kept above the window for its edge — so on a
//! 200 per cent screen a task window asks for twice the pixels and is the same
//! size to the eye as the Dock beside it. At 100 nothing changes.
//!
//! **Placed once, at the scale of the display it was placed on.** A window
//! that is already on the screen keeps its rectangle when the person changes
//! that display's scale, by rule 6's *later redraws must not unexpectedly
//! reposition it*; it is laid out inside at the new scale.

use alo_strings::Direction;
use alo_tasks::layout::{the_least_height, the_least_width};
use smithay::utils::{Logical, Physical, Point, Rectangle, Size};

use crate::display_scale::DisplayScale;

/// How large a task window asks to be, in logical pixels, before the display
/// has its say.
///
/// The size the design draws it at. **A request, not a guarantee** — rule 6:
/// *use the application's requested initial size, constrained by the usable
/// display area and its supported minimum size.*
pub(crate) const AS_LARGE_AS_THE_DESIGN_DRAWS_IT: (u32, u32) = (1040, 800);

/// What size a task window asks for on a display this big, drawn at `scale`.
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
pub(crate) fn what_size_it_asks_for(
    view: Size<i32, Logical>,
    scale: DisplayScale,
) -> Option<Size<i32, Logical>> {
    let (least_w, least_h) = (scale.px(the_least_width()), scale.px(the_least_height()));
    if view.w < least_w || view.h < least_h {
        return None;
    }
    let (wanted_w, wanted_h) = (
        scale.px(AS_LARGE_AS_THE_DESIGN_DRAWS_IT.0),
        scale.px(AS_LARGE_AS_THE_DESIGN_DRAWS_IT.1),
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
/// Answers [`None`] when the view cannot hold the window at all. `scale` is
/// the display's, which every measure here goes through.
#[must_use]
pub(crate) fn where_one_more_goes(
    view: Rectangle<i32, Logical>,
    already: &[Rectangle<i32, Logical>],
    reading: Direction,
    fixed: &[Rectangle<i32, Logical>],
    scale: DisplayScale,
) -> Option<Rectangle<i32, Logical>> {
    // **Below its own edge.** A task window carries the design's external
    // edge above it, and Close is on that edge — so a window placed at the
    // very top of the view would have its Close off the screen, and the one
    // pointer road to closing it would be gone. The edge's own height comes
    // off the top of the view before anything is sized or placed.
    // At the display's scale, because `crate::window_edge_picture` draws the
    // edge at it.
    let edge = scale.px(u32::try_from(crate::window_edge::THE_REGION_IS_TALL).unwrap_or(0));
    let view = Rectangle::new(
        Point::from((view.loc.x, view.loc.y + edge)),
        Size::from((view.size.w, (view.size.h - edge).max(0))),
    );
    let wanted = what_size_it_asks_for(view.size, scale)?;
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
/// One number to one number, for the reason in this file's header: the seat's
/// coordinates are the display's pixels, and the measures were scaled when the
/// size was asked for.
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
