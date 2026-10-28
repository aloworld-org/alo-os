//! Notification cards on a display drawn at 200 per cent, held against the same
//! cards at 100: twice the size, the same words, and the one-to-one cards the
//! same bytes they were before they took a scale.
//!
//! **There is no press to hold here**, and that is a fact about the cards
//! rather than a gap in these tests: nothing hit-tests a card. Its actions are
//! labels drawn as `alo-notifying` worded them, so a scale cannot move what a
//! press chooses.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_dock::Dock;
use alo_notifying::Shown;

use super::tests::{from_an_agent, from_the_machine, light, words};
use super::{NotificationPicture, picture};
use crate::WindowControlLabels;
use crate::a_picture_hashed::Hashed;
use crate::display_scale::DisplayScale;
use crate::egress_status_place::TheRoom;

/// A display one to one.
const SPARSE: (i32, i32) = (1920, 1080);
/// The same display with twice the pixels each way.
const DENSE: (i32, i32) = (3840, 2160);

/// The dense display's scale.
fn two_times() -> DisplayScale {
    DisplayScale::of_per_cent(200)
}

/// One card from alo OS with two actions, and one from the agent.
fn two_cards() -> [Shown; 2] {
    [
        from_the_machine(
            "Update ready",
            "Restart when you are ready",
            &[("later", "Later"), ("now", "Restart now")],
        ),
        from_an_agent("Your mail is sorted"),
    ]
}

/// The cards drawn on a display of `size` at `scale`, with nothing put aside.
fn drawn(size: (i32, i32), scale: DisplayScale) -> NotificationPicture {
    let mut labels = WindowControlLabels::new().unwrap();
    let mut room = TheRoom::with_nothing_put_aside(size);
    room.scale = scale;
    picture(
        &two_cards(),
        &words(),
        &Dock::shipped(),
        &mut labels,
        room,
        light(),
    )
    .unwrap()
}

/// **At 100 the cards are the bytes they drew before they took a scale.**
///
/// Both cards' places, their edges and grounds, and every word on them. The
/// hash was taken once, by hand, on 2026-10-28 from this fixture run against
/// the tree this change was branched from (`56abab9a`), and this test repeats
/// the comparison on every build.
#[test]
fn at_one_hundred_the_cards_draw_the_bytes_they_drew_before() {
    let cards = drawn(SPARSE, DisplayScale::ONE);
    let mut hash = Hashed::begun();
    for card in &cards.cards {
        hash.area(card.area);
    }
    hash.solids(&cards.solids);
    hash.inked(&cards.inked);
    let now = hash.done();
    assert_eq!(
        now, BEFORE,
        "the cards at 100 drew {now:#018x}, before {BEFORE:#018x}"
    );
}

/// The hash of the cards before this change. See the test above.
const BEFORE: u64 = 0x2550_6af7_8625_fe28;

/// Whether `two` is within a few pixels of twice `one`.
///
/// **A few, and that is type rather than slack.** A card's height and width
/// are its words' box plus a measured margin; the margin doubles exactly and a
/// line shaped at twice the size is within a pixel or two of twice as wide.
fn near_twice(two: i32, one: i32) -> bool {
    (two - 2 * one).abs() <= 4
}

/// **At 200 every card is twice the size and twice as far from the corner**,
/// says the same words, and its edge is twice as thick — the measure that
/// doubles exactly, held exactly.
#[test]
fn at_two_hundred_every_card_is_twice_the_size() {
    let sparse = drawn(SPARSE, DisplayScale::ONE);
    let dense = drawn(DENSE, two_times());
    assert_eq!(sparse.cards.len(), 2, "the premise: two cards to compare");
    assert_eq!(dense.cards.len(), sparse.cards.len(), "every card is drawn");
    for (two, one) in dense.cards.iter().zip(&sparse.cards) {
        assert_eq!(two.lines, one.lines, "a card says what it said");
        for (a, b) in [
            (two.area.loc.x, one.area.loc.x),
            (two.area.loc.y, one.area.loc.y),
            (two.area.size.w, one.area.size.w),
            (two.area.size.h, one.area.size.h),
        ] {
            assert!(
                near_twice(a, b),
                "the card {:?} is not near twice {:?}",
                two.area,
                one.area
            );
        }
    }
    // The edge: the ground is inset from the card by its edge, 2 at 100 and 4
    // at 200, exactly.
    let inset = |cards: &NotificationPicture| {
        let (edge, ground) = (cards.solids.first().unwrap(), cards.solids.get(1).unwrap());
        ground.area.loc.x - edge.area.loc.x
    };
    assert_eq!((inset(&sparse), inset(&dense)), (2, 4), "the card's edge");
}

/// **Two displays at two scales each get cards laid out for their own**: a
/// two-times laptop and a one-to-one monitor beside it.
#[test]
fn two_displays_at_two_scales_each_get_cards_of_their_own() {
    let laptop = drawn((2560, 1600), two_times());
    let monitor = drawn(SPARSE, DisplayScale::ONE);
    let (Some(on_the_laptop), Some(on_the_monitor)) = (laptop.cards.first(), monitor.cards.first())
    else {
        unreachable!("a card that fits on both displays was drawn on neither");
    };
    assert!(
        near_twice(on_the_laptop.area.size.h, on_the_monitor.area.size.h),
        "the laptop's card {:?} against the monitor's {:?}",
        on_the_laptop.area,
        on_the_monitor.area
    );
}
