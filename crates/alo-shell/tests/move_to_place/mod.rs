//! **A frame moves between Places by keyboard** — task 3 of the canvas plan, the
//! road that needs no pointer.
//!
//! The plan gives task 3 two roads and says why: *neither road is the only road —
//! a person who cannot drag can still move a window between Places.* The pointer
//! road drags a frame out through the World and is task 2's to finish. **This one
//! needs no pointer, no World and no aim**, which is what makes it the
//! accessibility road rather than a convenience beside the gesture.
//!
//! Driven through `dispatch_canvas_command` with the real chord, never by calling
//! `move_the_window_in_front_to_the_next_place`. A test that called the function
//! would pass just as well with nothing bound to it — which is how task 6's rule
//! sat with no caller for a fortnight, and how this lane once spent an afternoon
//! diagnosing a machine with no pull request open.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use super::support::{Application, Fixture};
use alo_shortcuts::{Action, Shortcuts};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

/// The viewport these tests render to.
const VIEWPORT: (i32, i32) = (1280, 720);

/// A real display with a keyboard and an output.
fn fixture() -> Fixture {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    f.render(VIEWPORT, false, 1).expect("an output to look at");
    f
}

/// A mapped frame with the keyboard on it, kept alive by the returned handle.
fn a_focused_frame(f: &Fixture) -> (Application, WlSurface) {
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
            move |s| s.activate_window(&surface).is_ok()
        }),
        "the frame could not be given the keyboard"
    );
    (app, surface)
}

/// Press the shipped chord for an action, as a person's keyboard would.
fn press(f: &Fixture, action: Action) {
    let shortcuts = Shortcuts::shipped();
    let chord = shortcuts
        .chord_for(action)
        .expect("the action has a shipped chord");
    f.backend(move |s| {
        let shortcuts = Shortcuts::shipped();
        let _ = s.dispatch_canvas_command(&shortcuts, chord, a_moment());
    });
}

/// **The chord sends the window in front to the next Place.**
///
/// Task 3's keyboard half, through the key a person actually presses.
#[test]
fn the_chord_sends_the_window_in_front_to_the_next_place() {
    let f = fixture();
    let (_app, surface) = a_focused_frame(&f);

    let before = f.backend({
        let surface = surface.clone();
        move |s| s.the_place_of_the_window(&surface)
    });
    assert_eq!(before, Some(alo_canvas::Place::FIRST));

    press(&f, Action::MoveTheWindowToTheNextPlace);

    let after = f.backend({
        let surface = surface.clone();
        move |s| s.the_place_of_the_window(&surface)
    });
    assert_eq!(
        after,
        alo_canvas::Place::FIRST.next(),
        "the chord did not move the window to the next Place"
    );
}

/// **It is a move and never a copy: the window is not on the Place it left.**
///
/// The task's constraint — *a frame belongs to exactly one Place at every
/// moment* — asserted as the absence rather than only the arrival. A version
/// that added the new Place without clearing the old would pass a test that only
/// checked where the window now is.
#[test]
fn the_window_is_not_left_behind_on_the_place_it_came_from() {
    let f = fixture();
    let (_app, _surface) = a_focused_frame(&f);

    press(&f, Action::MoveTheWindowToTheNextPlace);

    let frames_here = f.backend(|s| {
        let on = s.the_place_now();
        s.the_frames_on_the_plane()
            .into_iter()
            .filter(|frame| frame.place() == on)
            .count()
    });
    assert_eq!(
        frames_here, 0,
        "the window is still on the Place it was sent away from"
    );
}

/// **The work goes with it: the frame keeps its point and its size.**
///
/// *The work goes with it* is the plan's own wording, and the whole of what a
/// move does to a frame's geometry is nothing.
#[test]
fn the_frame_keeps_its_point_and_its_size() {
    let f = fixture();
    let (_app, surface) = a_focused_frame(&f);
    assert!(
        f.backend({
            let surface = surface.clone();
            move |s| s.place_window(&surface, (320, 240)).is_ok()
        }),
        "the frame could not be placed"
    );

    let before = f.backend(|s| {
        s.the_frames_on_the_plane()
            .into_iter()
            .next()
            .map(|frame| (frame.at(), frame.size()))
    });
    press(&f, Action::MoveTheWindowToTheNextPlace);
    let after = f.backend(|s| {
        s.the_frames_on_the_plane()
            .into_iter()
            .next()
            .map(|frame| (frame.at(), frame.size()))
    });

    assert_eq!(
        before, after,
        "a move changed the frame's point or its size"
    );
}

/// **The person does not travel with it.**
///
/// Sending a window away is not asking to be taken to it. Travelling is
/// `canvas_a_restore_travels`'s and happens on a restore; a version that followed
/// the window would make *send this away* and *go there* the same key, and a
/// person tidying three windows off their surface would find themselves three
/// Places away.
#[test]
fn moving_a_window_away_does_not_take_the_person_with_it() {
    let f = fixture();
    let (_app, _surface) = a_focused_frame(&f);
    let here = f.backend(|s| s.the_place_now());

    press(&f, Action::MoveTheWindowToTheNextPlace);

    assert_eq!(
        f.backend(|s| s.the_place_now()),
        here,
        "moving a window to another Place took the person there too"
    );
}

/// **Pressing it with nothing in front is an ordinary thing to do.**
///
/// Not a fault, for the reason `dispatch_canvas_command`'s own header gives about
/// zooming at the end of the ladder: a refusal there would make key repeat
/// produce a stream of faults about a thing the screen already shows.
#[test]
fn pressing_it_with_no_window_in_front_is_not_a_fault() {
    let f = fixture();
    let shortcuts = Shortcuts::shipped();
    let chord = shortcuts
        .chord_for(Action::MoveTheWindowToTheNextPlace)
        .expect("the action has a shipped chord");

    let answered = f.backend(move |s| {
        let shortcuts = Shortcuts::shipped();
        s.dispatch_canvas_command(&shortcuts, chord, a_moment())
            .is_ok()
    });
    assert!(answered, "pressing it with nothing in front was a fault");
}

/// A moment a test chooses, so nothing here depends on a clock.
///
/// Only one canvas action reads it — putting a Place back holds the state
/// being left under a moment — and no test in this file presses that chord.
/// A fixed value says so more plainly than `SystemTime::now()` would.
fn a_moment() -> std::time::SystemTime {
    std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_760_000_000)
}
