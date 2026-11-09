//! Whether a window's controls are shown, and what a pointer does to that.
//!
//! `docs/design/the-window-controls.md`, §4: the title and the application's
//! name are always on the band; **the three controls appear as the pointer
//! enters anywhere in the band** — on an inactive window too, without
//! activating or raising it — and stay while the pointer is in the band or on a
//! control, while the keyboard is on them, and while the window panel is open.
//! *Always show window controls* keeps them shown.
//!
//! # Nothing new is decided here
//!
//! [`alo_dock::revealing::Revealing`] already holds those conditions, one for
//! one, and carries **no timer** as a property of its type: there is no
//! instant, duration or wait anywhere in it, so *a person must never have to
//! move quickly to reach a control* cannot be broken by this file. The owner's
//! ruling of 2026-11-09 — *no wait, continuous path* — is that property.
//!
//! **The fade is not a wait.** `crate::the_controls_fade` draws the change over
//! about 120 ms in and 160 ms out, beginning the moment the answer here
//! changes; a press reaches a control exactly while this answers *shown*.
//!
//! # The whole band reveals, and nothing on it can refuse to
//!
//! A pointer anywhere in the band is [`ThePointer::AtTheEdge`], the answer that
//! reveals. Until 2026-11-09 a pointer on the drawn strip or on a control was
//! *on the surface* — which only holds an edge already revealed — and that was
//! safe only because a concealed edge drew no strip. **The band is now drawn at
//! rest**, carrying the title, so the same reading would have left every band
//! unable to reveal at all: a pointer arriving on the title would be *on the
//! surface* of something nothing had revealed.

use std::time::Instant;

use alo_dock::revealing::{Revealing, ThePointer};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point, Rectangle};

use crate::the_controls_fade::TheFade;
use crate::window_edge::WindowEdge;

/// What this pointer is to one window band's reveal machine: in the band, or
/// elsewhere.
pub(crate) fn what_the_pointer_is(edge: &WindowEdge, at: Point<i32, Logical>) -> ThePointer {
    if holds(edge.region, at) {
        ThePointer::AtTheEdge
    } else {
        ThePointer::Elsewhere
    }
}

/// Whether a rectangle contains a point, half-open, with no area containing
/// nothing — so two bands that share an edge do not both contain the points
/// along it.
fn holds(rectangle: Rectangle<i32, Logical>, at: Point<i32, Logical>) -> bool {
    if rectangle.size.w <= 0 || rectangle.size.h <= 0 {
        return false;
    }
    let right = rectangle.loc.x.saturating_add(rectangle.size.w);
    let bottom = rectangle.loc.y.saturating_add(rectangle.size.h);
    at.x >= rectangle.loc.x && at.x < right && at.y >= rectangle.loc.y && at.y < bottom
}

impl crate::Server {
    /// **The output a narrow window's band is clamped inside**: the same
    /// extent the draw uses, so the band a pointer is classified against is
    /// the band that was painted. No extent yet means nothing has drawn, and
    /// the band is left unclamped until the first frame.
    pub(crate) fn the_room_an_edge_is_clamped_in(&self) -> smithay::utils::Size<i32, Logical> {
        self.surfaces.popups.output_size.map_or_else(
            || smithay::utils::Size::from((i32::MAX, i32::MAX)),
            |pixels| {
                crate::window_edge::the_usable_output(
                    pixels,
                    self.the_scale_an_edge_is_measured_at(),
                )
            },
        )
    }

    /// Read the pointer once and tell every mapped window's band about it.
    ///
    /// **Called once a frame, by the draw**, so the band is laid out from the
    /// reveal state where the layout happens, and one answer holds per frame.
    ///
    /// | condition | fed from |
    /// |---|---|
    /// | the pointer | this file, once a frame |
    /// | keyboard focus | `TheKeyboardOnTheEdge::whether_the_keyboard_holds_it` |
    /// | the window panel | `TheKeyboardOnTheEdge::whether_a_menu_holds_it` |
    ///
    /// **A drag holds nothing**: the band is in the window's place while it is
    /// dragged, so the pointer is in it the whole way.
    pub fn the_edges_were_told_where_the_pointer_is(&mut self) {
        let now = Instant::now();
        let mapped: Vec<WlSurface> = self.mapped_surfaces().cloned().collect();
        // **Pruned against what is mapped**, so a window that closed takes its
        // state with it: a `WlSurface` is a protocol object at an id the server
        // reuses.
        self.edges_revealed
            .retain(|(surface, _)| mapped.contains(surface));
        self.controls_fading
            .retain(|(surface, _)| mapped.contains(surface));
        self.only_the_edges_still_open(&mapped);

        // **No pointer is not the origin**: a seat that has never had a
        // pointing device would otherwise reveal whichever band contains
        // `(0, 0)`. The keyboard and the panel still count.
        let at = self.where_the_pointer_is_in_logical_pixels();
        let decorations = crate::window_edge_who_draws::who_draws_a_frame();
        let room = self.the_room_an_edge_is_clamped_in();
        let scale = self.the_scale_an_edge_is_measured_at();
        let at = at.map(|at| crate::an_edge_in_its_displays_units::a_point_in_the_edges_units(at, scale));
        let less_motion = self.motion_is_reduced();

        for surface in mapped {
            let was = self
                .edges_revealed
                .iter()
                .find(|(it, _)| *it == surface)
                .map_or_else(Revealing::covered, |(_, machine)| *machine);
            let edge = crate::window_edge::edge_of(
                crate::an_edge_in_its_displays_units::a_window_in_the_edges_units(
                    self.where_a_window_is_drawn(&surface),
                    scale,
                ),
                decorations,
                was.is_revealed(),
                room,
            );
            let keyboard = self.where_the_keyboard_is_on_this_windows_edge(&surface);
            let pointer = at.map_or(ThePointer::Elsewhere, |at| what_the_pointer_is(&edge, at));
            let revealed = was
                .the_pointer_is(pointer)
                .the_keyboard(keyboard.whether_the_keyboard_holds_it())
                .a_menu(keyboard.whether_a_menu_holds_it());
            match self
                .edges_revealed
                .iter_mut()
                .find(|(it, _)| *it == surface)
            {
                Some((_, machine)) => *machine = revealed,
                None => self.edges_revealed.push((surface.clone(), revealed)),
            }
            let shown = revealed.is_revealed() || self.window_controls.are_always_shown();
            match self
                .controls_fading
                .iter_mut()
                .find(|(it, _)| *it == surface)
            {
                Some((_, fade)) => *fade = fade.heading(shown, now, less_motion),
                // **A window seen for the first time starts where it is**, so a
                // window opening under the pointer does not fade in from
                // nothing it ever showed.
                None => self
                    .controls_fading
                    .push((surface, TheFade::settled(shown, now))),
            }
        }
    }

    /// Whether this window's controls are shown — which is also whether they
    /// can be pressed.
    ///
    /// **`false` for a window nothing has told about a pointer yet**, unless
    /// the person keeps the controls always shown.
    #[must_use]
    pub fn is_this_windows_edge_revealed(&self, surface: &WlSurface) -> bool {
        self.window_controls.are_always_shown()
            || self
                .edges_revealed
                .iter()
                .find(|(it, _)| it == surface)
                .is_some_and(|(_, machine)| machine.is_revealed())
    }

    /// How shown this window's controls are drawn at `now`, in thousandths:
    /// the fade's answer, or all the way where the controls are always shown.
    #[must_use]
    pub(crate) fn how_shown_its_controls_are(&self, surface: &WlSurface, now: Instant) -> u16 {
        if self.window_controls.are_always_shown() {
            return crate::the_controls_fade::ALL_THE_WAY;
        }
        let less_motion = self.motion_is_reduced();
        self.controls_fading
            .iter()
            .find(|(it, _)| it == surface)
            .map_or_else(
                || {
                    if self.is_this_windows_edge_revealed(surface) {
                        crate::the_controls_fade::ALL_THE_WAY
                    } else {
                        0
                    }
                },
                |(_, fade)| fade.how_shown(now, less_motion),
            )
    }
}

#[cfg(test)]
#[path = "the_window_edge_reveals_tests.rs"]
mod tests;
