//! **Dragging a frame** — the canvas plan's task 3, and ADR 0071's sentence
//! whole: *the edge and the corners resize; the name above the frame moves it.*
//!
//! # Why this is one test and not two
//!
//! *A press inside the content never moves it* is satisfied perfectly by a frame
//! nothing can move. So the refusal is worth nothing on its own, and it is
//! asserted here only beside a move that has just been shown to work — on the
//! same frame, at the place that move left it. The order is deliberate: the
//! content press comes second, so a frame that refuses to move cannot be mistaken
//! for a frame that was already stuck.
//!
//! The distance is the plan's own number: a drag of 150 screen pixels becomes 375
//! plane units at 40 per cent, 150 at life size and 60 at 250 per cent. That
//! arithmetic arrived with task 2 — `window_press` records a press in plane units
//! and `pointer_motion` hands the drag the same converted point — and nothing
//! checked it above life size until this file.
//!
//! # What a client can no longer do, and what it costs
//!
//! `xdg_toplevel.move` is refused outright. An application that draws its own
//! title bar therefore has a title bar that looks draggable and does nothing —
//! the visible symptom until decoration negotiation exists, which is task 3 of
//! `docs/autonomy/applications-people-already-use.md`. That is understood rather
//! than unnoticed, and it is why `crate::surfaces`' refusal carries the reason.
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

/// **The name moves a frame; a press in its content does not.**
///
/// ADR 0071's whole sentence, in one test, because neither half means anything
/// alone. *A press inside the content never moves it* is satisfied perfectly by a
/// frame nothing can move — so the refusal is only worth asserting beside a move
/// that demonstrably works, on the same frame, in the same case. That is why this
/// is one test and not two.
///
/// The order matters too: the content press is tried **after** the name has been
/// proved to move the frame, so a refusal cannot be mistaken for a frame that was
/// already stuck.
#[test]
fn the_name_moves_a_frame_and_a_press_in_its_content_does_not() {
    let f = fixture();
    let mut app = mapped(&f);
    let root = f.root();
    let placed = (200, 100);
    let drag = (150.0, -90.0);

    for at in TWO_PANS {
        for zoom in THREE_ZOOMS {
            {
                let root = root.clone();
                assert!(f.backend(move |s| s.place_window(&root, placed)).is_ok());
            }
            looking(&f, at, zoom);
            let scale = f64::from(zoom) / 1000.0;

            // --- The name moves it. ---
            //
            // A point inside the band above the frame: half its thickness up from
            // the frame's own top edge, and in from the left so a frame only a few
            // pixels wide on the glass is still aimed at inside its own width.
            let top_left = on_the_screen((0.0, 0.0), placed, at, zoom);
            let in_the_name = (top_left.0 + 2.0, top_left.1 - 16.0);
            motion(&f, in_the_name);
            assert_eq!(
                f.backend(|s| s.pointer_button(0x110, Pressed, 2)).ok(),
                Some(false),
                "at {at:?} and {zoom} thousandths a press on the name was delivered \
                 to a client, and the name is the shell's"
            );
            motion(&f, (in_the_name.0 + drag.0, in_the_name.1 + drag.1));
            let expected = (
                f64::from(placed.0) + drag.0 / scale,
                f64::from(placed.1) + drag.1 / scale,
            );
            let moved = origin(&f, &root);
            assert!(
                (moved.0 - expected.0).abs() <= 1.0 && (moved.1 - expected.1).abs() <= 1.0,
                "at {at:?} and {zoom} thousandths dragging the name moved the frame \
                 to {moved:?} and it belongs at {expected:?}"
            );
            assert_eq!(
                f.backend(|s| s.pointer_button(0x110, Released, 3)).ok(),
                Some(false)
            );

            // --- And a press in the content does not, on the frame that just
            // --- moved, from the place it moved to.
            let now = (moved.0 as i32, moved.1 as i32);
            let middle = on_the_screen((8.0, 8.0), now, at, zoom);
            motion(&f, middle);
            assert_eq!(
                f.backend(|s| s.pointer_button(0x110, Pressed, 4)).ok(),
                Some(true),
                "at {at:?} and {zoom} thousandths a press inside the frame was not \
                 delivered to the application, which is the half of this that must \
                 not break"
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
            motion(&f, (middle.0 + drag.0, middle.1 + drag.1));
            let after = origin(&f, &root);
            assert_eq!(
                (after.0 as i32, after.1 as i32),
                now,
                "at {at:?} and {zoom} thousandths an application asked to be moved \
                 from a press inside its own content and the frame moved"
            );
            assert_eq!(
                f.backend(|s| s.pointer_button(0x110, Released, 5)).ok(),
                Some(true)
            );
            app.sync();
        }
    }
}
