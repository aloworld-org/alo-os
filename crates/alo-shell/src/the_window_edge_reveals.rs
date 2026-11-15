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

use alo_dock::revealing::{AMenu, Revealing, TheKeyboard, ThePointer};
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
        let room = self.the_room_an_edge_is_clamped_in();
        let scale = self.the_scale_an_edge_is_measured_at();
        let at = at
            .map(|at| crate::an_edge_in_its_displays_units::a_point_in_the_edges_units(at, scale));
        let less_motion = self.motion_is_reduced();
        // **A full-screen window's band is brought out or put away first**, so
        // the reveal below asks about the band where it now hangs.
        self.the_full_screen_bands_were_told(&mapped, at, scale);

        for surface in mapped {
            let was = self
                .edges_revealed
                .iter()
                .find(|(it, _)| *it == surface)
                .map_or_else(Revealing::covered, |(_, machine)| *machine);
            let edge = crate::window_edge::edge_of(
                self.the_window_a_band_hangs_from(&surface, scale),
                self.who_draws_the_header_of(&surface),
                was.is_revealed(),
                room,
            );
            let keyboard = self.where_the_keyboard_is_on_this_windows_edge(&surface);
            let pointer = at.map_or(ThePointer::Elsewhere, |at| what_the_pointer_is(&edge, at));
            let fade = self
                .controls_fading
                .iter()
                .find(|(it, _)| *it == surface)
                .map(|(_, fade)| *fade);
            let (revealed, fade) = a_band_is_told(
                (was, fade),
                (
                    pointer,
                    keyboard.whether_the_keyboard_holds_it(),
                    keyboard.whether_a_menu_holds_it(),
                ),
                self.window_controls.are_always_shown(),
                (now, less_motion),
            );
            match self
                .edges_revealed
                .iter_mut()
                .find(|(it, _)| *it == surface)
            {
                Some((_, machine)) => *machine = revealed,
                None => self.edges_revealed.push((surface.clone(), revealed)),
            }
            match self
                .controls_fading
                .iter_mut()
                .find(|(it, _)| *it == surface)
            {
                Some((_, it)) => *it = fade,
                None => self.controls_fading.push((surface, fade)),
            }
        }
        // **And the bands over alo's own windows, from the same pointer and by
        // the same rule** (`crate::the_bands_over_alos_windows`), so a band
        // alo draws for itself cannot reveal differently from one it draws for
        // an application. Until 2026-11-14 each of those was laid out revealed
        // and drawn shown, whatever the pointer did.
        self.alos_bands_were_told(at, (room, scale), (now, less_motion));
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
        how_shown(
            self.window_controls.are_always_shown(),
            self.is_this_windows_edge_revealed(surface),
            self.controls_fading
                .iter()
                .find(|(it, _)| it == surface)
                .map(|(_, fade)| fade),
            (now, self.motion_is_reduced()),
        )
    }
}

/// **One band told where the pointer, the keyboard and the panel are**: its
/// reveal advanced, and its fade turned toward what that answer shows.
///
/// The one rule every band follows — an application's, a task window's and
/// each of alo's own windows' — so no band can be drawn shown at rest by a
/// reveal of its own. `fade` is [`None`] on a band seen for the first time,
/// which **starts where it is**, so a window opening under the pointer does
/// not fade in from nothing it ever showed. `always` is the person's *Always
/// show window controls*.
pub(crate) fn a_band_is_told(
    (was, fade): (Revealing, Option<TheFade>),
    (pointer, keyboard, menu): (ThePointer, TheKeyboard, AMenu),
    always: bool,
    (now, less_motion): (Instant, bool),
) -> (Revealing, TheFade) {
    let revealed = was
        .the_pointer_is(pointer)
        .the_keyboard(keyboard)
        .a_menu(menu);
    let shown = revealed.is_revealed() || always;
    let fade = fade.map_or_else(
        || TheFade::settled(shown, now),
        |fade| fade.heading(shown, now, less_motion),
    );
    (revealed, fade)
}

/// How shown one band's controls are drawn at `now`, in thousandths: all the
/// way where the person keeps them always shown, the fade's answer where the
/// band has one, and otherwise whether it is revealed.
pub(crate) fn how_shown(
    always: bool,
    revealed: bool,
    fade: Option<&TheFade>,
    (now, less_motion): (Instant, bool),
) -> u16 {
    if always {
        return crate::the_controls_fade::ALL_THE_WAY;
    }
    fade.map_or_else(
        || {
            if revealed {
                crate::the_controls_fade::ALL_THE_WAY
            } else {
                0
            }
        },
        |fade| fade.how_shown(now, less_motion),
    )
}

#[cfg(test)]
#[path = "the_window_edge_reveals_tests.rs"]
mod tests;
