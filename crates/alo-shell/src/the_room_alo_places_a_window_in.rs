//! **A window alo places lands wholly on the screen, above the Dock**, and one
//! too big for it is asked to be smaller.
//!
//! `docs/design/where-a-new-window-opens.md`, rule 6: *use the application's
//! requested initial size, **constrained by the usable display area** and its
//! supported minimum size*; rule 7: *repeated openings must not drift
//! off-screen*; rule 5: *losing access to a window is not* permitted.
//!
//! # What was wrong
//!
//! Seen on a booted machine on 2026-11-12 (qemu, 1 280 × 800, Dolphin open at
//! about (260, 107), 760 × 586). Firefox opened twice larger than the screen:
//! once with its top at the screen's top and its bottom under the Dock and off
//! the edge, Dolphin nowhere to be seen; once with its top at y ≈ 710, only its
//! tab strip showing, under the Dock. Two rules let that happen, and both are
//! read off the code rather than measured on the machine:
//!
//! - **a window taller than the view counted as seen when its corner was**
//!   (`crate::room_in_a_full_view::there_is_room_at`,
//!   `crate::the_nearest_empty_space`), and nothing asked it to be smaller —
//!   rule 6's *constrained* was never built;
//! - **a window that grew was placed again only if it now covered another**
//!   (`crate::a_window_that_grows_is_placed_again`). One whose first frame
//!   fitted below Dolphin and which then grew down past the bottom of the
//!   screen covered nothing, and stayed.
//!
//! And the Dock was never part of the room: `crate::a_new_window_is_placed`
//! placed against the whole view, so a window could be placed with its bottom
//! under the Dock.
//!
//! # The rule now
//!
//! **The room** is the view less the Dock's band. The Dock is fixed to the
//! bottom edge (ADR 0076), so the room is the view down to the Dock's top.
//!
//! - A window too big for the room, with the 16 gap kept round it and its
//!   strip above it, is **asked to be the size that fits** — never smaller than
//!   the application says it can be. The application may refuse; it is placed
//!   at the size it really has, and placed again when it settles.
//! - A window that fits is **kept wholly inside the room**: rules 3 to 7's
//!   answer is moved the least it must be to be inside, with the gap.
//! - A window alo placed that **grows so that part of it leaves the room** is
//!   placed again, as one that grows over another already was. One wholly out
//!   of the view — the person panned away from it — is left where it is.
//!
//! A window the person moved, resized, maximised or pressed on is never placed
//! again by any of this (`crate::a_window_that_grows_is_placed_again`).
//!
//! # And clear of the put-aside panel — the owner, 2026-11-13
//!
//! Asked whether a new window may open under the right-hand panel of windows
//! put aside, the owner answered *"Windows open clear of it"*. So the room is
//! also the view left of the panel's column (right of it, read right to left).
//! **The column, not the rail**: the panel keeps its column whether it holds
//! anything or is showing (`crate::panel_raster::the_column_it_reserves`), and
//! a window placed under an empty panel would be covered the moment the person
//! put one aside. It is the panel's extent as last drawn, asked rather than
//! worked out here, so a denser display or a larger panel moves it with them.
//! A window too wide for what is left is asked to be narrower, never below
//! its minimum. **A person may still move a window under the panel
//! themselves**; nothing here moves it back.
//!
//! **This makes an existing promise true rather than adding one**: rule 6's
//! *constrained by the usable display area* and rule 5's *losing access to a
//! window is not* permitted. It adds no setting. A person may still put a
//! window under the Dock, or off the screen, themselves.

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point, Rectangle, Size};

use crate::a_window_and_its_strip::ItsStrip;

/// The gap kept round a window alo places, inside the room as beside another
/// window: the same 16 rule 3 keeps (`crate::where_a_window_opens::A_SMALL_GAP`).
const THE_GAP: i32 = crate::where_a_window_opens::A_SMALL_GAP;

/// **The room a window is placed in**: the view, down to the top of the Dock
/// where the Dock's band reaches into it. The Dock is fixed to the bottom edge
/// (ADR 0076), so only the bottom is taken; a band that does not reach into
/// the view takes nothing.
#[must_use]
pub(crate) fn the_room_the_dock_leaves(
    view: Rectangle<i32, Logical>,
    dock: Option<Rectangle<i32, Logical>>,
) -> Rectangle<i32, Logical> {
    let Some(dock) = dock.and_then(|dock| dock.intersection(view)) else {
        return view;
    };
    let tall = dock.loc.y.saturating_sub(view.loc.y);
    if tall <= 0 {
        return view;
    }
    Rectangle::new(view.loc, Size::from((view.size.w, tall.min(view.size.h))))
}

/// **`room`, clear of the put-aside panel's column** where the column reaches
/// into it. The column runs the display's whole height on the edge the panel
/// belongs to, so it takes the side its middle is on — the right, or the left
/// on a screen read right to left — as `crate::the_room_a_window_fills`'s work
/// area does. A column that does not reach into the room, or one of no width,
/// takes nothing.
///
/// **Asked, never assumed**: `panel` is the column as the panel last drew it
/// (`crate::canvas_fixed_controls::FixedControlsDrawn::panel_reserved`), which
/// it keeps whether it holds anything or not, and whether it is showing.
#[must_use]
pub(crate) fn the_room_the_panel_leaves(
    room: Rectangle<i32, Logical>,
    panel: Option<Rectangle<i32, Logical>>,
) -> Rectangle<i32, Logical> {
    let Some(panel) = panel.and_then(|panel| panel.intersection(room)) else {
        return room;
    };
    if panel.size.w <= 0 || panel.size.h <= 0 {
        return room;
    }
    let its_middle = panel.loc.x.saturating_add(panel.size.w / 2);
    let the_rooms_middle = room.loc.x.saturating_add(room.size.w / 2);
    if its_middle >= the_rooms_middle {
        let wide = panel.loc.x.saturating_sub(room.loc.x).max(0);
        Rectangle::new(room.loc, Size::from((wide, room.size.h)))
    } else {
        let from = panel.loc.x.saturating_add(panel.size.w);
        let right = room.loc.x.saturating_add(room.size.w);
        Rectangle::new(
            Point::from((from, room.loc.y)),
            Size::from((right.saturating_sub(from).max(0), room.size.h)),
        )
    }
}

/// **The room a window alo places lands in**: `view` above the Dock and clear
/// of the put-aside panel's column — the two fixed controls the owner has said
/// a new window opens clear of (2026-11-12 and 2026-11-13). Both are the
/// controls as last drawn, on the plane.
#[must_use]
pub(crate) fn the_room_the_controls_leave(
    view: Rectangle<i32, Logical>,
    dock: Option<Rectangle<i32, Logical>>,
    panel: Option<Rectangle<i32, Logical>>,
) -> Rectangle<i32, Logical> {
    the_room_the_panel_leaves(the_room_the_dock_leaves(view, dock), panel)
}

/// **The size a window of `wanted` is offered so that it, its strip and the
/// gap round them fit `room`**: `wanted` on each side where it fits, else the
/// most that fits — and never less than `smallest`, the least the application
/// says it can be (nought on a side is no limit).
#[must_use]
pub(crate) fn the_size_that_fits(
    wanted: Size<i32, Logical>,
    room: Rectangle<i32, Logical>,
    strip: ItsStrip,
    smallest: Size<i32, Logical>,
) -> Size<i32, Logical> {
    let with_the_strip = strip.round_a_size(Size::from((1, 1)));
    let strip_tall = with_the_strip.h.saturating_sub(1);
    let most = Size::<i32, Logical>::from((
        room.size.w.saturating_sub(2 * THE_GAP),
        room.size
            .h
            .saturating_sub(2 * THE_GAP)
            .saturating_sub(strip_tall),
    ));
    let side = |wanted: i32, most: i32, smallest: i32| wanted.min(most).max(smallest).max(1);
    Size::from((
        side(wanted.w, most.w, smallest.w),
        side(wanted.h, most.h, smallest.h),
    ))
}

/// Whether a window at `at`, `wanted` big, lies wholly in `room` with its
/// strip and the gap: what [`kept_inside`] answers, asked by its tests.
#[cfg(test)]
#[must_use]
pub(crate) fn lies_wholly_in(
    at: Point<i32, Logical>,
    wanted: Size<i32, Logical>,
    room: Rectangle<i32, Logical>,
    strip: ItsStrip,
) -> bool {
    inside_with_the_gap(room).contains_rect(strip.round(Rectangle::new(at, wanted)))
}

/// Whether part of a window at `at`, `wanted` big, with its strip, is in
/// `room` and part of it is not: the window a person can see some of and not
/// all. **The room itself, not the room less the gap**, so a window that grows
/// a little into the gap at the room's edge is not moved for it.
#[must_use]
pub(crate) fn is_partly_out_of(
    at: Point<i32, Logical>,
    wanted: Size<i32, Logical>,
    room: Rectangle<i32, Logical>,
    strip: ItsStrip,
) -> bool {
    let taken = strip.round(Rectangle::new(at, wanted));
    taken.overlaps(room) && !room.contains_rect(taken)
}

/// **`at`, moved the least it must be so the window lies wholly in `room`**
/// with its strip and the gap. A window too big for the room on a side is put
/// at the room's start on that side, so its strip — where a person takes hold
/// of it — is on the screen.
#[must_use]
pub(crate) fn kept_inside(
    at: Point<i32, Logical>,
    wanted: Size<i32, Logical>,
    room: Rectangle<i32, Logical>,
    strip: ItsStrip,
) -> Point<i32, Logical> {
    let inside = inside_with_the_gap(room);
    let taken = strip.round(Rectangle::new(at, wanted));
    let along = |from: i32, length: i32, inside_from: i32, inside_length: i32| -> i32 {
        let inside_to = inside_from.saturating_add(inside_length);
        if length > inside_length || from < inside_from {
            inside_from
        } else if from.saturating_add(length) > inside_to {
            inside_to.saturating_sub(length)
        } else {
            from
        }
    };
    let corner = Point::from((
        along(taken.loc.x, taken.size.w, inside.loc.x, inside.size.w),
        along(taken.loc.y, taken.size.h, inside.loc.y, inside.size.h),
    ));
    strip.the_window_in(corner, wanted)
}

/// `room` less the gap on every side.
fn inside_with_the_gap(room: Rectangle<i32, Logical>) -> Rectangle<i32, Logical> {
    Rectangle::new(
        Point::from((
            room.loc.x.saturating_add(THE_GAP),
            room.loc.y.saturating_add(THE_GAP),
        )),
        Size::from((
            room.size.w.saturating_sub(2 * THE_GAP).max(0),
            room.size.h.saturating_sub(2 * THE_GAP).max(0),
        )),
    )
}

impl crate::Server {
    /// **Ask a window too big for `room` to be the size that fits**, and say
    /// what was asked: [`None`] where it fits already or cannot be asked — a
    /// maximised window is where its mode put it, and the request is the
    /// application's to refuse either way.
    pub(crate) fn ask_it_to_fit(
        &mut self,
        surface: &WlSurface,
        wanted: Size<i32, Logical>,
        room: Rectangle<i32, Logical>,
        strip: ItsStrip,
    ) -> Option<Size<i32, Logical>> {
        let smallest = smithay::wayland::compositor::with_states(surface, |states| {
            let mut cached = states
                .cached_state
                .get::<smithay::wayland::shell::xdg::SurfaceCachedState>();
            cached.current().min_size
        });
        let fits = the_size_that_fits(wanted, room, strip, smallest);
        if fits == wanted {
            return None;
        }
        match self.request_window_size(surface, (fits.w, fits.h)) {
            Ok(_) => Some(fits),
            Err(why) => {
                eprintln!(
                    "alo-shell: a window {}×{} does not fit the {}×{} the Dock and the panel \
                     leave, and could not be asked to be {}×{} ({why}), so it is placed at its \
                     own size",
                    wanted.w, wanted.h, room.size.w, room.size.h, fits.w, fits.h
                );
                None
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
#[path = "the_room_alo_places_a_window_in_tests.rs"]
mod tests;
