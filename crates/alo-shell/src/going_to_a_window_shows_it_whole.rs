//! **Going to a window shows it whole: its name band, its controls and as much
//! of it as fits, above the Dock.**
//!
//! `docs/design/the-canvas-as-a-workspace.md` §11: *if the chosen window is
//! elsewhere: travel to its Place and location*. The travel itself is
//! `Server::the_view_travels_to`; this file answers **where the view stops**.
//!
//! # What was wrong
//!
//! Seen on a booted machine on 2026-11-13 (qemu, 1 280 × 800): Super+Tab to
//! Dolphin, which was out of sight, brought it into view with its name band
//! off the top of the screen (`w26-37`). Confirmed by reading the code: the
//! view was pointed at the window's own corner, so the window's top-left was
//! the screen's top-left and the band, which is drawn **above** the window
//! (`crate::a_window_and_its_strip`), was off the screen — and with it the
//! window's close and put-aside controls. The Dock was not counted either, so a
//! tall window's bottom stopped under it.
//!
//! # The rule now
//!
//! The view stops where the window **and its band** lie in the room the Dock
//! leaves (`crate::the_room_alo_places_a_window_in::the_room_the_dock_leaves`),
//! **centred** on each side where it fits — as alo's own windows already travel
//! (`crate::the_view_travels_to_a_window`). A window too big for the room on a
//! side has its band's edge the 16 gap inside the room on that side, so what a
//! person takes hold of is on the screen.
//!
//! When the view travels is unchanged: always to another Place, and on this one
//! only when none of the window is in sight (`crate::canvas_a_restore_travels`).
//! The window is never moved.
//!
//! **This makes an existing promise true rather than adding one**: §11's
//! *travel to its location* means arriving where the window can be seen and
//! used, and a window whose band is off the screen cannot be closed or put
//! aside from where the travel left it.

use alo_canvas::{At, Frame};
use smithay::utils::{Logical, Physical, Rectangle, Size};

use crate::a_window_and_its_strip::ItsStrip;

/// The gap kept inside the room for a window too big for it: the same 16 a
/// window alo places is kept from the room's edge.
const THE_GAP: i32 = crate::where_a_window_opens::A_SMALL_GAP;

/// **Where the view's corner goes on the plane** so that `taken` — a window
/// with its band — lies in a room `room` big that starts at the view's corner:
/// centred on each side where it fits, else its band's edge the gap inside.
#[must_use]
pub(crate) fn where_the_view_looks(
    taken: Rectangle<i32, Logical>,
    room: Size<i32, Logical>,
) -> (i32, i32) {
    let along = |from: i32, length: i32, room: i32| -> i32 {
        if length <= room {
            from.saturating_sub(room.saturating_sub(length) / 2)
        } else {
            from.saturating_sub(THE_GAP.min(room.max(0)))
        }
    };
    (
        along(taken.loc.x, taken.size.w, room.w),
        along(taken.loc.y, taken.size.h, room.h),
    )
}

/// **The room the Dock leaves on a screen `screen` big, as a size on the
/// plane** at `zoom`: the screen down to the Dock's top, divided by the zoom
/// and rounded down so a window never counts as fitting a room it overflows.
/// [`None`] for a zoom that is not a number above nought.
#[must_use]
pub(crate) fn the_room_on_the_plane(
    screen: Size<i32, Physical>,
    dock: Option<Rectangle<i32, Physical>>,
    zoom: f64,
) -> Option<Size<i32, Logical>> {
    if !zoom.is_finite() || zoom <= 0.0 {
        return None;
    }
    let as_logical = |it: Rectangle<i32, Physical>| -> Rectangle<i32, Logical> {
        Rectangle::new((it.loc.x, it.loc.y).into(), (it.size.w, it.size.h).into())
    };
    let room = crate::the_room_alo_places_a_window_in::the_room_the_dock_leaves(
        as_logical(Rectangle::from_size(screen)),
        dock.map(as_logical),
    );
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a display's pixels over a zoom of at least a twentieth fit an i32"
    )]
    let on_the_plane = |pixels: i32| -> i32 { (f64::from(pixels) / zoom).floor() as i32 };
    Some(Size::from((
        on_the_plane(room.size.w),
        on_the_plane(room.size.h),
    )))
}

impl crate::Server {
    /// **Where the view goes to show `frame` whole**, band and all, above the
    /// Dock, at the zoom the person has. [`None`] before anything is drawn,
    /// where there is no screen to fit it to.
    pub(crate) fn where_the_view_shows_it_whole(&self, frame: Frame) -> Option<At> {
        let screen = self.surfaces.popups.output_size?;
        let zoom = crate::scene::drawn_at(self.the_camera());
        let dock = self
            .the_controls_of_the_display_for(None)
            .and_then(crate::canvas_fixed_controls::FixedControls::labelled)
            .and_then(|drawn| drawn.dock_band)
            .filter(|band| band.size.w > 0 && band.size.h > 0);
        let room = the_room_on_the_plane(screen, dock, zoom)?;
        let strip = ItsStrip::at(self.the_display_scale_edges_are_measured_at());
        let size = frame.size();
        let window = Rectangle::<i32, Logical>::new(
            (frame.at().x, frame.at().y).into(),
            (
                i32::try_from(size.width()).ok()?,
                i32::try_from(size.height()).ok()?,
            )
                .into(),
        );
        let (x, y) = where_the_view_looks(strip.round(window), room);
        At::checked(x, y)
    }
}

#[cfg(test)]
#[path = "going_to_a_window_shows_it_whole_tests.rs"]
mod tests;
