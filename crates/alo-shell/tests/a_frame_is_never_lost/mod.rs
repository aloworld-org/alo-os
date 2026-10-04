//! **A frame is never lost** — task 8 of the canvas plan.
//!
//! The acceptance names two tests and they are the two here: a frame dragged far
//! away is still found by *Show all*, and a frame cannot be left entirely under
//! the dock.
//!
//! # What the numbers made clear
//!
//! *Show all* fits the frames' own extent, bounded by `Zoom::FURTHEST_OUT`: on
//! this 1280×720 output, 25,600 × 14,400 plane units. The plane reaches
//! ±1,000,000, seventy-eight times further across. So *a frame dragged far away is
//! still found by Show all* is not something the canvas had — it is something a
//! rule has to keep, and these tests are what hold it.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use super::support::{Application, Fixture};
use alo_canvas::At;
use smithay::utils::Rectangle;

/// The viewport these tests render to.
const VIEWPORT: (i32, i32) = (1280, 720);

/// A dock along the bottom, as the desktop draws one: its own band, in pixels.
fn a_dock_along_the_bottom() -> Rectangle<i32, smithay::utils::Physical> {
    Rectangle::new((0, VIEWPORT.1 - 64).into(), (VIEWPORT.0, 64).into())
}

/// A real display with a pointer and an output.
fn fixture() -> Fixture {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    f.render(VIEWPORT, false, 1).expect("an output to look at");
    f
}

/// Complete the real XDG handshake and attach a buffer. **Kept alive**: dropping
/// an `Application` disconnects its client, which unmaps the frame.
fn mapped(f: &Fixture) -> Application {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    app
}

/// **How far *Show all* reaches is the output's own size, not the plane's.**
///
/// The measurement this whole task turns on, asserted rather than left in a
/// comment: a 1280×720 output can be shown 25,600 × 14,400 plane units at once.
#[test]
fn show_all_reaches_the_output_divided_by_the_furthest_out() {
    let f = fixture();
    let reach = f
        .backend(|s| s.as_far_as_show_all_reaches())
        .expect("an output to fit into");
    assert_eq!((reach.width(), reach.height()), (25_600, 14_400));
}

/// **A frame dragged far away is still found by *Show all*.**
///
/// The first acceptance sentence. The drag stops at the edge of what the fit can
/// hold, so *Show all* still gathers every frame with none clipped — rather than
/// the frame going where it likes and the fit silently refusing afterwards.
#[test]
fn a_frame_dragged_far_away_is_still_found_by_show_all() {
    let f = fixture();
    let _near = mapped(&f);
    let _far = mapped(&f);
    let frame = f
        .backend(|s| s.mapped_surfaces().last().cloned())
        .expect("two frames are mapped");

    let dock = a_dock_along_the_bottom();
    let here = At::checked(0, 0).expect("the origin is on the plane");
    let miles = At::checked(500_000, 500_000).expect("a place on the plane");

    let stopped = {
        let frame = frame.clone();
        f.backend(move |s| s.as_far_as_a_frame_may_be_dragged(&frame, here, miles, dock))
    };
    assert_eq!(
        stopped, here,
        "a frame was allowed to be dragged beyond where Show all could bring it back"
    );

    // A move well inside the reach is allowed, so this is a limit and not a wall.
    let along = At::checked(4_000, 2_000).expect("a place on the plane");
    let allowed = {
        let frame = frame.clone();
        f.backend(move |s| s.as_far_as_a_frame_may_be_dragged(&frame, here, along, dock))
    };
    assert_eq!(allowed, along, "an ordinary drag was refused");

    // And having moved there, Show all really does gather them unclipped.
    {
        let frame = frame.clone();
        assert!(f.backend(move |s| s.place_window(&frame, (4_000, 2_000)).is_ok()));
    }
    assert!(
        f.backend(|s| s.show_all_on_the_canvas()).is_some(),
        "Show all refused a spread the rule says it can hold"
    );
    let clipped = f.backend(|s| {
        s.the_frames_on_the_plane()
            .into_iter()
            .filter(|frame| {
                let inside = || {
                    let (left, top) = s.the_camera().screen_of(frame.at())?;
                    let (right, bottom) = s.the_camera().screen_of(frame.opposite()?)?;
                    Some(left >= 0 && top >= 0 && right <= VIEWPORT.0 && bottom <= VIEWPORT.1)
                };
                inside() != Some(true)
            })
            .count()
    });
    assert_eq!(clipped, 0, "Show all left a frame outside the viewport");
}

/// **A frame cannot be dragged entirely under the dock.**
///
/// The second acceptance sentence. ADR 0071 makes the name the only handle, so a
/// name entirely under the dock is a frame nothing can pick up. A name half under
/// it is still one a pointer can land on, which is why *entirely* is the
/// condition and why the second half of this test matters as much as the first.
#[test]
fn a_frame_cannot_be_dragged_entirely_under_the_dock() {
    let f = fixture();
    let _app = mapped(&f);
    let frame = f
        .backend(|s| s.mapped_surfaces().next().cloned())
        .expect("a frame is mapped");

    let dock = a_dock_along_the_bottom();
    // The frame starts with its name band just above the viewport's top edge, so
    // these moves are measured from a place the dock is nowhere near.
    let here = At::checked(0, 0).expect("the origin is on the plane");

    // The band sits *above* the frame — 48 pixels of it — so for the whole band
    // to land inside a dock band spanning 656..720 the frame must be at 704 or
    // below. 712 puts the band at 664..712, entirely inside it.
    let under = At::checked(0, 712).expect("a place on the plane");
    {
        let frame = frame.clone();
        assert!(
            f.backend(move |s| s.the_name_would_be_under(&frame, here, under, dock)),
            "a name that would land inside the dock was not reported as under it"
        );
    }
    {
        let frame = frame.clone();
        assert_eq!(
            f.backend(move |s| s.as_far_as_a_frame_may_be_dragged(&frame, here, under, dock)),
            here,
            "a frame was allowed to be dragged entirely under the dock"
        );
    }

    // Half under it is still reachable, and still allowed.
    // 680 puts the band at 632..680: its lower half is over the dock and its top
    // is not, which is the case a pointer can still reach.
    let half = At::checked(0, 680).expect("a place on the plane");
    {
        let frame = frame.clone();
        assert!(
            !f.backend(move |s| s.the_name_would_be_under(&frame, here, half, dock)),
            "a name only half under the dock was reported as under it"
        );
    }
    let frame = frame.clone();
    assert_eq!(
        f.backend(move |s| s.as_far_as_a_frame_may_be_dragged(&frame, here, half, dock)),
        half,
        "a drag leaving the name half reachable was refused"
    );
}

/// Where a frame's name band sits on the glass: `(left, top, width, height)`.
///
/// Computed from the two public numbers rather than reached for through a
/// test-only accessor — `alo_shell::the_names_band` and
/// `alo_shell::window_buffer_origin` — which is also what keeps this test honest
/// about aiming where a person would. The fixture's windows carry a 16x16 buffer
/// and the camera is at the origin at life size, so the band is directly above the
/// frame and as wide as it.
fn the_name_band(
    f: &Fixture,
    frame: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
) -> (f64, f64, f64, f64) {
    let at = {
        let frame = frame.clone();
        f.backend(move |_| alo_shell::window_buffer_origin(&frame))
    };
    let band = alo_shell::the_names_band();
    (at.x, at.y - band, 16.0, band)
}

/// The status area, where the owner put it: **fixed at the top-right**.
///
/// A viewport control like the dock, so it is a rectangle in screen pixels here
/// rather than anything this test computes from the plane.
///
/// **This one really is the status area, and it is the only fixture in the crate
/// that is.** The clock, battery, network and volume, at the owner's position of
/// 2026-09-30 — a surface **nothing draws yet**, modelled here because task 8's
/// acceptance names it and the rule can be held to it before the pixels exist.
/// `alo_shell::canvas_fixed_controls::FixedControlsDrawn::what_is_leaving` is the
/// field that was called `status_area` until 2026-10-04 and never carried this: it
/// is handed the **egress indicator's** band, at the far end of the Dock. One word
/// over two surfaces is what put a reconciliation that did not exist into
/// ADR 0086.
fn a_status_area_at_the_top_right() -> Rectangle<i32, smithay::utils::Physical> {
    Rectangle::new((VIEWPORT.0 - 200, 0).into(), (200, 32).into())
}

/// A control covering this frame's name wherever it is, for total occlusion.
///
/// **Not the viewport**, which was the first attempt and does not cover it: a
/// frame at the plane origin has its name band at `y = -48`, above the top of the
/// screen, so a rectangle over the visible area misses it entirely. That is not a
/// fault in the rule — a name above the viewport is reached by panning, which is
/// why this task's own reasoning counts only the fit and the fixed controls as
/// ways a frame is lost.
fn over_the_whole_name(name: (f64, f64, f64, f64)) -> Rectangle<i32, smithay::utils::Physical> {
    Rectangle::new(
        ((name.0 - 10.0) as i32, (name.1 - 10.0) as i32).into(),
        (name.2 as i32 + 20, name.3 as i32 + 20).into(),
    )
}

/// **A person who made everything larger made this larger too.**
///
/// A fixed 44 × 24 would shrink against everything around it for exactly the
/// person who most needs it not to. Asserted against `TextScale`'s own ordinary
/// value rather than against 100, so the two cannot drift.
#[test]
fn the_handle_grows_with_the_text_scale() {
    let ordinary = alo_shell::Server::a_usable_handle_at(alo_appearance::TextScale::ordinary());
    let larger = alo_shell::Server::a_usable_handle_at(
        alo_appearance::TextScale::percent(200).expect("two hundred per cent is offered"),
    );
    assert_eq!(ordinary, (44.0, 24.0));
    assert_eq!(
        larger,
        (88.0, 48.0),
        "at twice the text size the handle is twice the size"
    );
    assert!(
        larger.0 > ordinary.0 && larger.1 > ordinary.1,
        "a larger text scale did not make the handle larger"
    );
}

/// **A frame with no reachable name at all is refused**, which is the case the
/// whole rule exists for and the one the dock test already held for one control.
#[test]
fn a_name_wholly_covered_by_the_controls_is_not_reachable() {
    let f = fixture();
    let _app = mapped(&f);
    let frame = f
        .backend(|s| s.mapped_surfaces().next().cloned())
        .expect("a frame is mapped");
    let here = At::checked(0, 0).expect("the origin is on the plane");
    let handle = alo_shell::Server::a_usable_handle_at(alo_appearance::TextScale::ordinary());
    let name = the_name_band(&f, &frame);
    let covered = f.backend(move |s| {
        s.enough_of_the_name_is_reachable(&frame, here, here, &[over_the_whole_name(name)], handle)
    });
    assert!(
        !covered,
        "a name entirely under a fixed control was called reachable"
    );
}

/// **The status area is a control like any other**, now it has a position.
///
/// ADR 0076 left it with nowhere to be and the owner placed it at the top-right on
/// 2026-09-30. This is the half of task 8's acceptance that could not be written
/// until then — *a frame cannot be left entirely under the dock **or the status
/// area***, with *entirely* replaced by the usable-handle rule.
#[test]
fn the_status_area_hides_a_name_the_same_way_the_dock_does() {
    let f = fixture();
    let _app = mapped(&f);
    let frame = f
        .backend(|s| s.mapped_surfaces().next().cloned())
        .expect("a frame is mapped");
    let here = At::checked(0, 0).expect("the origin is on the plane");
    let handle = alo_shell::Server::a_usable_handle_at(alo_appearance::TextScale::ordinary());
    let name = the_name_band(&f, &frame);

    // The status area moved onto this frame's name, leaving nothing of it.
    let over_the_name = Rectangle::new(
        ((name.0 - 10.0) as i32, name.1 as i32).into(),
        (name.2 as i32 + 20, name.3 as i32 + 2).into(),
    );
    let frame_a = frame.clone();
    assert!(
        !f.backend(move |s| s.enough_of_the_name_is_reachable(
            &frame_a,
            here,
            here,
            &[over_the_name],
            handle
        )),
        "a name entirely under the status area was called reachable"
    );

    // And where it actually sits — top-right, clear of a frame at the origin — it
    // takes nothing, so the rule is about the geometry rather than about the
    // control's existence.
    assert!(
        f.backend(move |s| s.enough_of_the_name_is_reachable(
            &frame,
            here,
            here,
            &[a_status_area_at_the_top_right()],
            handle
        )),
        "the status area at the top-right hid a frame at the origin"
    );
}
