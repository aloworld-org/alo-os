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
