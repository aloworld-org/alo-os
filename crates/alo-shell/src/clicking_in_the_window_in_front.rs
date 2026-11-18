//! **One click at one point of the window in front, for alo** — the screen
//! road's click ([ADR 0147](../../../docs/decisions/0147-the-screen-road-and-who-reads-the-screen.md),
//! amended 2026-11-17: *"Allow clicking by position"*), and
//! `docs/contracts/application-door.md`, *`click-at`*.
//!
//! - **Only the window that holds the keyboard**, and only when it is the named
//!   application's and one alo may reach at the person's setting — the same
//!   rule as typing and the picture (`crate::the_window_in_front_alo_may_reach`).
//!   The keyboard is never moved to make a click possible.
//! - **At a proportion of that window**, as its picture showed it: the point is
//!   measured from the window's own geometry, the part a picture of it is
//!   drawn from, so *halfway across the picture* is halfway across the window.
//! - **Delivered to that client alone**: the primary button pressed and
//!   released once at that point of the window's surface, then the pointer
//!   given back to whatever is under the person's own pointer — so the person's
//!   pointer is where they left it.
//! - **Never into the middle of what the person is doing**: while they hold a
//!   button, nothing is clicked.

use alo_acting::{Clicked, ToClick};
use smithay::backend::input::ButtonState;
use smithay::input::pointer::{ButtonEvent, MotionEvent};
use smithay::utils::{Logical, Point, SERIAL_COUNTER};

use crate::Server;
use crate::the_window_in_front_alo_may_reach::NotReached;
use crate::typing_into_the_window_in_front::MayTypeInto;

/// The primary button, as Linux numbers it (`BTN_LEFT`).
const THE_PRIMARY_BUTTON: u32 = 0x110;

impl Server {
    /// Click once at the point `asked` names of the window in front, if it is
    /// that application's and alo may reach it.
    pub fn clicked_in_the_window_in_front(
        &mut self,
        asked: &ToClick,
        may: &mut dyn MayTypeInto,
    ) -> Clicked {
        let identifier = asked.application.identifier();
        let root = match self.the_window_in_front_alo_may_reach(identifier, may) {
            Ok((root, _)) => root,
            Err(NotReached::NotInFront) => return Clicked::NotInFront,
            Err(NotReached::NotAlosCopy) => return Clicked::NotAlosCopy,
        };
        let window = crate::scene::geometry(&root);
        if window.size.w <= 0.0 || window.size.h <= 0.0 {
            return Clicked::CouldNotClick;
        }
        let Some(pointer) = self.surfaces.pointer.as_ref() else {
            return Clicked::CouldNotClick;
        };
        if !pointer.buttons.is_empty() {
            return Clicked::ThePersonIsPressing;
        }
        let handle = pointer.handle.clone();
        let theirs = pointer.location;
        let time = pointer.time;
        let _ = self.alo_acts_in(&root, identifier);

        // The point on the window's surface: its geometry's corner, then the
        // proportion of its width and height, never past its last pixel.
        let at = |thousandths: u16, along: f64, from: f64| {
            let share = f64::from(thousandths) / f64::from(ToClick::THE_WHOLE);
            from + (share * along).min(along - 1.0).max(0.0)
        };
        let here: Point<f64, Logical> = (
            at(asked.across, window.size.w, window.loc.x),
            at(asked.down, window.size.h, window.loc.y),
        )
            .into();
        // **Surface-local on both sides**: the surface given at the origin, so
        // the client is told exactly `here`.
        handle.motion(
            &mut self.surfaces,
            Some((root, (0.0, 0.0).into())),
            &MotionEvent {
                location: here,
                serial: SERIAL_COUNTER.next_serial(),
                time,
            },
        );
        for state in [ButtonState::Pressed, ButtonState::Released] {
            handle.button(
                &mut self.surfaces,
                &ButtonEvent {
                    button: THE_PRIMARY_BUTTON,
                    state,
                    time,
                    serial: SERIAL_COUNTER.next_serial(),
                },
            );
        }
        handle.frame(&mut self.surfaces);

        // **And the pointer given back** to what is under the person's own.
        let focus = self.pointer_target(theirs);
        let on_the_plane = self.surfaces.on_the_plane(theirs);
        handle.motion(
            &mut self.surfaces,
            focus,
            &MotionEvent {
                location: on_the_plane,
                serial: SERIAL_COUNTER.next_serial(),
                time,
            },
        );
        handle.frame(&mut self.surfaces);
        Clicked::Clicked
    }
}
