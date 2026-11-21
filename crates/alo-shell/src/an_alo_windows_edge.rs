//! **The edge over one of alo's own windows** — the terminal, Settings and the
//! record — laid out, drawn and pressed through the application edge itself.
//!
//! Each of those windows is drawn by alo and placed on the canvas by ADR 0145,
//! and each wears the band every application's window wears: its title and
//! Close, in the focused look while the keys go to it and the unfocused one
//! otherwise (`crate::window_edge_look`), and Minimise, which puts the window
//! in the right-hand panel (`crate::alos_own_window_goes_aside`). The edge is
//! laid out by `crate::window_edge::edge_of` with `Decorations::AlosOwnWindow`,
//! painted by `crate::window_edge_picture::EdgePicture` and pressed through
//! `crate::window_edge_pointing` — one edge, one set of measures — **from where
//! the camera draws the window**, so it pans and zooms with it as #1084 made
//! every application's edge do.
//!
//! **Revealed as every band is** (`crate::the_bands_over_alos_windows`): no
//! controls at rest, shown while the pointer is in the band, always shown for a
//! person who asked for that. Until 2026-11-14 this band was laid out revealed
//! and drawn shown whatever the pointer did.

use std::time::Instant;

use alo_put_aside::AlosOwnWindow;
use smithay::utils::{Logical, Physical, Point, Rectangle, Size};

use crate::an_edge_in_its_displays_units::{
    a_point_in_the_edges_units, a_window_in_the_edges_units,
};
use crate::the_bands_over_alos_windows::AlosBand;
use crate::window_edge::{OnTheEdge, WindowEdge};
use crate::window_edge_picture::{EdgePicture, OverTheEdge};
use crate::window_edge_pointing::{UnderThePointer, pointing_at, what_is_under};

/// The edge over a window drawn at `window` on the display, laid out at
/// `scale` per cent in `room`, **with its controls** — the geometry a test
/// reads where they would be. The machine never lays one out like this: it
/// asks the band's reveal (`Server::alos_band_as_pressed`).
#[cfg(test)]
#[must_use]
pub(crate) fn the_edge_over(
    window: Rectangle<i32, Physical>,
    room: Size<i32, Logical>,
    scale: u32,
) -> WindowEdge {
    crate::window_edge::edge_of(
        a_window_in_the_edges_units(window, scale),
        crate::window_edge::Decorations::AlosOwnWindow,
        true,
        room,
    )
}

/// What one of alo's windows shows on its edge.
#[derive(Debug, Clone, Copy)]
pub(crate) struct WhatItShows<'a> {
    /// Its title, in the person's language.
    pub(crate) title: &'a str,
    /// Whether the keys go to it.
    pub(crate) focused: bool,
}

/// The edge over `which`, drawn at `window`, as a picture for a display
/// `output` pixels big at `scale` per cent — its controls as shown as its
/// band's reveal says.
pub(crate) fn its_picture(
    server: &crate::Server,
    (which, window): (AlosOwnWindow, Rectangle<i32, Physical>),
    shows: WhatItShows<'_>,
    fonts: &mut cosmic_text::FontSystem,
    (output, scale): ((i32, i32), u32),
) -> Option<EdgePicture> {
    let room = crate::window_edge::the_usable_output(Size::from(output), scale);
    let (edge, how_shown) = server.alos_band_as_drawn(
        AlosBand::Its(which),
        a_window_in_the_edges_units(window, scale),
        room,
        Instant::now(),
    );
    let pointing = pointing_at(
        &edge,
        server
            .where_the_pointer_is_in_logical_pixels()
            .map(|at| a_point_in_the_edges_units(at, scale)),
        None,
    );
    let look = crate::window_edge_look::WhichLook::of(shows.focused);
    EdgePicture::of(
        &edge,
        OverTheEdge {
            menu: None,
            field: None,
            showing: None,
            alo: None,
        },
        // **No body from the edge**: each of these windows draws its own
        // panel, hairline and shadow.
        crate::window_edge_picture::TheWindow {
            title: Some(shows.title),
            application: None,
            look,
            body: None,
            laid_over: false,
            controls: crate::window_edge_paint::HowTheControlsLook {
                pointing,
                open: false,
                how_shown,
                look,
            },
        },
        fonts,
        output,
        scale,
    )
}

#[cfg(test)]
#[path = "an_alo_windows_edge_tests.rs"]
mod tests;
