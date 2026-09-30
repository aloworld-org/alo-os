//! What the plane refuses, and what *Show all* would have to fit.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;

/// A frame of an ordinary size, on the Place a machine starts on.
fn a_frame(id: u64, x: i32, y: i32) -> Frame {
    on_a_place(id, Place::FIRST, x, y)
}

/// The same, on whichever Place is named.
fn on_a_place(id: u64, place: Place, x: i32, y: i32) -> Frame {
    Frame::of(
        id,
        place,
        At::checked(x, y).unwrap(),
        Size::checked(800, 600).unwrap(),
    )
}

/// The second Place, for the tests that need two.
fn the_second_place() -> Place {
    Place::FIRST.next().unwrap()
}

/// **A plane has no corner: a frame may sit left of and above the origin.**
///
/// This is what makes it a surface rather than a page. A person who drags the
/// canvas right puts their frames at negative coordinates, and if that were
/// refused the drag would stop for a reason nobody could see.
#[test]
fn a_frame_may_sit_left_of_and_above_the_origin() {
    let at = At::checked(-4_000, -2_500).unwrap();
    assert_eq!(at.x, -4_000);
    assert_eq!(at.y, -2_500);
}

/// **Past the furthest is refused by name, not clamped.**
///
/// A frame quietly moved to a wall is a frame somebody put somewhere and found
/// somewhere else.
#[test]
fn past_the_furthest_is_refused_rather_than_clamped() {
    assert!(At::checked(FURTHEST, FURTHEST).is_some());
    assert!(At::checked(FURTHEST + 1, 0).is_none());
    assert!(At::checked(0, -FURTHEST - 1).is_none());
}

/// **A size is never zero along either axis.**
#[test]
fn a_frame_with_no_width_is_not_a_frame() {
    assert!(Size::checked(800, 600).is_some());
    assert!(Size::checked(0, 600).is_none());
    assert!(Size::checked(800, 0).is_none());
}

/// **What *Show all* fits is the frames' own reach, computed from them.**
///
/// The plan's task 6 constraint in arithmetic: there is no declared extent here
/// to drift from what is open, so a frame dragged far away is inside the answer
/// by construction rather than by somebody remembering to widen a field.
#[test]
fn what_show_all_fits_is_computed_from_the_frames_themselves() {
    let span = reached_by(&[a_frame(1, 0, 0), a_frame(2, 3_000, -1_000)], Place::FIRST).unwrap();

    assert_eq!(span.from, At::checked(0, -1_000).unwrap());
    assert_eq!(span.to, At::checked(3_800, 600).unwrap());
}

/// **A frame dragged far away is still inside what *Show all* fits.**
///
/// Task 8 is *a frame is never lost*, and this is the half of it that is
/// arithmetic: the reach grows to hold whatever is open, however far it went.
#[test]
fn a_frame_dragged_far_away_is_still_inside_the_reach() {
    let far = a_frame(2, 90_000, 70_000);
    let span = reached_by(&[a_frame(1, 0, 0), far], Place::FIRST).unwrap();

    assert!(span.to.x >= far.opposite().unwrap().x);
    assert!(span.to.y >= far.opposite().unwrap().y);
}

/// **No frames reach nowhere**, which is a real state and not an empty rectangle.
#[test]
fn nothing_open_reaches_nowhere() {
    assert!(reached_by(&[], Place::FIRST).is_none());
}

/// **Two windows at the same point on different Places are two windows.**
///
/// The acceptance condition of `the-canvas-and-its-places.md` task 1, and the
/// one it says must be *asserted by a test that would pass today and must not*.
/// Before this change the two frames below were equal in every respect a type
/// could see — same point, same size — and `(4200, 0)` meant one location
/// because there was only one surface for it to be on.
///
/// This is not a hypothetical about a feature nobody has yet. `alo-put-aside`'s
/// `restoring_a_window_the_view_already_shows_needs_no_travel` takes a branch on
/// exactly this comparison: a window whose rectangle overlaps the current view
/// is restored *without travelling*. On a second Place that is a window restored
/// onto the wrong surface, silently, and no assertion in the tree could have
/// caught it.
#[test]
fn two_frames_at_one_point_on_two_places_are_told_apart() {
    let here = on_a_place(1, Place::FIRST, 4_200, 0);
    let there = on_a_place(2, the_second_place(), 4_200, 0);

    assert_eq!(here.at(), there.at(), "the point is deliberately the same");
    assert_eq!(here.size(), there.size(), "and so is the size");
    assert_ne!(
        here.place(),
        there.place(),
        "a patch alone was never an answer to where a window is"
    );
    assert_ne!(here, there);
}

/// **A frame that is not on this Place is not in what *Show all* fits.**
///
/// The failure this would otherwise have been is a quiet one: the arithmetic
/// still works, so nothing refuses. *Show all* simply zooms out far enough to
/// include a window on a surface the person is not looking at, and the window
/// they asked to see becomes a speck.
#[test]
fn show_all_fits_this_place_and_not_the_other_one() {
    let mine = a_frame(1, 0, 0);
    let elsewhere = on_a_place(2, the_second_place(), 90_000, 70_000);
    let both = [mine, elsewhere];

    let here = reached_by(&both, Place::FIRST).unwrap();
    assert_eq!(here.from, mine.at());
    assert_eq!(here.to, mine.opposite().unwrap());

    let there = reached_by(&both, the_second_place()).unwrap();
    assert_eq!(there.from, elsewhere.at());
}

/// **A Place with nothing on it reaches nowhere**, even while another has frames.
#[test]
fn an_empty_place_reaches_nowhere_though_the_canvas_is_not_empty() {
    let somewhere_else = [a_frame(1, 0, 0), a_frame(2, 500, 500)];
    assert!(reached_by(&somewhere_else, the_second_place()).is_none());
}

/// **Moving a frame to another Place changes nothing about its geometry.**
///
/// Task 3's *the work goes with it*: a window dragged through the World into
/// another surface is the same window, at the same point, the same size. What
/// changed is which plane those coordinates are on.
#[test]
fn a_frame_moved_to_another_place_keeps_its_point_and_its_size() {
    let before = a_frame(7, 1_200, -300);
    let after = before.now_on(the_second_place());

    assert_eq!(after.id(), before.id());
    assert_eq!(after.at(), before.at());
    assert_eq!(after.size(), before.size());
    assert_eq!(after.place(), the_second_place());
    assert_ne!(after.place(), before.place());
}
