//! Where alo puts a window when alo is the one choosing.
//!
//! `docs/design/where-a-new-window-opens.md` is the contract, recorded by the
//! owner on 2026-10-09 **before this was written**. This file implements rules
//! 3 to 7 of it; 1, 2 and 8 belong to the caller and are named below so that
//! nobody looks for them here.
//!
//! # What this decides, and the thing it must never decide
//!
//! It answers *where does this window go* **only when nothing else has
//! answered**. Rule 1: *explicit placement wins* — a position the person chose
//! never reaches this function. Rule 2: *restoring is different from opening*
//! — a window coming back goes to its saved geometry and this is not asked.
//! Rule 8 is about focus and the camera, which are the host's.
//!
//! **Overlap is permitted; losing access to a window is not.** That is rule 5
//! and it is the whole shape of this file. `docs/features.md` promises
//! *nothing is stacked — no window is buried behind another **where the person
//! cannot find it***, and on 2026-10-09 this lane read that to the dash and
//! was about to forbid one window covering another — which would have taken a
//! choice away from the person, the thing `CLAUDE.md`'s fifth law calls a bug
//! whatever the reason given. **A person may stack windows as they please.**
//! What alo may not do is put one somewhere unreachable without being asked.
//!
//! # Nothing here moves anything that is already open
//!
//! Rule 4: *do not move or resize existing windows to make room, do not
//! automatically zoom out or impose a grid.* So this reads the windows that
//! are open and never returns an instruction about them — the return value is
//! one point, for one new window, and there is no shape this function could
//! take that would rearrange a desk.
//!
//! # Not yet called, and what its caller has to hold
//!
//! On 2026-10-09 nothing calls this. The place it belongs is
//! `crate::surfaces`' `commit`, at `window.mapped = ...` — the moment a
//! window becomes mapped, which is the first moment its size is known.
//! `new_toplevel` is too early: a toplevel has a Place and no buffer there,
//! and placing a window before it has a size would be choosing a position for
//! something with no extent.
//!
//! **What stops that being a line of wiring is two arguments.** `Surfaces`
//! holds protocol state, window modes, popups and cameras; it holds **no view
//! extent and no reading direction**, and both are the host's — the view is
//! the camera over this display and the direction is the person's. Passing
//! them in is a change to the compositor's own shape rather than to this file,
//! so it is named here rather than guessed at.
//!
//! Until then every window is placed at the origin by nothing choosing at all,
//! which is what `docs/design/where-a-new-window-opens.md` records as the
//! state this was written to end.

use alo_strings::Direction;
use smithay::utils::{Logical, Point, Rectangle, Size};

/// The gap between a new window and the one it opens beside.
///
/// **16, which is the design file's own `space/4`** — measured live from
/// `Canvas / Window title` on 2026-10-08 and chosen by the owner on
/// 2026-10-09. A number this product already uses rather than one invented
/// here.
pub const A_SMALL_GAP: i32 = 16;

/// How far in front a window opens when there is no room beside.
///
/// **48, which is one name band** (`crate::window_name_band::BAND_IS_TALL`),
/// down and to the right. Chosen by the owner on 2026-10-09, and the reason it
/// is that number rather than a smaller one: offsetting by exactly a band
/// leaves the covered window's own band **completely uncovered**, so rule 5's
/// *keep its title band and controls reachable* holds for the window
/// underneath as well as the new one.
pub const A_DELIBERATE_OFFSET: i32 = crate::window_name_band::BAND_IS_TALL;

/// The smallest corner of a new window that must be inside the view for it to
/// be reachable: `canvas_never_lost::A_USABLE_HANDLE`, as whole pixels.
///
/// A window whose handle is outside the view is one a person would have to
/// find before they could move it, which rule 7 exists to prevent.
const A_REACHABLE_CORNER: (i32, i32) = (44, 24);

/// Where a new window of `wanted` size opens.
///
/// `view` is what the person can currently see, in the same logical space as
/// `open` and `active`. `open` is every window already on this Place; `active`
/// is the one being worked in, where there is one.
///
/// Always returns a point: rule 5 means there is no case where alo declines to
/// open a window, only cases where it opens in front of something.
#[must_use]
pub fn where_a_window_opens(
    view: Rectangle<i32, Logical>,
    open: &[Rectangle<i32, Logical>],
    active: Option<Rectangle<i32, Logical>>,
    wanted: Size<i32, Logical>,
    reading: Direction,
) -> Point<i32, Logical> {
    if let Some(active) = active {
        // **Rule 3: beside the active window, preferred side first.** Right in
        // a left-to-right layout and left in a right-to-left one — the same
        // rule mirrored, not a second rule.
        let beside = match reading {
            Direction::LeftToRight => [
                to_the_right_of(active, wanted),
                to_the_left_of(active, wanted),
            ],
            Direction::RightToLeft => [
                to_the_left_of(active, wanted),
                to_the_right_of(active, wanted),
            ],
        };
        for at in beside {
            if is_free(at, wanted, view, open) {
                return at;
            }
        }
        // **Rule 5: no room, so overlap on purpose**, in front and offset.
        let offset = Point::from((
            active.loc.x + A_DELIBERATE_OFFSET,
            active.loc.y + A_DELIBERATE_OFFSET,
        ));
        // **Rule 7: and never off the edge.** An offset that would put the new
        // window's handle outside the view restarts placement inside it rather
        // than walking the next one further out. A person opening ten
        // terminals ends with ten reachable terminals.
        if is_reachable_in(offset, wanted, view) {
            return offset;
        }
        return the_view_starts_here(view);
    }
    // **Rule 6: no active window, so near the centre of the view.**
    let middle = Point::from((
        view.loc.x + (view.size.w - wanted.w) / 2,
        view.loc.y + (view.size.h - wanted.h) / 2,
    ));
    if is_free(middle, wanted, view, open) {
        return middle;
    }
    // Something is in the middle already. Step by the same deliberate offset
    // rather than inventing a second kind of movement, and restart in the view
    // when stepping would leave it — rule 7 again, for the same reason.
    let stepped = Point::from((
        middle.x + A_DELIBERATE_OFFSET,
        middle.y + A_DELIBERATE_OFFSET,
    ));
    if is_reachable_in(stepped, wanted, view) {
        stepped
    } else {
        the_view_starts_here(view)
    }
}

/// Where placement restarts when another offset would leave the view.
fn the_view_starts_here(view: Rectangle<i32, Logical>) -> Point<i32, Logical> {
    Point::from((view.loc.x + A_SMALL_GAP, view.loc.y + A_SMALL_GAP))
}

/// Immediately to the right of this window, one gap away, at its top.
fn to_the_right_of(
    active: Rectangle<i32, Logical>,
    _wanted: Size<i32, Logical>,
) -> Point<i32, Logical> {
    Point::from((active.loc.x + active.size.w + A_SMALL_GAP, active.loc.y))
}

/// Immediately to the left of this window, one gap away, at its top.
fn to_the_left_of(
    active: Rectangle<i32, Logical>,
    wanted: Size<i32, Logical>,
) -> Point<i32, Logical> {
    Point::from((active.loc.x - A_SMALL_GAP - wanted.w, active.loc.y))
}

/// Whether a window of `wanted` size at `at` fits in the view and touches
/// nothing already open.
///
/// **Whole, not partly.** Rule 3 asks for *free space*, and a position that
/// put most of a window in free space and a corner over another window would
/// be an overlap nobody chose — which rule 5 allows only when there is no
/// alternative.
fn is_free(
    at: Point<i32, Logical>,
    wanted: Size<i32, Logical>,
    view: Rectangle<i32, Logical>,
    open: &[Rectangle<i32, Logical>],
) -> bool {
    let want = Rectangle::new(at, wanted);
    if view.intersection(want) != Some(want) {
        return false;
    }
    !open.iter().any(|window| window.overlaps(want))
}

/// Whether enough of a window at `at` is inside the view to be grabbed.
///
/// Its handle, which is the top-left corner of its name band. A window whose
/// band is outside the view is one a person has to go and find before they can
/// move it.
fn is_reachable_in(
    at: Point<i32, Logical>,
    wanted: Size<i32, Logical>,
    view: Rectangle<i32, Logical>,
) -> bool {
    let handle = Rectangle::new(
        at,
        Size::from((
            A_REACHABLE_CORNER.0.min(wanted.w),
            A_REACHABLE_CORNER.1.min(wanted.h),
        )),
    );
    view.intersection(handle) == Some(handle)
}

#[cfg(test)]
#[path = "where_a_window_opens_tests.rs"]
mod tests;
