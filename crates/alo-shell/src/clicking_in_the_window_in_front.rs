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
use smithay::utils::{Logical, Point, Rectangle, SERIAL_COUNTER};

use crate::Server;
use crate::the_window_in_front_alo_may_reach::NotReached;
use crate::typing_into_the_window_in_front::MayTypeInto;

/// The primary button, as Linux numbers it (`BTN_LEFT`).
const THE_PRIMARY_BUTTON: u32 = 0x110;

/// **The point on the window's surface** that `asked` names: the corner of
/// the window's geometry, then the proportion of its width and height, never
/// past its last pixel — or [`None`] for a window with no width or no height,
/// where no point is inside it.
fn the_point(window: Rectangle<f64, Logical>, asked: &ToClick) -> Option<Point<f64, Logical>> {
    if window.size.w <= 0.0 || window.size.h <= 0.0 {
        return None;
    }
    let at = |thousandths: u16, along: f64, from: f64| {
        let share = f64::from(thousandths) / f64::from(ToClick::THE_WHOLE);
        from + (share * along).min(along - 1.0).max(0.0)
    };
    Some(
        (
            at(asked.across, window.size.w, window.loc.x),
            at(asked.down, window.size.h, window.loc.y),
        )
            .into(),
    )
}

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
        let Some(here) = the_point(crate::scene::geometry(&root), asked) else {
            return Clicked::CouldNotClick;
        };
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

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
mod tests {
    use super::*;

    fn at(across: u16, down: u16) -> ToClick {
        ToClick::at(
            alo_applications::Application::identified("net.supertuxkart.SuperTuxKart")
                .expect("an identifier"),
            across,
            down,
        )
        .expect("a point")
    }

    fn a_window(x: f64, y: f64, w: f64, h: f64) -> Rectangle<f64, Logical> {
        Rectangle::new((x, y).into(), (w, h).into())
    }

    /// **From the geometry's corner, by proportion, never past the last pixel.**
    #[test]
    fn the_point_is_a_proportion_of_the_window_from_its_corner() {
        let window = a_window(10.0, 20.0, 200.0, 100.0);
        let point = the_point(window, &at(500, 250)).expect("inside");
        let near = |a: f64, b: f64| (a - b).abs() < 1e-9;
        assert!(near(point.x, 110.0) && near(point.y, 45.0), "{point:?}");
        let corner = the_point(window, &at(1000, 1000)).expect("inside");
        assert!(near(corner.x, 209.0) && near(corner.y, 119.0), "{corner:?}");
    }

    /// **A window with no width, or no height, has no point to click.**
    #[test]
    fn a_window_with_no_width_or_no_height_has_no_point() {
        for window in [
            a_window(0.0, 0.0, 0.0, 100.0),
            a_window(0.0, 0.0, 100.0, 0.0),
            a_window(0.0, 0.0, -1.0, 100.0),
        ] {
            assert!(the_point(window, &at(500, 500)).is_none(), "{window:?}");
        }
    }
}
