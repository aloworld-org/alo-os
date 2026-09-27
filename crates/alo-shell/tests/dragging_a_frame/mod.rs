//! **Dragging a frame** — the canvas plan's task 3, first clause: *dragging at
//! three zoom levels moves the frame by the pointer's own distance in plane
//! units.*
//!
//! The plan states it as a number: *a drag of 100 screen pixels at half zoom
//! moves it 200 plane units.* That is what each case here asserts, at 40 per
//! cent, life size and 250 per cent, and at two pan offsets.
//!
//! # The arithmetic arrived with task 2 and was not tested by it
//!
//! `Surfaces::window_press` records a press in plane units and `pointer_motion`
//! hands the drag the same converted point, so `origin + (pointer - press)` is
//! already entirely in plane units. Nothing checked it above life size. A
//! correct-by-construction claim that no test holds is a claim that survives
//! exactly until somebody moves the conversion.
//!
//! # What this does not hold, and why it is the rest of task 3
//!
//! *A press inside the content never moves it.* Today a client's
//! `xdg_toplevel.move` is honoured from a press anywhere in its window, which is
//! the drag these cases use — so that clause is **false right now**, and it
//! cannot be tested by removing the only drag there is: a frame nothing can move
//! passes *a press inside the content never moves it* perfectly.
//!
//! ADR 0065 says where the handle belongs instead — *handles, not chrome*: **the
//! name is the handle, and so is the frame's edge. Inside the frame every click
//! belongs to the application, so nothing is moved by accident.** So the second
//! clause needs the name and the edge to be grabbable before it can be held, and
//! it is named in the plan under task 3 rather than ticked here.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use super::support::{Application, Fixture, THREE_ZOOMS, TWO_PANS, looking, motion, on_the_screen};
use smithay::{
    backend::input::ButtonState::{Pressed, Released},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
};

/// A real display with a pointer on its seat.
fn fixture() -> Fixture {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    f
}

/// Complete the real XDG handshake and attach the 16x16 buffer.
fn mapped(f: &Fixture) -> Application {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    app
}

/// Where this frame's own origin sits on the plane.
fn origin(f: &Fixture, root: &WlSurface) -> (f64, f64) {
    let root = root.clone();
    f.backend(move |_| {
        let point = alo_shell::window_buffer_origin(&root);
        (point.x, point.y)
    })
}

/// **A drag moves a frame by the pointer's own distance in plane units.**
///
/// The pointer is moved by the same hundred-and-fifty screen pixels in every
/// case, and the frame is required to move by that distance *divided by the
/// zoom*: 375 plane units at 40 per cent, 150 at life size, 60 at 250 per cent.
/// A drag that moved a frame by the screen's distance would pass at life size
/// and be wrong by more than a factor of two either side of it.
#[test]
fn a_drag_moves_a_frame_by_the_pointers_distance_in_plane_units() {
    let f = fixture();
    let mut app = mapped(&f);
    let root = f.root();
    let placed = (200, 100);
    let middle = (8.0, 8.0);
    // A drag in screen pixels, both axes, neither a multiple of the other.
    let drag = (150.0, -90.0);

    for at in TWO_PANS {
        for zoom in THREE_ZOOMS {
            {
                let root = root.clone();
                assert!(f.backend(move |s| s.place_window(&root, placed)).is_ok());
            }
            assert_eq!(
                origin(&f, &root),
                (f64::from(placed.0), f64::from(placed.1)),
                "the frame was not put back before this case"
            );
            looking(&f, at, zoom);

            // Press the middle of the frame, which is where it looks, and ask for
            // the move with that press's own serial — the drag any application
            // starts today.
            let from = on_the_screen(middle, placed, at, zoom);
            motion(&f, from);
            assert_eq!(
                f.backend(|s| s.pointer_button(0x110, Pressed, 2)).ok(),
                Some(true),
                "at {at:?} and {zoom} thousandths the frame was not pressed at all"
            );
            app.sync();
            let serial = app.events.pointer.button_serial;
            let seat = app
                .events
                .keyboard
                .seat
                .clone()
                .expect("a seat this client was told about");
            app.toplevel._move(&seat, serial);
            app.sync();

            motion(&f, (from.0 + drag.0, from.1 + drag.1));

            let scale = f64::from(zoom) / 1000.0;
            let expected = (
                f64::from(placed.0) + drag.0 / scale,
                f64::from(placed.1) + drag.1 / scale,
            );
            let moved = origin(&f, &root);
            assert!(
                (moved.0 - expected.0).abs() <= 1.0 && (moved.1 - expected.1).abs() <= 1.0,
                "at {at:?} and {zoom} thousandths a drag of {drag:?} screen pixels \
                 left the frame at {moved:?} and it belongs at {expected:?}"
            );

            // End the drag, so the next case starts with no gesture held. The
            // final release is consumed by the drag rather than delivered.
            assert_eq!(
                f.backend(|s| s.pointer_button(0x110, Released, 3)).ok(),
                Some(false)
            );
            app.sync();
        }
    }
}
