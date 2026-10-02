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

/// A display that draws one pixel per logical one, in hundredths.
///
/// Named rather than written as `100` at each site, because **every test in this
/// file meant it implicitly until 2026-10-02** and a bare number would not say
/// which of the two scales it is. The handle floor is logical; these rectangles
/// are in the display's pixels; this is what converts between them.
const ONE_TO_ONE: u16 = 100;

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
                display_scale: ONE_TO_ONE,
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
                display_scale: ONE_TO_ONE,
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

/// Tell the server where the controls are, on a display of `scale` hundredths.
fn drawn_at_a_display_scale(f: &Fixture, scale: u16) {
    f.backend(move |s| {
        s.the_fixed_controls_were_drawn(
            alo_shell::FixedControlsDrawn {
                dock_band: Some(a_dock_along_the_bottom()),
                panel_reserved: Rectangle::default(),
                display_scale: scale,
            },
            alo_appearance::TextScale::ordinary(),
        );
    });
}

/// How far right a drag gets with the Dock drawn at this display scale.
fn how_far_a_drag_gets_at(scale: u16) -> f64 {
    let f = fixture();
    let (_app, surface) = a_frame(&f);
    drawn_at_a_display_scale(&f, scale);
    assert!(
        f.backend({
            let surface = surface.clone();
            move |s| s.place_window(&surface, (200, 100))
        })
        .is_ok()
    );
    take_hold_of_the_name(&f, (200.0, 100.0));
    // **Chosen so the floor is the only thing that decides.** The grab sits 16
    // above the frame's top, so a pointer at 662 puts the top at 678 and the
    // 48-tall name band at 630..678. The Dock's band starts at 656, leaving
    // **26 pixels** of name clear.
    //
    // 26 clears a 24-pixel floor and fails a 30-, 36- or 48-pixel one. So this
    // exact motion is allowed at one to one and refused at 125, 150 and 200 per
    // cent — which is only true if the logical floor is converted into the
    // pixels these rectangles are measured in.
    //
    // *A first attempt aimed at 670, leaving 34 clear. That is allowed at 125
    // because 34 still clears a floor of 30 — the code was right and the
    // expectation was wrong, which is worth recording because the failure looked
    // exactly like the bug.*
    motion(&f, (202.0, 662.0));
    origin_of(&f, &surface).1
}

/// **The handle floor is logical and the controls are in the display's pixels,
/// so the floor is converted once — and a dense display protects more pixels,
/// not the same number.**
///
/// `A_USABLE_HANDLE` is 44 × 24 **logical**, by the owner's ruling, scaled by
/// the person's text size and nothing else. The rectangles it is compared
/// against are laid out from `target.size()`, the framebuffer. Comparing them
/// without converting made the protected area **too small by the display's
/// scale**: on a screen drawing two pixels per logical one, a floor of 24 was 24
/// framebuffer pixels where the promise is 48.
///
/// **Every test in this file ran at one to one, so nothing could see it.** That
/// is the same reason the division's scale sat at a literal `1` for as long as
/// the file existed.
///
/// At 125, 150 and 200 by the owner's direction. The fractional two matter most:
/// a converter written as `scale / 100` floors both to one, and a test at 1
/// against 2 cannot tell that apart from a correct one.
#[test]
fn the_handle_floor_converts_with_the_displays_scale() {
    let at_one = how_far_a_drag_gets_at(100);

    for scale in [125u16, 150, 200] {
        let dense = how_far_a_drag_gets_at(scale);
        assert!(
            dense < at_one,
            "at {scale} per cent the drag reached y={dense} and at 100 it reached \
             y={at_one}: a denser display did not protect more of the name band, \
             so the logical floor was never converted into the pixels it is \
             compared against"
        );
    }
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
