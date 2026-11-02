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
    let open: Vec<_> = server.mapped_surfaces().cloned().collect();
    open.iter()
        .filter_map(|surface| {
            // **In the edge's units at this display's scale**: the window is in
            // the display's pixels and `EdgePicture::of` multiplies by the
            // scale, so without the division a dense display drew every edge
            // at twice its window's distance from the corner
            // (`crate::an_edge_in_its_displays_units`).
            let window = crate::an_edge_in_its_displays_units::a_window_in_the_edges_units(
                server.where_a_window_is_drawn(surface),
                scale,
            );
            let edge = crate::window_edge::edge_of(
                window,
                crate::window_edge_who_draws::who_draws_a_frame(),
                server.is_this_windows_edge_revealed(surface),
                // The same output this picture is laid out against, in the
                // units the layout uses. A narrow window's edge is clamped
                // inside it, so painting and the clamp read one number.
                crate::window_edge::the_usable_output(
                    smithay::utils::Size::from((output.0, output.1)),
                    scale,
                ),
            );
            let title = server.the_name_of(surface);
            // **Where the pointer and the keyboard are, not a default.** This
            // passed `Pointing::default()` until 2026-10-12, so no control on
            // any window on a real machine ever showed a highlight or a focus
            // ring: both were built, tested, and fed only by the example.
            let keyboard = server.where_the_keyboard_is_on_this_windows_edge(surface);
            let pointing = server.what_this_edge_is_pointing_at_on(
                &edge,
                keyboard.what_the_painter_shows(),
                scale,
            );
            // **And this window's menu, where it has one open.** Laid out from
            // the same edge the picture is drawn from, so what a person sees
            // and what `a_press_in_the_menu` hit-tests are one layout.
            let menu = keyboard.the_menu().and_then(|in_the_menu| {
                crate::window_menu::the_menu_of(
                    &edge,
                    crate::window_menu::WhatTheWindowAllows::everything(),
                    server.surfaces.has_window_mode(surface),
                    crate::window_menu::THE_TITLE_BLOCK_IS_TALL,
                    crate::window_edge::the_usable_output(
                        smithay::utils::Size::from((output.0, output.1)),
                        scale,
                    ),
                )
                .map(|menu| (menu, in_the_menu.on()))
            });
            // **Which window it is, for how its edge looks**: the one the
            // person's typing goes to wears the focused look (`557:456`), and
            // every window is framed by alo's line and shadow, drawn round its
            // own rectangle and never over it.
            let artwork = std::sync::Arc::clone(&server.what_each_application_draws);
            let icon = server
                .the_application_of(surface)
                .and_then(|application| artwork.of(&application));
            EdgePicture::of(
                &edge,
                pointing,
                crate::window_edge_picture::OverTheEdge {
                    menu: menu.as_ref().map(|(menu, on)| {
                        (
                            menu,
                            crate::window_menu_paint::PointingAtTheMenu {
                                hovered: None,
                                focused: *on,
                            },
                        )
                    }),
                    // **A reason to show the whole title**, where there is one.
                    // The title sits in the drag region, so a pointer on the
                    // title is one the edge classifies as `Drag`; the keyboard's
                    // road is being on the edge at all, which is the nearest
                    // thing to focusing the title until a tab stop exists for it.
                    showing: the_reason_to_show_the_whole_title(server, &edge, keyboard),
                },
                crate::window_edge_picture::TheWindow {
                    title: title.its_own_words(),
                    look: crate::window_edge_look::WhichLook::of(
                        server.is_the_window_typing_goes_to(surface),
                    ),
                    icon,
                    body: Some(window),
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

/// Why the whole title would be shown for this window, if it was cut.
///
/// [`None`] where there is no reason, which is most windows most of the time.
/// `crate::window_edge_tooltip::tooltip_of` answers `None` again for a title
/// that fits, so this says *a person is looking at this title* and nothing
/// about whether there is more of it to see.
fn the_reason_to_show_the_whole_title(
    server: &crate::Server,
    edge: &crate::window_edge::WindowEdge,
    keyboard: crate::window_edge_keys::TheKeyboardOnTheEdge,
) -> Option<crate::window_edge_tooltip::Because> {
    // **The pointer first**, because it is the specific one: a person pointing
    // at a title is asking about that title, where the keyboard being
    // somewhere on the edge is weaker evidence.
    if let Some(at) = server.where_the_pointer_is_in_logical_pixels()
        && matches!(
            crate::window_edge_pointing::what_is_under(edge, at),
            Some(crate::window_edge_pointing::UnderThePointer::Drag)
        )
    {
        return Some(crate::window_edge_tooltip::Because::APointerIsOnIt);
    }
    match keyboard {
        crate::window_edge_keys::TheKeyboardOnTheEdge::Nowhere => None,
        crate::window_edge_keys::TheKeyboardOnTheEdge::OnAControl(_)
        | crate::window_edge_keys::TheKeyboardOnTheEdge::InTheMenu(_) => {
            Some(crate::window_edge_tooltip::Because::TheKeyboardIsOnIt)
        }
    }
}
