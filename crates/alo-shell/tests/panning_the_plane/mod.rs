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
//! # The keyboard route, and the reason it was missing for two days
//!
//! It is here now, at the foot of this file: arrows pan, Shift + arrow pans a
//! whole viewport, and an application with focus keeps its own arrow keys.
//! `crate::canvas_arrow_pan` holds the road.
//!
//! **This header used to say it was somebody else's gap, and that was wrong in a
//! way worth keeping.** It reasoned: a chord reaches a native operation through
//! `alo_shortcuts::Action`; `alo_shortcuts::Chord` requires Super, Ctrl or Alt;
//! the design file gives panning the bare arrow keys; therefore *a pan shortcut is
//! not a thing that enum can hold*, and the question belongs to `alo-shortcuts`.
//!
//! Every step of that is true and the conclusion does not follow. The road did not
//! need to be a chord — `crate::canvas_space_drag` was already panning from a bare
//! Space, in this crate, without one, and this file's own last test exercises it.
//! **The answer was beside the question the whole time.** What the reasoning
//! actually established was that the road is *not in `alo-shortcuts`*, and that
//! got written down as *not ours*, which is a different claim and the one that let
//! task 5 be marked done for all three roads while one of them did not exist.
//!
//! # All three roads are held in this file now
//!
//! The wheel, the trackpad's two-finger form, Space-and-drag and the arrows. This
//! header said *neither is held here; the plan says so* about the last two long
//! after `a_two_finger_scroll_pans_exactly_as_a_wheel_does` and
//! `space_and_drag_pans_the_plane_and_only_while_space_is_held` were sitting a
//! screen below it. **Third stale claim found in this one header** — a document
//! and the code beneath it disagreeing, with the code right each time.
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

/// `KEY_UP`, `KEY_LEFT`, `KEY_RIGHT`, `KEY_DOWN` and `KEY_LEFTSHIFT`.
const UP: u32 = 103;
/// `KEY_LEFT`.
const LEFT: u32 = 105;
/// `KEY_RIGHT`.
const RIGHT: u32 = 106;
/// `KEY_DOWN`.
const DOWN: u32 = 108;
/// `KEY_LEFTSHIFT`, which the design file's *pan faster* is held with.
const SHIFT: u32 = 42;

/// An output wide enough that a quarter of it is a round number either way.
const VIEWPORT: (i32, i32) = (400, 200);

/// A display with a keyboard, a pointer and an output of a known size.
///
/// The output matters here and nowhere else in this file: a keyboard pan is
/// measured as a fraction of the viewport, so a fixture with no output has
/// nothing to take a fraction of.
fn on_a_screen() -> Fixture {
    let f = fixture();
    assert!(f.render(VIEWPORT, false, 1).is_ok());
    f
}

/// Press and release one key, and say whether a client was told.
fn press(f: &Fixture, code: u32) -> bool {
    let told = f.key(code, KeyState::Pressed).expect("a real key");
    assert!(f.key(code, KeyState::Released).is_ok());
    told
}

/// **Arrows pan the canvas, and one press is a quarter of the viewport.**
///
/// The keyboard road of task 5's three, which `docs/design/the-shortcuts-and-the-
/// edges.md` gives as *pan in any direction: arrows while the canvas is focused*.
/// It was missing while task 5 was marked done for all three roads — the header
/// of this very file recorded it as a gap, and the plan's status line said
/// otherwise.
///
/// All four directions, because each is a different arm of the same arithmetic
/// and a sign error in one would pass a test that only pressed Right. The
/// distance is asserted too: a road that panned by some amount would satisfy
/// *the plane moved* while making the canvas unusable by keyboard.
#[test]
fn an_arrow_pans_the_plane_and_a_step_is_a_quarter_of_the_viewport() {
    let f = on_a_screen();
    let _app = mapped(&f);
    let (quarter_across, quarter_down) = (VIEWPORT.0 / 4, VIEWPORT.1 / 4);

    for (key, expected) in [
        (RIGHT, (quarter_across, 0)),
        (LEFT, (-quarter_across, 0)),
        (DOWN, (0, quarter_down)),
        (UP, (0, -quarter_down)),
    ] {
        looking(&f, (1000, 1000), 1000);
        let before = looking_at(&f);
        assert!(
            !press(&f, key),
            "an arrow over the canvas was handed to an application"
        );
        let moved = (looking_at(&f).0 - before.0, looking_at(&f).1 - before.1);
        assert_eq!(moved, expected, "key {key} moved the plane by {moved:?}");
    }
}

/// **Shift goes faster, and *faster* is exactly four ordinary presses.**
///
/// The design file's second row, *pan faster: Shift + arrow*. Asserted against
/// the plain press's own answer rather than against a number written here, so the
/// two cannot drift apart: if the step changes, this still says Shift is four of
/// them. That is the same discipline `a_two_finger_scroll_pans_exactly_as_a_wheel
/// _does` keeps one road over.
#[test]
fn shift_and_an_arrow_pans_exactly_four_ordinary_presses() {
    let f = on_a_screen();
    let _app = mapped(&f);

    looking(&f, (1000, 1000), 1000);
    let before = looking_at(&f);
    assert!(!press(&f, RIGHT));
    let plain = looking_at(&f).0 - before.0;
    assert_ne!(
        plain, 0,
        "the ordinary press moved nothing to compare against"
    );

    looking(&f, (1000, 1000), 1000);
    let before = looking_at(&f);
    assert!(f.key(SHIFT, KeyState::Pressed).is_ok());
    assert!(
        !press(&f, RIGHT),
        "Shift and an arrow reached an application"
    );
    assert!(f.key(SHIFT, KeyState::Released).is_ok());
    let faster = looking_at(&f).0 - before.0;

    assert_eq!(
        faster,
        plain * 4,
        "Shift moved {faster} where four ordinary presses move {}",
        plain * 4
    );
}

/// **An application with focus owns its own arrow keys, and the plane stays.**
///
/// *While the canvas is focused* is the design file's condition and it is the
/// whole of what keeps this road from breaking every text field on the machine.
/// Both halves asserted, as the acceptance's *asserts which moved* asks
/// everywhere else in this file: the application is told, **and** the camera did
/// not move. A road that panned as well as delivering would pass a test that only
/// watched the client.
#[test]
fn an_arrow_belongs_to_a_focused_application_and_the_plane_stays_where_it_is() {
    let f = on_a_screen();
    let mut app = mapped(&f);
    assert!(f.focus(Some(0)).is_ok(), "a window takes focus");
    app.sync();

    looking(&f, (1000, 1000), 1000);
    let before = looking_at(&f);
    let keys = app.events.keyboard.keys.len();

    assert!(
        press(&f, RIGHT),
        "an arrow was kept from the application that had focus"
    );
    app.sync();

    assert_eq!(
        looking_at(&f),
        before,
        "an arrow meant for an application panned the canvas underneath it"
    );
    assert!(
        app.events.keyboard.keys.len() > keys,
        "the focused application was never told about the arrow"
    );
}

/// **An arrow at the edge of the plane is still the canvas's.**
///
/// Pressing left at the left-hand edge is an ordinary thing to do, and the screen
/// already shows there is nowhere further. Handing the key on to nobody would make
/// one key mean two things depending on where the canvas happens to sit — the same
/// argument `crate::canvas_command` makes about zooming at the end of the ladder,
/// where a refusal would turn key repeat into a stream of faults.
#[test]
fn an_arrow_at_the_edge_of_the_plane_is_not_handed_on() {
    let f = on_a_screen();
    let _app = mapped(&f);

    // As far left as the plane goes, so the next press cannot be honoured.
    f.backend(|s| {
        let at = At::checked(-alo_canvas::plane::FURTHEST, 0).expect("the plane's own edge");
        assert!(s.look_at_the_canvas(at).is_some());
    });
    let before = looking_at(&f);

    assert!(
        !press(&f, LEFT),
        "an arrow refused by the plane's edge was handed to an application"
    );
    assert_eq!(
        looking_at(&f),
        before,
        "a pan past the plane's edge moved the camera anyway"
    );
}
