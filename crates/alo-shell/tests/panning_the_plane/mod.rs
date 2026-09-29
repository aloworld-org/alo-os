//! **Pan** — the canvas plan's task 5, and the rule it states outright: *scroll
//! over a frame scrolls that frame's content; scroll over empty canvas pans.*
//!
//! The acceptance asks for a test that **scrolls over both and asserts which
//! moved**, which is stronger than it first reads: each case has to check the
//! thing that moved *and* the thing that did not. A scroll that panned the canvas
//! and also reached the application would pass a test that only looked at the
//! camera, and a person reading a long document would watch the whole canvas
//! slide away underneath it.
//!
//! # What is not here, and it is somebody else's crate
//!
//! **The keyboard route for panning.** A chord reaches a native operation through
//! `alo_shortcuts::Action`, which had no canvas action in it at all when this was
//! written — a finding recorded under task 5. It now has three, and they are
//! zoom's: `ZoomTheCanvasIn`, `ZoomTheCanvasOut` and `ShowAllOnTheCanvas`, held in
//! `crates/alo-shell/tests/zoom_and_show_all/mod.rs`. **Panning still has no
//! chord**, because the design file gives it the arrow keys rather than a chord —
//! `alo_shortcuts::Chord` requires Super, Ctrl or Alt and a bare arrow is not one,
//! so a pan shortcut is not a thing that enum can hold. That is task 5's remaining
//! gap and it is a question about `alo-shortcuts`, not about this file.
//!
//! **Space-and-drag**, which ADR 0065 names beside the wheel, and the trackpad's
//! two-finger form. Both are gestures rather than arithmetic and neither is held
//! here; the plan says so.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use super::support::{Application, Fixture, THREE_ZOOMS, looking, motion, on_the_screen};
use alo_canvas::At;
/// `KEY_SPACE`, as a real keyboard sends it.
const SPACE: u32 = 57;

use smithay::{
    backend::input::{Axis, AxisSource, KeyState},
    input::pointer::AxisFrame,
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

/// One wheel event, both axes, as a real device sends it.
fn scroll(f: &Fixture, (across, down): (f64, f64)) -> bool {
    f.backend(move |s| {
        s.pointer_axis(
            AxisFrame::new(11)
                .source(AxisSource::Wheel)
                .value(Axis::Horizontal, across)
                .value(Axis::Vertical, down),
        )
    })
    .expect("a scroll with real values")
}

/// Where this session is looking, as a pair.
fn looking_at(f: &Fixture) -> (i32, i32) {
    f.backend(|s| {
        let at = s.the_camera().at();
        (at.x, at.y)
    })
}

/// **A scroll over empty canvas pans the plane and reaches no application.**
#[test]
fn a_scroll_over_the_plane_pans_it_and_the_application_hears_nothing() {
    let f = fixture();
    let mut app = mapped(&f);
    let placed = (200, 100);
    {
        let root = f.root();
        assert!(f.backend(move |s| s.place_window(&root, placed)).is_ok());
    }

    for zoom in THREE_ZOOMS {
        looking(&f, (0, 0), zoom);
        // Well clear of the frame, whatever the zoom puts it at.
        let corner = on_the_screen((0.0, 0.0), placed, (0, 0), zoom);
        motion(&f, (corner.0 - 200.0, corner.1 - 200.0));
        app.sync();
        let axes = app.events.pointer.axes.len();
        let before = looking_at(&f);

        assert!(
            scroll(&f, (30.0, 45.0)),
            "at {zoom} thousandths a scroll over the plane moved nothing"
        );

        // Screen pixels, which is the camera's own contract: what was under the
        // finger moves by what was scrolled, whatever the zoom. So the plane units
        // travelled are the pixels divided by it.
        let scale = f64::from(zoom) / 1000.0;
        let expected = (
            before.0 + (30.0 / scale) as i32,
            before.1 + (45.0 / scale) as i32,
        );
        let moved = looking_at(&f);
        assert!(
            (moved.0 - expected.0).abs() <= 1 && (moved.1 - expected.1).abs() <= 1,
            "at {zoom} thousandths a scroll of (30, 45) screen pixels left the \
             camera at {moved:?} and it belongs at {expected:?}"
        );

        app.sync();
        assert_eq!(
            app.events.pointer.axes.len(),
            axes,
            "at {zoom} thousandths a scroll over empty canvas also reached the \
             application, so a person reading a document would have the canvas \
             slide out from under it"
        );
    }
}

/// **A scroll over a frame reaches the application and does not pan the plane.**
#[test]
fn a_scroll_over_a_frame_reaches_it_and_the_plane_stays_where_it_is() {
    let f = fixture();
    let mut app = mapped(&f);
    let placed = (200, 100);
    {
        let root = f.root();
        assert!(f.backend(move |s| s.place_window(&root, placed)).is_ok());
    }

    for zoom in THREE_ZOOMS {
        looking(&f, (0, 0), zoom);
        // The middle of the frame, which is where it looks.
        motion(&f, on_the_screen((8.0, 8.0), placed, (0, 0), zoom));
        app.sync();
        let axes = app.events.pointer.axes.len();
        let before = looking_at(&f);

        assert!(
            scroll(&f, (30.0, 45.0)),
            "at {zoom} thousandths a scroll on a frame was not delivered"
        );
        app.sync();

        assert!(
            app.events.pointer.axes.len() > axes,
            "at {zoom} thousandths the frame under the arrow was scrolled and the \
             application was never told"
        );
        assert_eq!(
            looking_at(&f),
            before,
            "at {zoom} thousandths a scroll inside a frame panned the whole canvas"
        );
    }
}

/// **A trackpad's tenths are kept rather than dropped.**
///
/// The plane is measured in whole units and a scroll arrives as a fraction, so
/// truncating each event would make a slow trackpad a canvas that never moves at
/// all — the failure would look like broken hardware rather than like arithmetic.
/// Four tenths five times is two whole pixels, and nothing less.
#[test]
fn a_scroll_too_small_to_spend_is_kept_until_it_is_worth_a_pixel() {
    let f = fixture();
    let _app = mapped(&f);
    looking(&f, (0, 0), 1000);
    motion(&f, (900.0, 700.0));
    let before = looking_at(&f);

    let mut moved_on = 0;
    for _ in 0..5 {
        if scroll(&f, (0.0, 0.4)) {
            moved_on += 1;
        }
    }
    assert_eq!(
        moved_on, 2,
        "five scrolls of four tenths are worth two whole pixels, and this canvas \
         spent them on {moved_on} of the five"
    );
    assert_eq!(
        looking_at(&f),
        (before.0, before.1 + 2),
        "five scrolls of four tenths moved the camera somewhere other than two units"
    );
}

/// **A scroll off the edge of the plane is refused and keeps nothing.**
///
/// `alo-canvas` refuses a pan past the plane by name rather than clamping, and a
/// held remainder would be spent the instant the person turned round — a canvas
/// that jumped when they scrolled back.
#[test]
fn a_scroll_past_the_edge_of_the_plane_is_refused_and_holds_nothing() {
    let f = fixture();
    let _app = mapped(&f);
    looking(&f, (0, 0), 1000);
    motion(&f, (900.0, 700.0));
    let edge = alo_canvas::plane::FURTHEST;
    assert!(f.backend(move |s| {
        s.look_at_the_canvas(At::checked(edge, 0).expect("the plane's own edge"))
            .is_some()
    }));
    let before = looking_at(&f);

    assert!(
        !scroll(&f, (10.0, 0.0)),
        "a scroll off the plane reported that it had moved the canvas"
    );
    assert_eq!(before, looking_at(&f), "a refused scroll panned anyway");
    // And it is not owed back: scrolling the other way moves by its own amount.
    assert!(scroll(&f, (-10.0, 0.0)));
    assert_eq!(
        looking_at(&f),
        (before.0 - 10, before.1),
        "a refused scroll was kept and spent on the next one"
    );
}

/// One two-finger scroll, as a touchpad sends one.
///
/// `AxisSource::Finger` rather than `Wheel`, which is the whole point of the
/// test below: libinput reports the source and nothing on the pan road reads it.
fn two_fingers(f: &Fixture, (across, down): (f64, f64)) -> bool {
    f.backend(move |s| {
        s.pointer_axis(
            AxisFrame::new(31)
                .source(AxisSource::Finger)
                .value(Axis::Horizontal, across)
                .value(Axis::Vertical, down),
        )
    })
    .expect("a scroll with real values")
}

/// **A two-finger scroll pans exactly as a wheel does.**
///
/// Task 5 names *two-finger scroll* beside the wheel and this was recorded as
/// still owed. It was not: `crate::libinput_scroll` turns `ScrollFinger` into an
/// ordinary `AxisFrame` carrying `AxisSource::Finger`, and `crate::canvas_pan`
/// never asks what the source was. So the road already existed and nothing held
/// it — which is the same shape as a capability that is claimed and untested,
/// and the reason this test is here rather than a line in the plan saying it
/// works.
///
/// Asserted against the wheel's own answer rather than against a number written
/// here, so the two cannot drift apart without this failing.
#[test]
fn a_two_finger_scroll_pans_exactly_as_a_wheel_does() {
    let f = fixture();
    let _app = mapped(&f);

    let start = looking_at(&f);
    assert!(scroll(&f, (13.0, -21.0)), "a wheel over the plane pans it");
    let by_wheel = (looking_at(&f).0 - start.0, looking_at(&f).1 - start.1);
    assert_ne!(
        by_wheel,
        (0, 0),
        "the wheel moved nothing to compare against"
    );

    let before = looking_at(&f);
    assert!(
        two_fingers(&f, (13.0, -21.0)),
        "two fingers over the plane pan it"
    );
    let by_fingers = (looking_at(&f).0 - before.0, looking_at(&f).1 - before.1);

    assert_eq!(
        by_fingers, by_wheel,
        "a touchpad and a wheel moved the plane by different amounts for one gesture"
    );
}

/// **Space and drag pans, and only while Space is held.**
///
/// ADR 0065 names it beside the wheel. It needed the fix that let this
/// compositor see a key held with nothing focused — `Server::keyboard_key` used
/// to return before the call that advances XKB whenever `current_focus()` was
/// `None`, which is the only state this gesture happens in.
///
/// Both halves are asserted: that holding Space makes a drag pan, and that
/// letting go stops it. A gesture that panned whether or not the key was down
/// would pass the first half alone.
#[test]
fn space_and_drag_pans_the_plane_and_only_while_space_is_held() {
    let f = fixture();
    let _app = mapped(&f);
    assert!(f.focus(Some(0)).is_ok(), "a window takes focus");

    // With Space up, a motion moves the pointer and not the plane.
    let start = looking_at(&f);
    motion(&f, (400.0, 300.0));
    motion(&f, (460.0, 330.0));
    assert_eq!(
        looking_at(&f),
        start,
        "the plane moved for a drag with no key held"
    );

    // Space down: the same movement pans, and the plane goes the other way —
    // dragging right brings what is on the left into view, as a sheet of paper
    // does.
    assert!(f.key(SPACE, KeyState::Pressed).is_ok());
    motion(&f, (560.0, 330.0));
    let panned = looking_at(&f);
    assert_eq!(
        (panned.0 - start.0, panned.1 - start.1),
        (-100, 0),
        "a hundred pixels of space-drag did not move the plane a hundred units the other way"
    );

    // And letting go stops it.
    assert!(f.key(SPACE, KeyState::Released).is_ok());
    motion(&f, (660.0, 330.0));
    assert_eq!(
        looking_at(&f),
        panned,
        "the plane kept panning after Space was let go"
    );
}
