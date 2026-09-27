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

use super::{Application, Fixture};
use alo_canvas::{At, Zoom};
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

/// Look at the plane from exactly here, at exactly this zoom.
///
/// The zoom is set while the camera is at the origin and held at the viewport's
/// own corner, so the pointer-centred arithmetic leaves the camera where it is;
/// the pan is then set outright. Both are read back, because a camera that refused
/// one of the two would otherwise make the case below a case about life size.
fn looking(f: &Fixture, at: (i32, i32), zoom: u32) {
    f.backend(move |s| {
        assert!(s.look_at_the_canvas(At::origin()).is_some());
        let zoom = Zoom::of(zoom).expect("a zoom inside the canvas's own bounds");
        assert!(s.zoom_the_canvas(zoom, (0, 0)).is_some());
        let at = At::checked(at.0, at.1).expect("a place on the plane");
        assert!(s.look_at_the_canvas(at).is_some());
        assert_eq!(s.the_camera().zoom(), zoom, "the camera refused the zoom");
        assert_eq!(s.the_camera().at(), at, "the camera refused the pan");
    });
}

/// Where a point of a frame at `origin` appears on the screen, under this camera.
///
/// Written out here rather than asked of the compositor: this is the sentence the
/// person experiences — *the middle of that window is there on the glass* — and
/// the test is worth nothing if it borrows the answer from the code it checks.
fn on_the_screen(point: (f64, f64), origin: (i32, i32), at: (i32, i32), zoom: u32) -> (f64, f64) {
    let scale = f64::from(zoom) / 1000.0;
    (
        (point.0 + f64::from(origin.0) - f64::from(at.0)) * scale,
        (point.1 + f64::from(origin.1) - f64::from(at.1)) * scale,
    )
}

/// Move the pointer there, in the screen's own pixels.
fn motion(f: &Fixture, (x, y): (f64, f64)) {
    assert!(f.backend(move |s| s.pointer_motion(x, y, 7)).is_ok());
}

/// A pointer coordinate a client was told, to within Wayland's own fixed point.
///
/// `wl_fixed` is 1/256 of a unit, so an exact comparison would be a test of the
/// order this crate's divisions happen in rather than of where the arrow was.
fn close_enough(got: (f64, f64), want: (f64, f64)) -> bool {
    const A_FRACTION_OF_A_UNIT: f64 = 2.0 / 256.0;
    (got.0 - want.0).abs() <= A_FRACTION_OF_A_UNIT && (got.1 - want.1).abs() <= A_FRACTION_OF_A_UNIT
}

/// The zooms the plan names, and life size between them.
const THREE_ZOOMS: [u32; 3] = [400, 1000, 2500];

/// Two pans, one of them off the plane's origin in both directions.
const TWO_PANS: [(i32, i32); 2] = [(0, 0), (-37, 64)];

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
