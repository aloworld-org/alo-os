//! **The World is reached by stepping out, on the roads that already exist** —
//! task 2's gesture.
//!
//! #364 landed the arithmetic and said in its own status that nothing in
//! `alo-shell` answered a step out with it. These drive the **key** and the
//! **wheel** a person actually uses, never `step_out_into_the_world` — a test
//! that called it would pass with nothing wired to it, which is how task 6's
//! rule sat unreachable for a fortnight and how this task's own first half was
//! marked Done with its gesture owed.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use super::support::{Application, Fixture};
use alo_canvas::{Place, Showing, Zoom};
use alo_shortcuts::{Action, Shortcuts};
use smithay::{
    backend::input::{Axis, AxisSource, KeyState},
    input::pointer::AxisFrame,
};

/// `KEY_LEFTCTRL`, as a real keyboard sends it.
const LEFT_CTRL: u32 = 29;

/// One wheel event over the plane, in notches. Positive scrolls out.
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

/// The viewport these tests render to.
const VIEWPORT: (i32, i32) = (1280, 720);

/// A real display with a pointer and an output.
fn fixture() -> Fixture {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    f.render(VIEWPORT, false, 1).expect("an output to look at");
    f
}

/// A mapped frame, kept alive by the returned handle.
fn a_frame(f: &Fixture) -> Application {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    app
}

/// Press the shipped chord for an action, as a person's keyboard would.
fn press(f: &Fixture, action: Action) {
    let chord = Shortcuts::shipped()
        .chord_for(action)
        .expect("the action has a shipped chord");
    f.backend(move |s| {
        let _ = s.dispatch_canvas_command(&Shortcuts::shipped(), chord, a_moment());
    });
}

/// Put the camera at the furthest out a Place goes.
fn as_far_out_as_a_place_goes(f: &Fixture) {
    f.backend(|s| {
        let furthest = Zoom::of(Zoom::FURTHEST_OUT).expect("the furthest out is a zoom");
        assert!(s.zoom_the_canvas(furthest, (0, 0)).is_some());
    });
}

/// **A fresh machine is looking at one Place, not at the World.**
#[test]
fn a_fresh_machine_is_on_one_place() {
    let f = fixture();
    assert_eq!(
        f.backend(|s| s.showing()),
        Showing::OnePlace(Place::FIRST),
        "a machine nobody has used was showing the World"
    );
}

/// **Zooming out at the last rung steps into the World.**
///
/// The whole of task 2's gesture, through the key. Before this the press did
/// nothing: `one_step_out` answered `None` and the branch was a comment saying
/// *nowhere to go, and nothing wrong with having asked*.
#[test]
fn the_last_step_out_reaches_the_world() {
    let f = fixture();
    let _app = a_frame(&f);
    as_far_out_as_a_place_goes(&f);
    assert_eq!(f.backend(|s| s.showing()), Showing::OnePlace(Place::FIRST));

    press(&f, Action::ZoomTheCanvasOut);

    assert_eq!(
        f.backend(|s| s.showing()),
        Showing::TheWorld,
        "the step out at the last rung did not reach the World"
    );
}

/// **A step out before the last rung is an ordinary zoom and stays on the Place.**
///
/// The World must not be reachable early, or a person zooming out to see more of
/// their surface would be taken off it.
#[test]
fn a_step_out_that_has_somewhere_to_go_stays_on_the_place() {
    let f = fixture();
    let _app = a_frame(&f);
    // Life size: several rungs above the bottom.
    press(&f, Action::ZoomTheCanvasOut);

    assert_eq!(
        f.backend(|s| s.showing()),
        Showing::OnePlace(Place::FIRST),
        "an ordinary zoom out left the Place"
    );
}

/// **The wheel reaches it too, by the same step.**
///
/// Task 2's constraint is that the gesture is the canvas's existing zoom *on all
/// three of its roads*. A World reachable by key and not by wheel would be a
/// second navigation model wearing one name.
#[test]
fn the_wheel_reaches_the_world_by_the_same_step() {
    let f = fixture();
    let _app = a_frame(&f);
    as_far_out_as_a_place_goes(&f);

    // Ctrl chooses the zoom road rather than the pan road, exactly as
    // `zoom_and_show_all` drives it. Then scroll out past the last rung, which
    // previously did nothing at all.
    // A key needs somewhere to go before it can be held, which is what
    // `zoom_and_show_all` does before the same press.
    assert!(f.focus(Some(0)).is_ok());
    assert!(
        f.key(LEFT_CTRL, KeyState::Pressed)
            .expect("a real modifier press"),
        "the Ctrl press was not routed at all"
    );
    scroll(&f, 1.0);

    assert_eq!(
        f.backend(|s| s.showing()),
        Showing::TheWorld,
        "the wheel could not reach the World the key reaches"
    );
}

/// **With nothing open there is nowhere to step out to, and the press does
/// nothing.**
///
/// A person with no windows has no Places in their World, and a view moved
/// somewhere arbitrary would be worse than a key that does nothing — which is
/// exactly what the key did before this change.
#[test]
fn with_nothing_open_the_step_out_does_nothing() {
    let f = fixture();
    as_far_out_as_a_place_goes(&f);

    press(&f, Action::ZoomTheCanvasOut);

    assert_eq!(
        f.backend(|s| s.showing()),
        Showing::OnePlace(Place::FIRST),
        "a machine with nothing open was taken to an empty World"
    );
}

/// **Stepping into a tile enters that Place and fills the screen.**
///
/// The other half of the acceptance: *zooming into one fills the screen with
/// it*. Driven from the World the gesture actually reached rather than from a
/// state set by hand.
#[test]
fn stepping_into_a_tile_enters_that_place() {
    let f = fixture();
    let _app = a_frame(&f);
    as_far_out_as_a_place_goes(&f);
    press(&f, Action::ZoomTheCanvasOut);
    assert_eq!(f.backend(|s| s.showing()), Showing::TheWorld);

    // The first tile sits at the origin of the World's own plane.
    let entered = f.backend(|s| s.step_into_the_place_at(alo_canvas::At::origin()));

    assert_eq!(entered, Some(Place::FIRST));
    assert_eq!(f.backend(|s| s.showing()), Showing::OnePlace(Place::FIRST));
    assert_eq!(
        f.backend(|s| s.the_camera().zoom()),
        Zoom::LIFE_SIZE,
        "entering a Place did not fill the screen with it"
    );
}

/// **A point between tiles enters nothing.**
///
/// A gap is a real answer. Taking a person to the nearest Place when they aimed
/// at nothing is the canvas's standing refusal to move somebody without their
/// saying so.
#[test]
fn a_point_between_tiles_enters_nothing() {
    let f = fixture();
    let _app = a_frame(&f);
    as_far_out_as_a_place_goes(&f);
    press(&f, Action::ZoomTheCanvasOut);

    // Well past the only tile, which is a viewport wide at the origin.
    let far = alo_canvas::At::checked(VIEWPORT.0 * 4, 0).expect("a point on the plane");
    assert_eq!(f.backend(move |s| s.step_into_the_place_at(far)), None);
    assert_eq!(
        f.backend(|s| s.showing()),
        Showing::TheWorld,
        "aiming at nothing took the person out of the World anyway"
    );
}

/// A moment a test chooses, so nothing here depends on a clock.
///
/// Only one canvas action reads it — putting a Place back holds the state
/// being left under a moment — and no test in this file presses that chord.
/// A fixed value says so more plainly than `SystemTime::now()` would.
fn a_moment() -> std::time::SystemTime {
    std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_760_000_000)
}
