//! Where each card is, on which edge, when there is none, and what an outline
//! is drawn as.
//!
//! Every expected figure is worked out from the panel's derivation in
//! `crate::panel_raster` — `ICON` 48, `MARGIN` and `GAP` 8, a clearance of
//! three margins, the rail a tenth of the way down — never read off a run.

use alo_dock::WindowId;
use alo_strings::Direction;
use smithay::utils::{Physical, Rectangle};

use super::TheCards;
use crate::desktop_testing::{an_appearance, noon_look, three_windows_put_aside};
use crate::painted::Solid;

/// The design's own frame, used as *a* display rather than *the* display.
const AS_DRAWN: (i32, i32) = (1440, 960);

/// A rectangle on a display.
fn rect(x: i32, y: i32, w: i32, h: i32) -> Rectangle<i32, Physical> {
    Rectangle::new((x, y).into(), (w, h).into())
}

/// The cards of three windows put aside, on a display of `size`.
fn cards(reading: Direction, size: (i32, i32), scale: u16) -> TheCards {
    TheCards::as_revealed(
        three_windows_put_aside(),
        noon_look(&an_appearance(), reading),
        size,
        scale,
    )
}

/// **Each window's card, on the right for a left-to-right reader, newest at
/// the top.** The rail is 64 wide and 24 in from the edge, so it starts at
/// `1440 − 24 − 64 = 1352` and a card a margin in at 1360; the rail's top is a
/// tenth of 960, and the first card sits a margin and one 56 slot (the
/// expansion control's) below it, at `96 + 8 + 56 = 160`. Window 3 was put
/// aside last, so it holds that card, and each older one is a slot below.
#[test]
fn each_card_is_on_the_right_and_the_newest_is_at_the_top() {
    let cards = cards(Direction::LeftToRight, AS_DRAWN, 100);
    assert_eq!(cards.of(WindowId::numbered(3)), Some(rect(1360, 160, 48, 48)));
    assert_eq!(cards.of(WindowId::numbered(2)), Some(rect(1360, 216, 48, 48)));
    assert_eq!(cards.of(WindowId::numbered(1)), Some(rect(1360, 272, 48, 48)));
}

/// **On the left for a right-to-left reader**: the rail 24 in from the left
/// edge, a card a margin in at 32, at the same heights.
#[test]
fn for_a_right_to_left_reader_the_cards_are_on_the_left() {
    let cards = cards(Direction::RightToLeft, AS_DRAWN, 100);
    assert_eq!(cards.of(WindowId::numbered(3)), Some(rect(32, 160, 48, 48)));
}

/// **Through the display's scale**: at 200 per cent every measure doubles, so
/// the rail is 128 wide and 48 in — starting at `2880 − 48 − 128 = 2704`, a
/// card 16 in at 2720 — and the first card is `192 + 16 + 112 = 320` down.
#[test]
fn on_a_dense_display_the_card_is_where_the_panel_draws_it() {
    let cards = cards(Direction::LeftToRight, (2880, 1920), 200);
    assert_eq!(cards.of(WindowId::numbered(3)), Some(rect(2720, 320, 96, 96)));
}

/// **The refusal: a window the panel does not hold has no card.**
#[test]
fn a_window_not_put_aside_has_no_card() {
    let cards = cards(Direction::LeftToRight, AS_DRAWN, 100);
    assert_eq!(cards.of(WindowId::numbered(99)), None);
}

/// **The refusal: no room for the panel's column, no cards.** The column is
/// `64 + 2 × 24 = 112` wide; a display 100 wide cannot hold it.
#[test]
fn a_display_with_no_room_for_the_panel_has_no_cards() {
    let cards = cards(Direction::LeftToRight, (100, 960), 100);
    for window in 1..=3 {
        assert_eq!(cards.of(WindowId::numbered(window)), None);
    }
    assert!(
        cards.the_outlines(&[rect(0, 0, 40, 40)]).is_empty(),
        "an outline was drawn with no card to land on"
    );
}

/// **The refusal: a rail too short drops the card.** On a display 120 tall the
/// rail starts at 12 and may run to 120; the first card would be
/// `12 + 8 + 56 = 76` down and end at 124, past it — so there is none.
#[test]
fn a_rail_too_short_for_the_card_has_none() {
    let cards = cards(Direction::LeftToRight, (1440, 120), 100);
    assert_eq!(cards.of(WindowId::numbered(3)), None);
}

/// **An outline is four sides of the focus ring's weight, in the cards'
/// colour**: 2 at 100 per cent, inside the rectangle.
#[test]
fn an_outline_is_four_sides_inside_the_rectangle() {
    let cards = cards(Direction::LeftToRight, AS_DRAWN, 100);
    let accent = noon_look(&an_appearance(), Direction::LeftToRight)
        .palette()
        .map(|palette| palette.accent)
        .ok();
    let drawn = cards.the_outlines(&[rect(100, 200, 800, 600)]);
    let areas: Vec<_> = drawn.iter().map(|solid| solid.area).collect();
    assert_eq!(
        areas,
        vec![
            rect(100, 200, 800, 2),
            rect(100, 798, 800, 2),
            rect(100, 202, 2, 596),
            rect(898, 202, 2, 596),
        ]
    );
    assert!(
        drawn.iter().all(|solid| Some(solid.colour) == accent),
        "an outline was not drawn in the cards' colour"
    );
}

/// **Twice as thick on a display at 200 per cent**, as every panel measure is.
#[test]
fn an_outline_is_thicker_on_a_dense_display() {
    let cards = cards(Direction::LeftToRight, (2880, 1920), 200);
    let drawn = cards.the_outlines(&[rect(0, 0, 400, 400)]);
    assert_eq!(drawn.first().map(|solid| solid.area), Some(rect(0, 0, 400, 4)));
}

/// **An outline too small to have an inside is drawn filled**, and one partly
/// off the display is cut to it rather than drawn outside it.
#[test]
fn a_small_outline_is_filled_and_one_off_the_display_is_cut() {
    let cards = cards(Direction::LeftToRight, AS_DRAWN, 100);
    let small: Vec<Solid> = cards.the_outlines(&[rect(10, 10, 4, 3)]);
    assert_eq!(
        small.iter().map(|solid| solid.area).collect::<Vec<_>>(),
        vec![rect(10, 10, 4, 3)]
    );
    let off = cards.the_outlines(&[rect(-50, -50, 100, 100)]);
    assert!(
        off.iter()
            .all(|solid| solid.area.loc.x >= 0 && solid.area.loc.y >= 0),
        "an outline was drawn off the display"
    );
    assert_eq!(off.len(), 2, "the two sides on the display, and only those");
}
