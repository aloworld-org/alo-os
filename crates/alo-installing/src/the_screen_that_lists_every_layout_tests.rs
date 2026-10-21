//! That the cards go down the screen in order, that the ring is on one of
//! them, and that the screen says which way the list continues.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use super::EveryLayoutThisComputerKnows;
use crate::where_a_screen_goes::THE_DESIGN_WAS_DRAWN_AT;

/// The eight the environment offers, by their own names.
const THE_EIGHT: [&str; 8] = [
    "Belgisch",
    "Français",
    "Deutsch",
    "Español",
    "Italiano",
    "Nederlands",
    "English (UK)",
    "English (US)",
];

fn opened() -> EveryLayoutThisComputerKnows<'static> {
    let (across, down) = THE_DESIGN_WAS_DRAWN_AT;
    EveryLayoutThisComputerKnows::opened(across, down, None, &THE_EIGHT)
        .expect("the reference display lays out")
}

#[test]
fn eight_layouts_in_five_cards_shows_five_and_says_there_are_more() {
    let screen = opened();
    let cards = screen.cards();
    assert_eq!(cards.len(), 5, "the design size has room for five");
    assert_eq!(cards[0].called, "Belgisch");
    assert_eq!(cards[4].called, "Italiano");
    assert!(!screen.more_above(), "at the top and claiming more above");
    assert!(
        screen.more_below(),
        "eight in five and claiming nothing below"
    );
    assert_eq!(screen.on(), Some("Belgisch"));
}

#[test]
fn the_cards_go_down_the_screen_one_gap_apart() {
    let screen = opened();
    let places = screen.places();
    let cards = screen.cards();

    for (down_the_screen, card) in cards.iter().enumerate() {
        assert_eq!(
            card.at.down,
            places.first_card.down + (down_the_screen as u32) * places.card_to_card,
            "card {down_the_screen} is not where the gap puts it"
        );
        assert_eq!(card.at.across, places.first_card.across);
        assert_eq!(card.at.wide, places.first_card.wide);
        assert_eq!(card.at.tall, places.first_card.tall);
    }

    // And the last one is clear of the way forward, which is the whole reason
    // the actions moved on a list screen.
    let last = cards.last().expect("five cards");
    assert!(
        last.at.down + last.at.tall <= places.onward.down,
        "the last card at {} reaches over the actions at {}",
        last.at.down + last.at.tall,
        places.onward.down
    );
}

#[test]
fn exactly_one_card_has_the_ring_wherever_the_cursor_is() {
    // A screen with no ring is a screen a person cannot see themselves on; a
    // screen with two is a screen that lies about which Enter will take.
    let mut screen = opened();
    for step in 0..THE_EIGHT.len() + 3 {
        let with_the_ring = screen
            .cards()
            .iter()
            .filter(|card| card.has_the_ring)
            .count();
        assert_eq!(
            with_the_ring, 1,
            "after {step} steps, {with_the_ring} cards have the ring"
        );
        screen.down();
    }
}

#[test]
fn the_ring_is_on_the_card_the_screen_says_it_is_on() {
    let mut screen = opened();
    for _ in 0..THE_EIGHT.len() + 2 {
        let named = screen.on().expect("a layout is always under the cursor");
        let ringed = screen
            .cards()
            .into_iter()
            .find(|card| card.has_the_ring)
            .expect("one card has the ring");
        assert_eq!(
            ringed.called, named,
            "the ring is on {} and the screen says the cursor is on {named}",
            ringed.called
        );
        screen.down();
    }
}

#[test]
fn scrolling_to_the_end_shows_the_last_layout_and_says_nothing_is_below() {
    let mut screen = opened();
    for _ in 0..THE_EIGHT.len() {
        screen.down();
    }
    let cards = screen.cards();
    assert_eq!(screen.on(), Some("English (US)"));
    assert_eq!(cards.last().expect("cards").called, "English (US)");
    assert!(
        screen.more_above(),
        "scrolled to the end and nothing is above"
    );
    assert!(
        !screen.more_below(),
        "scrolled to the end and more is below"
    );
}

#[test]
fn the_cursor_does_not_wrap_at_either_end() {
    let mut screen = opened();
    screen.up();
    assert_eq!(screen.on(), Some("Belgisch"), "up from the first wrapped");

    for _ in 0..20 {
        screen.down();
    }
    assert_eq!(
        screen.on(),
        Some("English (US)"),
        "down past the end wrapped"
    );
}

#[test]
fn a_shorter_list_than_the_screen_shows_all_of_it_and_no_arrows() {
    let (across, down) = THE_DESIGN_WAS_DRAWN_AT;
    let three = ["Deutsch", "Nederlands", "English (UK)"];
    let screen =
        EveryLayoutThisComputerKnows::opened(across, down, None, &three).expect("lays out");
    assert_eq!(screen.cards().len(), 3);
    assert!(!screen.more_above() && !screen.more_below());
}

#[test]
fn a_display_too_small_opens_no_screen() {
    assert!(EveryLayoutThisComputerKnows::opened(320, 240, None, &THE_EIGHT).is_none());
}

#[test]
fn the_words_are_the_keyboard_screens_own() {
    // Deliberate, and asserted so that a later change has to argue with this
    // rather than quietly re-word it: the list is the same question asked a
    // second way, and new wording would read as somewhere else in the
    // installer. Asserted by key rather than by English, because the key is
    // what ADR 0068 says is stable and the English is what translators move.
    let screen = opened();
    assert_eq!(
        screen.eyebrow().key().as_str(),
        "installing.before-anything-typed"
    );
    assert_eq!(screen.title().key().as_str(), "installing.every-layout");
    assert_eq!(
        screen.under_the_cards().key().as_str(),
        "installing.if-it-does-not-match"
    );
}

#[test]
fn every_sentence_this_screen_says_is_one_the_machine_has() {
    // The check that would catch a key this crate declares and the machine
    // cannot say — which would draw an empty line on the screen rather than
    // failing anywhere a person could see.
    let vocabulary = alo_saying::everything_this_machine_can_say().expect("alo OS's own words");
    let strings = alo_strings::Strings::of(vocabulary);
    let screen = opened();

    for word in [screen.eyebrow(), screen.title(), screen.under_the_cards()] {
        let said = strings.say(&word.key(), &alo_strings::Filling::nothing());
        assert!(
            !said.text().is_empty(),
            "{} says nothing",
            word.key().as_str()
        );
    }
}
