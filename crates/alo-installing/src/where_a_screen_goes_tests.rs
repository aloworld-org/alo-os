//! That a screen reproduces its own frame at the size it was drawn, keeps its
//! shape on screens nobody drew, and never draws body text too small to read.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::{HowMuchThereIs, THE_DESIGN_WAS_DRAWN_AT, where_a_screen_goes};
use crate::what_display_there_is::Glass;

/// **At the size the design was drawn, the arithmetic lands on the design.**
///
/// This is the licence for every proportion: a rule that does not reproduce
/// the frame it came from is not a rule, it is a different layout.
#[test]
fn at_the_reference_size_it_is_what_the_frames_draw() {
    let (across, down) = THE_DESIGN_WAS_DRAWN_AT;
    let places = where_a_screen_goes(
        across,
        down,
        None,
        HowMuchThereIs::as_the_frames_were_drawn(),
    )
    .expect("the reference lays out");

    assert_eq!(
        (places.eyebrow.across, places.eyebrow.down),
        (160, 164),
        "the block does not begin where the frames draw it"
    );
    assert_eq!(places.eyebrow.wide, 600, "the block is not 600 wide");
    assert_eq!(
        places.first_card.down - places.eyebrow.down,
        177,
        "the first card is not where the frames draw it"
    );
    assert_eq!(places.first_card.tall, 86, "a card is not 86 tall");
    assert_eq!(places.card_to_card, 96, "the cards are not 96 apart");
    assert_eq!(places.onward.wide, 160, "a button is not 160 wide");
    assert_eq!(places.onward.tall, 48, "a button is not 48 tall");
    assert_eq!(
        places.back.across - places.onward.across,
        172,
        "the second button is not 172 along, as the frames draw it"
    );
    assert_eq!(places.title_type.tall, 36, "the title is not 36");
    assert_eq!(places.body.tall, 16, "the introduction is not 16");
    assert_eq!(places.quiet.tall, 14, "a card's description is not 14");
}

/// **The shape is kept on a screen nobody drew.**
#[test]
fn the_shape_holds_on_a_display_the_design_never_saw() {
    let big = where_a_screen_goes(3840, 2160, None, HowMuchThereIs::as_the_frames_were_drawn())
        .expect("a 4K panel lays out");
    let small = where_a_screen_goes(1280, 800, None, HowMuchThereIs::as_the_frames_were_drawn())
        .expect("a small laptop lays out");

    for (places, across, down) in [(big, 3840u32, 2160u32), (small, 1280, 800)] {
        let left = f64::from(places.eyebrow.across) / f64::from(across);
        assert!(
            (0.10..=0.12).contains(&left),
            "the block begins {left} of the way in, and the frames begin it at 0.111"
        );
        assert!(
            places.accessibility.down + places.accessibility.tall <= down,
            "the accessibility control hangs off the bottom of the display"
        );
        assert!(
            places.back.across + places.back.wide < across,
            "the second button hangs off the right of the display"
        );
    }
    assert!(
        big.title_type.tall > small.title_type.tall * 2,
        "the title did not grow with the display: {} against {}",
        big.title_type.tall,
        small.title_type.tall
    );
}

/// **Body text is never drawn smaller than the design's own physical size.**
///
/// A 15-inch 4K panel is where proportion alone draws it at about
/// three-quarters of what the design intends, and it is the screen asking
/// whether to erase a disk.
#[test]
fn body_text_is_never_smaller_than_the_design_draws_it() {
    // 3840 × 2160 on a 15.6-inch panel: 345 × 194 millimetres.
    let glass = Glass::of(345, 194);
    let floored = where_a_screen_goes(
        3840,
        2160,
        Some(glass),
        HowMuchThereIs::as_the_frames_were_drawn(),
    )
    .expect("it lays out");
    let unfloored =
        where_a_screen_goes(3840, 2160, None, HowMuchThereIs::as_the_frames_were_drawn())
            .expect("it lays out");

    assert!(
        floored.body.tall > unfloored.body.tall,
        "the floor did nothing on the panel it exists for: {} against {}",
        floored.body.tall,
        unfloored.body.tall
    );
    let per_mm = f64::from(2160) / f64::from(194);
    let cap_mm = f64::from(floored.body.tall) * 0.72 / per_mm;
    assert!(
        cap_mm >= 3.0,
        "body text is {cap_mm:.2} mm of capital on a screen asking to erase a disk"
    );

    // **The control**: on a screen where proportion is already large enough,
    // the floor changes nothing — so the growth above is the floor and not
    // something that enlarges everything.
    let roomy = Glass::of(700, 393);
    assert_eq!(
        where_a_screen_goes(
            3840,
            2160,
            Some(roomy),
            HowMuchThereIs::as_the_frames_were_drawn()
        )
        .expect("it lays out")
        .body
        .tall,
        unfloored.body.tall,
        "the floor grew text on a panel that did not need it"
    );
}

/// **A display that says nothing about its size still lays out**, and is not
/// given a scale guessed from its pixel count.
#[test]
fn a_display_that_reports_no_size_is_not_guessed_at() {
    let told_nothing =
        where_a_screen_goes(3840, 2160, None, HowMuchThereIs::as_the_frames_were_drawn())
            .expect("it lays out");
    let told_nought = where_a_screen_goes(
        3840,
        2160,
        Some(Glass::of(0, 0)),
        HowMuchThereIs::as_the_frames_were_drawn(),
    )
    .expect("it lays out");
    assert_eq!(
        told_nought.body.tall, told_nothing.body.tall,
        "a panel reporting nought millimetres was treated as a measurement"
    );
}

/// **A display too small for the words is told so**, rather than being given a
/// screen squeezed into nothing.
#[test]
fn a_display_too_small_has_no_screen() {
    assert!(
        where_a_screen_goes(320, 200, None, HowMuchThereIs::as_the_frames_were_drawn()).is_none()
    );
    // **The control**: one ordinary step up still lays out.
    assert!(
        where_a_screen_goes(1024, 768, None, HowMuchThereIs::as_the_frames_were_drawn()).is_some()
    );
}

#[test]
fn the_flow_reproduces_every_offset_the_frames_were_drawn_with() {
    // **This is the test that makes the flow a reading of the design rather
    // than a replacement for it.** Six offsets were measured off the adopted
    // frames; the flow derives all six from one gap of sixteen. If it is the
    // right reading, laying out with what the frames hold gives those six
    // numbers back exactly — and if it is not, this says which one moved.
    let (across, down) = THE_DESIGN_WAS_DRAWN_AT;
    let places = where_a_screen_goes(
        across,
        down,
        None,
        HowMuchThereIs::as_the_frames_were_drawn(),
    )
    .expect("the reference lays out");
    let top = places.eyebrow.down;

    for (what, was, now) in [
        ("the eyebrow", 0, places.eyebrow.down - top),
        ("the title", 32, places.title.down - top),
        ("the introduction", 136, places.introduction.down - top),
        ("the first card", 177, places.first_card.down - top),
        (
            "the line under the cards",
            375,
            places.consequence.down - top,
        ),
        ("the actions", 413, places.onward.down - top),
        ("the other action", 413, places.back.down - top),
    ] {
        assert_eq!(
            now, was,
            "{what} was drawn at {was} and the flow puts it at {now}"
        );
    }
}

#[test]
fn a_sentence_that_wraps_pushes_the_cards_down_instead_of_being_drawn_over_them() {
    // The fault the flow exists to remove, stated as the difference between
    // one line and two. `words.rs` gives the keyboard screen an introduction
    // of 120 characters, which is two lines at this size.
    let (across, down) = THE_DESIGN_WAS_DRAWN_AT;
    let one = where_a_screen_goes(
        across,
        down,
        None,
        HowMuchThereIs::as_the_frames_were_drawn(),
    )
    .expect("lays out");
    let two = where_a_screen_goes(
        across,
        down,
        None,
        HowMuchThereIs {
            introduction_lines: 2,
            ..HowMuchThereIs::as_the_frames_were_drawn()
        },
    )
    .expect("lays out");

    assert_eq!(
        two.introduction.tall,
        one.introduction.tall * 2,
        "a two-line introduction is not twice as tall"
    );
    assert_eq!(
        two.first_card.down - one.first_card.down,
        one.introduction.tall,
        "the cards did not move down by the line that was added"
    );
    assert!(
        two.introduction.down + two.introduction.tall <= two.first_card.down,
        "the introduction still reaches into the first card: it ends at {} and \
         the card starts at {}",
        two.introduction.down + two.introduction.tall,
        two.first_card.down
    );

    // And everything below moved with it, by the same amount, rather than
    // some parts moving and others staying.
    let by = one.introduction.tall;
    assert_eq!(two.consequence.down - one.consequence.down, by);
    assert_eq!(two.onward.down - one.onward.down, by);
    assert_eq!(two.back.down - one.back.down, by);
    assert_eq!(
        two.accessibility.down, one.accessibility.down,
        "the accessibility button is fixed to the bottom edge and moved"
    );
}

#[test]
fn nothing_in_the_block_ever_overlaps_anything_below_it() {
    // Swept, because the flow is only worth having if this holds at every
    // amount of text and every size — and an overlap is the one fault a
    // person sees immediately and no other test here would catch.
    for down in [480_u32, 600, 768, 800, 960, 1080, 1600, 2160] {
        for introduction_lines in 1..=6 {
            for cards in 0..=5 {
                let how_much = HowMuchThereIs {
                    title_lines: 2,
                    introduction_lines,
                    cards,
                    consequence_lines: 1,
                };
                let Some(places) = where_a_screen_goes(1280, down, None, how_much) else {
                    continue;
                };
                let cards_end = if cards == 0 {
                    places.first_card.down
                } else {
                    places.first_card.down
                        + (cards - 1) * places.card_to_card
                        + places.first_card.tall
                };
                for (above, below, what) in [
                    (
                        places.eyebrow.down + places.eyebrow.tall,
                        places.title.down,
                        "eyebrow into title",
                    ),
                    (
                        places.title.down + places.title.tall,
                        places.introduction.down,
                        "title into introduction",
                    ),
                    (
                        places.introduction.down + places.introduction.tall,
                        places.first_card.down,
                        "introduction into the cards",
                    ),
                    (
                        cards_end,
                        places.consequence.down,
                        "the cards into the line below them",
                    ),
                    (
                        places.consequence.down + places.consequence.tall,
                        places.onward.down,
                        "the line into the actions",
                    ),
                    (
                        places.onward.down + places.onward.tall,
                        places.accessibility.down,
                        "the actions into accessibility",
                    ),
                ] {
                    assert!(
                        above <= below,
                        "at {down} tall with {introduction_lines} lines and {cards} cards, \
                         {what}: {above} is below {below}"
                    );
                }
            }
        }
    }
}

#[test]
fn more_than_the_block_holds_is_refused_rather_than_drawn_over_itself() {
    // The honest answer when the flow runs out of room. A screen whose way
    // forward is underneath the accessibility button is a screen a person
    // cannot leave, so it is not drawn at all.
    let too_much = HowMuchThereIs {
        title_lines: 20,
        introduction_lines: 20,
        cards: 20,
        consequence_lines: 20,
    };
    assert!(
        where_a_screen_goes(1280, 800, None, too_much).is_none(),
        "a screen with twenty of everything claimed to lay out"
    );

    // And the control: the same display lays out what it actually holds, so
    // the refusal is about the amount of text and not about the display.
    assert!(
        where_a_screen_goes(1280, 800, None, HowMuchThereIs::as_the_frames_were_drawn()).is_some()
    );
}
