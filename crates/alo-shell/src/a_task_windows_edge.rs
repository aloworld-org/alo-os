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

use smithay::utils::{Logical, Physical, Point, Rectangle, Size};

use crate::Server;
use crate::an_edge_in_its_displays_units::{
    a_point_in_the_edges_units, a_window_in_the_edges_units,
};
use crate::the_bands_over_alos_windows::AlosBand;
use crate::the_tasks_a_person_opened::WhichTask;
use crate::which_band::ABand;
use crate::window_edge::OnTheEdge;
#[cfg(test)]
use crate::window_edge::WindowEdge;
use crate::window_edge_picture::{EdgePicture, OverTheEdge};
use crate::window_edge_pointing::{UnderThePointer, pointing_at, what_is_under};

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
/// cent, exactly as `crate::every_windows_edge` takes them.
pub(crate) fn every_task_windows_edge(
    server: &Server,
    fonts: &mut cosmic_text::FontSystem,
    output: (i32, i32),
    scale: u32,
) -> Vec<EdgePicture> {
    let room = crate::window_edge::the_usable_output(Size::from(output), scale);
    let pointer = server.where_the_pointer_is_in_logical_pixels();
    let now = std::time::Instant::now();
    server
        .the_task_windows_on_the_screen()
        .into_iter()
        .filter_map(|(which, window)| {
            let (edge, how_shown) = server.alos_band_as_drawn(
                AlosBand::ATask(which),
                a_window_in_the_edges_units(window, scale),
                room,
                now,
            );
            let keyboard =
                server.where_the_keyboard_is_on_this_band(&ABand::Alos(AlosBand::ATask(which)));
            let pointing = pointing_at(
                &edge,
                pointer.map(|at| a_point_in_the_edges_units(at, scale)),
                keyboard.what_the_painter_shows(),
            );
            // **Its window panel, while it is open** (`crate::alos_windows_panel`).
            let (panel, pointing_at_it) = server
                .the_panel_drawn_over(AlosBand::ATask(which), &edge, (room, scale))
                .map_or((None, None), |(menu, pointing)| {
                    (Some(menu), Some(pointing))
                });
            let look = crate::window_edge_look::WhichLook::of(
                server.the_kind_in_front()
                    == crate::which_window_is_in_front::AKindOfWindow::TaskWindows
                    && server.tasks().the_one_in_front() == Some(which),
            );
            EdgePicture::of(
                &edge,
                OverTheEdge {
                    menu: panel.as_ref().zip(pointing_at_it),
                    field: None,
                    showing: None,
                    alo: None,
                },
                // **No title and no body from the edge**: a task window carries
                // its own title inside it and draws its own body. Its look is
                // the focused one while it is the window the keys go to.
                crate::window_edge_picture::TheWindow {
                    title: None,
                    application: None,
                    look,
                    body: None,
                    laid_over: false,
                    controls: crate::window_edge_paint::HowTheControlsLook {
                        pointing,
                        open: keyboard.the_menu().is_some(),
                        how_shown,
                        look,
                    },
                },
                fonts,
                output,
                scale,
            )
        })
        .collect()
}

#[cfg(test)]
#[path = "a_task_windows_edge_tests.rs"]
mod tests;
