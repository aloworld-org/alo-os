//! A task window's edge: the design's external window edge above it, carrying
//! **Close** and nothing else.
//!
//! The design draws every task window with the same `External window edge`
//! client windows have — 44 tall, above the window, its controls 44 × 44 at the
//! trailing end (`docs/design/the-ask-alo-task-window.md`). **The owner decided
//! on 2026-10-28 to draw Close now**, and minimise and maximise later with the
//! right-hand panel. So this lays the edge out through `crate::window_edge`
//! itself, with `Decorations::ATaskWindow`, and paints it through
//! `crate::window_edge_picture` — one edge, one set of measures, scaled at the
//! one boundary every other edge is.
//!
//! # Revealed as every band is
//!
//! No controls at rest: they appear as the pointer enters the band and go as
//! it leaves, and *Always show window controls* keeps them shown
//! (`crate::the_bands_over_alos_windows`), as on every window since the
//! owner's specification of 2026-11-09 (`docs/design/the-window-controls.md`).
//! Until 2026-11-14 this edge was laid out revealed — *a Close that appears
//! only under a pointer is one a person must hunt for* — a reading of
//! 2026-10-28 the owner overruled on seeing a booted machine. There is
//! nothing to drag on it, because a task window does not move.
//!
//! # A press on Close follows Alt+F4's rules
//!
//! It is `Server::the_person_closes_a_task`, the same door the keyboard uses:
//! a finished task closes, a running one does not yet, and a change waiting is
//! never answered.
//!
//! # Scale
//!
//! A task window is placed and drawn with logical and physical as one number
//! (`crate::where_a_task_window_opens`). The edge is laid out from the window's
//! pixels divided by the display's scale, then scaled back by
//! `EdgePicture::of`, so it sits on the window at any scale and its own figures
//! are the design's, scaled. A press and the pointer's highlight are read the
//! same way: the pointer divided by the scale and tested against the edge laid
//! out from the divided window, which is `crate::an_edge_in_its_displays_units`.
//! Until 2026-10-28 a press was read against the edge laid out at 100, so on a
//! 200 per cent display Close was drawn where a press did not find it.

#[cfg(test)]
use smithay::utils::{Logical, Rectangle, Size};

use crate::Server;
use crate::the_bands_over_alos_windows::AlosBand;
#[cfg(test)]
use crate::window_edge::WindowEdge;
use crate::window_edge_picture::EdgePicture;

/// The edge above one task window **with its controls**, in `room` — the
/// geometry a test reads where they would be. The machine never lays one out
/// like this: it asks the band's reveal (`Server::alos_band_as_pressed`).
#[cfg(test)]
#[must_use]
pub(crate) fn the_edge_of_a_task_window(
    window: Rectangle<i32, Logical>,
    room: Size<i32, Logical>,
) -> WindowEdge {
    crate::window_edge::edge_of(
        window,
        crate::window_edge::Decorations::ATaskWindow,
        true,
        room,
    )
}

/// Every open task window's edge, as pictures for this display.
///
/// `output` is the display's size in pixels and `scale` its scale in per
/// cent, exactly as `crate::every_windows_edge` takes them. Each is drawn by
/// the one function every band is (`crate::a_band_is_drawn`).
pub(crate) fn every_task_windows_edge(
    server: &Server,
    fonts: &mut cosmic_text::FontSystem,
    says: &alo_strings::Strings,
    output: (i32, i32),
    scale: u32,
) -> Vec<EdgePicture> {
    server
        .the_task_windows_on_the_screen()
        .into_iter()
        .filter_map(|(which, _)| {
            server.the_band_drawn(
                &crate::which_band::ABand::Alos(AlosBand::ATask(which)),
                says,
                fonts,
                (output, scale),
            )
        })
        .collect()
}

#[cfg(test)]
#[path = "a_task_windows_edge_tests.rs"]
mod tests;
