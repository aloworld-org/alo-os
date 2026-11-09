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
pub(crate) fn every_windows_edge(
    server: &mut crate::Server,
    labels: &mut crate::WindowControlLabels,
    output: (i32, i32),
    scale: u32,
) -> Vec<EdgePicture> {
    server.the_edges_were_told_where_the_pointer_is();
    let now = std::time::Instant::now();
    let room = crate::window_edge::the_usable_output(
        smithay::utils::Size::from((output.0, output.1)),
        scale,
    );
    let open: Vec<_> = server.mapped_surfaces().cloned().collect();
    open.iter()
        .filter_map(|surface| {
            let window = server.the_window_a_band_hangs_from(surface, scale);
            // **Laid out with its controls while any of them is still to be
            // seen**, so a fade out has something to draw. What a press
            // reaches is asked separately, of the reveal itself
            // (`crate::a_press_on_an_edge`), so a control fading out is drawn
            // and is not pressed.
            let how_shown = server.how_shown_its_controls_are(surface, now);
            let edge = crate::window_edge::edge_of(
                window,
                crate::window_edge_who_draws::who_draws_a_frame(),
                how_shown > 0 || server.is_this_windows_edge_revealed(surface),
                room,
            );
            let title = server.the_name_of(surface);
            let application = server.the_application_on_the_band(surface);
            let keyboard = server.where_the_keyboard_is_on_this_windows_edge(surface);
            let pointing = server.what_this_edge_is_pointing_at_on(
                &edge,
                keyboard.what_the_painter_shows(),
                scale,
            );
            let menu = keyboard.the_menu().and_then(|in_the_menu| {
                crate::window_menu::the_menu_of(&edge, server.how_this_window_is(surface), room)
                    .map(|menu| (menu, in_the_menu.on()))
            });
            let hovered_in_the_menu = menu.as_ref().and_then(|(menu, _)| {
                server
                    .where_the_pointer_is_in_logical_pixels()
                    .map(|at| {
                        crate::an_edge_in_its_displays_units::a_point_in_the_edges_units(at, scale)
                    })
                    .and_then(|at| menu.what_is_under(at))
                    .map(|row| row.does)
            });
            EdgePicture::of(
                &edge,
                crate::window_edge_picture::OverTheEdge {
                    menu: menu.as_ref().map(|(menu, on)| {
                        (
                            menu,
                            crate::window_menu_paint::PointingAtTheMenu {
                                hovered: hovered_in_the_menu,
                                focused: *on,
                            },
                        )
                    }),
                    showing: the_reason_to_show_a_tooltip(server, &edge, keyboard, scale),
                },
                crate::window_edge_picture::TheWindow {
                    title: crate::the_name_on_the_band::the_title_to_show(&title),
                    application: application.as_deref(),
                    look: crate::window_edge_look::WhichLook::of(
                        server.is_the_window_typing_goes_to(surface),
                    ),
                    // **No line or shadow round a band out over full
                    // screen**: there is no window card to draw it round,
                    // only the band laid over the application's top.
                    body: (!server.is_its_band_out_over_full_screen(surface)).then_some(window),
                    controls: crate::window_edge_paint::HowTheControlsLook {
                        pointing,
                        open: keyboard.the_menu().is_some(),
                        how_shown,
                        look: crate::window_edge_look::WhichLook::of(
                            server.is_the_window_typing_goes_to(surface),
                        ),
                    },
                },
                &mut labels.fonts,
                output,
                scale,
            )
            .map(|picture| picture.on_the_window(surface.clone()))
        })
        .collect()
}

#[cfg(test)]
#[path = "every_windows_edge_tests.rs"]
mod tests;

/// Why a tooltip is shown on this band: the pointer on its band or a control,
/// or the keyboard on it — and nothing while the panel is open, which says
/// everything a tooltip would.
fn the_reason_to_show_a_tooltip(
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
