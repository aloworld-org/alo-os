//! That a screen reproduces its own frame at the size it was drawn, keeps its
//! shape on screens nobody drew, and never draws body text too small to read.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::{THE_DESIGN_WAS_DRAWN_AT, where_a_screen_goes};
use crate::what_display_there_is::Glass;

/// **At the size the design was drawn, the arithmetic lands on the design.**
///
/// This is the licence for every proportion: a rule that does not reproduce
/// the frame it came from is not a rule, it is a different layout.
#[test]
fn at_the_reference_size_it_is_what_the_frames_draw() {
    let (across, down) = THE_DESIGN_WAS_DRAWN_AT;
    let places = where_a_screen_goes(across, down, None).expect("the reference lays out");

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
    let big = where_a_screen_goes(3840, 2160, None).expect("a 4K panel lays out");
    let small = where_a_screen_goes(1280, 800, None).expect("a small laptop lays out");

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
    let floored = where_a_screen_goes(3840, 2160, Some(glass)).expect("it lays out");
    let unfloored = where_a_screen_goes(3840, 2160, None).expect("it lays out");

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
        where_a_screen_goes(3840, 2160, Some(roomy))
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
    let told_nothing = where_a_screen_goes(3840, 2160, None).expect("it lays out");
    let told_nought = where_a_screen_goes(3840, 2160, Some(Glass::of(0, 0))).expect("it lays out");
    assert_eq!(
        told_nought.body.tall, told_nothing.body.tall,
        "a panel reporting nought millimetres was treated as a measurement"
    );
}

/// **A display too small for the words is told so**, rather than being given a
/// screen squeezed into nothing.
#[test]
fn a_display_too_small_has_no_screen() {
    assert!(where_a_screen_goes(320, 200, None).is_none());
    // **The control**: one ordinary step up still lays out.
    assert!(where_a_screen_goes(1024, 768, None).is_some());
}
