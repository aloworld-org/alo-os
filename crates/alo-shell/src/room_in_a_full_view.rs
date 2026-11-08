//! **One rule for every window a person opens into a full view**: rules 3 to
//! 7's answer where there is room, and where there is none and the person kept
//! *Glide to empty space*, the nearest empty space and a glide to it.
//!
//! `docs/design/where-a-new-window-opens.md`, amended 2026-11-07. The owner's
//! first answer was about application windows; asked whether alo's own — the
//! terminal, Settings, the record and task windows — glide too, the owner
//! answered the same day: *"Yes, the same"*. So this is the one place the rule
//! is written, and each caller hands it the answer rules 3 to 7 gave and the
//! windows that answer must not cover:
//!
//! - `crate::a_new_window_is_placed`, for an application window the person
//!   opened;
//! - `crate::the_terminal_on_the_plane`, `crate::settings_on_the_plane` and
//!   `crate::record_on_the_plane`, for alo's terminal, Settings and the record;
//! - `crate::the_task_windows_find_room`, for a task window.
//!
//! **Who may ask is the caller's.** alo's own windows open only when the person
//! opens them — a chord, a press, a request they made — so each of those
//! callers asks every time; an application's window asks only when the person
//! opened it (rule 8).

use smithay::utils::{Logical, Point, Rectangle, Size};

/// Whether rules 3 to 7 found a window of `wanted` size **room in view** at
/// `at`: over no open window, and wholly in the view — or, for a window larger
/// than the view, with its handle in it, which is as much of it as any view
/// can show. Anything else is the case the person's choice is about.
pub(crate) fn there_is_room_at(
    at: Point<i32, Logical>,
    wanted: Size<i32, Logical>,
    open: &[Rectangle<i32, Logical>],
    view: Rectangle<i32, Logical>,
) -> bool {
    let window = Rectangle::new(at, wanted);
    if open.iter().any(|other| other.overlaps(window)) {
        return false;
    }
    let fits = wanted.w <= view.size.w && wanted.h <= view.size.h;
    let seen = if fits {
        window
    } else {
        Rectangle::new(at, Size::from((wanted.w.min(44), wanted.h.min(24))))
    };
    view.contains_rect(seen)
}

/// What a full view asks of one opening: the windows already open on the
/// Place, the view, and the fixed controls on the plane.
pub(crate) struct AnOpening<'a> {
    /// What the person is looking at, on the plane.
    pub(crate) view: Rectangle<i32, Logical>,
    /// Every window on this Place the new one must not cover.
    pub(crate) open: &'a [Rectangle<i32, Logical>],
    /// How big the new window is.
    pub(crate) wanted: Size<i32, Logical>,
    /// The fixed controls, on the plane at the view as it is.
    pub(crate) fixed: &'a [Rectangle<i32, Logical>],
    /// The person's reading direction.
    pub(crate) reading: alo_strings::Direction,
    /// The title strip above every window at the zoom this opens at: what
    /// each window takes is itself and its strip (`crate::a_window_and_its_strip`).
    /// `open`, `wanted`, `first` and the answer are the windows alone.
    pub(crate) strip: crate::a_window_and_its_strip::ItsStrip,
}

impl crate::Server {
    /// **Where a window the person opened goes**, given rules 3 to 7's answer
    /// `first`: `first` itself where there was room or the person chose *Open
    /// in front*; otherwise the nearest empty space, with the view set gliding
    /// to it — or jumping, with less motion asked for. A Place with no space
    /// left within *Show all*'s reach keeps `first`, rule 5, rather than a
    /// window that could not be brought back.
    pub(crate) fn room_for_what_the_person_opened(
        &mut self,
        first: Point<i32, Logical>,
        opening: &AnOpening<'_>,
        now: std::time::Instant,
    ) -> Point<i32, Logical> {
        let strip = opening.strip;
        let open = strip.round_each(opening.open);
        let wanted = strip.round_a_size(opening.wanted);
        let taken = strip.round(Rectangle::new(first, opening.wanted)).loc;
        if !self.when_there_is_no_room.glides()
            || there_is_room_at(taken, wanted, &open, opening.view)
        {
            return first;
        }
        let reach = self.as_far_as_show_all_reaches().and_then(|reach| {
            Some(Size::<i32, Logical>::from((
                i32::try_from(reach.width()).ok()?,
                i32::try_from(reach.height()).ok()?,
            )))
        });
        let Some(found) = crate::the_nearest_empty_space::the_nearest_empty_space(
            opening.view,
            &open,
            wanted,
            opening.fixed,
            reach,
            opening.reading,
        ) else {
            return first;
        };
        if found.the_view_moves(opening.view.loc)
            && let Some(to) = alo_canvas::At::checked(found.view.x, found.view.y)
        {
            let _moving = self.the_view_glides_to(to, now);
        }
        strip.the_window_in(found.window, opening.wanted)
    }
}

#[cfg(test)]
#[path = "room_in_a_full_view_tests.rs"]
mod tests;
