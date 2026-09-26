//! What the plane refuses, and what *Show all* would have to fit.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;

/// A frame of an ordinary size.
fn a_frame(id: u64, x: i32, y: i32) -> Frame {
    Frame::of(
        id,
        At::checked(x, y).unwrap(),
        Size::checked(800, 600).unwrap(),
    )
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
    let span = reached_by(&[a_frame(1, 0, 0), a_frame(2, 3_000, -1_000)]).unwrap();

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
    let span = reached_by(&[a_frame(1, 0, 0), far]).unwrap();

    assert!(span.to.x >= far.opposite().unwrap().x);
    assert!(span.to.y >= far.opposite().unwrap().y);
}

/// **No frames reach nowhere**, which is a real state and not an empty rectangle.
#[test]
fn nothing_open_reaches_nowhere() {
    assert!(reached_by(&[]).is_none());
}
