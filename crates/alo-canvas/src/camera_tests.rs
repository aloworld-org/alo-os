//! What the camera answers, at the zooms the plan names.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;
use crate::plane::{At, Frame, Size};

/// 40 %, which task 2 names.
fn far_out() -> Zoom {
    Zoom::of(400).unwrap()
}

/// 250 %, which task 2 names.
fn far_in() -> Zoom {
    Zoom::of(2500).unwrap()
}

/// **A point on the plane lands where the arithmetic says, at every zoom.**
///
/// Integer thousandths, so these are exact numbers rather than nearly-right ones.
#[test]
fn a_plane_point_lands_where_the_arithmetic_says() {
    let camera = Camera::new();
    assert_eq!(
        camera.screen_of(At::checked(100, 50).unwrap()),
        Some((100, 50))
    );

    let out = camera.zoomed_to(far_out(), (0, 0)).unwrap();
    assert_eq!(out.screen_of(At::checked(100, 50).unwrap()), Some((40, 20)));

    let in_close = camera.zoomed_to(far_in(), (0, 0)).unwrap();
    assert_eq!(
        in_close.screen_of(At::checked(100, 50).unwrap()),
        Some((250, 125))
    );
}

/// **The point under the pointer does not move when a person zooms.**
///
/// Task 6's acceptance, and the reason zooming takes the point to hold rather
/// than a corner. Checked at both of task 2's zooms and at two pointer positions,
/// because a transform that holds the origin still is not the same as one that
/// holds a pointer still.
#[test]
fn the_point_under_the_pointer_is_unchanged_by_a_zoom() {
    for held in [(0, 0), (640, 360), (1279, 719)] {
        for zoom in [far_out(), far_in(), Zoom::LIFE_SIZE] {
            let camera = Camera::new().panned_by(-300, -200).unwrap();
            let under = camera.plane_of(held).unwrap();

            let zoomed = camera.zoomed_to(zoom, held).unwrap();

            assert_eq!(
                zoomed.plane_of(held),
                Some(under),
                "zooming to {zoom:?} about {held:?} moved what was under the pointer"
            );
        }
    }
}

/// **A pan moves the plane by the pointer's own distance, in plane units.**
///
/// Task 3's arithmetic: 100 screen pixels at 40 % is 250 plane units, because a
/// person's hand moves in pixels and the plane moves underneath it.
#[test]
fn a_pan_moves_the_plane_by_the_pointers_distance_in_plane_units() {
    let out = Camera::new().zoomed_to(far_out(), (0, 0)).unwrap();
    let panned = out.panned_by(100, 0).unwrap();
    assert_eq!(panned.at().x - out.at().x, 250);

    let in_close = Camera::new().zoomed_to(far_in(), (0, 0)).unwrap();
    let panned = in_close.panned_by(100, 0).unwrap();
    assert_eq!(panned.at().x - in_close.at().x, 40);
}

/// **A press arrives at the right coordinate inside the frame, at both zooms.**
///
/// The other half of task 2, held here because it is the same transform. A
/// coordinate inside a frame is the application's own, so it is unsigned and it
/// is refused outright for a point that is not in that frame — a clamped one
/// would tell an application somebody clicked its edge when they clicked its
/// neighbour.
#[test]
fn a_press_arrives_at_the_right_place_inside_a_frame() {
    let frame = Frame::of(
        1,
        crate::Place::FIRST,
        At::checked(500, 400).unwrap(),
        Size::checked(800, 600).unwrap(),
    );
    assert_eq!(
        Camera::inside(frame, At::checked(500, 400).unwrap()),
        Some((0, 0))
    );
    assert_eq!(
        Camera::inside(frame, At::checked(900, 700).unwrap()),
        Some((400, 300))
    );
    assert_eq!(Camera::inside(frame, At::checked(499, 400).unwrap()), None);
    assert_eq!(Camera::inside(frame, At::checked(1300, 400).unwrap()), None);
}

/// **A screen press reaches the same place inside a frame however far a person
/// has zoomed or panned.**
///
/// The whole of task 2 in one clause: three zooms, two pan offsets, one frame,
/// and the surface coordinate is the same every time.
#[test]
fn a_screen_press_reaches_the_same_place_at_every_zoom_and_pan() {
    let frame = Frame::of(
        1,
        crate::Place::FIRST,
        At::checked(1_000, 800).unwrap(),
        Size::checked(800, 600).unwrap(),
    );
    // A point a third of the way into the frame, chosen so that every zoom below
    // divides it exactly: what is being held is the transform, not a rounding.
    let wanted = At::checked(1_400, 1_000).unwrap();
    let expected = Camera::inside(frame, wanted).unwrap();

    for zoom in [Zoom::LIFE_SIZE, far_out(), far_in()] {
        for pan in [(0, 0), (-250, -150)] {
            let camera = Camera::new()
                .looking_at(At::checked(pan.0, pan.1).unwrap())
                .unwrap()
                .zoomed_to(zoom, (0, 0))
                .unwrap();
            let screen = camera.screen_of(wanted).unwrap();

            let landed = camera.plane_of(screen).unwrap();

            assert_eq!(
                Camera::inside(frame, landed),
                Some(expected),
                "at {zoom:?} panned to {pan:?}, a press at {screen:?} reached the \
                 wrong place inside the frame"
            );
        }
    }
}

/// **A zoom outside the canvas's range is refused by name.**
#[test]
fn a_zoom_further_than_the_canvas_goes_is_refused_by_name() {
    assert!(matches!(
        Zoom::of(10),
        Err(NotZoomed::TooFarOut { thousandths: 10 })
    ));
    assert!(matches!(
        Zoom::of(99_000),
        Err(NotZoomed::TooFarIn {
            thousandths: 99_000
        })
    ));
    assert!(Zoom::of(Zoom::FURTHEST_OUT).is_ok());
    assert!(Zoom::of(Zoom::FURTHEST_IN).is_ok());
}

/// **Zooming out shows more of the plane**, which is the sentence the canvas has
/// to earn.
#[test]
fn zooming_out_shows_more_of_the_plane() {
    let viewport = Size::checked(1366, 768).unwrap();
    let life = Camera::new().sees(viewport).unwrap();
    let out = Camera::new()
        .zoomed_to(far_out(), (0, 0))
        .unwrap()
        .sees(viewport)
        .unwrap();

    assert!(out.width() > life.width() && out.height() > life.height());
    assert_eq!(out.width(), 3415);
}

/// The frames of one arrangement, as *Show all* is handed them.
fn frames(placed: &[(i32, i32, u32, u32)]) -> Vec<Frame> {
    placed
        .iter()
        .enumerate()
        .map(|(id, (x, y, width, height))| {
            Frame::of(
                u64::try_from(id).unwrap(),
                crate::Place::FIRST,
                At::checked(*x, *y).unwrap(),
                Size::checked(*width, *height).unwrap(),
            )
        })
        .collect()
}

/// **The ladder is ordered, and it ends where the type does.**
///
/// A rung out of order would make a step go backwards, and a ladder that stopped
/// short of the bounds would leave zooms a person can set but not step to.
#[test]
fn the_ladder_is_ordered_and_reaches_both_bounds() {
    let mut rungs = Zoom::STOPS.into_iter();
    let mut last = rungs.next().unwrap();
    assert_eq!(last, Zoom::FURTHEST_OUT);
    for rung in rungs {
        assert!(rung > last, "the ladder goes backwards at {rung}");
        assert!(
            Zoom::of(rung).is_ok(),
            "{rung} is not a zoom this type allows"
        );
        last = rung;
    }
    assert_eq!(last, Zoom::FURTHEST_IN);
    assert!(
        Zoom::STOPS.contains(&Zoom::LIFE_SIZE.thousandths()),
        "life size is not a rung, so returning to 100 % is a jump rather than a step back"
    );
}

/// **A step in and a step out come back to exactly where they started.**
///
/// This is the whole argument for a ladder over a multiplier: in integer
/// thousandths a repeated multiply drifts, and a canvas that cannot return to
/// life size is one somebody stops trusting.
#[test]
fn a_step_in_and_a_step_out_come_back() {
    for rung in Zoom::STOPS {
        let zoom = Zoom::of(rung).unwrap();
        if let Some(further_in) = zoom.one_step_in() {
            assert_eq!(
                further_in.one_step_out(),
                Some(zoom),
                "stepping in from {rung} and out again did not come back"
            );
        }
        if let Some(further_out) = zoom.one_step_out() {
            assert_eq!(
                further_out.one_step_in(),
                Some(zoom),
                "stepping out from {rung} and in again did not come back"
            );
        }
    }
}

/// **The ends have nowhere further to go, and say so rather than staying put
/// silently.**
#[test]
fn the_ends_of_the_ladder_refuse_to_step_past_themselves() {
    assert_eq!(Zoom::of(Zoom::FURTHEST_IN).unwrap().one_step_in(), None);
    assert_eq!(Zoom::of(Zoom::FURTHEST_OUT).unwrap().one_step_out(), None);
    assert!(
        Zoom::of(Zoom::FURTHEST_IN)
            .unwrap()
            .one_step_out()
            .is_some()
    );
    assert!(
        Zoom::of(Zoom::FURTHEST_OUT)
            .unwrap()
            .one_step_in()
            .is_some()
    );
}

/// **A zoom between rungs steps to the rung beyond it, not backwards.**
///
/// *Show all* leaves the camera between rungs, because the fit is whatever fits;
/// the first press afterwards must carry on in the direction it was pressed.
#[test]
fn a_zoom_between_rungs_steps_past_itself() {
    let between = Zoom::of(1200).unwrap();
    assert_eq!(between.one_step_in(), Some(Zoom::of(1500).unwrap()));
    assert_eq!(between.one_step_out(), Some(Zoom::LIFE_SIZE));
}

/// **Show all leaves every frame inside the viewport, with none clipped.**
///
/// Task 6's acceptance, held against the frames' own extent — `reached_by` — and
/// not against any declared bound. Every corner of every frame is asked where it
/// lands and has to be on the screen, at arrangements that are wide, tall,
/// scattered across the origin and nearly a point.
#[test]
fn show_all_leaves_every_frame_inside_the_viewport() {
    let viewport = Size::checked(1280, 720).unwrap();
    let (room_across, room_down) = (1280, 720);
    for arrangement in [
        vec![(0, 0, 800, 600)],
        vec![(0, 0, 800, 600), (2000, 1500, 640, 480)],
        vec![(-5000, -4000, 300, 200), (9000, 7000, 1920, 1080)],
        // Wider than it is tall by a factor of thousands, and the other way.
        vec![(0, 0, 25_000, 10)],
        vec![(0, 0, 10, 14_000)],
        // Two frames almost on top of each other: the fit is capped by how far
        // in the canvas goes, not by the span.
        vec![(37, -91, 1, 1), (38, -90, 1, 1)],
    ] {
        let placed = frames(&arrangement);
        let span = crate::plane::reached_by(&placed, crate::Place::FIRST).unwrap();
        let camera = Camera::showing(span, viewport).unwrap();
        for frame in &placed {
            let (left, top) = camera.screen_of(frame.at()).unwrap();
            let (right, bottom) = camera.screen_of(frame.opposite().unwrap()).unwrap();
            assert!(
                left >= 0 && top >= 0,
                "{arrangement:?}: a frame starts off the top-left at ({left}, {top})"
            );
            assert!(
                right <= room_across && bottom <= room_down,
                "{arrangement:?}: a frame is clipped, ending at ({right}, {bottom})"
            );
        }
    }
}

/// **What Show all shows is centred**, to within what integer halving costs.
///
/// The tolerance is derived rather than chosen. Centring halves a count of whole
/// **plane units**, so it can be off by one of those — and a plane unit is
/// `zoom / 1000` pixels, which above life size is more than a pixel. Adding one
/// for the screen conversion's own truncation gives the bound below. Asserting a
/// single pixel here failed at 159 against 161 on a 1200-thousandths fit, which is
/// the arithmetic behaving exactly as it says it does.
#[test]
fn show_all_centres_what_it_shows() {
    let viewport = Size::checked(1280, 720).unwrap();
    let placed = frames(&[(400, 300, 800, 600)]);
    let span = crate::plane::reached_by(&placed, crate::Place::FIRST).unwrap();
    let camera = Camera::showing(span, viewport).unwrap();
    let a_unit = i32::try_from(camera.zoom().thousandths().div_ceil(1000)).unwrap();
    let slack = a_unit + 1;

    let (left, top) = camera.screen_of(span.from).unwrap();
    let (right, bottom) = camera.screen_of(span.to).unwrap();
    assert!(
        (left - (1280 - right)).abs() <= slack,
        "left margin {left} and right margin {} differ by more than {slack}",
        1280 - right
    );
    assert!(
        (top - (720 - bottom)).abs() <= slack,
        "top margin {top} and bottom margin {} differ by more than {slack}",
        720 - bottom
    );
    // This arrangement is 4:3 in a 16:9 viewport, so height is what limits the
    // fit and the spare room is at the sides. Margins of zero on both axes would
    // satisfy the symmetry above while proving nothing.
    assert!(
        left > 0,
        "a fit limited by height left no room at the sides"
    );
}

/// **Frames spread further than the canvas goes are refused, not shown clipped.**
///
/// *Show all* makes exactly one promise. Breaking it silently is worse than
/// saying it cannot be kept.
#[test]
fn show_all_refuses_frames_it_cannot_fit() {
    let viewport = Size::checked(1280, 720).unwrap();
    let placed = frames(&[(0, 0, 10, 10), (900_000, 0, 10, 10)]);
    let span = crate::plane::reached_by(&placed, crate::Place::FIRST).unwrap();
    assert_eq!(
        Camera::showing(span, viewport),
        None,
        "a span no zoom this canvas has can hold was fitted anyway"
    );
}
