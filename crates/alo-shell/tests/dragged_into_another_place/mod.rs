//! **A frame dragged out to the World and dropped into another Place** — task 3's
//! pointer road.
//!
//! Every case here drives the three gestures a person uses and nothing else:
//! take hold of the frame by its name, zoom out until the World appears, let go
//! over a tile. No test calls `drop_the_frame_into_the_place_at`; one that did
//! would pass with nothing wired to the release.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use super::support::{Application, Fixture};
use alo_canvas::{Place, Showing, Zoom};
use alo_shortcuts::{Action, Shortcuts};
use smithay::{
    backend::input::ButtonState, reexports::wayland_server::protocol::wl_surface::WlSurface,
};

/// The viewport these tests render to.
const VIEWPORT: (i32, i32) = (1280, 720);

/// `BTN_LEFT`, as a real pointer sends it.
const LEFT: u32 = 0x110;

/// A real display with a pointer and an output.
fn fixture() -> Fixture {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    f.render(VIEWPORT, false, 1).expect("an output to look at");
    f
}

/// A mapped frame at a known point, kept alive by the returned handle.
fn a_frame_at(f: &Fixture, at: (i32, i32)) -> (Application, WlSurface) {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    let surface = f
        .backend(|s| s.mapped_surfaces().last().cloned())
        .expect("one frame is mapped");
    assert!(
        f.backend({
            let surface = surface.clone();
            move |s| s.place_window(&surface, at).is_ok()
        }),
        "the frame could not be placed"
    );
    (app, surface)
}

/// Move the pointer there, in the screen's own pixels.
fn motion(f: &Fixture, (x, y): (f64, f64)) {
    assert!(f.backend(move |s| s.pointer_motion(x, y, 7)).is_ok());
}

/// Take hold of the frame whose corner is at this screen point, by its name band.
fn take_hold_of(f: &Fixture, at: (f64, f64)) {
    motion(f, (at.0 + 2.0, at.1 - 16.0));
    assert_eq!(
        f.backend(|s| s.pointer_button(LEFT, ButtonState::Pressed, 2))
            .ok(),
        Some(false),
        "a press on the name band was delivered to a client, and the band is the shell's"
    );
}

/// Press the shipped chord for an action.
fn press(f: &Fixture, action: Action) {
    let chord = Shortcuts::shipped()
        .chord_for(action)
        .expect("the action has a shipped chord");
    f.backend(move |s| {
        let _ = s.dispatch_canvas_command(&Shortcuts::shipped(), chord, a_moment());
    });
}

/// Zoom out until the World appears, which is the last rung plus one step.
fn zoom_out_to_the_world(f: &Fixture) {
    f.backend(|s| {
        let furthest = Zoom::of(Zoom::FURTHEST_OUT).expect("the furthest out is a zoom");
        assert!(s.zoom_the_canvas(furthest, (0, 0)).is_some());
    });
    press(f, Action::ZoomTheCanvasOut);
    assert_eq!(
        f.backend(|s| s.showing()),
        Showing::TheWorld,
        "the canvas did not reach the World"
    );
}

/// Where this frame's buffer origin is.
fn origin_of(f: &Fixture, surface: &WlSurface) -> (f64, f64) {
    let surface = surface.clone();
    f.backend(move |_| {
        let point = alo_shell::window_buffer_origin(&surface);
        (point.x, point.y)
    })
}

/// Let go at this screen point.
fn let_go_at(f: &Fixture, at: (f64, f64)) {
    motion(f, at);
    let _ = f.backend(|s| s.pointer_button(LEFT, ButtonState::Released, 9));
}

/// **A frame held while zooming out is still held, and the World is reached.**
///
/// The first of the three gestures, asserted on its own because the rest is
/// meaningless if letting go of nothing is what is being tested.
#[test]
fn a_frame_stays_held_while_the_canvas_zooms_out_to_the_world() {
    let f = fixture();
    let (_app, _surface) = a_frame_at(&f, (100, 100));
    take_hold_of(&f, (100.0, 100.0));

    zoom_out_to_the_world(&f);

    assert!(
        f.backend(|s| s.the_frame_a_drag_holds().is_some()),
        "zooming out let go of the frame"
    );
    assert!(
        f.backend(|s| s.a_drag_is_choosing_a_place()),
        "a drag in the World is not choosing a Place"
    );
}

/// **A drag in the World moves no window.**
///
/// The World's coordinates are the tiles' and a frame's are its own Place's, so
/// writing one into the other would move a window by the distance between two
/// unrelated planes. The frame must be exactly where it was.
#[test]
fn a_drag_in_the_world_does_not_move_the_frame() {
    let f = fixture();
    let (_app, surface) = a_frame_at(&f, (100, 100));
    take_hold_of(&f, (100.0, 100.0));
    zoom_out_to_the_world(&f);

    let before = origin_of(&f, &surface);
    motion(&f, (900.0, 500.0));
    motion(&f, (40.0, 600.0));
    let after = origin_of(&f, &surface);

    assert_eq!(
        before, after,
        "a drag in the World moved the frame on its own Place"
    );
}

/// **Letting go between tiles leaves the frame where it was, on the Place it was
/// already on.**
///
/// A drop on nothing is a drop on nothing. Choosing the nearest Place for
/// somebody who aimed between two would move a window rather than a view, which
/// is the worse half of the canvas's standing refusal.
#[test]
fn letting_go_between_tiles_changes_nothing() {
    let f = fixture();
    let (_app, surface) = a_frame_at(&f, (100, 100));
    let before_place = f.backend({
        let surface = surface.clone();
        move |s| s.the_place_of_the_window(&surface)
    });
    take_hold_of(&f, (100.0, 100.0));
    zoom_out_to_the_world(&f);
    let before_origin = origin_of(&f, &surface);

    // Far to the right of the only tile, which sits at the World's origin.
    let_go_at(&f, (1_270.0, 700.0));

    assert_eq!(
        f.backend({
            let surface = surface.clone();
            move |s| s.the_place_of_the_window(&surface)
        }),
        before_place,
        "a drop on the gap moved the frame to another Place"
    );
    assert_eq!(
        origin_of(&f, &surface),
        before_origin,
        "a drop on the gap moved the frame on its own Place"
    );
}

/// **Dropped on the tile it is already on, nothing happens.**
///
/// A drop that changed nothing must not read as a move — and must not be
/// answered as one, or a record of *where it was* would be written for a window
/// that did not go anywhere.
#[test]
fn dropping_a_frame_on_its_own_place_changes_nothing() {
    let f = fixture();
    let (_app, surface) = a_frame_at(&f, (100, 100));
    take_hold_of(&f, (100.0, 100.0));
    zoom_out_to_the_world(&f);

    // The only tile is the Place this frame is already on.
    let_go_at(&f, (10.0, 10.0));

    assert_eq!(
        f.backend({
            let surface = surface.clone();
            move |s| s.the_place_of_the_window(&surface)
        }),
        Some(Place::FIRST),
        "a frame dropped on its own Place left it"
    );
}

/// **A frame dragged to the World and dropped on another Place is on that Place
/// and not on the first, with its point and its size.**
///
/// Task 3's acceptance sentence for the pointer road, and the only test here that
/// asserts the move rather than a refusal to move. The World needs two tiles for
/// there to be anywhere to drop, and a tile exists when something is on the Place
/// — so a second frame is sent to the second Place first, by the keyboard road,
/// which is the other half of this same task.
#[test]
fn a_frame_dropped_on_another_place_is_on_it_and_not_on_the_first() {
    let f = fixture();
    let elsewhere = Place::FIRST.next().expect("there is a second place");

    // A second frame, sent to the second Place so that the World has two tiles.
    let (_other, other) = a_frame_at(&f, (900, 400));
    f.backend({
        let other = other.clone();
        move |s| s.move_the_window_to(&other, elsewhere)
    });

    // The frame this test is about, still on the first Place.
    let (_app, surface) = a_frame_at(&f, (100, 100));
    let before = origin_of(&f, &surface);
    assert_eq!(
        f.backend({
            let surface = surface.clone();
            move |s| s.the_place_of_the_window(&surface)
        }),
        Some(Place::FIRST)
    );

    take_hold_of(&f, (100.0, 100.0));
    zoom_out_to_the_world(&f);

    // Where the second tile is on the glass, asked of the camera rather than
    // worked out here — a test that computed it would be asserting this file's
    // arithmetic rather than the canvas's.
    let onto = f
        .backend(move |s| {
            let tile = s
                .the_world()
                .each()
                .position(|place| place == elsewhere)
                .map(|nth| nth as i32)?;
            let across = VIEWPORT.0;
            let at = alo_canvas::At::checked(tile * across * 2 + 8, 8)?;
            s.the_camera().screen_of(at)
        })
        .expect("the second tile is on the glass");

    let_go_at(&f, (f64::from(onto.0), f64::from(onto.1)));

    assert_eq!(
        f.backend({
            let surface = surface.clone();
            move |s| s.the_place_of_the_window(&surface)
        }),
        Some(elsewhere),
        "the frame was not put on the Place it was dropped on"
    );
    assert_eq!(
        origin_of(&f, &surface),
        before,
        "the work did not go with it: the frame's point changed"
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
