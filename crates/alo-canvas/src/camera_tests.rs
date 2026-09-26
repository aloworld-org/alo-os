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
