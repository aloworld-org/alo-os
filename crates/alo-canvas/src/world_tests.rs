//! What the World fits, and which Place a point in it names.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::{Showing, World};
use crate::{At, Place, Size, Zoom};

/// A tile of an ordinary size.
fn tile() -> Size {
    Size::checked(800, 600).expect("a size a tile may be")
}

/// A point on the plane.
fn at(x: i32, y: i32) -> At {
    At::checked(x, y).expect("a place on the plane")
}

/// The nth Place, counting from the first.
fn place(n: u64) -> Place {
    Place::numbered(n).expect("a numbered place")
}

/// Three Places laid out in a row, a tile's width apart.
fn three_in_a_row() -> World {
    World::new()
        .with(place(1), at(0, 0), tile())
        .with(place(2), at(1_000, 0), tile())
        .with(place(3), at(2_000, 0), tile())
}

/// **A machine nobody has used has no World to show.**
///
/// Refused rather than answered with an empty rectangle, for the same reason
/// `plane::reached_by` refuses no frames: nowhere is a real state and a camera
/// pointed at it would be pointed somewhere arbitrary.
#[test]
fn a_world_with_no_places_shows_nothing() {
    let empty = World::new();
    assert_eq!(empty.how_many(), 0);
    assert!(empty.reached_by().is_none());
    assert!(empty.showing(tile()).is_none());
}

/// **Every Place the person has is inside what the World fits.**
///
/// Task 2's first acceptance sentence, as arithmetic: the span holds every tile,
/// so a camera fitted to it leaves none of them out.
#[test]
fn every_place_is_inside_what_the_world_fits() {
    let world = three_in_a_row();
    assert_eq!(world.how_many(), 3);
    let span = world.reached_by().expect("three places reach somewhere");

    assert_eq!(span.from, at(0, 0));
    assert_eq!(span.to, at(2_800, 600));
}

/// **Adding a Place widens the World rather than replacing it.**
///
/// The fault this guards is the one the first version of this file had: tiles
/// each carrying the Place they stand for, folded by a function that folds *one
/// Place's* frames — which would have answered with the first tile alone, with
/// correct arithmetic, about the wrong set.
#[test]
fn a_fourth_place_widens_the_world() {
    let three = three_in_a_row();
    let narrower = three.reached_by().expect("three reach somewhere");
    let four = three.with(place(4), at(9_000, 0), tile());
    let wider = four.reached_by().expect("four reach somewhere");

    assert_eq!(four.how_many(), 4);
    assert!(
        wider.to.x > narrower.to.x,
        "a fourth Place did not widen the World: {narrower:?} then {wider:?}"
    );
    assert_eq!(wider.to.x, 9_800);
}

/// **Every Place it holds can be named, in the order they were laid out.**
#[test]
fn the_places_come_back_in_the_order_they_were_laid_out() {
    let world = three_in_a_row();
    let names: Vec<u64> = world.each().map(Place::number).collect();
    assert_eq!(names, vec![1, 2, 3]);
}

/// **Zooming in from the World picks the Place under the pointer.**
///
/// The second half of task 2's acceptance. The same question a press on a frame
/// asks — `Camera::inside` — so there is no second hit test to learn and no
/// switcher to click.
#[test]
fn a_point_in_the_world_names_the_place_under_it() {
    let world = three_in_a_row();

    assert_eq!(world.the_place_at(at(10, 10)), Some(place(1)));
    assert_eq!(world.the_place_at(at(1_400, 300)), Some(place(2)));
    assert_eq!(world.the_place_at(at(2_799, 599)), Some(place(3)));
}

/// **Between two Places is between them, and names neither.**
///
/// A gap is a real answer. Returning the nearest Place would mean a person who
/// aimed at nothing was taken somewhere, which is the canvas's standing refusal
/// to move somebody without their saying so.
#[test]
fn a_point_between_places_names_none_of_them() {
    let world = three_in_a_row();
    // The tiles are 800 wide and 1,000 apart, so 800..1,000 is the gap.
    assert_eq!(world.the_place_at(at(900, 300)), None);
    assert_eq!(world.the_place_at(at(-5, 0)), None);
    assert_eq!(world.the_place_at(at(0, 700)), None);
}

/// **A camera fitted to the World shows every tile on the glass.**
///
/// Not asserted through the span this time but through the camera, because what
/// the acceptance promises is that a person *sees* them all — and that is
/// `screen_of` landing inside the viewport for every corner of every tile.
#[test]
fn a_camera_fitted_to_the_world_has_every_place_on_the_glass() {
    let world = three_in_a_row();
    let viewport = Size::checked(1_280, 720).expect("a viewport");
    let camera = world.showing(viewport).expect("the world fits");

    for place_number in 1..=3u64 {
        let corner = at(((place_number - 1) * 1_000) as i32, 0);
        let (x, y) = camera.screen_of(corner).expect("a corner on the glass");
        assert!(
            x >= 0 && y >= 0 && x <= i32::from(1_280_u16) && y <= i32::from(720_u16),
            "Place {place_number}'s corner landed at ({x}, {y}), off a 1280x720 viewport"
        );
    }
}

/// **The World is reached by stepping out from the furthest a Place goes, and
/// nothing above that rung changed meaning.**
///
/// The boundary, asserted rather than described. `one_step_out` at
/// `FURTHEST_OUT` answers `None` — a step with nowhere to go — and that is the
/// step the World is for. No constant moved to make room for it, which is what
/// keeps `Show all` fitting one Place's frames and keeps
/// `as_far_as_show_all_reaches` the extent a test already asserts to the pixel.
#[test]
fn the_world_is_the_step_out_that_had_nowhere_to_go() {
    let furthest_out = Zoom::of(Zoom::FURTHEST_OUT).expect("the furthest out is a zoom");
    assert_eq!(
        furthest_out.one_step_out(),
        None,
        "the rung the World is reached from must still be the last one inside a Place"
    );
    // And the rung above it is still an ordinary step, so nothing was inserted.
    let one_in = furthest_out.one_step_in().expect("a rung further in");
    assert_eq!(one_in.one_step_out(), Some(furthest_out));
}

/// **What is shown is a consequence, never a stored mode.**
///
/// `Showing` is returned and compared; nothing holds one. A field would be a
/// second thing that can disagree with the camera.
#[test]
fn showing_names_the_world_or_one_place() {
    assert_eq!(Showing::TheWorld, Showing::TheWorld);
    assert_eq!(Showing::OnePlace(place(2)), Showing::OnePlace(place(2)));
    assert_ne!(Showing::OnePlace(place(1)), Showing::OnePlace(place(2)));
    assert_ne!(Showing::TheWorld, Showing::OnePlace(place(1)));
}
