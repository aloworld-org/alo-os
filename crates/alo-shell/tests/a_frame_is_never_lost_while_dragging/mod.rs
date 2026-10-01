//! **The rule is in force on the path a person takes**, which it was not before.
//!
//! `crate::canvas_never_lost` has held *a frame keeps a usable part of its name
//! outside every fixed control* since it was written, with unit tests and with
//! integration tests that call it directly. **Nothing on any path a person could
//! take ever asked it.** `window_move.rs` and `canvas_space_drag.rs` mentioned it
//! zero times; its only public entry, `put_back_where_it_was`, was called from
//! one test and nothing else. A person could drag a frame under the Dock and the
//! rule forbidding it was never consulted.
//!
//! So every test here drives a **real pointer drag** — press on the name band,
//! move the pointer — and asks where the frame actually ended up. None of them
//! calls the rule. A test that called it would pass exactly as well before this
//! change as after, which is the property that let the gap survive.
//!
//! # Why the Dock's bounds are handed in
//!
//! `Server::the_dock_was_drawn` is what the draw calls, and these tests call it
//! for the same reason: the Dock's band is laid out in the raster path and there
//! is no draw in a headless fixture. **The band passed here is the one
//! `direct_desktop` passes** — `pictures.desktop.dock.band` — so the shape of the
//! fixture's input is the shape of production's, not a convenience.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use super::support::{Application, Fixture, motion};
use smithay::{
    backend::input::ButtonState::Pressed,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Rectangle},
};

/// The viewport these tests render to.
const VIEWPORT: (i32, i32) = (1280, 720);

/// How thick the Dock's band is here.
const DOCK: i32 = 64;

/// A real display with a pointer and an output.
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
        s.the_dock_was_drawn(
            Some(a_dock_along_the_bottom()),
            alo_appearance::TextScale::ordinary(),
        );
    });
}

/// A mapped frame, and the surface it is. **Kept alive** by the returned handle.
fn a_frame(f: &Fixture) -> (Application, WlSurface) {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    let surface = f
        .backend(|s| s.mapped_surfaces().last().cloned())
        .expect("one frame is mapped");
    (app, surface)
}

/// Where this frame's buffer origin is, which is where a drag puts it.
///
/// Read through `alo_shell::window_buffer_origin`, the same function every scene
/// consumer reads, rather than through anything this test could arrange.
fn origin_of(f: &Fixture, surface: &WlSurface) -> (f64, f64) {
    let surface = surface.clone();
    f.backend(move |_| {
        let point = alo_shell::window_buffer_origin(&surface);
        (point.x, point.y)
    })
}

/// Press on the name band of a frame whose corner is at `(x, y)` on the screen.
fn take_hold_of_the_name(f: &Fixture, at: (f64, f64)) {
    let in_the_name = (at.0 + 2.0, at.1 - 16.0);
    motion(f, in_the_name);
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Pressed, 2)).ok(),
        Some(false),
        "a press on the name band was delivered to a client, and the band is the shell's"
    );
}

/// **A drag cannot leave a frame's name under the Dock.**
///
/// The sentence the whole task is for, asked of a real drag rather than of the
/// rule. Before this change the frame went wherever the pointer went.
#[test]
fn a_drag_cannot_put_a_frames_name_under_the_dock() {
    let f = fixture();
    let (_app, surface) = a_frame(&f);
    drawn_with_a_dock(&f);

    assert!(
        f.backend({
            let surface = surface.clone();
            move |s| s.place_window(&surface, (200, 100))
        })
        .is_ok()
    );
    take_hold_of_the_name(&f, (200.0, 100.0));
    // **Aimed so the name band lands wholly inside the Dock's band**, which is
    // what the rule is about. The band sits `THE_NAMES_BAND` above the frame's
    // top edge, so a frame at y=700 has its name at 676..700 and the Dock's
    // 656..720 covers every row of it.
    //
    // Dragging further is a different loss and this rule does not cover it: at
    // y=736 the band is 712..736, the Dock's bottom edge no longer crosses the
    // band's full height, and the rule correctly answers *reachable* about a
    // band that is in fact below the screen. See the note at the foot of this
    // file.
    motion(&f, (202.0, 694.0));

    let (_, y) = origin_of(&f, &surface);
    assert!(
        y < 704.0,
        "a drag left the frame at y={y}, with its whole name band under the Dock"
    );
}

/// **The frame keeps the last position that was allowed.**
///
/// The owner's ruling of 2026-09-30: *keep the last valid position while the
/// pointer continues moving; never accept an invalid placement and then pull the
/// frame back.* So a refused motion leaves the frame **exactly** where the last
/// allowed one put it — not near it, and not corrected into place afterwards.
#[test]
fn a_refused_motion_leaves_the_frame_exactly_where_it_was() {
    let f = fixture();
    let (_app, surface) = a_frame(&f);
    drawn_with_a_dock(&f);

    assert!(
        f.backend({
            let surface = surface.clone();
            move |s| s.place_window(&surface, (300, 120))
        })
        .is_ok()
    );
    take_hold_of_the_name(&f, (300.0, 120.0));

    // One allowed motion, then one whose name band would be wholly under the
    // Dock. Both are ordinary pointer motions; only the second is refused.
    motion(&f, (302.0, 200.0));
    let after_the_allowed_one = origin_of(&f, &surface);
    motion(&f, (302.0, 694.0));
    let after_the_refused_one = origin_of(&f, &surface);

    assert_eq!(
        after_the_refused_one, after_the_allowed_one,
        "a refused motion moved the frame; it must keep the last allowed position"
    );
}

/// **With nothing drawn, a drag is not held back.**
///
/// Before the first frame the shell does not know where the Dock is, and a Dock
/// nobody has drawn covers nothing. The honest behaviour is to allow the drag
/// rather than to protect against a rectangle that has never existed — and the
/// alternative, treating unknown as *everywhere*, would freeze every drag on a
/// machine that has not drawn yet.
#[test]
fn with_nothing_drawn_a_drag_is_not_held_back() {
    let f = fixture();
    let (_app, surface) = a_frame(&f);
    // Deliberately no `drawn_with_a_dock`.

    assert!(
        f.backend({
            let surface = surface.clone();
            move |s| s.place_window(&surface, (200, 100))
        })
        .is_ok()
    );
    take_hold_of_the_name(&f, (200.0, 100.0));
    motion(&f, (202.0, 300.0));

    let (_, y) = origin_of(&f, &surface);
    assert!(
        y > 100.0,
        "with no controls known the drag was held back anyway, at y={y}"
    );
}

// # What this rule does not cover, found by a test that asserted it did
//
// **The fixed controls, and not the viewport's own edges.** The first version of
// `a_drag_cannot_put_a_frames_name_under_the_dock` dragged to the bottom of the
// screen and expected a refusal. It got none, and the rule was right: at that
// point the name band had passed *below* the Dock's bottom edge, so the Dock no
// longer crossed the band's full height — which
// `crate::canvas_never_lost::the_longest_reachable_run` requires, deliberately,
// because a control clipping a few rows of a band leaves a shorter but still
// grabbable strip.
//
// So a frame can still be dragged off the bottom of the screen, where its name is
// not under any control and is not reachable either. `show_all_would_still_reach`
// bounds the **plane**, not the glass, and nothing bounds the glass.
//
// **Recorded rather than fixed here.** It is a real gap in a v0.01 promise and it
// is not the gap this change closes; conflating them would let *the rule now runs
// on the drag path* read as *a frame can no longer be lost*, which is not yet
// true. Whether the viewport's edges join the set of things a name must stay
// clear of is a question for the owner: the Dock is a control a person can move
// a frame out from under, and a screen edge is not.
