//! **The edge over one of alo's own windows** — the terminal, Settings and the
//! record — laid out, drawn and pressed through the application edge itself.
//!
//! Each of those windows is drawn by alo and placed on the canvas by ADR 0145,
//! and each wears the edge every application's window wears: its icon, its
//! title and Close, in the focused look while the keys go to it and the
//! unfocused one otherwise (`crate::window_edge_look`). The edge is laid out by
//! `crate::window_edge::edge_of` with `Decorations::AlosOwnWindow`, painted by
//! `crate::window_edge_picture::EdgePicture` and pressed through
//! `crate::window_edge_pointing` — one edge, one set of measures — **from where
//! the camera draws the window**, so it pans and zooms with it as #1084 made
//! every application's edge do.
//!
//! **Always revealed**, because it carries the only Close the window has.

use smithay::utils::{Logical, Physical, Point, Rectangle, Size};

use crate::an_edge_in_its_displays_units::{
    a_point_in_the_edges_units, a_window_in_the_edges_units,
};
use crate::window_edge::{Decorations, OnTheEdge, WindowEdge, edge_of};
use crate::window_edge_picture::{EdgePicture, OverTheEdge};
use crate::window_edge_pointing::{UnderThePointer, pointing_at, what_is_under};

/// The edge over a window drawn at `window` on the display, laid out at
/// `scale` per cent in `room`.
#[must_use]
pub(crate) fn the_edge_over(
    window: Rectangle<i32, Physical>,
    room: Size<i32, Logical>,
    scale: u32,
) -> WindowEdge {
    edge_of(
        a_window_in_the_edges_units(window, scale),
        Decorations::AlosOwnWindow,
        true,
        room,
    )
}

/// What one of alo's windows shows on its edge.
#[derive(Debug, Clone, Copy)]
pub(crate) struct WhatItShows<'a> {
    /// Its title, in the person's language.
    pub(crate) title: &'a str,
    /// Its icon, where it has one; the grip's mark is drawn where it has none,
    /// as for an application whose icon this machine does not know.
    pub(crate) icon: Option<&'a alo_dock::artwork::APicture>,
    /// Whether the keys go to it.
    pub(crate) focused: bool,
}

/// The edge over a window drawn at `window`, as a picture for a display
/// `output` pixels big at `scale` per cent, with the pointer at `pointer`.
pub(crate) fn its_picture(
    window: Rectangle<i32, Physical>,
    shows: WhatItShows<'_>,
    pointer: Option<Point<i32, Logical>>,
    fonts: &mut cosmic_text::FontSystem,
    (output, scale): ((i32, i32), u32),
) -> Option<EdgePicture> {
    let room = crate::window_edge::the_usable_output(Size::from(output), scale);
    let edge = the_edge_over(window, room, scale);
    let pointing = pointing_at(
        &edge,
        pointer.map(|at| a_point_in_the_edges_units(at, scale)),
        None,
    );
    EdgePicture::of(
        &edge,
        pointing,
        OverTheEdge {
            menu: None,
            showing: None,
        },
        // **No body from the edge**: each of these windows draws its own
        // panel, hairline and shadow.
        crate::window_edge_picture::TheWindow {
            title: Some(shows.title),
            look: crate::window_edge_look::WhichLook::of(shows.focused),
            icon: shows.icon,
            body: None,
        },
        fonts,
        output,
        scale,
    )
}

/// What a press at `at`, in the display's pixels, is on, on the edge over a
/// window drawn at `window` — [`None`] off it.
pub(crate) fn what_a_press_is_on(
    window: Rectangle<i32, Physical>,
    at: Point<i32, Logical>,
    (room, scale): (Size<i32, Logical>, u32),
) -> Option<OnItsEdge> {
    let edge = the_edge_over(window, room, scale);
    match what_is_under(&edge, a_point_in_the_edges_units(at, scale))? {
        UnderThePointer::Control(OnTheEdge::Close) => Some(OnItsEdge::Close),
        UnderThePointer::Control(_)
        | UnderThePointer::Drag
        | UnderThePointer::TheEdgeAndNothingOnIt => Some(OnItsEdge::TheRest),
    }
}

/// Where on one of alo's windows' edges a press is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OnItsEdge {
    /// Close.
    Close,
    /// Anywhere else on it, which brings the window in front.
    TheRest,
}

#[cfg(test)]
#[path = "an_alo_windows_edge_tests.rs"]
mod tests;
