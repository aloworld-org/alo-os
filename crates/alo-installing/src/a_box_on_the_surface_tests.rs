//! That a card is a card: solid in the middle, cut at the corners, and
//! bordered by a ring that is concentric with it.

#![expect(
    clippy::indexing_slicing,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use super::{ABoxToDraw, THE_CORNER, THE_RING, as_this_display_draws_it, the_edge_of, the_fill_of};
use crate::a_surface_to_draw_on::Grey;
use crate::where_a_screen_goes::AnArea;

fn a_card() -> ABoxToDraw {
    ABoxToDraw {
        area: AnArea {
            across: 160,
            down: 200,
            wide: 600,
            tall: 87,
        },
        fill: Grey::a_cards_surface(),
        edge: Grey::a_cards_edge(),
        edge_is: 1,
        corner: THE_CORNER,
    }
}

#[test]
fn the_middle_of_a_card_is_solid_all_the_way_across() {
    // The row halfway down is past every corner, so the card covers it
    // completely. If this row has a gap, the shape is not a rectangle at all.
    let card = a_card();
    let middle = the_fill_of(card, card.area.tall / 2);
    assert_eq!(middle.len(), card.area.wide as usize);
    let uncovered = middle.iter().filter(|how_much| **how_much < 1.0).count();
    assert_eq!(
        uncovered, 0,
        "{uncovered} pixel(s) of the card's middle row are not covered"
    );
}

#[test]
fn the_corners_are_cut_and_the_cut_is_the_radius() {
    // The very first row passes through both top corners, so it is narrower
    // than the card by about the radius at each end — that is what a rounded
    // corner *is*, and a box that drew it square would cover the whole row.
    let card = a_card();
    let first = the_fill_of(card, 0);
    let covered: f64 = first.iter().sum();
    let whole = f64::from(card.area.wide);
    assert!(
        covered < whole,
        "the top row covers the full width, so the corners are square"
    );
    // **How much two corners take, derived rather than guessed.** The first
    // attempt at this assertion expected twice the radius, which is what a
    // corner takes at the very top *edge* — but a row is sampled at its
    // centre, half a pixel down, and the circle has already come in by then.
    // The test caught that, which is the whole reason to write the arithmetic
    // out instead of writing down a number that looked right.
    let radius = f64::from(THE_CORNER);
    let into_the_row = radius - 0.5;
    let one_corner = radius - (radius * radius - into_the_row * into_the_row).sqrt();
    let missing = whole - covered;
    assert!(
        (missing - one_corner * 2.0).abs() < 1.0,
        "the top row is missing {missing:.2} units where two {THE_CORNER}-unit \
         corners sampled half a pixel down should take {:.2}",
        one_corner * 2.0
    );
    assert!(
        missing > 2.0,
        "the top row is missing only {missing:.2} units, which is not a \
         rounded corner at all"
    );
}

#[test]
fn a_cards_first_and_last_rows_are_not_empty() {
    // **The control for the test above.** A shape that covered *nothing* on
    // its extreme rows would also be narrower than the card there, and would
    // pass that assertion while drawing a card with its top and bottom sliced
    // off entirely.
    let card = a_card();
    for row in [0, card.area.tall - 1] {
        let covered: f64 = the_fill_of(card, row).iter().sum();
        assert!(
            covered > f64::from(card.area.wide) / 2.0,
            "row {row} of the card covers only {covered:.1} of \
             {} units, which is a card with its edge cut off rather than \
             rounded",
            card.area.wide
        );
    }
}

#[test]
fn nothing_is_drawn_outside_the_card() {
    // A row past the bottom belongs to whatever is below, and a card that drew
    // there would draw over the next card in the list.
    let card = a_card();
    let past: f64 = the_fill_of(card, card.area.tall + 1).iter().sum();
    assert_eq!(
        past, 0.0,
        "the card draws {past} units below its own bottom"
    );
}

#[test]
fn the_border_is_a_ring_and_not_a_second_card() {
    // The border covers the outside and leaves the middle alone, which is what
    // makes it a border. A fill mistaken for a border would cover the row.
    let mut card = a_card();
    card.edge_is = THE_RING;
    let middle = the_edge_of(card, card.area.tall / 2);

    let left = middle[0];
    let centre = middle[card.area.wide as usize / 2];
    let right = middle[card.area.wide as usize - 1];

    assert!(left > 0.9, "the ring's left side is not drawn: {left}");
    assert!(right > 0.9, "the ring's right side is not drawn: {right}");
    assert_eq!(
        centre, 0.0,
        "the ring covers the middle of the card, so it is a fill and not a ring"
    );
}

#[test]
fn a_box_with_no_border_draws_no_border() {
    // **The second control.** Every assertion about the ring would hold on an
    // implementation that always drew one, and then an unchosen card would
    // look chosen — which is the one thing the ring exists to distinguish.
    let mut card = a_card();
    card.edge_is = 0;
    let anywhere: f64 = the_edge_of(card, card.area.tall / 2).iter().sum();
    assert_eq!(
        anywhere, 0.0,
        "a card asked for no border drew {anywhere} units of one"
    );
}

#[test]
fn a_measure_is_scaled_by_the_display_and_never_rounds_away() {
    // 12 at the design's own height is 12. On a taller screen it is more, on a
    // shorter one less — and never nought, because a border that rounds to
    // nothing is a border a person cannot see.
    let (_, drawn_at) = crate::where_a_screen_goes::THE_DESIGN_WAS_DRAWN_AT;
    assert_eq!(as_this_display_draws_it(THE_CORNER, drawn_at), THE_CORNER);
    assert!(as_this_display_draws_it(THE_CORNER, drawn_at * 2) > THE_CORNER);
    assert!(as_this_display_draws_it(THE_CORNER, drawn_at / 2) < THE_CORNER);
    assert!(
        as_this_display_draws_it(1, 240) >= 1,
        "a one-unit border rounded away to nothing on a short screen"
    );
}
