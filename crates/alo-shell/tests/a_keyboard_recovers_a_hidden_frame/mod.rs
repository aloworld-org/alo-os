//! **A keyboard can recover a frame the fixed controls hide** — task 6's fourth
//! acceptance check, asked of the roads that already exist rather than answered
//! with a new one.
//!
//! The check reads *keyboard users can find and move a frame without reaching its
//! name band*, and the plan records honestly that **moving a frame within one
//! Place by keyboard is not promised**: `docs/features.md`'s frames line attaches
//! *each with a keyboard form* to fitting, filling and working inside, not to
//! dragging. So nothing here invents that road.
//!
//! What **is** promised, and landed as task 3, is *Move to Place* — and a frame
//! whose name is under the Dock is reachable again the moment it is on a Place
//! where the Dock is not over it. **So the question this file answers is whether
//! the promised road is enough for the check, rather than whether a new one could
//! be.** It is enough for recovery and not for placement, and the difference is
//! stated at the foot of this file rather than left for somebody to discover.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use super::support::{Application, Fixture};
use alo_canvas::Place;
use alo_shortcuts::{Action, Shortcuts};
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Rectangle},
};

/// The viewport these tests render to.
const VIEWPORT: (i32, i32) = (1280, 720);

/// How thick the Dock's band is here.
const DOCK: i32 = 64;

/// A real display with a keyboard, a pointer and an output.
fn fixture() -> Fixture {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    f.render(VIEWPORT, false, 1).expect("an output to look at");
    f
}

/// A Dock along the bottom, as `dock_raster` lays one out.
fn a_dock_along_the_bottom() -> Rectangle<i32, Physical> {
    Rectangle::new((0, VIEWPORT.1 - DOCK).into(), (VIEWPORT.0, DOCK).into())
}

/// Tell the server where the fixed controls are, as a draw would.
fn drawn_with_a_dock(f: &Fixture) {
    f.backend(|s| {
        s.the_fixed_controls_were_drawn(
            alo_shell::FixedControlsDrawn {
                dock_band: Some(a_dock_along_the_bottom()),
                // A panel nobody has put a window into covers nothing, which is
                // the true answer rather than a placeholder.
                panel_reserved: Rectangle::default(),
                status_area: None,
            },
            alo_appearance::TextScale::ordinary(),
        );
    });
}

/// A mapped frame holding the keyboard, kept alive by the returned handle.
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

/// Press the shipped chord for an action.
fn press(f: &Fixture, action: Action) {
    let chord = Shortcuts::shipped()
        .chord_for(action)
        .expect("the action has a shipped chord");
    f.backend(move |s| {
        let _ = s.dispatch_canvas_command(&Shortcuts::shipped(), chord);
    });
}

/// Put this frame where the Dock covers its whole name band.
fn hidden_under_the_dock(f: &Fixture, surface: &WlSurface) {
    let surface = surface.clone();
    assert!(
        f.backend(move |s| s.place_window(&surface, (300, 710)).is_ok()),
        "the frame could not be placed under the Dock"
    );
}

/// **A frame under the Dock is reported as hidden, and that is the premise.**
///
/// Asserted on its own so the recovery test cannot pass by the frame never having
/// been hidden — which is how a check that could not fail gets written.
#[test]
fn a_frame_under_the_dock_is_reported_as_hidden() {
    let f = fixture();
    let (_app, surface) = a_focused_frame(&f);
    drawn_with_a_dock(&f);
    hidden_under_the_dock(&f, &surface);

    let hidden = f.backend(|s| s.frames_the_controls_now_hide());
    assert_eq!(
        hidden.len(),
        1,
        "a frame whose name is wholly under the Dock was not reported hidden"
    );
}

/// **One keypress recovers it, without reaching its name band.**
///
/// Task 6's fourth check, answered by *Move to Place* — a road `docs/features.md`
/// already promises. The frame moves to a Place the Dock is not over, so it is
/// reachable again, and the person touched nothing but a key.
#[test]
fn one_keypress_recovers_a_frame_the_dock_hides() {
    let f = fixture();
    let (_app, surface) = a_focused_frame(&f);
    drawn_with_a_dock(&f);
    hidden_under_the_dock(&f, &surface);
    assert_eq!(f.backend(|s| s.frames_the_controls_now_hide()).len(), 1);

    press(&f, Action::MoveTheWindowToTheNextPlace);

    assert_eq!(
        f.backend({
            let surface = surface.clone();
            move |s| s.the_place_of_the_window(&surface)
        }),
        Place::FIRST.next(),
        "the keypress did not move the frame off the Place it was hidden on"
    );
    assert!(
        f.backend(|s| s.frames_the_controls_now_hide()).is_empty(),
        "the frame is still reported hidden on the Place being looked at"
    );
}

/// **The frame keeps its point and its size: recovery is not a tidy-up.**
///
/// What makes this recovery rather than rearrangement. A road that recovered a
/// frame by moving it somewhere convenient would be the machine editing a
/// person's arrangement, which this repository forbids without showing the move
/// and recording where it was.
#[test]
fn recovering_by_keyboard_does_not_move_the_frame_on_its_own_place() {
    let f = fixture();
    let (_app, surface) = a_focused_frame(&f);
    drawn_with_a_dock(&f);
    hidden_under_the_dock(&f, &surface);

    let before = {
        let surface = surface.clone();
        f.backend(move |_| {
            let point = alo_shell::window_buffer_origin(&surface);
            (point.x, point.y)
        })
    };
    press(&f, Action::MoveTheWindowToTheNextPlace);
    let after = {
        let surface = surface.clone();
        f.backend(move |_| {
            let point = alo_shell::window_buffer_origin(&surface);
            (point.x, point.y)
        })
    };

    assert_eq!(
        before, after,
        "recovery moved the frame on its own Place, which is a tidy-up rather than a recovery"
    );
}

// # What this does not answer, and whose question it is
//
// **Recovery, yes. Placement, no.** *Move to Place* gets a keyboard user to a
// frame the controls hide, which is what the fourth check is for — but it
// recovers by changing **which surface** the frame is on, not by moving it a few
// hundred units out from under the Dock on the surface it is already on.
//
// A person who wants the second has no keyboard road, and `docs/features.md` does
// not promise them one: the frames line attaches *each with a keyboard form* to
// fitting the Place to the screen, filling the screen with what is selected, and
// double-clicking to work inside — **not to dragging**. Moving a frame within one
// Place by keyboard is unpromised, and this lane has not built it on that basis.
//
// **That is the owner's to settle, and it is a real question rather than a
// formality:** the case the check is mostly about is a frame whose name is under
// a control and which wants to be moved *a little*, and the answer today is *send
// it to another Place*, which is a larger act than the person asked for.
