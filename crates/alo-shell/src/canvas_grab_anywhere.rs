//! **Super and drag takes hold of the canvas from anywhere**, a window included.
//!
//! The owner's decision of 2026-11-13. Every other pointer road to a pan —
//! the wheel, two fingers, Shift and the wheel — starts over **empty canvas**,
//! and with several windows covering the screen there is no empty canvas to
//! start on. Asked *"many canvas apps also let you grab from anywhere, even over
//! a window, by holding a key such as Super while dragging. Would you like
//! it?"*, the owner answered **"yes add it please"**.
//!
//! # What it does, which is all of what a person can see of it
//!
//! - While `⊞` is held, a left press **anywhere on the canvas** takes hold of
//!   the whole canvas: over an application, over its band, over its edge, over
//!   empty canvas — the same gesture everywhere, so it never changes meaning
//!   with what happens to be under the pointer.
//! - Moving pans every window together, **one for one in screen pixels**, which
//!   is `alo_canvas::Camera::panned_by`'s contract and the same arithmetic as
//!   `crate::canvas_space_drag`: what is under the hand moves by exactly as much
//!   as the hand did, at every zoom.
//! - Letting go of the button lets go of the canvas, and so does letting go of
//!   `⊞` mid-drag. Nothing carries on afterwards — no glide, no inertia — so a
//!   person who asked for less motion has nothing to switch off.
//! - **The window under the pointer is told nothing**: not the press, not the
//!   movement, not the release. It is never clicked by accident, and the
//!   keyboard stays where it was.
//! - **Fixed surfaces are not canvas.** The Dock, what the Dock opened, the
//!   Bar's matches, the put-aside panel and the egress indicator lie on top of
//!   the canvas and do not move with it, so a press on one of them is that
//!   surface's as it always was. Sheets never reach here at all: the one on the
//!   desktop takes the whole seat before a press is routed
//!   (`crate::a_key_reaches_the_open_with_sheet`).
//!
//! # Why the view the press landed on, and not the one the pointer is on now
//!
//! There is one camera per display (`crate::canvas_camera`). A grab names the
//! display it started on and pans that one to the end, so a drag that wanders
//! onto the next monitor keeps moving the view the person took hold of rather
//! than jumping to another one half-way through the stroke.
//!
//! # Why it is not in Settings → Keyboard and shortcuts
//!
//! That page lists `alo_shortcuts::Action`s, each with a chord a person can
//! change, and a chord is a set of modifiers **and a key**
//! (`alo_shortcuts::Chord`). A held modifier and a pointer drag is not a key,
//! so it cannot be one of those rows without changing what the settings file
//! holds. Space and drag and the arrows are not listed there for the same
//! reason (`crate::canvas_space_drag`'s header). The gesture is recorded in
//! `docs/design/the-shortcuts-and-the-edges.md` beside them.
//!
//! # `⊞` alone is still free to open the Bar
//!
//! The design gives a lone `⊞` to the alo Bar, and nothing builds that yet.
//! When it is built, a `⊞` that took hold of the canvas was not pressed alone,
//! and must not open the Bar on its release — the same rule every desktop keeps
//! for a Super that was part of a chord.

use smithay::{
    backend::input::ButtonState,
    utils::{Logical, Physical, Point, Rectangle},
};

/// `BTN_LEFT`: the button a person takes hold of things with.
const THE_LEFT_BUTTON: u32 = 0x110;

/// The canvas, while a person holds it with `⊞` and a press.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TheCanvasHeld {
    /// The button that took hold. Its release lets go.
    button: u32,
    /// The display whose view is moving, named when the press landed.
    display: String,
    /// Movement not yet worth a whole pixel, kept for `crate::canvas_pan`'s
    /// reason: a slow hand arrives in fractions and must not be dropped.
    unspent: (f64, f64),
}

impl crate::Server {
    /// Whether `⊞` is held right now, as XKB sees it.
    ///
    /// Read from the seat's modifier state rather than kept beside it, the
    /// discipline `Server::ctrl_is_held` keeps for the wheel: one answer to *is
    /// it down*. XKB advances with nothing focused too (`Server::keyboard_key`),
    /// which is the state a person is in over empty canvas.
    #[must_use]
    pub(crate) fn super_is_held(&self) -> bool {
        self.surfaces
            .keyboard
            .as_ref()
            .is_some_and(|keyboard| keyboard.handle.modifier_state().logo)
    }

    /// Whether a person is holding the canvas right now.
    #[must_use]
    pub fn the_canvas_is_held(&self) -> bool {
        self.surfaces
            .pointer
            .as_ref()
            .is_some_and(|pointer| pointer.canvas_held.is_some())
    }

    /// Whether a fixed surface — something drawn over the canvas that does not
    /// move with it — is at this point on the screen.
    pub(crate) fn a_fixed_surface_is_at(&self, at: Point<i32, Physical>) -> bool {
        // **The bands the draw recorded for this point's display**: the Dock,
        // the put-aside panel's column and the egress indicator. Not the top
        // controls' room: it is reserved and nothing is painted in it yet, so a
        // person pressing there is pressing on the window they can see.
        let recorded = self
            .the_controls_of_the_display_for(Some(Rectangle::new(at, (1, 1).into())))
            .and_then(crate::canvas_fixed_controls::FixedControls::labelled)
            .is_some_and(|drawn| {
                [
                    drawn.dock_band,
                    Some(drawn.panel_reserved),
                    drawn.what_is_leaving,
                ]
                .into_iter()
                .flatten()
                .any(|band| band.contains(at))
            });
        // **And what the Dock knows it drew**, on every display, because the
        // pointer's position is global: its band, whatever it opened, and the
        // Bar's matches risen from its field.
        let on_the_dock = self
            .dock_as_drawn
            .values()
            .any(|dock| dock.how_far_along(at).is_some());
        let on_what_the_dock_opened = self.the_docks_surface_as_drawn.values().any(|drawn| {
            crate::where_the_docks_surface_is::what_a_press_on_the_surface_does(drawn, at)
                != crate::where_the_docks_surface_is::WhatAPressOnTheSurfaceDoes::Dismiss
        });
        let on_the_bars_matches = self
            .the_bars_matches_places
            .as_ref()
            .is_some_and(|places| places.panel.contains(at));
        recorded || on_the_dock || on_what_the_dock_opened || on_the_bars_matches
    }

    /// Whether this button belongs to a held canvas, taking hold where it may.
    ///
    /// `true` means the event goes no further: not to the window under the
    /// pointer, not to its band or edge, not to the keyboard's focus. While the
    /// canvas is held **every** button is its own until the one that took hold
    /// is let go, so a second finger on the trackpad cannot click an
    /// application in the middle of a pan.
    ///
    /// A press takes hold only with `⊞` held, only with the left button, only
    /// when no other button is already down on a window and no frame is being
    /// moved or resized, and only off the fixed surfaces.
    pub(crate) fn a_button_holds_the_canvas(
        &mut self,
        button: u32,
        state: ButtonState,
        time: u32,
    ) -> bool {
        if let Some(held) = self
            .surfaces
            .pointer
            .as_ref()
            .and_then(|pointer| pointer.canvas_held.as_ref())
        {
            if held.button == button && state == ButtonState::Released {
                self.let_go_of_the_canvas(time);
            }
            return true;
        }
        if state != ButtonState::Pressed || button != THE_LEFT_BUTTON || !self.super_is_held() {
            return false;
        }
        let Some(pointer) = self.surfaces.pointer.as_ref() else {
            return false;
        };
        if !pointer.buttons.is_empty()
            || self.surfaces.window_move.is_some()
            || self.surfaces.resizing_pointer()
        {
            return false;
        }
        let at = (pointer.location.x as i32, pointer.location.y as i32).into();
        if self.a_fixed_surface_is_at(at) {
            return false;
        }
        let display = self.the_display_being_worked_on();
        if let Some(pointer) = self.surfaces.pointer.as_mut() {
            pointer.canvas_held = Some(TheCanvasHeld {
                button,
                display,
                unspent: (0.0, 0.0),
            });
        }
        true
    }

    /// Pan by this pointer movement, if the canvas is held.
    ///
    /// `true` means the movement was the canvas's and reaches no window —
    /// including a movement too small to spend yet, and one the plane's edge
    /// refused. `false` when nothing is held, and when `⊞` turned out to be up,
    /// which lets go first so the movement goes where it would have.
    pub(crate) fn pan_the_plane_by_a_held_canvas(
        &mut self,
        to: Point<f64, Logical>,
        time: u32,
    ) -> bool {
        if !self.the_canvas_is_held() {
            return false;
        }
        if !self.super_is_held() {
            self.let_go_of_the_canvas(time);
            return false;
        }
        let Some(pointer) = self.surfaces.pointer.as_mut() else {
            return false;
        };
        // The hand moves one way and the plane the other, as a sheet of paper
        // does: dragging right brings what is on the left into view.
        let moved = (pointer.location.x - to.x, pointer.location.y - to.y);
        let Some(held) = pointer.canvas_held.as_mut() else {
            return false;
        };
        let wanted = (held.unspent.0 + moved.0, held.unspent.1 + moved.1);
        let (across, down) = (wanted.0.trunc(), wanted.1.trunc());
        held.unspent = (wanted.0 - across, wanted.1 - down);
        let display = held.display.clone();
        let (Ok(across), Ok(down)) = (i32::try_from(across as i64), i32::try_from(down as i64))
        else {
            return true;
        };
        if across == 0 && down == 0 {
            return true;
        }
        match self.surfaces.camera_of(&display).panned_by(across, down) {
            Some(camera) => self.set_the_camera_of(&display, camera),
            // **Refused at the plane's edge, and nothing kept**, for
            // `crate::canvas_pan`'s reason: held movement would be spent the
            // moment the person turned round.
            None => {
                if let Some(held) = self
                    .surfaces
                    .pointer
                    .as_mut()
                    .and_then(|pointer| pointer.canvas_held.as_mut())
                {
                    held.unspent = (0.0, 0.0);
                }
            }
        }
        true
    }

    /// Let go of the canvas if `⊞` is no longer held. Asked after every key.
    pub(crate) fn the_canvas_is_let_go_unless_super_is_held(&mut self, time: u32) {
        if self.the_canvas_is_held() && !self.super_is_held() {
            self.let_go_of_the_canvas(time);
        }
    }

    /// Let go, and ask again what is under the pointer.
    ///
    /// **The asking is the part that matters.** The windows moved under a
    /// pointer that stood still on the screen, so whichever window the seat
    /// last pointed into may not be there any more — and a press before the
    /// next movement would be delivered to it.
    fn let_go_of_the_canvas(&mut self, time: u32) {
        let Some(pointer) = self.surfaces.pointer.as_mut() else {
            return;
        };
        pointer.canvas_held = None;
        let at = pointer.location;
        let _ = self.pointer_motion(at.x, at.y, time);
    }

    /// The hand the pointer shows for this gesture, or [`None`] where it shows
    /// whatever it would have.
    ///
    /// **A closed hand while the canvas is held**, and **an open one while `⊞`
    /// is held over somewhere a press would take hold** — which is how a person
    /// learns the gesture exists: the pointer changes the moment the key goes
    /// down. Over a fixed surface it does not, because a press there is that
    /// surface's.
    pub(crate) fn the_hand_for_holding_the_canvas(&self) -> Option<alo_cursors::CursorIcon> {
        let pointer = self.surfaces.pointer.as_ref()?;
        if pointer.canvas_held.is_some() {
            return Some(alo_cursors::CursorIcon::Grabbing);
        }
        let at = (pointer.location.x as i32, pointer.location.y as i32).into();
        (self.super_is_held()
            && pointer.buttons.is_empty()
            && self.surfaces.window_move.is_none()
            && !self.surfaces.resizing_pointer()
            && !self.a_fixed_surface_is_at(at))
        .then_some(alo_cursors::CursorIcon::Grab)
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "canvas_grab_anywhere_tests.rs"]
mod tests;
