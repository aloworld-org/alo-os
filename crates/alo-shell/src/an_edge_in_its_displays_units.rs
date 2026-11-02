//! A window and the pointer, in the units a window's edge is laid out in.
//!
//! `crate::window_edge` lays an edge out in **logical** units — the design's,
//! which a display's scale multiplies — and `crate::window_edge_picture`
//! multiplies them into that display's pixels exactly once. But a window's
//! rectangle and the pointer are kept in the display's **pixels**: clients are
//! told scale one (`crate::their_displays`), so the seat's coordinates are the
//! pixels a frame draws.
//!
//! So both are divided by the display's scale on the way in, here and only
//! here, by every road that lays an edge out: the draw, a press, a press in the
//! menu, the pointer revealing an edge, and the keyboard arriving on one. Until
//! 2026-10-28 only the task window's draw divided, and nothing else did — so on
//! a display at 200 per cent a client window's edge was drawn at twice its
//! window's distance from the corner, and a press on any edge, a task window's
//! Close included, was read against an edge laid out at 100: drawn in one
//! place and pressed in another. At 100 every division is the identity.
//!
//! # Which display's scale
//!
//! **The display whose frame was drawn last**, which is the display
//! `crate::Server::the_room_an_edge_is_clamped_in` already measures the room
//! from: the two are written together, in `crate::Server::render_frame`, so an
//! edge is never laid out in one display's room at another's scale. With two
//! displays at two scales a press on the other one is read at this one's, the
//! same limit the room has had since it was written.

use smithay::utils::{Logical, Point, Rectangle};

/// The ordinary scale, in per cent.
const ORDINARY: u32 = 100;

/// One length in the display's pixels as edge units at `scale` per cent,
/// rounded down — so every pixel of an edge drawn back at that scale falls
/// inside the edge unit it was drawn from.
///
/// **A scale of nought is one-to-one**, not a divisor of one: dividing by
/// `max(1)` multiplies every figure by a hundred, which is the fault
/// `crate::window_edge::the_usable_output` records about itself.
fn down(length: i32, scale: u32) -> i32 {
    let scale = if scale == 0 { ORDINARY } else { scale };
    i32::try_from(i64::from(length) * i64::from(ORDINARY) / i64::from(scale)).unwrap_or(length)
}

/// A window's rectangle in the display's pixels, as edge units at `scale`.
pub(crate) fn a_window_in_the_edges_units<Space>(
    at: Rectangle<i32, Space>,
    scale: u32,
) -> Rectangle<i32, Logical> {
    Rectangle::new(
        (down(at.loc.x, scale), down(at.loc.y, scale)).into(),
        (down(at.size.w, scale), down(at.size.h, scale)).into(),
    )
}

/// The pointer in the display's pixels, as edge units at `scale`.
pub(crate) fn a_point_in_the_edges_units(
    at: Point<i32, Logical>,
    scale: u32,
) -> Point<i32, Logical> {
    Point::from((down(at.x, scale), down(at.y, scale)))
}

impl crate::Server {
    /// Where an application's window is **drawn**, in the display's pixels:
    /// its rectangle on the plane, moved and scaled by the canvas camera.
    ///
    /// **The one road every edge takes to its window**, since 2026-11-01. Until
    /// then the draw, a press, a press in the menu, the pointer revealing an
    /// edge and the keyboard arriving on one each read `where_a_window_is`,
    /// which is the window on the **plane**, as though it were on the screen —
    /// so the moment a person panned or zoomed, every window's edge, line and
    /// shadow stayed where the window had been at 100 % and the window moved
    /// away from it. The window itself has always been drawn through the
    /// camera (`crate::scene::trees`); now its edge is drawn through the same
    /// arithmetic (`crate::the_terminal_on_the_plane::on_the_screen`).
    ///
    /// The edge's own furniture is not zoomed: a title and its controls stay
    /// the size the display draws them at, round a window of whatever size the
    /// zoom makes it, which is what a design canvas does with a frame's name.
    pub(crate) fn where_a_window_is_drawn(
        &self,
        surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
    ) -> Rectangle<i32, Logical> {
        let drawn = crate::the_terminal_on_the_plane::on_the_screen(
            crate::where_a_window_is(surface),
            self.the_camera(),
        );
        Rectangle::new(
            (drawn.loc.x, drawn.loc.y).into(),
            (drawn.size.w, drawn.size.h).into(),
        )
    }

    /// The scale an edge is laid out at for a press, a reveal or the keyboard:
    /// the last drawn display's, in per cent. See this file's header.
    pub(crate) fn the_scale_an_edge_is_measured_at(&self) -> u32 {
        u32::from(self.the_scale_edges_are_measured_at)
    }
}

#[cfg(test)]
#[path = "an_edge_in_its_displays_units_tests.rs"]
mod tests;
