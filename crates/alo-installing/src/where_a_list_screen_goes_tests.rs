//! That a list screen moves the way forward out of the scrolling region, and
//! shows whole cards only.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use super::{how_many_cards_fit, where_a_list_screen_goes};
use crate::where_a_screen_goes::{HowMuchThereIs, THE_DESIGN_WAS_DRAWN_AT, where_a_screen_goes};

/// What a list screen holds above its cards: the keyboard screen's own
/// eyebrow, question and quiet line, with the cards counted by the room
/// rather than declared here.
fn a_list_screens_text() -> HowMuchThereIs {
    HowMuchThereIs {
        cards: 0,
        ..HowMuchThereIs::as_the_frames_were_drawn()
    }
}

#[test]
fn at_the_design_size_five_whole_cards_fit() {
    let (across, down) = THE_DESIGN_WAS_DRAWN_AT;
    let places = where_a_list_screen_goes(across, down, None, a_list_screens_text())
        .expect("the reference lays out");

    // The figures this module's header records, so a change to any of them
    // shows up here as a number rather than as a screen somebody has to look
    // at.
    assert_eq!(places.first_card.down, 341, "the first card has moved");
    assert_eq!(
        places.onward.down, 812,
        "the actions are not where the sheet puts them"
    );
    assert_eq!(places.first_card.tall, 86);
    assert_eq!(places.card_to_card, 96);
    assert_eq!(how_many_cards_fit(&places), 5);
}

#[test]
fn the_actions_move_and_nothing_else_does() {
    let (across, down) = THE_DESIGN_WAS_DRAWN_AT;
    let ordinary = where_a_screen_goes(across, down, None, a_list_screens_text())
        .expect("the six frames lay out");
    let list = where_a_list_screen_goes(across, down, None, a_list_screens_text())
        .expect("a list screen lays out");

    assert!(
        list.onward.down > ordinary.onward.down,
        "the way forward did not move down, so it is still inside the part that scrolls"
    );
    assert_eq!(
        list.onward.down - ordinary.onward.down,
        list.back.down - ordinary.back.down,
        "the two actions moved by different amounts, so they are no longer on one row"
    );

    // Everything above the actions is the same screen, which is the point of
    // this being one shape with one thing moved rather than a second layout.
    assert_eq!(list.eyebrow, ordinary.eyebrow);
    assert_eq!(list.title, ordinary.title);
    assert_eq!(list.introduction, ordinary.introduction);
    assert_eq!(list.first_card, ordinary.first_card);
    assert_eq!(list.card_to_card, ordinary.card_to_card);
    assert_eq!(list.accessibility, ordinary.accessibility);
}

#[test]
fn a_list_screen_fits_more_than_the_six_frames_do() {
    // The control for the whole module: if an ordinary screen already fitted
    // five, moving the actions would be achieving nothing and none of this
    // would be worth its own file.
    //
    // **Each screen is laid out with its own content**, which is the whole
    // comparison. Laying the ordinary one out with a list screen's text — no
    // cards — made it fit one, because its actions then flow up to just under
    // the introduction. That is correct of a screen with no cards and says
    // nothing about the six frames.
    let (across, down) = THE_DESIGN_WAS_DRAWN_AT;
    let ordinary = where_a_screen_goes(
        across,
        down,
        None,
        HowMuchThereIs::as_the_frames_were_drawn(),
    )
    .expect("lays out");
    let list =
        where_a_list_screen_goes(across, down, None, a_list_screens_text()).expect("lays out");

    assert_eq!(
        how_many_cards_fit(&ordinary),
        2,
        "the adopted frames fit two cards, which is what each of them draws"
    );
    assert!(
        how_many_cards_fit(&list) > how_many_cards_fit(&ordinary),
        "moving the actions bought no extra cards"
    );
}

#[test]
fn only_whole_cards_are_counted() {
    // Taken across every height the installer lays out for: the cards must
    // always end at or above the actions. A count that was one too generous
    // would draw the last card over the way forward, and a person would be
    // looking at a button with a card on top of it.
    for down in (480..=2160).step_by(37) {
        let Some(places) = where_a_list_screen_goes(1920, down, None, a_list_screens_text()) else {
            continue;
        };
        let how_many = how_many_cards_fit(&places) as u32;
        let bottom =
            places.first_card.down + (how_many - 1) * places.card_to_card + places.first_card.tall;
        assert!(
            bottom <= places.onward.down,
            "at {down} tall, {how_many} cards reach {bottom} and the actions are at {}",
            places.onward.down
        );

        // And one more card would not have fitted, so the count is not merely
        // safe but as large as it can be. A check that only tested the first
        // half of this would pass on a function that always answered one.
        let with_another = bottom + places.card_to_card;
        assert!(
            with_another > places.onward.down,
            "at {down} tall, {} cards would also have fitted",
            how_many + 1
        );
    }
}

#[test]
fn five_cards_fit_on_every_screen_this_lays_out_for() {
    // **Measured, and it is the point rather than a coincidence.** Every
    // figure in this layout is a proportion of the display — the card's
    // height, the gap, where the first card starts, where the actions sit —
    // so scaling the screen scales all four together and the count cannot
    // move. Swept over 10,086 sizes when this was written, and all of them
    // answered five.
    //
    // So this test is really about the rule *nothing is built to one screen
    // size*: the day somebody puts a raw pixel figure into this layout, the
    // count starts drifting with the display and this fails. That is the fault
    // worth catching, and a test asserting *five at the design size* would not
    // have caught it.
    let mut counts = std::collections::BTreeSet::new();
    for down in (480..=2160).step_by(7) {
        for across in [640_u32, 1024, 1280, 1366, 1920, 2560, 3840] {
            if let Some(places) =
                where_a_list_screen_goes(across, down, None, a_list_screens_text())
            {
                counts.insert(how_many_cards_fit(&places));
            }
        }
    }
    assert_eq!(
        counts,
        std::collections::BTreeSet::from([5]),
        "the number of cards a list screen shows is not the same on every \
         display, which means something in this layout is no longer a \
         proportion of it"
    );
}

#[test]
fn nine_keyboards_in_five_cards_is_one_scroll() {
    // What that means for the person the keyboard screen is for: nine
    // layouts, five visible, so they scroll once and never twice.
    let (across, down) = THE_DESIGN_WAS_DRAWN_AT;
    let places =
        where_a_list_screen_goes(across, down, None, a_list_screens_text()).expect("lays out");
    let fit = how_many_cards_fit(&places);
    assert!(
        fit * 2 >= 9,
        "nine keyboards in windows of {fit} takes more than one scroll"
    );
}

#[test]
fn a_display_too_small_has_no_list_screen_either() {
    // The same floor as every other screen, rather than a second answer to
    // *is there room*.
    assert!(where_a_list_screen_goes(320, 240, None, a_list_screens_text()).is_none());
    assert!(where_a_screen_goes(320, 240, None, a_list_screens_text()).is_none());
}

#[test]
fn at_least_one_card_is_always_shown() {
    // A list screen that showed nothing would be a question with no visible
    // answers. Swept rather than asserted at one size, because the floor is
    // where this would break.
    for down in 480..=700 {
        if let Some(places) = where_a_list_screen_goes(640, down, None, a_list_screens_text()) {
            assert!(
                how_many_cards_fit(&places) >= 1,
                "nothing shown at {down} tall"
            );
        }
    }
}
