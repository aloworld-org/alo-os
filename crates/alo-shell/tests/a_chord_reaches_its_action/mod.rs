//! **A chord pressed on a running desktop reaches its action** —
//! `docs/autonomy/the-shell-plan.md` task 19.
//!
//! # Why this is not the fifteen tests that already pass
//!
//! That task's central observation is that every existing test of a shortcut
//! **enters the chain at `dispatch_canvas_command`**, handing it a `Chord` and a
//! `Shortcuts` directly. Fifteen actions were green that way while no key press
//! on a machine could reach any of them, and *no suite could fail for that
//! reason*.
//!
//! So nothing here calls a dispatcher. Every test below presses **evdev codes**
//! through `Server::keyboard_key` — the function the libinput road calls on a
//! real machine, through `libinput_update` and `direct_seat` — and asks what the
//! canvas did. The chord is the one this release ships rather than one written
//! out here, so a default moved in `alo-shortcuts` moves this test with it.
//!
//! What is still not proved here, said plainly: a *display*. The acceptance asks
//! for a person pressing a key and seeing it, and that is the half this lane has
//! no machine for. What this file closes is the other half — that the road
//! exists and that a key travels it.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use super::support::{Application, Fixture};
use alo_shortcuts::Shortcuts;
use smithay::backend::input::KeyState;

/// `KEY_LEFTMETA`, which is `⊞` as a real keyboard sends it.
const SUPER: u32 = 125;
/// `KEY_0`, the other half of the shipped *Show all* chord.
const DIGIT0: u32 = 11;
/// `KEY_A`, which this release binds to nothing.
const LETTER_A: u32 = 30;

/// The viewport every test here renders to, so *Show all* has room to fit into.
const VIEWPORT: (i32, i32) = (1280, 720);

/// A real display with an output and two windows open, looking nowhere near
/// them.
///
/// Panned away deliberately: *Show all* that did nothing would be
/// indistinguishable from a chord that never arrived, which is the one thing
/// these tests exist to tell apart.
fn a_desktop_looking_away() -> (Fixture, Application, Application) {
    let f = Fixture::keyboard();
    f.render(VIEWPORT, false, 1).expect("an output to look at");
    let first = mapped(&f);
    let second = mapped(&f);
    assert!(
        f.backend(|s| s.pan_the_canvas(-4000, -3000)).is_some(),
        "the canvas would not pan away from the frames"
    );
    (f, first, second)
}

/// Complete the real XDG handshake and attach a buffer.
fn mapped(f: &Fixture) -> Application {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    app
}

/// Where this session is looking, which is what *Show all* moves.
fn looking_at(f: &Fixture) -> (i32, i32) {
    f.backend(|s| {
        let at = s.the_camera().at();
        (at.x, at.y)
    })
}

/// Press a key through the road libinput takes, and say whether a client got it.
fn press(f: &Fixture, code: u32) -> bool {
    f.backend(move |s| s.keyboard_key(code, KeyState::Pressed, 7))
        .expect("a real evdev code on a seat with a keyboard")
}

/// Let a key go, the same way.
fn release(f: &Fixture, code: u32) -> bool {
    f.backend(move |s| s.keyboard_key(code, KeyState::Released, 8))
        .expect("a real evdev code on a seat with a keyboard")
}

/// **The whole road: `⊞`+0 pressed on a running desktop shows all of it.**
///
/// This is task 19's acceptance in the part a test can hold — a key press
/// reaching `dispatch_canvas_command` with the chord it made, and one of the
/// shipped defaults doing what it says.
#[test]
fn a_chord_pressed_on_a_running_desktop_reaches_its_action() {
    let (f, _first, _second) = a_desktop_looking_away();
    f.backend(|s| s.the_shortcuts_are(Shortcuts::shipped()));
    let away = looking_at(&f);

    assert!(!press(&f, SUPER), "a modifier alone is not a chord");
    assert!(
        !press(&f, DIGIT0),
        "a chord the shell took must not also reach a client"
    );

    assert_ne!(
        looking_at(&f),
        away,
        "the chord arrived and Show all did not move the camera"
    );
}

/// **A seat nobody has told about shortcuts routes every key as it always did.**
///
/// The sign-in screen, the nested lane and every test that existed before this
/// road did are all in this state, and none of them may change behaviour. The
/// same two presses that moved the camera above move nothing here.
#[test]
fn a_seat_told_no_shortcuts_takes_no_key() {
    let (f, _first, _second) = a_desktop_looking_away();
    let away = looking_at(&f);

    press(&f, SUPER);
    press(&f, DIGIT0);

    assert_eq!(
        looking_at(&f),
        away,
        "a seat that was told no shortcuts looked one up anyway"
    );
}

/// **A chord beats the window in front, and an unbound key does not.**
///
/// The property that makes a system shortcut a system shortcut. `⊞`+0 is bound
/// so that somebody typing in a document can still reach their canvas; a road
/// that worked only with nothing focused would work on an empty desktop and
/// nowhere else.
///
/// `a` in the same breath is the control: with a window focused it is delivered,
/// so the test above cannot be passing because nothing is ever delivered.
#[test]
fn a_chord_is_taken_from_the_window_in_front() {
    let (f, _first, _second) = a_desktop_looking_away();
    f.backend(|s| s.the_shortcuts_are(Shortcuts::shipped()));
    let root = f.root();
    f.focus_surface(root)
        .expect("the window would not take focus");
    let away = looking_at(&f);

    assert!(
        press(&f, LETTER_A),
        "a key this release binds to nothing must still reach the focused window"
    );
    assert!(release(&f, LETTER_A));

    press(&f, SUPER);
    assert!(
        !press(&f, DIGIT0),
        "a bound chord reached the window instead of the shell"
    );
    assert_ne!(
        looking_at(&f),
        away,
        "a focused window swallowed a system chord"
    );
}

/// **The release of a chord is not delivered either.**
///
/// A client that saw no press must never see the release: an unpaired release
/// tells it a key it never saw held has gone up. Nothing in this road remembers
/// which keys it took — the keyboard's own `forwarded` set already holds only
/// the keys a client was told about, and a press that never entered it has a
/// release with nowhere to go.
#[test]
fn the_release_of_a_chord_a_client_never_saw_is_not_delivered() {
    let (f, _first, _second) = a_desktop_looking_away();
    f.backend(|s| s.the_shortcuts_are(Shortcuts::shipped()));
    let root = f.root();
    f.focus_surface(root)
        .expect("the window would not take focus");

    press(&f, SUPER);
    assert!(!press(&f, DIGIT0), "the press was delivered");
    assert!(
        !release(&f, DIGIT0),
        "a client was sent the release of a press it never saw"
    );
    assert!(release(&f, SUPER));
}
