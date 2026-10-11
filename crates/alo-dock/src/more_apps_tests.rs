//! That the caps hold, whatever is in the list and whatever output it is on.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::*;
use crate::measures::{MORE_APPS_IS_AT_MOST_TALL, MORE_APPS_IS_WIDE, THE_APP_LIST_IS_AT_MOST_TALL};

/// The room a 768-high output leaves: the Dock's band, the gap to it, and the
/// same clearance at the top.
const ON_A_768_OUTPUT: u32 = 768 - 76 - 12 - 12;

/// **Sixty applications do not make it taller.**
///
/// The owner's sentence, and the one this file exists for: *sixty applications
/// must scroll inside the same bounded viewport. A 644px-tall popover exceeds
/// the specification.* 644 is what this repository drew on 2026-10-11, on a
/// 768-high output, with the list design that grew until it ran out of screen.
///
/// Measured at both ends, because a cap that is never reached is not being
/// tested: four applications and sixty give the same two numbers.
#[test]
fn sixty_applications_are_the_same_size_as_four() {
    let room = Room::pixels(ON_A_768_OUTPUT);
    let four = TheOverview::of(4, 0, room);
    let sixty = TheOverview::of(4, 56, room);

    assert_eq!(
        sixty.height().as_pixels(),
        MORE_APPS_IS_AT_MOST_TALL,
        "sixty applications did not stop at the cap"
    );
    assert_eq!(
        sixty.the_list_is_tall().as_pixels(),
        THE_APP_LIST_IS_AT_MOST_TALL,
        "the list did not stop at its own cap"
    );
    assert!(
        sixty.scrolls(),
        "sixty applications fitted without scrolling, so the cap was not reached"
    );
    assert!(
        four.height().as_pixels() < sixty.height().as_pixels(),
        "four applications are already at the cap, so this test proves nothing: {} and {}",
        four.height().as_pixels(),
        sixty.height().as_pixels()
    );
}

/// **However many there are, it is never taller than the cap.**
///
/// The sweep rather than one case, because a cap that holds at sixty and not at
/// six hundred is a cap that holds by accident.
#[test]
fn nothing_makes_it_taller_than_the_cap() {
    let room = Room::pixels(ON_A_768_OUTPUT);
    for how_many in [0_usize, 1, 3, 4, 5, 8, 17, 60, 200, 2_000] {
        let it = TheOverview::of(4, how_many, room);
        assert!(
            it.height().as_pixels() <= MORE_APPS_IS_AT_MOST_TALL,
            "{how_many} applications gave a popover {} tall",
            it.height().as_pixels()
        );
        assert!(
            it.the_list_is_tall().as_pixels() <= THE_APP_LIST_IS_AT_MOST_TALL,
            "{how_many} applications gave a list {} tall",
            it.the_list_is_tall().as_pixels()
        );
    }
}

/// **On a short output the list gives way and the whole thing still fits.**
///
/// The owner's second rule: *on shorter screens, reduce the list viewport
/// further so the complete popover fits the available space.*
///
/// Swept down to a room smaller than the fixed parts, because that is where a
/// subtraction wraps round and a popover becomes enormous rather than small.
#[test]
fn a_short_output_shrinks_the_list_and_not_the_rest() {
    for room in [800_u32, 600, 536, 400, 300, 200, 100, 50, 10, 0] {
        let it = TheOverview::of(4, 56, Room::pixels(room));
        assert!(
            it.height().as_pixels() <= MORE_APPS_IS_AT_MOST_TALL,
            "in {room} of room it was {} tall",
            it.height().as_pixels()
        );
        assert!(
            it.height().as_pixels() <= room.max(FIXED),
            "in {room} of room it was {} tall, which does not fit",
            it.height().as_pixels()
        );
    }
}

/// **Four columns**, which is what 448 wide gives.
///
/// Asserted against the arithmetic rather than the number, so that a narrower
/// popover would be caught changing the answer rather than silently drawing
/// four columns off its own edge.
#[test]
fn the_grid_has_four_columns_and_they_fit_across_it() {
    let it = TheOverview::of(4, 8, Room::pixels(ON_A_768_OUTPUT));
    assert_eq!(it.columns(), 4);

    let inside = MORE_APPS_IS_WIDE - INSIDE_MORE_APPS * 2;
    let used = u32::try_from(it.columns()).unwrap() * AN_APP_CELL_IS_WIDE
        + (u32::try_from(it.columns()).unwrap() - 1) * BETWEEN_APP_CELLS;
    assert!(
        used <= inside,
        "{} columns need {used} and there are {inside}",
        it.columns()
    );
    assert!(
        used + BETWEEN_APP_CELLS + AN_APP_CELL_IS_WIDE > inside,
        "another column would have fitted, so this is not as many as fit"
    );
}

/// **The cells are where the design puts them.**
///
/// The first four at 0, 102, 204 and 306 across, which is the frame's own
/// measurement, and the fifth back at 0 on the next row.
#[test]
fn the_cells_sit_where_the_design_draws_them() {
    let it = TheOverview::of(8, 0, Room::pixels(ON_A_768_OUTPUT));
    let across: Vec<u32> = (0..5)
        .map(|which| it.where_a_cell_sits(which).unwrap().0.as_pixels())
        .collect();
    assert_eq!(across, vec![0, 102, 204, 306, 0]);

    let first = it.where_a_cell_sits(0).unwrap().1.as_pixels();
    let fifth = it.where_a_cell_sits(4).unwrap().1.as_pixels();
    assert_eq!(
        first, 26,
        "the first row should sit under an 18-high heading and an 8 gap"
    );
    assert_eq!(fifth - first, 96, "a row is 88 tall with 8 between rows");
}

/// **Past the last application there is no cell.**
#[test]
fn there_is_no_cell_past_the_last_application() {
    let it = TheOverview::of(4, 3, Room::pixels(ON_A_768_OUTPUT));
    assert!(it.where_a_cell_sits(6).is_some(), "the seventh of seven");
    assert_eq!(it.where_a_cell_sits(7), None, "one past the last");
    assert_eq!(it.where_a_cell_sits(usize::MAX), None, "far past it");
}

/// **An empty group takes no room**, so two headings never appear with a gap
/// and nothing between them.
#[test]
fn an_empty_group_takes_no_room() {
    let room = Room::pixels(ON_A_768_OUTPUT);
    let both = TheOverview::of(4, 4, room);
    let only_others = TheOverview::of(0, 4, room);

    assert_eq!(only_others.pinned().tall().as_pixels(), 0);
    assert_eq!(
        only_others.others().at().as_pixels(),
        0,
        "the second group did not move up into the empty first one"
    );
    assert!(
        both.others().at().as_pixels() > 0,
        "the premise: with both groups the second one is lower down"
    );
}

/// **A focused application is scrolled into view, and the least it can be.**
///
/// The owner's *scroll keyboard-focused items into view*. Both directions:
/// something below the viewport scrolls down to it, and something above
/// scrolls back up — and the second is the one a single-direction
/// implementation gets wrong.
#[test]
fn a_focused_application_is_scrolled_into_view_from_either_direction() {
    let it = TheOverview::of(4, 56, Room::pixels(ON_A_768_OUTPUT));
    assert!(it.scrolls(), "the premise: there is more than fits");
    assert!(!it.is_shown(59), "the premise: the last one is out of view");

    let down = it.showing(59);
    assert!(down.is_shown(59), "it did not scroll down to the last one");

    let up = down.showing(0);
    assert!(up.is_shown(0), "it did not scroll back up to the first one");
    assert_eq!(up.scrolled().as_pixels(), 0, "the first one is at the top");
}

/// **It never scrolls past the end.**
#[test]
fn it_does_not_scroll_past_the_last_row() {
    let it = TheOverview::of(4, 56, Room::pixels(ON_A_768_OUTPUT)).showing(59);
    assert!(
        it.scrolled().as_pixels() <= it.the_furthest_it_scrolls().as_pixels(),
        "scrolled {} of a possible {}",
        it.scrolled().as_pixels(),
        it.the_furthest_it_scrolls().as_pixels()
    );
}

/// **A list that fits does not scroll**, which is the other side of the cap.
#[test]
fn a_short_list_does_not_scroll() {
    let it = TheOverview::of(4, 4, Room::pixels(ON_A_768_OUTPUT));
    assert!(!it.scrolls());
    assert_eq!(it.the_furthest_it_scrolls().as_pixels(), 0);
    assert!(
        it.the_list_is_tall().as_pixels() < THE_APP_LIST_IS_AT_MOST_TALL,
        "eight applications should not need the whole viewport"
    );
}
