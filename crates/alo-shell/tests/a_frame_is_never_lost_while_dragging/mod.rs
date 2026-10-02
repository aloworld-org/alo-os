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
                status_area: None,
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
                status_area: None,
            },
            alo_appearance::TextScale::ordinary(),
        );
    });
}

/// The same, with the status area drawn where the owner placed it.
///
/// Top-right, as ADR 0076's gap was filled on 2026-09-30. No Dock and no panel,
/// so a frame held back here is held back by the status area and nothing else —
/// a bound that is one of three cannot be tested while the other two can hide
/// its absence.
fn drawn_with_a_status_area(f: &Fixture) {
    f.backend(|s| {
        s.the_fixed_controls_were_drawn(
            alo_shell::FixedControlsDrawn {
                dock_band: None,
                panel_reserved: Rectangle::default(),
                status_area: Some(a_status_area_at_the_top_right()),
            },
            alo_appearance::TextScale::ordinary(),
        );
    });
}

/// Where the indicator is drawn: the far end of the Dock's edge, at the top.
fn a_status_area_at_the_top_right() -> Rectangle<i32, Physical> {
    Rectangle::new((VIEWPORT.0 - 400, 0).into(), (400, 240).into())
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

/// **The status area reaches the rule, and nothing asked until 2026-10-02.**
///
/// Canvas task 6 names three fixed controls as a set and the rule held two. The
/// status area could not join: `EgressStatusPicture` carried rows and words and
/// no rectangle, so there was nothing to hand over.
///
/// # Why the test that already existed could not catch this
///
/// `the_status_area_hides_a_name_the_same_way_the_dock_does` passes a rectangle
/// **straight to the rule** and proves the rule handles any rectangle. It
/// cannot notice whether the status area's rectangle is ever handed over —
/// and when the field was removed from the bounds, **all nine hundred and fifty
/// tests stayed green.** A rule that takes a list is tested by what is in the
/// list, never by what the caller put there.
///
/// So this asks at the seam: the draw says where the status area is, and a
/// frame under it is named. Nothing else is drawn, so a frame named here is
/// named by the status area alone.
#[test]
fn a_frame_under_the_status_area_is_named_by_the_recheck() {
    let f = fixture();
    let (_app, surface) = a_frame(&f);

    // Under where the status area will be, clear of everything else.
    let put_at = (VIEWPORT.0 - 200, 200);
    assert!(
        f.backend({
            let surface = surface.clone();
            move |s| s.place_window(&surface, put_at)
        })
        .is_ok()
    );

    drawn_with_a_status_area(&f);
    let hidden = f.backend(|s| s.frames_the_controls_now_hide());

    assert!(
        !hidden.is_empty(),
        "a frame sits under the status area and the recheck named nobody. The \
         status area is one of the three controls task 6 promises, and a bound \
         the draw never hands over is a promise the rule cannot keep."
    );
}

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

/// **The controls moving is what owes a recheck, and a frame redrawn owes
/// nothing.**
///
/// The first half of task 6's third clause: *recovery is rechecked when the
/// display, the scale, the Dock's bounds or the panel's state changes.* The
/// detector and the mover above were both written, both tested, and **both called
/// by tests alone** until 2026-10-02 — on a running machine the Dock could grow
/// over a frame and nothing asked either of them.
///
/// So this drives the trigger rather than the recovery, and the three outcomes it
/// walks through are three different answers that a plain empty list would have
/// rendered as one:
///
/// - the first draw, which is a change because before it nothing was known;
/// - **the same draw again, which must owe nothing** — the assertion that fails
///   if a redraw counts as a move, and the reason the trigger is a flag set on a
///   comparison rather than on every call;
/// - the panel's column appearing, which moves a frame.
///
/// A test asserting *no recoveries* after an unchanged frame would pass against a
/// trigger wired to nothing at all, which is why [`None`] and an empty [`Vec`] are
/// different answers here and are asserted apart.
#[test]
fn the_controls_moving_is_what_owes_a_recheck() {
    let f = fixture();
    let (_app, surface) = a_frame(&f);

    // Under where the panel's column will be, clear of the Dock: a frame that is
    // reachable now and will not be when the panel appears, without ever moving.
    assert!(
        f.backend({
            let surface = surface.clone();
            move |s| s.place_window(&surface, (VIEWPORT.0 - 40, 300))
        })
        .is_ok()
    );

    // **The first draw is a change.** Before it the shell did not know where any
    // control was, and *unknown* becoming *the Dock is here* is exactly the event
    // this trigger is for.
    drawn_with_a_dock(&f);
    let first = f
        .backend(|s| s.bring_back_frames_the_moved_controls_hide())
        .expect("the first draw told the shell where the Dock is, which is a change");
    assert!(
        first.is_empty(),
        "a frame with nothing over it was moved by the first draw: {first:?}"
    );

    // **The same draw again, and nothing may be owed.** The draw runs every frame
    // with the same bounds whenever nothing has moved; a trigger that fired here
    // would run the search sixty times a second and, worse, would make every
    // assertion below meaningless — a recheck that always fires cannot show that
    // a change is what caused one.
    drawn_with_a_dock(&f);
    assert!(
        f.backend(|s| s.bring_back_frames_the_moved_controls_hide())
            .is_none(),
        "an identical draw owed a recheck, so the recovery runs on every frame \
         and nothing distinguishes the Dock moving from the clock ticking"
    );

    // **Now the panel's column appears over it.** The frame has not moved; the
    // controls have, which is the whole of the clause.
    drawn_with_a_dock_and_a_panel(&f);
    let moved = f
        .backend(|s| s.bring_back_frames_the_moved_controls_hide())
        .expect("the panel's column appeared over a frame, which is a change");
    let outcome = *moved
        .first()
        .expect("the controls moved over a frame and nothing was brought back");
    assert!(
        matches!(outcome, alo_shell::Recovery::BroughtBack { .. }),
        "the controls moved over a frame and the search declined to move it: {outcome:?}"
    );

    // And the debt is settled rather than standing: asked again with no draw in
    // between, there is nothing owed. A trigger that never cleared would move a
    // frame once and then re-examine it forever.
    assert!(
        f.backend(|s| s.bring_back_frames_the_moved_controls_hide())
            .is_none(),
        "the recheck ran and stayed owed, so it would run again on every frame"
    );
}

/// **A display that changes owes a recheck, which is the clause's own first
/// trigger.**
///
/// The clause names four: *the display, the scale, the Dock's bounds or the
/// panel's state.* The test above moves the panel; this one moves the **display**,
/// by drawing the same Dock on a shorter screen — which is what `direct_desktop`
/// hands over when a mode changes, because the band is laid out from the target's
/// own size.
///
/// Worth its own test rather than a second case in the one above because the
/// mechanism differs: the panel appears as a **new rectangle** and a display
/// change **moves an existing one**, and a comparison that noticed only
/// appearances would pass the test above and fail a person whose screen resized.
#[test]
fn a_display_that_changed_owes_a_recheck() {
    let f = fixture();
    let (_app, surface) = a_frame(&f);

    // Low on the screen, and clear of the Dock as the full-height display lays it
    // out.
    assert!(
        f.backend({
            let surface = surface.clone();
            move |s| s.place_window(&surface, (200, VIEWPORT.1 - DOCK - 120))
        })
        .is_ok()
    );

    drawn_with_a_dock(&f);
    let settled = f
        .backend(|s| s.bring_back_frames_the_moved_controls_hide())
        .expect("the first draw is a change");
    assert!(
        settled.is_empty(),
        "a frame clear of the Dock was moved: {settled:?}"
    );

    // **The screen got shorter, so the Dock's band is somewhere else.** Nothing
    // about the frame changed and nothing about the panel did.
    let shorter = VIEWPORT.1 - 200;
    f.backend(move |s| {
        s.the_fixed_controls_were_drawn(
            alo_shell::FixedControlsDrawn {
                dock_band: Some(Rectangle::new(
                    (0, shorter - DOCK).into(),
                    (VIEWPORT.0, DOCK).into(),
                )),
                panel_reserved: Rectangle::default(),
                status_area: None,
            },
            alo_appearance::TextScale::ordinary(),
        );
    });

    assert!(
        f.backend(|s| s.bring_back_frames_the_moved_controls_hide())
            .is_some(),
        "the display changed and the Dock's band moved with it, and no recheck \
         was owed — so a frame the new band covers stays under it"
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
