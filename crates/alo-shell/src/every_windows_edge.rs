//! Every open window's edge, for one display, built where a machine draws.
//!
//! # The gap this closes
//!
//! `crate::window_edge` laid an edge out, `crate::window_edge_paint` coloured
//! it, `crate::window_edge_picture` turned it into pixels, and
//! `crate::scene_native` carried it — and until this existed the only caller of
//! `edge_of` outside a test was `examples/a_real_application.rs`. Every piece of
//! the edge was built, measured against the design file's own coordinates and
//! tested, and **no window on a machine had one**.
//!
//! That is the fault this repository keeps meeting from the far end: a surface
//! complete in every part and reached by nothing. The owner's bar for this work
//! was written for it — *an exported function with no caller is not evidence of
//! working behaviour*.
//!
//! # Per display, because an edge is pixels and pixels belong to a screen
//!
//! `EdgePicture::of` converts logical units to physical ones exactly once,
//! against one output's size and one display's scale. So this is asked per
//! display rather than once per frame: the same window on two displays of
//! different scales is two different pictures, and a single list would be right
//! for whichever display it was built against and silently wrong for the other.
//!
//! An edge that would fall off its output is **not drawn** — `EdgePicture::of`
//! answers `None` and `filter_map` drops it, which is the refusal that stops a
//! window near the top of one display having its edge painted onto the next.
//!
//! # What it asks, and what it does not decide
//!
//! Nothing here chooses anything. Where the window is is
//! `crate::window_placement`'s; who draws its header is
//! `crate::window_edge_who_draws`'s one answer, shared with the accessibility
//! tree so a control drawn and a control named cannot disagree; whether the
//! edge is revealed is the host's, through §5's `Revealing`; and the name is the
//! application's own string, never translated.

use crate::window_edge_picture::EdgePicture;

/// Every edge to draw on this display, in `mapped_surfaces` order.
///
/// `output` is this display's size in physical pixels and `scale` its own
/// scale — see this file's header for why both are per display.
///
/// **The pointer is told first.** `the_edges_were_told_where_the_pointer_is`
/// advances one `alo_dock::Revealing` per window from where the pointer
/// actually is, and asking each window afterwards is what makes the reveal this
/// window's own answer rather than a literal. A frame that asked without
/// telling would draw the previous frame's reveal.
///
/// **Each band is drawn by the one function every band is**,
/// `Server::the_band_drawn` (`crate::a_band_is_drawn`).
pub(crate) fn every_windows_edge(
    server: &mut crate::Server,
    labels: &mut crate::WindowControlLabels,
    says: &alo_strings::Strings,
    output: (i32, i32),
    scale: u32,
) -> Vec<EdgePicture> {
    server.the_edges_were_told_where_the_pointer_is();
    let open: Vec<_> = server.mapped_surfaces().cloned().collect();
    open.into_iter()
        .filter_map(|surface| {
            let picture = server.the_band_drawn(
                &crate::which_band::ABand::App(surface.clone()),
                says,
                &mut labels.fonts,
                (output, scale),
            )?;
            // **alo's border was drawn round this window**, so the corner may
            // leave its copy's sending to the border (the owner, 2026-11-15:
            // *the border is enough*). Only a border drawn counts: a picture
            // left off the screen returned before this.
            if server.alo_wears_its_border_round(&surface) {
                server.alos_border_was_drawn_round(&surface);
            }
            Some(picture)
        })
        .collect()
}

#[cfg(test)]
#[path = "every_windows_edge_tests.rs"]
mod tests;

/// Why a tooltip is shown on this band: the pointer on its band or a control,
/// or the keyboard on it — and nothing while the panel is open, which says
/// everything a tooltip would.
pub(crate) fn the_reason_to_show_a_tooltip(
    server: &crate::Server,
    edge: &crate::window_edge::WindowEdge,
    keyboard: crate::window_edge_keys::TheKeyboardOnTheEdge,
    scale: u32,
) -> Option<crate::window_edge_tooltip::Because> {
    match keyboard {
        crate::window_edge_keys::TheKeyboardOnTheEdge::InTheMenu(_) => return None,
        crate::window_edge_keys::TheKeyboardOnTheEdge::OnAControl(_) => {
            return Some(crate::window_edge_tooltip::Because::TheKeyboardIsOnIt);
        }
        crate::window_edge_keys::TheKeyboardOnTheEdge::Nowhere => {}
    }
    let at = server
        .where_the_pointer_is_in_logical_pixels()
        .map(|at| crate::an_edge_in_its_displays_units::a_point_in_the_edges_units(at, scale))?;
    crate::window_edge_pointing::what_is_under(edge, at)
        .map(|_| crate::window_edge_tooltip::Because::APointerIsOnIt)
}
