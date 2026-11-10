//! **A full-screen window's band, out of sight until the top edge is reached.**
//!
//! `docs/design/the-window-controls.md`: *Full screen fills the display and
//! hides the band … In full screen a 12-unit zone at the top of the display
//! reveals the title and controls together as an overlay that does not resize
//! or push anything.* The owner's ruling of 2026-11-09 holds here as on every
//! band: a continuous path and no delays — the band stays while the pointer is
//! in it or the panel it opened, or while the keyboard is on it, and goes as
//! soon as none of those is true.
//!
//! # Hidden is where the band already is
//!
//! A band hangs **above** its window (`crate::window_edge`), and a full-screen
//! window's top is the display's, so its band is off the top of the display:
//! not drawn, because an edge that falls off its output is not
//! (`crate::window_edge_picture`), and not pressed or revealed, because no
//! pointer reaches above a display. Nothing has to hide it.
//!
//! **Out is the same band hung 54 units lower**, over the window's own top —
//! [`crate::Server::the_window_a_band_hangs_from`] is the one rectangle every
//! band reader asks for, so drawing, presses, the reveal and the keyboard all
//! see the band in the same place. The window itself is never moved or
//! resized: only the rectangle the band is laid out from changes.
//!
//! # What brings it out and what keeps it
//!
//! [`is_the_band_out`]: the pointer in the top-edge zone across the window, or
//! the keyboard on the band or in its panel — `Alt+F3` is the keyboard's road
//! to the same actions — brings it out; once out, the pointer anywhere in the
//! band keeps it. The panel being open keeps it because the panel is the
//! keyboard's state here (`crate::window_edge_keys`).

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point, Rectangle};

use crate::window_edge::THE_REGION_IS_TALL;

/// How tall the zone at the top of a full-screen window is that brings its
/// band out, in the band's units: the specification's *about 12*.
pub(crate) const THE_TOP_EDGE_IS: i32 = 12;

/// Whether a full-screen window's band is out, given whether it was, where the
/// pointer is and whether the keyboard holds it — all in the band's units, with
/// `window` the window as drawn.
#[must_use]
pub(crate) fn is_the_band_out(
    was_out: bool,
    window: Rectangle<i32, Logical>,
    pointer: Option<Point<i32, Logical>>,
    the_keyboard_holds_it: bool,
) -> bool {
    if the_keyboard_holds_it {
        return true;
    }
    let Some(at) = pointer else {
        return false;
    };
    let across = at.x >= window.loc.x && at.x < window.loc.x + window.size.w;
    let down = at.y - window.loc.y;
    let reach = if was_out {
        THE_REGION_IS_TALL
    } else {
        THE_TOP_EDGE_IS
    };
    across && (0..reach).contains(&down)
}

/// The rectangle a band out over full screen is laid out from: the window's
/// own, moved one band down so the band lies over its top — and **starting
/// after the egress indicator** where the indicator is on that band.
///
/// The indicator owns the display's top-left corner and is drawn above every
/// window (`crate::top_controls_region`, the owner's ruling of 2026-10-31), so
/// a band laid over a full-screen window's top meets it. A booted machine on
/// 2026-11-09 drew the title under it. The band gives way rather than the
/// indicator, which is law 1's: *nothing leaves silently* is never covered.
/// `indicators` are where it was last drawn on each display, in band units.
#[must_use]
pub(crate) fn hung_over_its_top(
    drawn: Rectangle<i32, Logical>,
    indicators: &[Rectangle<i32, Logical>],
) -> Rectangle<i32, Logical> {
    let band = Rectangle::new(drawn.loc, (drawn.size.w, THE_REGION_IS_TALL).into());
    let starts = indicators
        .iter()
        .filter(|it| it.size.w > 0 && it.size.h > 0 && it.overlaps(band))
        .map(|it| it.loc.x + it.size.w)
        .fold(drawn.loc.x, i32::max)
        .min(drawn.loc.x + drawn.size.w);
    Rectangle::new(
        (starts, drawn.loc.y + THE_REGION_IS_TALL).into(),
        (
            drawn.size.w - (starts - drawn.loc.x),
            (drawn.size.h - THE_REGION_IS_TALL).max(0),
        )
            .into(),
    )
}

impl crate::Server {
    /// The window a band is laid out from, in the band's units at `scale`.
    ///
    /// The window as drawn — except a full-screen window whose band is out,
    /// whose band is hung over its own top, so the band is on the display.
    pub(crate) fn the_window_a_band_hangs_from(
        &self,
        surface: &WlSurface,
        scale: u32,
    ) -> Rectangle<i32, Logical> {
        let drawn = crate::an_edge_in_its_displays_units::a_window_in_the_edges_units(
            self.where_a_window_is_drawn(surface),
            scale,
        );
        if !self.bands_out_over_full_screen.contains(surface) {
            return drawn;
        }
        let indicators: Vec<Rectangle<i32, Logical>> = self
            .surfaces
            .rooms
            .iter()
            .filter_map(|(_, _, reserved)| reserved.leaving)
            .map(|it| crate::an_edge_in_its_displays_units::a_window_in_the_edges_units(it, scale))
            .collect();
        hung_over_its_top(drawn, &indicators)
    }

    /// Bring each full-screen window's band out, or put it away, for where the
    /// pointer and the keyboard are now. Every other window's is forgotten.
    pub(crate) fn the_full_screen_bands_were_told(
        &mut self,
        mapped: &[WlSurface],
        pointer: Option<Point<i32, Logical>>,
        scale: u32,
    ) {
        let mut out = Vec::new();
        for surface in mapped {
            if self.surfaces.requested_window_mode(surface)
                != crate::window_mode::Mode::FillingTheScreen
            {
                continue;
            }
            let window = crate::an_edge_in_its_displays_units::a_window_in_the_edges_units(
                self.where_a_window_is_drawn(surface),
                scale,
            );
            let held = self.where_the_keyboard_is_on_this_windows_edge(surface)
                != crate::window_edge_keys::TheKeyboardOnTheEdge::Nowhere;
            let was_out = self.bands_out_over_full_screen.contains(surface);
            if is_the_band_out(was_out, window, pointer, held) {
                out.push(surface.clone());
            }
        }
        self.bands_out_over_full_screen = out;
    }

    /// Whether this full-screen window's band is out.
    #[must_use]
    pub(crate) fn is_its_band_out_over_full_screen(&self, surface: &WlSurface) -> bool {
        self.bands_out_over_full_screen.contains(surface)
    }
}

#[cfg(test)]
#[path = "the_band_over_full_screen_tests.rs"]
mod tests;
