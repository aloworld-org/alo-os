//! That a task window asks for the design's size, is cut to the display rather
//! than overflowing it, is refused on a display too small, and that repeated
//! openings stay reachable.
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on a window this file just asked to be placed is the \
              failure being reported"
)]

use alo_strings::Direction;
use alo_tasks::layout::{the_least_height, the_least_width};
use smithay::utils::{Rectangle, Size};

use super::{AS_LARGE_AS_THE_DESIGN_DRAWS_IT, what_size_it_asks_for, where_one_more_goes};

/// A display with room to spare.
fn a_large_view() -> Rectangle<i32, smithay::utils::Logical> {
    Rectangle::new((0, 0).into(), (3840, 2160).into())
}

/// **On a display with room, it asks for the size the design draws.**
#[test]
fn it_asks_for_the_designs_own_size_when_there_is_room() {
    let asked = what_size_it_asks_for(a_large_view().size).expect("room on a 4K display");

    assert_eq!(
        asked.w,
        i32::try_from(AS_LARGE_AS_THE_DESIGN_DRAWS_IT.0).unwrap_or(0)
    );
    assert_eq!(
        asked.h,
        i32::try_from(AS_LARGE_AS_THE_DESIGN_DRAWS_IT.1).unwrap_or(0)
    );
}

/// **On a smaller display it is cut to the display, not left overflowing it.**
#[test]
fn it_is_cut_to_a_display_smaller_than_the_design() {
    let small = Size::from((800, 600));
    let asked = what_size_it_asks_for(small).expect("800 by 600 is above the floor");

    assert_eq!(asked.w, 800, "it is no wider than the display");
    assert_eq!(asked.h, 600, "it is no taller than the display");
    assert!(asked.w < i32::try_from(AS_LARGE_AS_THE_DESIGN_DRAWS_IT.0).unwrap_or(0));
}

/// **It never asks for less than the layout will lay out.**
///
/// A window below its own floor is one the rasteriser refuses, and a refused
/// window is one a person cannot see at all — so the floor wins over the
/// display, and a display below the floor is refused outright instead.
#[test]
fn it_never_asks_for_less_than_the_layout_will_draw() {
    let least = Size::from((
        i32::try_from(the_least_width()).unwrap_or(0),
        i32::try_from(the_least_height()).unwrap_or(0),
    ));
    let asked = what_size_it_asks_for(least).expect("exactly the floor is allowed");

    assert_eq!(asked, least);
    assert!(
        alo_tasks::layout::TheRows::of(
            u32::try_from(asked.w).unwrap_or(0),
            u32::try_from(asked.h).unwrap_or(0)
        )
        .is_ok(),
        "the size it asked for is one the layout refuses"
    );
}

/// **A display too small for the floor is refused, and nothing is placed.**
#[test]
fn a_display_too_small_is_refused_rather_than_given_a_window_it_cannot_draw() {
    let tiny = Size::from((320, 200));

    assert_eq!(what_size_it_asks_for(tiny), None);
    assert_eq!(
        where_one_more_goes(
            Rectangle::new((0, 0).into(), tiny),
            &[],
            Direction::LeftToRight,
            &[]
        ),
        None
    );
}

/// **The first window opens inside the view, at the size it asked for.**
#[test]
fn the_first_window_opens_inside_the_view() {
    let view = a_large_view();

    let first =
        where_one_more_goes(view, &[], Direction::LeftToRight, &[]).expect("room on a 4K display");

    assert_eq!(
        first.size,
        what_size_it_asks_for(view.size).expect("a size"),
        "it was not given the size it asked for"
    );
    assert!(
        view.contains_rect(first),
        "the first window is outside the view: {first:?} in {view:?}"
    );
}

/// **A second window does not land exactly on the first.**
///
/// Rule 3 prefers beside; rule 5 allows in front and offset. Either is correct
/// and *the same position* is not: two windows at one rectangle is one window
/// a person can reach.
#[test]
fn a_second_window_does_not_land_on_the_first() {
    let view = a_large_view();
    let first = where_one_more_goes(view, &[], Direction::LeftToRight, &[]).expect("the first");
    let second =
        where_one_more_goes(view, &[first], Direction::LeftToRight, &[]).expect("the second");

    assert_ne!(
        first.loc, second.loc,
        "the second window landed on the first"
    );
}

/// **Ten openings leave ten windows a person can still grab.**
///
/// Rule 7: *repeated openings must not drift off-screen.* And the thing that
/// must not drift is the **handle** — 44 × 24 at a window's top-left corner —
/// not the whole window. *Overlap is permitted; losing access to a window is
/// not.*
///
/// **This test asserted containment and was wrong.** It required every window
/// to sit wholly inside the view, and the fourth opening came 4 pixels past the
/// bottom edge with its entire title band visible — reachable by the
/// specification's own standard, and refused by an assertion stricter than the
/// rule. A test may not be stricter than the contract it checks: it would have
/// forced a placer that reserves room rule 4 forbids it to take.
#[test]
fn ten_openings_leave_ten_windows_a_person_can_still_grab() {
    // The handle `where_a_window_opens` guarantees: the corner of the name
    // band, which is what a person puts a pointer on to move a window.
    const A_REACHABLE_CORNER: (i32, i32) = (44, 24);

    let view = Rectangle::new((0, 0).into(), (1920, 1080).into());
    let mut placed: Vec<Rectangle<i32, smithay::utils::Logical>> = Vec::new();

    for which in 0..10 {
        let next = where_one_more_goes(view, &placed, Direction::LeftToRight, &[])
            .unwrap_or_else(|| panic!("window {which} was not placed"));

        let handle = Rectangle::new(
            next.loc,
            Size::from((
                A_REACHABLE_CORNER.0.min(next.size.w),
                A_REACHABLE_CORNER.1.min(next.size.h),
            )),
        );
        assert_eq!(
            view.intersection(handle),
            Some(handle),
            "window {which}'s handle is outside the view, so a person cannot \
             grab it: handle {handle:?} in {view:?}"
        );
        placed.insert(0, next);
    }

    assert_eq!(placed.len(), 10);
    // And none of them is *entirely* gone, which is what drifting off-screen
    // would look like however the handle fared.
    assert!(
        placed.iter().all(|at| view.intersection(*at).is_some()),
        "a window left the view altogether after ten openings"
    );
}

/// **A right-to-left reader gets the mirror of the same rule**, not a second
/// rule — so the two are different outcomes and neither is the other's default.
#[test]
fn a_right_to_left_reader_gets_the_mirrored_side() {
    let view = a_large_view();
    let first = where_one_more_goes(view, &[], Direction::LeftToRight, &[]).expect("the first");

    let ltr = where_one_more_goes(view, &[first], Direction::LeftToRight, &[]).expect("ltr");
    let rtl = where_one_more_goes(view, &[first], Direction::RightToLeft, &[]).expect("rtl");

    assert_ne!(
        ltr.loc.x, rtl.loc.x,
        "both readings put the second window at the same x, so the mirror does nothing"
    );
    assert!(
        rtl.loc.x < ltr.loc.x,
        "the right-to-left reader's window is not the leftward one: rtl at {} \
         against ltr at {}",
        rtl.loc.x,
        ltr.loc.x
    );
}

/// **A window is never placed where its edge would leave the view**, so the
/// Close on that edge can always be reached.
#[test]
fn a_window_leaves_room_above_it_for_its_edge() {
    for view in [
        Rectangle::new((0, 0).into(), (1280, 800).into()),
        Rectangle::new((0, 0).into(), (1920, 1080).into()),
        a_large_view(),
    ] {
        let at =
            where_one_more_goes(view, &[], Direction::LeftToRight, &[]).expect("room for a window");
        assert!(
            at.loc.y - crate::window_edge::THE_REGION_IS_TALL >= view.loc.y,
            "on {view:?} the window at {at:?} leaves its edge above the view"
        );
    }
}
