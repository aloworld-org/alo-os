//! **Zoom, and *Show all*** — the canvas plan's task 6.
//!
//! The acceptance is three sentences and each is a test here: *the point under
//! the pointer is unchanged by a zoom, within a pixel*; *Show all leaves every
//! frame inside the viewport with no frame clipped*; *zoom has a keyboard route*.
//!
//! # What is held here and what is held in `alo-canvas`
//!
//! The arithmetic — the rungs, the fit, and that nothing is clipped by it — is
//! `alo_canvas::camera`'s and is tested there, against spans no compositor has to
//! be started to build. What is held **here** is everything that arithmetic cannot
//! see: that a chord reaches it, that Ctrl chooses the zoom road and its absence
//! chooses the pan road, and that the frames handed to the fit are the ones
//! actually open.
//!
//! That division is the lesson of this seam's first landing, where the zoom was
//! applied to a frame's size and not its position: a test of the arithmetic alone
//! passed while the compositor drew one of two windows.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use super::support::{Application, Fixture};
use alo_shortcuts::{Action, Shortcuts};
use smithay::{
    backend::input::{Axis, AxisSource, KeyState},
    input::pointer::AxisFrame,
};

/// `KEY_LEFTCTRL`, as a real keyboard sends it.
const LEFT_CTRL: u32 = 29;

/// The viewport every test here renders to, so *Show all* has room to fit into.
const VIEWPORT: (i32, i32) = (1280, 720);

/// A real display with a pointer, an output, and nothing open.
fn fixture() -> Fixture {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    f.render(VIEWPORT, false, 1).expect("an output to look at");
    f
}

/// Complete the real XDG handshake and attach a buffer.
fn mapped(f: &Fixture) -> Application {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    app
}

/// Press the chord this action ships with, through the canvas command road.
fn command(f: &Fixture, action: Action) -> Option<Action> {
    let settings = Shortcuts::shipped();
    let chord = settings
        .chord_for(action)
        .expect("every canvas action ships with a chord");
    f.backend(move |s| s.dispatch_canvas_command(&settings, chord))
        .expect("a shipped canvas chord is one this shell can carry out")
}

/// How far in this session is looking, in thousandths.
fn zoom(f: &Fixture) -> u32 {
    f.backend(|s| s.the_camera().zoom().thousandths())
}

/// One wheel event over the plane, in notches.
fn scroll(f: &Fixture, notches: f64) -> bool {
    f.backend(move |s| {
        s.pointer_axis(
            AxisFrame::new(11)
                .source(AxisSource::Wheel)
                .value(Axis::Vertical, notches * 15.0),
        )
    })
    .expect("a scroll with real values")
}

/// **Zoom has a keyboard route, and one press is one rung.**
///
/// The third of task 6's acceptance sentences. The chords are the shipped ones
/// rather than written out here, so a default moved in `alo-shortcuts` moves this
/// test with it instead of leaving it asserting a key nobody has.
#[test]
fn zoom_has_a_keyboard_route_and_one_press_is_one_rung() {
    let f = fixture();
    let _app = mapped(&f);
    assert_eq!(zoom(&f), 1000, "a canvas starts at life size");

    assert_eq!(
        command(&f, Action::ZoomTheCanvasIn),
        Some(Action::ZoomTheCanvasIn)
    );
    assert_eq!(zoom(&f), 1500, "one press in is one rung");
    assert_eq!(
        command(&f, Action::ZoomTheCanvasIn),
        Some(Action::ZoomTheCanvasIn)
    );
    assert_eq!(
        zoom(&f),
        2000,
        "a second press is a second rung, not a leap"
    );

    // **Nothing is accelerated**: presses in quick succession are rungs, not a
    // curve, so going back is exactly the way it came.
    assert_eq!(
        command(&f, Action::ZoomTheCanvasOut),
        Some(Action::ZoomTheCanvasOut)
    );
    assert_eq!(zoom(&f), 1500);
    assert_eq!(
        command(&f, Action::ZoomTheCanvasOut),
        Some(Action::ZoomTheCanvasOut)
    );
    assert_eq!(zoom(&f), 1000, "in and out again is exactly life size");
}

/// **A keyboard zoom holds the middle of the viewport still.**
///
/// The first acceptance sentence, for the route that has no pointer to be centred
/// on. *Within a pixel* is the acceptance's own tolerance; this asserts the plane
/// unit, which is stricter and is what the integer arithmetic actually promises.
#[test]
fn a_keyboard_zoom_holds_the_middle_of_the_viewport() {
    let f = fixture();
    let _app = mapped(&f);
    let middle = (VIEWPORT.0 / 2, VIEWPORT.1 / 2);
    let under = f.backend(move |s| s.the_camera().plane_of(middle));

    for _ in 0..3 {
        assert!(command(&f, Action::ZoomTheCanvasIn).is_some());
        assert_eq!(
            f.backend(move |s| s.the_camera().plane_of(middle)),
            under,
            "a zoom moved what was in the middle of the screen"
        );
    }
}

/// **Ctrl says which road a scroll takes, and nothing else does.**
///
/// This is the test the wheel route exists for. A scroll over the plane pans; the
/// same scroll with Ctrl held zooms; and each has to leave the *other* alone, or a
/// person zooming would also slide the canvas out from under themselves.
#[test]
fn ctrl_chooses_the_zoom_road_and_its_absence_chooses_the_pan_road() {
    let f = fixture();
    let _app = mapped(&f);

    let before = f.backend(|s| (s.the_camera().at().x, s.the_camera().at().y));
    assert!(scroll(&f, 1.0), "a scroll over the plane pans it");
    let panned = f.backend(|s| (s.the_camera().at().x, s.the_camera().at().y));
    assert_ne!(panned, before, "a plain scroll did not pan");
    assert_eq!(zoom(&f), 1000, "a plain scroll zoomed as well as panning");

    assert!(f.focus(Some(0)).is_ok());
    assert!(
        f.key(LEFT_CTRL, KeyState::Pressed)
            .expect("a real modifier press"),
        "the Ctrl press was not routed at all"
    );
    assert!(scroll(&f, -1.0), "Ctrl and a scroll up zooms in");
    assert_eq!(zoom(&f), 1500, "Ctrl and one notch is one rung");
    assert_eq!(
        f.backend(|s| (s.the_camera().at().x, s.the_camera().at().y)),
        panned,
        "a zoom panned the canvas as well"
    );

    f.key(LEFT_CTRL, KeyState::Released)
        .expect("a real modifier release");
    assert!(scroll(&f, 1.0), "the wheel pans again once Ctrl is let go");
    assert_eq!(zoom(&f), 1500, "letting Ctrl go did not stop the zooming");
}

/// **A wheel zoom holds the point under the pointer**, which is the acceptance's
/// first sentence for the route it was written about.
#[test]
fn a_wheel_zoom_holds_the_point_under_the_pointer() {
    let f = fixture();
    let _app = mapped(&f);
    super::support::motion(&f, (317.0, 211.0));
    let at = (317, 211);
    let under = f.backend(move |s| s.the_camera().plane_of(at));

    assert!(f.focus(Some(0)).is_ok());
    assert!(
        f.key(LEFT_CTRL, KeyState::Pressed)
            .expect("a real modifier press"),
        "the Ctrl press was not routed at all"
    );
    for notches in [-1.0, -1.0, 2.0] {
        assert!(scroll(&f, notches));
        assert_eq!(
            f.backend(move |s| s.the_camera().plane_of(at)),
            under,
            "zooming by {notches} notches moved what was under the pointer"
        );
    }
}

/// **Show all leaves every frame inside the viewport, with none clipped.**
///
/// The second acceptance sentence, asked of the compositor rather than of the
/// arithmetic: the frames fitted are the ones actually open, read back through
/// the same screen transform that draws them.
#[test]
fn show_all_leaves_every_frame_inside_the_viewport() {
    let f = fixture();
    let _first = mapped(&f);
    let _second = mapped(&f);

    // Somewhere the frames are certainly not all on screen from.
    assert!(f.backend(|s| s.pan_the_canvas(-4000, -3000)).is_some());
    assert!(command(&f, Action::ShowAllOnTheCanvas).is_some());

    let corners = f.backend(|s| {
        s.the_frames_on_the_plane()
            .into_iter()
            .filter_map(|frame| {
                Some((
                    s.the_camera().screen_of(frame.at())?,
                    s.the_camera().screen_of(frame.opposite()?)?,
                ))
            })
            .collect::<Vec<_>>()
    });
    assert!(!corners.is_empty(), "Show all was asked about no frames");
    for ((left, top), (right, bottom)) in corners {
        assert!(
            left >= 0 && top >= 0,
            "a frame starts off the top-left at ({left}, {top})"
        );
        assert!(
            right <= VIEWPORT.0 && bottom <= VIEWPORT.1,
            "a frame is clipped, ending at ({right}, {bottom})"
        );
    }
}

/// **Show all with nothing open refuses rather than moving somewhere arbitrary.**
#[test]
fn show_all_with_nothing_open_refuses() {
    let f = fixture();
    let before = f.backend(|s| s.the_camera());
    assert_eq!(f.backend(alo_shell::Server::show_all_on_the_canvas), None);
    assert_eq!(
        f.backend(|s| s.the_camera()),
        before,
        "refusing still moved the camera"
    );
}

/// **Ctrl is seen with nothing focused at all, and the wheel still zooms.**
///
/// This test used to assert the opposite, and pinned it deliberately so that
/// changing it would be a decision. This is that decision.
///
/// `keyboard_key` returned before `KeyboardHandle::input` — which is what
/// advances XKB — whenever `current_focus()` was `None`, so on a canvas with no
/// window focused the compositor could not read its own modifiers and a
/// Ctrl+wheel panned. A compositor has to know what is held to answer for its own
/// gestures; only forwarding to a client depends on focus.
///
/// **The two halves are asserted separately**, because the fix is only right if
/// both hold: the shell sees the modifier, *and* no client was told about a key
/// pressed while nothing was focused.
#[test]
fn ctrl_is_seen_with_nothing_focused_and_the_wheel_still_zooms() {
    let f = fixture();
    let _app = mapped(&f);
    assert!(f.focus(None).is_ok(), "nothing focused");

    assert!(
        !f.key(LEFT_CTRL, KeyState::Pressed)
            .expect("a real modifier press"),
        "a key pressed with nothing focused was reported as delivered to a client"
    );
    let before = f.backend(|s| (s.the_camera().at().x, s.the_camera().at().y));
    assert!(
        scroll(&f, -1.0),
        "Ctrl and a scroll up zooms with nothing focused"
    );
    assert_eq!(
        zoom(&f),
        1500,
        "the shell could not see Ctrl with nothing focused"
    );
    assert_eq!(
        f.backend(|s| (s.the_camera().at().x, s.the_camera().at().y)),
        before,
        "the zoom panned the canvas as well"
    );

    // And letting go is seen too, so the modifier does not stick.
    assert!(
        !f.key(LEFT_CTRL, KeyState::Released)
            .expect("a real modifier release"),
        "a key released with nothing focused was reported as delivered to a client"
    );
    assert!(scroll(&f, -1.0), "the wheel pans again once Ctrl is let go");
    assert_eq!(zoom(&f), 1500, "Ctrl stayed held after it was released");
}

/// Pinch the plane, as a touchpad does: begin, some scales, end.
fn pinch(f: &Fixture, scales: &[f64]) {
    f.backend(|s| s.pointer_pinch_begin(2, 20))
        .expect("a pinch begins on a seat with a pointer");
    for scale in scales {
        let scale = *scale;
        f.backend(move |s| s.pointer_pinch_update(scale, 0.0, (0.0, 0.0), 21))
            .expect("a pinch update with real values");
    }
    f.backend(|s| s.pointer_pinch_end(false, 22))
        .expect("a pinch ends");
}

/// **A pinch zooms the canvas, and its scale is against where it began.**
///
/// The plan's task 6 names a pinch beside the wheel, and it needed a protocol
/// rather than a branch: `zwp_pointer_gestures_v1`. The scale a touchpad reports
/// is absolute against the moment the fingers went down, so two updates of the
/// same scale are the same zoom rather than that zoom twice — which is the
/// difference between moving a thing with your hand and accelerating it.
#[test]
fn a_pinch_zooms_the_canvas_by_the_scale_it_reports() {
    let f = fixture();
    let _app = mapped(&f);
    assert_eq!(zoom(&f), 1000);

    pinch(&f, &[2.0]);
    assert_eq!(
        zoom(&f),
        2000,
        "a pinch to twice the size did not double the zoom"
    );

    // Back to life size, then the same scale sent twice in one gesture.
    pinch(&f, &[0.5]);
    assert_eq!(zoom(&f), 1000);
    pinch(&f, &[1.5, 1.5, 1.5]);
    assert_eq!(
        zoom(&f),
        1500,
        "the same scale three times was spent three times, so a pinch accelerates"
    );
}

/// **A pinch holds the point under the fingers**, which is task 6's acceptance
/// for every zoom road it names.
#[test]
fn a_pinch_holds_the_point_under_the_pointer() {
    let f = fixture();
    let _app = mapped(&f);
    super::support::motion(&f, (409.0, 277.0));
    let at = (409, 277);
    let under = f.backend(move |s| s.the_camera().plane_of(at));

    pinch(&f, &[1.5, 2.0, 1.2]);
    assert_eq!(
        f.backend(move |s| s.the_camera().plane_of(at)),
        under,
        "a pinch moved what was under the fingers"
    );
}

/// **A pinch past the end of the zoom range rests there rather than refusing.**
///
/// The one place the canvas clamps, and it is a person's fingers that make it
/// so: they do not stop at `FURTHEST_IN`, and abandoning the gesture would leave
/// the next update fighting the one before it.
#[test]
fn a_pinch_past_the_end_rests_at_the_end() {
    let f = fixture();
    let _app = mapped(&f);

    pinch(&f, &[1000.0]);
    assert_eq!(zoom(&f), alo_canvas::Zoom::FURTHEST_IN);
    pinch(&f, &[0.000_001]);
    assert_eq!(zoom(&f), alo_canvas::Zoom::FURTHEST_OUT);
}

/// **A pinch that never began moves nothing**, so a stray update from a device
/// that was reset cannot zoom a canvas nobody is touching.
#[test]
fn an_update_with_no_pinch_in_progress_moves_nothing() {
    let f = fixture();
    let _app = mapped(&f);
    let before = f.backend(|s| s.the_camera());

    assert!(f.focus(None).is_ok());
    assert!(
        !f.backend(|s| s.pointer_pinch_update(4.0, 0.0, (0.0, 0.0), 30))
            .expect("an update with real values"),
        "an update with no pinch in progress reported that something moved"
    );
    assert_eq!(f.backend(|s| s.the_camera()), before);
}

/// **A scale that is not a number is refused rather than turned into a zoom.**
#[test]
fn a_pinch_with_no_real_scale_is_refused() {
    let f = fixture();
    let _app = mapped(&f);
    for scale in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(
            f.backend(move |s| s.pointer_pinch_update(scale, 0.0, (0.0, 0.0), 31))
                .is_err(),
            "{scale} was accepted as a pinch"
        );
    }
}

/// **Turning off two-finger pinch zoom turns it off on the canvas too.**
///
/// `alo_desktops::gesture_settings::Preferences::pinch` is the person's setting
/// and it is named *enable two-finger pinch zoom*. It used to gate only the
/// gesture recogniser — and the canvas reads the touchpad directly rather than
/// through that recogniser, so for one landing the setting was a lie about the
/// single gesture it names.
#[test]
fn a_person_who_turns_pinch_zoom_off_is_not_pinch_zoomed() {
    let f = fixture();
    let _app = mapped(&f);

    let off = alo_desktops::gesture_settings::Preferences {
        pinch: false,
        ..alo_desktops::gesture_settings::Preferences::default()
    };
    f.backend(move |s| s.gestures_are_configured(off));

    assert!(
        !f.backend(|s| s.pointer_pinch_begin(2, 40))
            .expect("a pinch begins on a seat with a pointer"),
        "a pinch began on the plane with pinch zoom turned off"
    );
    pinch(&f, &[2.0]);
    assert_eq!(
        zoom(&f),
        1000,
        "the canvas zoomed with pinch zoom turned off"
    );

    // And back on again, so the setting is read each time rather than at start-up.
    f.backend(|s| {
        s.gestures_are_configured(alo_desktops::gesture_settings::Preferences::default())
    });
    pinch(&f, &[2.0]);
    assert_eq!(
        zoom(&f),
        2000,
        "turning pinch zoom back on did not restore it"
    );
}
