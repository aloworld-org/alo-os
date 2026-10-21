//! Where the parts go on a screen whose list is longer than the screen.
//!
//! # Why this is a second shape and not a parameter
//!
//! [`crate::where_a_screen_goes`] lays out the six frames the owner adopted,
//! and every one of them asks a person to pick between **two** things: try or
//! install, Wi-Fi or a cable. Two cards fit between the introduction and the
//! actions, and at the design size that is exactly what is drawn.
//!
//! The Wi-Fi road asks two questions that are not like that. *Which keyboard
//! is this?* has nine answers, and *which network?* has as many as the room a
//! person is standing in. **Neither is among the six frames**, and the design
//! file does not answer them: its scrolling passage is the small-screen
//! mechanism — *tighten first, then scroll, measured at 1280×720* — which is a
//! different problem.
//!
//! # So it is borrowed, and from the same product
//!
//! The installed system met this question first and answered it. At first
//! start a person picks their language out of twenty-four, and
//! `alo_shell::the_languages_offered` records the decision: **scroll, show a
//! few rows at a time, and move the actions out of the scrolling region** —
//! *at y 812 and y 884 rather than inside the column* — because
//!
//! > a build that scrolls this list without moving them would put the way
//! > forward out of reach.
//!
//! That sheet is drawn on this same 1440×960 canvas, so y 812 is not a figure
//! borrowed across two different pages: it is the same place on the same page.
//! This module does the same thing for the environment, which means **a person
//! meets one list twice** — once before installing and once after — rather
//! than two lists that behave differently.
//!
//! # What that leaves, measured
//!
//! | | at 1440 × 960 |
//! |---|---|
//! | the first card | 341 down, as on every other screen |
//! | the actions | **812** rather than 577 |
//! | room for cards | 471 |
//! | a card and its gap | 96 |
//! | whole cards that fit | **five** |
//!
//! Five of nine keyboards, so a person scrolls once. Everything is a
//! proportion of the display, so a shorter screen shows fewer and a taller one
//! more — and [`how_many_cards_fit`] counts **whole** cards, because half a
//! card at the bottom edge reads as a list that is broken rather than as a
//! list that continues.

use crate::what_display_there_is::Glass;
use crate::where_a_screen_goes::{AScreensPlaces, HowMuchThereIs, where_a_screen_goes};

/// How far down the actions sit on a screen that scrolls.
///
/// The first-start sheet's own figure on this canvas, for its own reason: the
/// way forward must not be inside the part that scrolls.
const THE_ACTIONS_MOVE_DOWN_TO: f32 = 812.0 / 960.0;

/// Where everything goes on a screen whose list a person scrolls through.
///
/// The same screen as [`where_a_screen_goes`] in every respect but one: the
/// actions move down, out of the scrolling region. The consequence line is
/// left where it is and is simply not drawn by a list screen — there is no
/// consequence to a cursor moving, and the screens that have one are the
/// screens that erase something.
///
/// [`None`] on a display too small to lay out at all, for the same reason and
/// at the same size as [`where_a_screen_goes`].
#[must_use]
pub fn where_a_list_screen_goes(
    across: u32,
    down: u32,
    glass: Option<Glass>,
    how_much: HowMuchThereIs,
) -> Option<AScreensPlaces> {
    let mut places = where_a_screen_goes(across, down, glass, how_much)?;
    let moved = (f64::from(down) * f64::from(THE_ACTIONS_MOVE_DOWN_TO)).round() as u32;
    let by = moved.checked_sub(places.onward.down)?;
    places.onward.down = moved;
    places.back.down = places.back.down.saturating_add(by);
    Some(places)
}

/// How many whole cards fit between the first one and the actions.
///
/// **Whole, and the word is doing work.** A card drawn half-off the bottom of
/// the region says *this list is broken*, where a list that simply stops says
/// *there is more below*. The second is what a person can act on, so a part
/// card is never counted.
///
/// Returns at least one wherever the screen laid out at all: a list screen
/// that showed nothing would be a question with no visible answers.
#[must_use]
pub fn how_many_cards_fit(places: &AScreensPlaces) -> usize {
    let room = places.onward.down.saturating_sub(places.first_card.down);
    let step = places.card_to_card.max(1);
    let tall = places.first_card.tall;

    // `n` cards reach from the first card's top to `(n - 1) * step + tall`, so
    // the room left after the first card is what buys the rest. The last card
    // needs no gap after it, which is why this is not simply `room / step`.
    let after_the_first = room.saturating_sub(tall) / step;
    (after_the_first as usize).saturating_add(1)
}

#[cfg(test)]
#[path = "where_a_list_screen_goes_tests.rs"]
mod tests;
