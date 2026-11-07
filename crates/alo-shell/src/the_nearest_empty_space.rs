//! **The nearest empty space on the Place**, for a new window that has no room
//! in what the person is looking at — and where the view has to go to show it.
//!
//! `docs/design/where-a-new-window-opens.md`, amended by the owner on
//! 2026-11-07: *the view moves smoothly to the **nearest** free space on the
//! current Place, from what the person is looking at, in any direction; the new
//! window opens there **without covering any window**.* Asked where it looks
//! first, the owner answered *"Nearest space"*.
//!
//! This file answers the arithmetic and nothing else. Whether to ask it at all —
//! the person's choice, whether a person opened the window, whether there was
//! room already — is `crate::a_new_window_is_placed`'s; moving the view is
//! `crate::the_view_glides`'.
//!
//! # What *nearest* is measured as
//!
//! **How far the view has to move**, not how far the window is from the middle
//! of the screen. A space the person can see already costs nothing, and one
//! just past the right edge costs the distance to bring it in with a gap to
//! spare. That is the thing a person experiences — the glide — and it is the
//! reading under which *nearest* and *the view moves as little as it can* are
//! one rule rather than two.
//!
//! Ties, which are common when several spaces are already in view, go to the
//! space whose middle is nearest the view's middle, then to the side a reader
//! goes on to — the right in a left-to-right layout and the left in a
//! right-to-left one, rule 3's own preference mirrored — and then to the
//! higher, so the same desk always gives the same answer.
//!
//! # Every candidate that could be the nearest is tried
//!
//! A free position for a window is any corner whose rectangle, grown by the gap,
//! touches no open window. The cost above is zero inside a box and grows
//! linearly outside it, separately across and down. So the nearest free corner
//! lies, on each axis, on one of finitely many lines: an open window's edge
//! (beside it, one gap away, or aligned with it), or the edge of the box where
//! the cost is zero. Trying every pair of those lines finds it exactly, with no
//! grid and no step size that a large or a small display would get wrong.
//!
//! Candidates are tried cheapest first and the first free one is the answer, so
//! a desk with room nearby answers after a few tests whatever is open elsewhere.
//!
//! # Never somewhere a person cannot get back
//!
//! Two bounds, both already promises:
//!
//! - **on the plane.** `alo_canvas::plane::FURTHEST` is as far as a position
//!   goes, for the window and for the view that shows it;
//! - **within *Show all***. `crate::canvas_never_lost`: *the system must not
//!   permit a distant window that Show all cannot include.* So the span of every
//!   window on the Place, the new one included, must fit what *Show all* can fit
//!   at its furthest out. A Place so full that no space is left inside that
//!   reach has **no** nearest empty space, and the caller falls back to opening
//!   in front, offset — rule 5 — rather than putting a window where it would be
//!   lost (rule 7: *repeated openings must not drift* out of reach).
//!
//! And the new window's handle — the corner of its name band — is not
//! completely under a fixed control once the view has moved, which is the
//! contract's *fixed controls must not completely cover the new window's usable
//! movement handle*.

use alo_strings::Direction;
use smithay::utils::{Logical, Point, Rectangle, Size};

/// The gap kept between the new window and every other, and between it and the
/// view's edge once the view has moved: the same 16 rule 3 keeps beside the
/// active window (`crate::where_a_window_opens::A_SMALL_GAP`).
const THE_GAP: i32 = crate::where_a_window_opens::A_SMALL_GAP;

/// The corner of a window that must be pressable: the same 44 × 24 the placer
/// protects (`crate::canvas_never_lost::A_USABLE_HANDLE`).
const A_HANDLE: (i32, i32) = (44, 24);

/// Where the new window goes, and where the view must look for it to be seen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WhereItGoes {
    /// The new window's top-left corner, on the plane.
    pub(crate) window: Point<i32, Logical>,
    /// The plane point the view's top-left corner must be at, at the zoom the
    /// person has. Equal to the view's corner now when the space is in view.
    pub(crate) view: Point<i32, Logical>,
}

impl WhereItGoes {
    /// Whether the view has to move to show it.
    pub(crate) fn the_view_moves(self, from: Point<i32, Logical>) -> bool {
        self.view != from
    }
}

/// The nearest empty space for a window of `wanted` size, and the view that
/// shows it.
///
/// Every rectangle is on the plane. `view` is what the person sees now; `open`
/// is every window on this Place, alo's own included; `fixed` is the fixed
/// controls moved onto the plane at the view as it is now, which move with the
/// view; `reach` is the widest span *Show all* can fit, or [`None`] where there
/// is no display to fit into; `reading` breaks a tie toward the side a reader
/// goes on to.
///
/// [`None`] where no space is left within reach — the caller's cue to open in
/// front, offset, as rule 5 says.
#[must_use]
pub(crate) fn the_nearest_empty_space(
    view: Rectangle<i32, Logical>,
    open: &[Rectangle<i32, Logical>],
    wanted: Size<i32, Logical>,
    fixed: &[Rectangle<i32, Logical>],
    reach: Option<Size<i32, Logical>>,
    reading: Direction,
) -> Option<WhereItGoes> {
    if wanted.w <= 0 || wanted.h <= 0 || view.size.w <= 0 || view.size.h <= 0 {
        return None;
    }
    let across = lines_on_one_axis(
        view.loc.x,
        view.size.w,
        wanted.w,
        open.iter().map(|w| (w.loc.x, w.size.w)),
    );
    let down = lines_on_one_axis(
        view.loc.y,
        view.size.h,
        wanted.h,
        open.iter().map(|w| (w.loc.y, w.size.h)),
    );
    let middle = (
        i64::from(view.loc.x) * 2 + i64::from(view.size.w),
        i64::from(view.loc.y) * 2 + i64::from(view.size.h),
    );
    let mut candidates: Vec<(Cost, i32, i32)> = Vec::with_capacity(across.len() * down.len());
    for &x in &across {
        let moved_across = how_far_the_view_moves(view.loc.x, view.size.w, x, wanted.w);
        for &y in &down {
            let moved_down = how_far_the_view_moves(view.loc.y, view.size.h, y, wanted.h);
            let from_the_middle = (
                i64::from(x) * 2 + i64::from(wanted.w) - middle.0,
                i64::from(y) * 2 + i64::from(wanted.h) - middle.1,
            );
            candidates.push((
                Cost {
                    travel: moved_across * moved_across + moved_down * moved_down,
                    off_centre: from_the_middle.0 * from_the_middle.0
                        + from_the_middle.1 * from_the_middle.1,
                    onward: match reading {
                        Direction::LeftToRight => -i64::from(x),
                        Direction::RightToLeft => i64::from(x),
                    },
                    down: y,
                },
                x,
                y,
            ));
        }
    }
    candidates.sort_unstable_by_key(|candidate| candidate.0);
    candidates.into_iter().find_map(|(_, x, y)| {
        let window = Rectangle::new(Point::from((x, y)), wanted);
        let shown_from = Point::from((
            view.loc.x.checked_add(
                i32::try_from(how_far_the_view_moves(view.loc.x, view.size.w, x, wanted.w)).ok()?,
            )?,
            view.loc.y.checked_add(
                i32::try_from(how_far_the_view_moves(view.loc.y, view.size.h, y, wanted.h)).ok()?,
            )?,
        ));
        (is_on_the_plane(window, shown_from, view.size)
            && touches_nothing(window, open)
            && show_all_still_reaches(window, open, reach)
            && the_handle_is_not_buried(window, view.loc, shown_from, fixed))
        .then_some(WhereItGoes {
            window: window.loc,
            view: shown_from,
        })
    })
}

/// What a candidate costs, compared field by field in this order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Cost {
    /// How far the view moves, squared.
    travel: i64,
    /// How far the window's middle is from the view's, squared, doubled.
    off_centre: i64,
    /// Further along the reading direction first.
    onward: i64,
    /// Then higher.
    down: i32,
}

/// Every line on one axis where the nearest free corner can lie.
///
/// Beside each open window one gap away on either side, aligned with either of
/// its edges, and the two ends of the stretch where the view need not move.
fn lines_on_one_axis(
    view_from: i32,
    view_length: i32,
    wanted: i32,
    open: impl Iterator<Item = (i32, i32)>,
) -> Vec<i32> {
    let mut lines = vec![
        view_from.saturating_add(THE_GAP),
        view_from
            .saturating_add(view_length)
            .saturating_sub(THE_GAP)
            .saturating_sub(wanted),
    ];
    for (from, length) in open {
        let to = from.saturating_add(length);
        lines.extend([
            to.saturating_add(THE_GAP),
            from.saturating_sub(THE_GAP).saturating_sub(wanted),
            from,
            to.saturating_sub(wanted),
        ]);
    }
    lines.sort_unstable();
    lines.dedup();
    lines
}

/// How far the view moves on one axis so a window from `at`, `wanted` long, is
/// in it with a gap to spare — or, where it is longer than the view, so its
/// start is one gap in, which is where its handle is.
fn how_far_the_view_moves(view_from: i32, view_length: i32, at: i32, wanted: i32) -> i64 {
    let (view_from, view_length, at, wanted) = (
        i64::from(view_from),
        i64::from(view_length),
        i64::from(at),
        i64::from(wanted),
    );
    let gap = i64::from(THE_GAP);
    let room = view_length - 2 * gap;
    if wanted > room {
        return at - gap - view_from;
    }
    let (inside_from, inside_to) = (view_from + gap, view_from + view_length - gap);
    if at < inside_from {
        at - inside_from
    } else if at + wanted > inside_to {
        at + wanted - inside_to
    } else {
        0
    }
}

/// Whether the window, and the view that shows it, are both on the plane.
fn is_on_the_plane(
    window: Rectangle<i32, Logical>,
    view: Point<i32, Logical>,
    view_size: Size<i32, Logical>,
) -> bool {
    let furthest = i64::from(alo_canvas::plane::FURTHEST);
    let inside = |from: i32, length: i32| {
        let from = i64::from(from);
        from >= -furthest && from + i64::from(length) <= furthest
    };
    inside(window.loc.x, window.size.w)
        && inside(window.loc.y, window.size.h)
        && inside(view.x, view_size.w)
        && inside(view.y, view_size.h)
}

/// Whether the window, grown by the gap, overlaps no open window.
fn touches_nothing(window: Rectangle<i32, Logical>, open: &[Rectangle<i32, Logical>]) -> bool {
    let grown = Rectangle::new(
        Point::from((
            window.loc.x.saturating_sub(THE_GAP),
            window.loc.y.saturating_sub(THE_GAP),
        )),
        Size::from((
            window.size.w.saturating_add(2 * THE_GAP),
            window.size.h.saturating_add(2 * THE_GAP),
        )),
    );
    !open.iter().any(|other| other.overlaps(grown))
}

/// Whether *Show all* could still gather every window with this one added.
fn show_all_still_reaches(
    window: Rectangle<i32, Logical>,
    open: &[Rectangle<i32, Logical>],
    reach: Option<Size<i32, Logical>>,
) -> bool {
    let Some(reach) = reach else {
        return true;
    };
    let mut from = (i64::from(window.loc.x), i64::from(window.loc.y));
    let mut to = (
        from.0 + i64::from(window.size.w),
        from.1 + i64::from(window.size.h),
    );
    for other in open {
        from.0 = from.0.min(i64::from(other.loc.x));
        from.1 = from.1.min(i64::from(other.loc.y));
        to.0 = to.0.max(i64::from(other.loc.x) + i64::from(other.size.w));
        to.1 = to.1.max(i64::from(other.loc.y) + i64::from(other.size.h));
    }
    to.0 - from.0 <= i64::from(reach.w) && to.1 - from.1 <= i64::from(reach.h)
}

/// Whether the window's handle is still pressable once the view has moved from
/// `now` to `then`: the fixed controls stay on the screen, so on the plane they
/// move with the view.
fn the_handle_is_not_buried(
    window: Rectangle<i32, Logical>,
    now: Point<i32, Logical>,
    then: Point<i32, Logical>,
    fixed: &[Rectangle<i32, Logical>],
) -> bool {
    let handle = Rectangle::new(
        window.loc,
        Size::from((A_HANDLE.0.min(window.size.w), A_HANDLE.1.min(window.size.h))),
    );
    let (across, down) = (then.x.saturating_sub(now.x), then.y.saturating_sub(now.y));
    !fixed.iter().any(|control| {
        let moved = Rectangle::new(
            Point::from((
                control.loc.x.saturating_add(across),
                control.loc.y.saturating_add(down),
            )),
            control.size,
        );
        moved.intersection(handle) == Some(handle)
    })
}

#[cfg(test)]
#[path = "the_nearest_empty_space_tests.rs"]
mod tests;
