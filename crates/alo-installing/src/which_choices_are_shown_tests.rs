//! That the list behaves like the one at first start, and that the cursor is
//! never scrolled out of sight.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use super::WhereInTheList;

/// The nine the keyboard screen offers, and the five a design-size screen
/// shows at once.
const NINE: usize = 9;
const FIVE: usize = 5;

#[test]
fn a_new_list_starts_on_the_first_choice_at_the_top() {
    let list = WhereInTheList::of(NINE);
    assert_eq!(list.on(), Some(0));
    assert_eq!(list.shown(FIVE), 0..5);
    assert_eq!(list.cursor_among_those_shown(FIVE), Some(0));
    assert!(
        !list.more_above(FIVE),
        "a list at the top says there is more above it"
    );
    assert!(
        list.more_below(FIVE),
        "nine in a window of five says there is nothing below"
    );
}

#[test]
fn the_cursor_stops_at_each_end_and_does_not_wrap() {
    let mut list = WhereInTheList::of(NINE);

    list.up();
    assert_eq!(
        list.on(),
        Some(0),
        "up from the first choice wrapped to the last"
    );

    for _ in 0..20 {
        list.down();
    }
    assert_eq!(list.on(), Some(NINE - 1), "down past the end did not stop");
    list.down();
    assert_eq!(
        list.on(),
        Some(NINE - 1),
        "one more down wrapped to the top"
    );
}

#[test]
fn the_cursor_is_always_among_the_choices_that_are_drawn() {
    // The property the whole module exists for, swept rather than sampled: a
    // cursor outside the window is a focus ring a person cannot see, and they
    // would be pressing Enter on a row they are not looking at.
    for all in 0..40_usize {
        for fit in 1..8_usize {
            let mut list = WhereInTheList::of(all);
            for step in 0..all + 3 {
                let shown = list.shown(fit);
                match list.on() {
                    None => {
                        assert_eq!(shown, 0..0, "an empty list draws rows");
                        assert_eq!(list.cursor_among_those_shown(fit), None);
                    }
                    Some(on) => {
                        assert!(
                            shown.contains(&on),
                            "with {all} choices, {fit} shown, after {step} steps the cursor \
                             is on {on} and the window is {shown:?}"
                        );
                        let within = list
                            .cursor_among_those_shown(fit)
                            .expect("a list with choices in it has a cursor somewhere");
                        assert_eq!(shown.start + within, on, "the ring is on the wrong row");
                        assert!(within < fit, "the ring is below the last row drawn");
                    }
                }
                list.down();
            }
        }
    }
}

#[test]
fn the_window_never_runs_past_either_end_into_blank_rows() {
    for all in 1..40_usize {
        for fit in 1..8_usize {
            let mut list = WhereInTheList::of(all);
            for _ in 0..all + 3 {
                let shown = list.shown(fit);
                assert!(
                    shown.end <= all,
                    "the window reaches {} of {all}",
                    shown.end
                );
                if all >= fit {
                    assert_eq!(
                        shown.end - shown.start,
                        fit,
                        "with {all} choices and room for {fit}, only {} were drawn",
                        shown.end - shown.start
                    );
                }
                list.down();
            }
        }
    }
}

#[test]
fn a_list_that_fits_entirely_never_says_there_is_more() {
    let mut list = WhereInTheList::of(3);
    for _ in 0..5 {
        assert!(!list.more_above(FIVE));
        assert!(!list.more_below(FIVE));
        assert_eq!(list.shown(FIVE), 0..3);
        list.down();
    }
}

#[test]
fn a_list_longer_than_the_screen_says_which_way_there_is_more() {
    let mut list = WhereInTheList::of(NINE);

    // At the top: more below, nothing above.
    assert!(!list.more_above(FIVE) && list.more_below(FIVE));

    // At the bottom: more above, nothing below. This is the pair that matters —
    // a check on one end only passes on code that always answers the same.
    for _ in 0..NINE {
        list.down();
    }
    assert!(
        list.more_above(FIVE),
        "scrolled to the end and nothing is above"
    );
    assert!(
        !list.more_below(FIVE),
        "scrolled to the end and something is still below"
    );
    assert_eq!(list.shown(FIVE), 4..9);
}

#[test]
fn the_cursor_moves_into_the_middle_before_the_list_scrolls() {
    // Borrowed behaviour, and worth asserting rather than trusting: with the
    // cursor in the middle a person sees what is on both sides of where they
    // are, which is how they know the list continues.
    let mut list = WhereInTheList::of(NINE);
    list.down();
    list.down();
    assert_eq!(list.on(), Some(2));
    assert_eq!(list.shown(FIVE), 0..5, "the list scrolled before it had to");
    assert_eq!(list.cursor_among_those_shown(FIVE), Some(2));

    list.down();
    assert_eq!(
        list.shown(FIVE),
        1..6,
        "the list did not scroll when it had to"
    );
    assert_eq!(
        list.cursor_among_those_shown(FIVE),
        Some(2),
        "the cursor did not stay in the middle"
    );
}

#[test]
fn an_empty_list_has_no_cursor_and_draws_nothing() {
    // A real state: a card with no firmware finds no networks at all.
    let mut list = WhereInTheList::of(0);
    assert_eq!(list.on(), None);
    assert_eq!(list.how_many(), 0);
    assert_eq!(list.shown(FIVE), 0..0);
    assert_eq!(list.cursor_among_those_shown(FIVE), None);
    assert!(!list.more_above(FIVE) && !list.more_below(FIVE));

    // And moving about in it changes nothing rather than panicking.
    list.down();
    list.up();
    assert_eq!(list.on(), None);
}

#[test]
fn one_choice_is_not_a_scrolling_list() {
    let list = WhereInTheList::of(1);
    assert_eq!(list.shown(FIVE), 0..1);
    assert_eq!(list.cursor_among_those_shown(FIVE), Some(0));
    assert!(!list.more_above(FIVE) && !list.more_below(FIVE));
}
