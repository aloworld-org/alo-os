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
//! # Why the controls' bounds are handed in
//!
//! `Server::the_fixed_controls_were_drawn` is what the draw calls, and these tests
//! call it
//! for the same reason: the Dock's band is laid out in the raster path and there
//! is no draw in a headless fixture. **The band passed here is the one
//! `direct_desktop` passes** — `pictures.desktop.dock.band` — so the shape of the
//! fixture's input is the shape of production's, not a convenience.
//!
//! # The rule was in force against one control, not every one
//!
//! The promise is *outside **every** fixed control*. Until 2026-10-02 the shell
//! handed over the Dock's band alone, so a frame could keep its name clear of the
//! Dock and sit entirely under the put-aside panel, and **nothing could tell** —
//! the rule was right, its caller existed, and the bounds it was given were a
//! third of the question. The last test here could not have been written before.
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

// There was a `ONE_TO_ONE: u16 = 100` here, and its own doc said *the handle
// floor is logical; these rectangles are in the display's pixels; this is what
// converts between them.* The middle clause was false and the constant existed
// to serve it. Both are gone: the draw takes no scale, so no test in this file
// has one to name.

/// How wide the put-aside panel's reserved column is here.
///
/// `alo_dock::measures` derives the real one; this is a width in the shape of
/// one, because the test is about the rule and not about the panel's arithmetic.
const PANEL: i32 = 112;

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

/// The put-aside panel's reserved column, down the right-hand edge, as
/// `panel_raster` lays one out for a panel that holds windows.
fn a_panel_down_the_right() -> Rectangle<i32, Physical> {
    Rectangle::new((VIEWPORT.0 - PANEL, 0).into(), (PANEL, VIEWPORT.1).into())
}

/// Tell the server where the fixed controls are, as a draw would — **the Dock
/// alone**, which is all the shell handed over until 2026-10-02.
fn drawn_with_a_dock(f: &Fixture) {
    f.backend(|s| {
        s.the_fixed_controls_were_drawn(
            alo_shell::FixedControlsDrawn {
                dock_band: Some(a_dock_along_the_bottom()),
                // A panel nobody has put a window into covers nothing, and a
                // rectangle of no extent is the true answer for one rather than
                // a placeholder.
                panel_reserved: Rectangle::default(),
            },
            alo_appearance::TextScale::ordinary(),
        );
    });
}

/// The same, with the put-aside panel drawn as well.
fn drawn_with_a_dock_and_a_panel(f: &Fixture) {
    f.backend(|s| {
        s.the_fixed_controls_were_drawn(
            alo_shell::FixedControlsDrawn {
                dock_band: Some(a_dock_along_the_bottom()),
                panel_reserved: a_panel_down_the_right(),
            },
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

/// **The put-aside panel is a fixed control too, and this is the test that
/// could not be written before 2026-10-02.**
///
/// The promise is *a frame keeps a usable part of its name outside **every**
/// fixed control*. The shell handed the rule the Dock's band alone, so a frame
/// could keep its name clear of the Dock, sit entirely under the panel's
/// reserved column, and **nothing could tell**: the rule was right, its caller
/// existed, and the bounds it was given were a third of the question.
///
/// Dragged to the right rather than down, so the Dock is not what refuses it.
/// The same drag is allowed when only the Dock was drawn — that is the companion
/// assertion, and without it this test would pass for a rule that refuses
/// everything.
#[test]
fn a_drag_cannot_put_a_frames_name_under_the_put_aside_panel() {
    let allowed_with_only_a_dock = {
        let f = fixture();
        let (_app, surface) = a_frame(&f);
        drawn_with_a_dock(&f);
        assert!(
            f.backend({
                let surface = surface.clone();
                move |s| s.place_window(&surface, (200, 300))
            })
            .is_ok()
        );
        take_hold_of_the_name(&f, (200.0, 300.0));
        // Into the column the panel reserves: 1168..1280 here.
        motion(&f, (1210.0, 302.0));
        origin_of(&f, &surface).0
    };

    let held_back_with_the_panel = {
        let f = fixture();
        let (_app, surface) = a_frame(&f);
        drawn_with_a_dock_and_a_panel(&f);
        assert!(
            f.backend({
                let surface = surface.clone();
                move |s| s.place_window(&surface, (200, 300))
            })
            .is_ok()
        );
        take_hold_of_the_name(&f, (200.0, 300.0));
        motion(&f, (1210.0, 302.0));
        origin_of(&f, &surface).0
    };

    assert!(
        allowed_with_only_a_dock > held_back_with_the_panel,
        "the panel's reserved column held nothing back: the drag reached x={held_back_with_the_panel} \
         with the panel drawn and x={allowed_with_only_a_dock} without it, so the same \
         position was allowed either way and the rule never saw the panel"
    );
}

/// **The floor is 24 along, and this is the only test that can feel its size.**
///
/// # Why it had to be written
///
/// Doubling `A_USABLE_HANDLE` passed **all eleven** other tests in this file.
/// That is the whole reason a conversion could be added to the draw, land, and
/// sit there: every other margin here is wide enough that a floor of 24 and a
/// floor of 48 both allow the same motions, so the suite could not tell a
/// correct floor from one twice the size.
///
/// The test that *could* feel it was the display-scale one, and it spent that
/// sensitivity asserting that the floor should change — so the one tight margin
/// in the file was pointed at the bug rather than at the promise.
///
/// # How it feels the size without naming a number
///
/// The same motion is driven twice: once with nothing drawn, which nothing holds
/// back, and once with the Dock drawn. **A motion leaving 26 pixels of the name
/// clear must reach the same place either way**, because 26 clears a floor of 24.
/// It would be held back by a floor of 30, 36 or 48.
///
/// Comparing the two runs rather than asserting a coordinate is deliberate: the
/// expected y depends on where the grab sits and how tall the name band is, and
/// a test that hardcodes it fails when either changes for an unrelated reason.
/// The free run *is* the expected value, measured in the same breath.
#[test]
fn a_motion_leaving_twenty_six_pixels_of_the_name_clear_is_allowed() {
    // The grab sits 16 above the frame's top, so a pointer at 662 puts the top
    // at 678 and the 48-tall name band at 630..678. The Dock's band starts at
    // 656, leaving 26 pixels of the name clear.
    let aim = (202.0, 662.0);

    let free = {
        let f = fixture();
        let (_app, surface) = a_frame(&f);
        assert!(
            f.backend({
                let surface = surface.clone();
                move |s| s.place_window(&surface, (200, 100))
            })
            .is_ok()
        );
        take_hold_of_the_name(&f, (200.0, 100.0));
        motion(&f, aim);
        origin_of(&f, &surface).1
    };

    let with_the_dock = {
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
        motion(&f, aim);
        origin_of(&f, &surface).1
    };

    assert_eq!(
        with_the_dock, free,
        "a motion leaving 26 pixels of the name clear was held back: it reached \
         y={with_the_dock} with the Dock drawn and y={free} with nothing drawn. \
         26 clears the 24 the owner's ruling promises, so the floor in force is \
         larger than 24 — which is what a conversion applied to a logical floor \
         does to it."
    );
}

// **The display's scale is not an input to this rule, and the test that used
// to stand here asserted that it was.**
//
// # What was wrong, and why it passed
//
// The owner's ruling of 2026-10-01 is that `A_USABLE_HANDLE` is 44 × 24
// **logical**, scaled by the person's text size *and nothing else*, and asks
// explicitly that any implication of a second conversion be removed. The two
// commits after that ruling added one.
//
// The test that guarded it asserted that a denser display reached a *smaller*
// y — that the drag was restricted further. It passed, and it was measuring the
// bug: a floor multiplied by the scale protects 88 × 48 at 200 per cent, so of
// course the drag stops sooner. **A denser display restricting the drag more is
// equally consistent with the floor arriving and with the floor being too
// large**, and nothing in the test told them apart.
//
// # What settled it
//
// `desktop_raster_tests::the_dock_band_and_the_panel_column_do_not_move_with_the_displays_scale`
// draws one display at 100, 125, 150 and 200 per cent and finds the Dock's band
// and the panel's reserved column **identical at all four**. They are laid out
// from the room, which arrives already divided by the scale. So they are
// logical, the floor they are compared against is logical, and there is nothing
// to convert. Their `Physical` marker is satisfied by construction and says
// nothing either way — the trap this module's header records two lanes losing an
// hour to, and a third hour here.
//
// # Why there is no replacement test in this file
//
// **The draw no longer hands a scale over, so there is no scale here to vary.**
// A test that took one and ignored it would be the same trap as the field it
// replaced: a number present and meaningless, which the next reader uses.
//
// The guarantee moved to where the scale actually exists. One test, in the
// raster, at the owner's three scales, asserting that these rectangles do not
// move — and the division beside them, which genuinely is converted, asserted
// by `the_displays_scale_reaches_what_is_drawn` to move. One place, both
// behaviours, told apart.
//
// What stays here is what this file can answer: the floor in logical units,
// against controls in the same units, driven by a real drag.

/// **A control that appears over a frame puts it out of reach, and until
/// 2026-10-02 nothing noticed.**
///
/// The rule is in force on the two roads a frame *moves* by — a drag, and a
/// recovery putting a window back where it was left. Both ask before placing.
/// **Neither asks again**, and the controls move underneath a frame that is not
/// moving at all: the Dock gives way and comes back, a window is put aside so
/// the panel's column appears where there was none, the display or its scale
/// changes, the Dock moves to another edge.
///
/// Task 8's third acceptance clause is exactly this: *reachability is rechecked
/// when those bounds change.*
///
/// Three positions, so the answer is not *everything* or *nothing*: a frame well
/// clear of the panel is not named, a frame under it is, and with no panel drawn
/// neither is.
#[test]
fn a_frame_the_panel_now_covers_is_named_by_the_recheck() {
    let f = fixture();
    let (_app, surface) = a_frame(&f);

    // Under where the panel's column will be, and clear of the Dock.
    assert!(
        f.backend({
            let surface = surface.clone();
            move |s| s.place_window(&surface, (VIEWPORT.0 - 40, 300))
        })
        .is_ok()
    );

    // **The Dock alone, which is what the draw hands over until a window is put
    // aside.** The frame is reachable: nothing is over it.
    drawn_with_a_dock(&f);
    let hidden_before = f.backend(|s| s.frames_the_controls_now_hide());
    assert!(
        hidden_before.is_empty(),
        "a frame with nothing over it was named as hidden: {hidden_before:?}"
    );

    // **Now a window is put aside and the panel takes its column.** The frame
    // has not moved; the controls have.
    drawn_with_a_dock_and_a_panel(&f);
    let hidden_after = f.backend(|s| s.frames_the_controls_now_hide());
    assert!(
        !hidden_after.is_empty(),
        "the panel's column appeared over a frame and the recheck named nobody, \
         so a frame can be put out of reach by a control moving and nothing notices"
    );
}

/// **A frame the controls hid is brought back, and where it was is kept.**
///
/// The other half of task 8's third clause: *where recovery needs a frame
/// moved, the move is shown and its previous position recorded.* A frame moved
/// by the machine is a person's arrangement edited without them, so `was` is
/// what makes the move undoable rather than merely visible.
///
/// Four things are asserted, because a mover can fail in four ways: it can move
/// nothing, it can move something that was fine, it can move the frame
/// somewhere still hidden, and it can forget where the frame came from.
#[test]
fn a_frame_the_panel_hid_is_brought_back_and_where_it_was_is_kept() {
    let f = fixture();
    let (_app, surface) = a_frame(&f);

    // Under where the panel's column will be, clear of the Dock.
    let put_at = (VIEWPORT.0 - 40, 300);
    assert!(
        f.backend({
            let surface = surface.clone();
            move |s| s.place_window(&surface, put_at)
        })
        .is_ok()
    );

    // Nothing over it yet: the mover must not touch it.
    drawn_with_a_dock(&f);
    let quiet = f.backend(|s| s.bring_back_frames_the_controls_hide());
    assert!(
        quiet.is_empty(),
        "a frame with nothing over it was moved: {quiet:?}"
    );
    let still_there = origin_of(&f, &surface);

    // The panel's column appears over it. The frame has not moved; the
    // controls have.
    drawn_with_a_dock_and_a_panel(&f);
    let done = f.backend(|s| s.bring_back_frames_the_controls_hide());

    // Two different failures, named apart: nothing was planned at all, and
    // something was planned and declined. A decline is a real outcome of the
    // search, so reading it as an absence would hide which half went wrong.
    let outcome = *done
        .first()
        .expect("the panel covered a frame and nothing was brought back");
    assert!(
        matches!(outcome, alo_shell::Recovery::BroughtBack { .. }),
        "the frame was hidden and the search declined to move it: {outcome:?}"
    );
    let (was, now) = match outcome {
        alo_shell::Recovery::BroughtBack { was, now, .. } => (was, now),
        // Refused by the assertion above. The arm is here because the match
        // must be total, and it returns a pair that fails the next assertion
        // too rather than one that would quietly pass.
        alo_shell::Recovery::CouldNotBeBroughtBack { at, .. } => (at, at),
    };
    assert_ne!(was, now, "it was reported as moved and did not move");
    assert_eq!(
        (f64::from(was.x), f64::from(was.y)),
        still_there,
        "where it was does not match where it actually was, so the record a \
         person would undo by is wrong"
    );

    // And the place it was moved to is one the controls do not hide — asked of
    // the shell rather than worked out here.
    let hidden_now = f.backend(|s| s.frames_the_controls_now_hide());
    assert!(
        hidden_now.is_empty(),
        "it was moved somewhere the controls still hide: {hidden_now:?}"
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
