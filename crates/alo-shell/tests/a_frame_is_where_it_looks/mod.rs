//! **A frame is where it looks, at any zoom** — the canvas plan's task 2, held
//! by a real client reading its own pointer events off the wire.
//!
//! The acceptance is that *a pointer press at a known screen point arrives at the
//! expected surface coordinate at three zoom levels and two pan offsets*, and the
//! constraint is that *the application is never told about the canvas: it is told
//! its size and gets its events, as it would on any compositor.*
//!
//! # Aimed at a named point of the window, not at an arithmetic result
//!
//! Every case here points at **the middle of a 16x16 window** and asserts the
//! client was told `(8, 8)`. The screen coordinate that lands there is different
//! in all six cases — it is `(8 + origin - camera) * zoom` — and the expected
//! answer never changes. That is the task's sentence turned into a test: a person
//! puts the arrow where the middle of the window *looks*, and the application
//! hears about its own middle, whatever the canvas is doing.
//!
//! An assertion written the other way round — compute the surface coordinate from
//! the same transform the compositor uses, then check it matches — would pass with
//! the transform inverted, doubled, or applied twice.
//!
//! # Why the pointer is moved away between cases
//!
//! Motion inside a surface sends `wl_pointer.motion`; only crossing into it sends
//! `enter`. Each case leaves the window first, so what is asserted is a fresh
//! `enter` with its own coordinates rather than the last event of the previous
//! case, which would still be there if a case delivered nothing at all.
//!
//! # What this does not hold, and it is not a canvas matter
//!
//! *Focus follows the frame that was pressed.* Nothing in this shell moves
//! **keyboard** focus on a press at any zoom — `Server::keyboard_focus` is an
//! explicit host API and no pointer path calls it — so that half of task 2's
//! sentence is about a shell policy this compositor has never had, not about the
//! plane's transform. Choosing click-to-focus is an ADR, not a line in a canvas
//! task. What is held here is that **the press reaches the frame that looks
//! pressed**, across two frames, at every zoom and pan — which is the part the
//! canvas can get wrong.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use super::support::{
    Application, Fixture, THREE_ZOOMS, TWO_PANS, close_enough, looking, motion, on_the_screen,
};
use smithay::backend::input::ButtonState;
use wayland_client::Proxy;

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

/// **The middle of a window is where it looks, at every zoom and pan.**
#[test]
fn a_press_where_the_middle_of_a_frame_looks_arrives_at_its_middle() {
    let f = fixture();
    let mut app = mapped(&f);
    let root = f.root();
    let origin = (200, 100);
    {
        let root = root.clone();
        assert!(f.backend(move |s| s.place_window(&root, origin)).is_ok());
    }
    // The placement is the frame's origin on the plane, which is what the camera
    // transforms. Read once and asserted, so a placement this shell declined
    // cannot make every case below a case about a window at the origin.
    let placed = {
        let root = root.clone();
        f.backend(move |_| alo_shell::window_buffer_origin(&root))
    };
    assert_eq!(
        (placed.x, placed.y),
        (f64::from(origin.0), f64::from(origin.1))
    );

    let middle = (8.0, 8.0);
    for at in TWO_PANS {
        for zoom in THREE_ZOOMS {
            looking(&f, at, zoom);
            // Off the window first: a hundred screen pixels left of its corner is
            // outside it at every zoom this tests, since the whole frame is at
            // most forty pixels across.
            let corner = on_the_screen((0.0, 0.0), origin, at, zoom);
            motion(&f, (corner.0 - 100.0, corner.1));
            app.sync();
            let before = app.events.pointer.enters.len();

            motion(&f, on_the_screen(middle, origin, at, zoom));
            app.sync();

            assert_eq!(
                app.events.pointer.enters.len(),
                before + 1,
                "at {at:?} and {zoom} thousandths the arrow was put where the middle \
                 of the window looks and the window was never entered"
            );
            let (entered, x, y) = *app
                .events
                .pointer
                .enters
                .last()
                .expect("an enter this client was just sent");
            assert_eq!(
                entered,
                app.surface.id().protocol_id(),
                "at {at:?} and {zoom} thousandths the press reached another surface"
            );
            assert!(
                close_enough((x, y), middle),
                "at {at:?} and {zoom} thousandths the middle of the window was \
                 reported at ({x}, {y}) and it is at {middle:?}"
            );
        }
    }
}

/// **A press reaches the frame that looks pressed, not the one beside it.**
///
/// Two frames far enough apart that a zoom which moved one onto the other's
/// screen position would send the press to the wrong application — which is the
/// failure a single-window test cannot see, and the one a person would describe
/// as *I clicked that window and the other one answered*.
#[test]
fn a_press_reaches_the_frame_it_looks_like_it_reached() {
    let f = fixture();
    let mut left = mapped(&f);
    let mut right = mapped(&f);
    let roots = f.backend(|s| s.mapped_surfaces().cloned().collect::<Vec<_>>());
    assert_eq!(roots.len(), 2, "two clients did not map");

    let places = [(100, 100), (500, 300)];
    for (root, place) in roots.iter().zip(places) {
        let root = root.clone();
        assert!(f.backend(move |s| s.place_window(&root, place)).is_ok());
    }

    let middle = (8.0, 8.0);
    for at in TWO_PANS {
        for zoom in THREE_ZOOMS {
            looking(&f, at, zoom);
            for (which, place) in places.iter().enumerate() {
                let corner = on_the_screen((0.0, 0.0), *place, at, zoom);
                motion(&f, (corner.0 - 100.0, corner.1));
                left.sync();
                right.sync();

                motion(&f, on_the_screen(middle, *place, at, zoom));
                assert_eq!(
                    f.backend(|s| s.pointer_button(0x110, ButtonState::Pressed, 8))
                        .ok(),
                    Some(true),
                    "at {at:?} and {zoom} thousandths a press on frame {which} was \
                     not delivered at all"
                );
                assert_eq!(
                    f.backend(|s| s.pointer_button(0x110, ButtonState::Released, 9))
                        .ok(),
                    Some(true)
                );
                left.sync();
                right.sync();

                // The frame at this place is the one that must have heard it, and
                // the other must not: `mapped_surfaces` is in the order they
                // mapped, which is the order `places` is written in.
                let (pressed, quiet) = if which == 0 {
                    (&mut left, &mut right)
                } else {
                    (&mut right, &mut left)
                };
                assert!(
                    !pressed.events.pointer.buttons.is_empty(),
                    "at {at:?} and {zoom} thousandths frame {which} was pressed \
                     where it looks and heard nothing"
                );
                assert!(
                    quiet.events.pointer.buttons.is_empty(),
                    "at {at:?} and {zoom} thousandths a press aimed at frame \
                     {which} was heard by the other frame"
                );
                let (_, x, y) = *pressed
                    .events
                    .pointer
                    .enters
                    .last()
                    .expect("the pressed frame was entered");
                assert!(
                    close_enough((x, y), middle),
                    "at {at:?} and {zoom} thousandths frame {which} was pressed in \
                     its middle and heard ({x}, {y})"
                );
                pressed.events.pointer.buttons.clear();
                quiet.events.pointer.buttons.clear();
            }
        }
    }
}
