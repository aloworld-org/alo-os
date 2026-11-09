//! The travel: where the outline is at its start, its middle and its end, a
//! duration however far it goes, and every way a travel ends early.
//!
//! The server's side — a real window put aside by the person's road, less
//! motion, a restore mid-travel — is driven through a client in
//! `tests/a_window_travels_aside`.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use std::time::{Duration, Instant};

use alo_dock::WindowId;
use smithay::utils::{Physical, Rectangle};

use super::{THE_TRAVEL_LASTS, TheTravel, TheTravels};
use crate::ease_in_and_out::eased;

/// A rectangle on a display.
fn rect(x: i32, y: i32, w: i32, h: i32) -> Rectangle<i32, Physical> {
    Rectangle::new((x, y).into(), (w, h).into())
}

/// Where a window was: an 800 × 600 window near the top left.
fn the_window() -> Rectangle<i32, Physical> {
    rect(100, 200, 800, 600)
}

/// Its card: the panel's first slot on the design's 1440 × 960 frame, as
/// `crate::panel_raster` lays it out — 24 in from the edge, 64 wide, a margin
/// in, and one 56 slot below the rail's top at a tenth of the height.
fn the_card() -> Rectangle<i32, Physical> {
    rect(1360, 160, 48, 48)
}

/// A travel that began at `began`.
fn travel(began: Instant) -> TheTravel {
    TheTravel::from(WindowId::numbered(7), the_window(), began)
        .expect("a window drawn at a size travels")
}

/// **It starts exactly where the window was**, so the outline leaves from the
/// window a person was looking at rather than jumping first.
#[test]
fn a_travel_starts_where_the_window_was() {
    let began = Instant::now();
    assert_eq!(travel(began).at(began, the_card()), (the_window(), false));
}

/// **Halfway through its time it is halfway there**, corner and size alike —
/// worked out from the two rectangles, because the curve passes its middle at
/// its middle.
#[test]
fn halfway_through_its_time_it_is_halfway_there_and_halfway_shrunk() {
    let began = Instant::now();
    let (from, to) = (the_window(), the_card());
    let half = |a: i32, b: i32| a + (b - a) / 2;
    assert!((eased(0.5) - 0.5).abs() < f64::EPSILON, "the curve's middle moved");
    assert_eq!(
        travel(began).at(began + THE_TRAVEL_LASTS / 2, to),
        (
            rect(
                half(from.loc.x, to.loc.x),
                half(from.loc.y, to.loc.y),
                half(from.size.w, to.size.w),
                half(from.size.h, to.size.h),
            ),
            false
        )
    );
}

/// **Eased in and out**: a tenth of its time covers less than a tenth of the
/// way, and nine tenths covers more than nine tenths of it.
#[test]
fn it_eases_in_and_out() {
    let began = Instant::now();
    let way = the_card().loc.x - the_window().loc.x;
    let (early, _) = travel(began).at(began + THE_TRAVEL_LASTS / 10, the_card());
    let early = early.loc.x - the_window().loc.x;
    assert!(early > 0 && early < way / 10, "a tenth of the time went {early} of {way}");
    let (late, _) = travel(began).at(began + THE_TRAVEL_LASTS * 9 / 10, the_card());
    let late = late.loc.x - the_window().loc.x;
    assert!(late > way * 9 / 10 && late < way, "nine tenths of the time went {late} of {way}");
}

/// **A duration, never a speed**: a short way and a long way end at the same
/// moment, neither a moment before, and each ends **exactly** on its card.
#[test]
fn a_travel_lasts_the_same_however_far_it_goes_and_ends_on_the_card() {
    for to in [rect(110, 205, 790, 590), rect(16_000, -9_000, 48, 48)] {
        let began = Instant::now();
        let travel = travel(began);
        let (_, ended) = travel.at(began + THE_TRAVEL_LASTS - Duration::from_millis(1), to);
        assert!(!ended, "it ended before its time, going to {to:?}");
        assert_eq!(
            travel.at(began + THE_TRAVEL_LASTS, to),
            (to, true),
            "it did not end on its card, at its time"
        );
    }
    assert_eq!(THE_TRAVEL_LASTS, Duration::from_millis(300), "the glide's length");
}

/// **A window drawn at no size does not travel**: there is no outline to follow.
#[test]
fn a_window_drawn_at_no_size_does_not_travel() {
    for nothing in [rect(10, 10, 0, 600), rect(10, 10, 800, 0), rect(0, 0, 0, 0)] {
        assert!(
            TheTravel::from(WindowId::numbered(1), nothing, Instant::now()).is_none(),
            "{nothing:?} travelled"
        );
    }
}

/// Travels keyed by a number, as the server keys them by a surface.
fn travelling(began: Instant, keys: &[u64]) -> TheTravels<u64> {
    let mut travels = TheTravels::none();
    for key in keys {
        let travel = TheTravel::from(WindowId::numbered(*key), the_window(), began)
            .expect("a window drawn at a size travels");
        travels.begin(*key, travel);
    }
    travels
}

/// Every window has the same card, which is enough for these.
fn every_window_has_a_card(_: WindowId) -> Option<Rectangle<i32, Physical>> {
    Some(the_card())
}

/// **Several put aside in a row each travel**, and a frame draws them all.
#[test]
fn several_put_aside_in_a_row_each_travel() {
    let began = Instant::now();
    let mut travels = travelling(began, &[1, 2, 3]);
    assert_eq!(travels.how_many(), 3);
    let drawn = travels.move_on(began + THE_TRAVEL_LASTS / 2, every_window_has_a_card);
    assert_eq!(drawn.len(), 3, "a travel was not drawn");
    assert_eq!(travels.how_many(), 3, "a travel ended halfway");
}

/// **The last frame is drawn on the card, and then nothing is**: the travel
/// is let go once it has arrived.
#[test]
fn the_last_frame_is_on_the_card_and_then_it_is_gone() {
    let began = Instant::now();
    let mut travels = travelling(began, &[1]);
    assert_eq!(
        travels.move_on(began + THE_TRAVEL_LASTS, every_window_has_a_card),
        vec![the_card()]
    );
    assert_eq!(travels.how_many(), 0, "it stayed after it arrived");
    assert!(
        travels
            .move_on(began + THE_TRAVEL_LASTS * 3 / 2, every_window_has_a_card)
            .is_empty(),
        "it was drawn after it arrived"
    );
}

/// **The refusal: a window with no card does not travel**, and is let go
/// undrawn — only the window that has one is drawn.
#[test]
fn a_window_with_no_card_is_let_go_undrawn() {
    let began = Instant::now();
    let mut travels = travelling(began, &[1, 2]);
    let only_two = |window: WindowId| (window == WindowId::numbered(2)).then(the_card);
    let drawn = travels.move_on(began + THE_TRAVEL_LASTS / 2, only_two);
    assert_eq!(drawn.len(), 1, "the window with no card was drawn");
    assert_eq!(travels.how_many(), 1, "the window with no card is still travelling");
}

/// **The refusal: a travel nobody saw is not drawn late.** Its time was up a
/// whole travel before the first frame — a locked screen — so it is let go.
#[test]
fn a_travel_nobody_saw_is_let_go_undrawn() {
    let began = Instant::now();
    let mut travels = travelling(began, &[1]);
    assert!(
        travels
            .move_on(began + THE_TRAVEL_LASTS * 2, every_window_has_a_card)
            .is_empty()
    );
    assert_eq!(travels.how_many(), 0);
}

/// **A window brought back mid-travel stops travelling**, and only that one.
#[test]
fn a_window_brought_back_stops_and_the_others_go_on() {
    let began = Instant::now();
    let mut travels = travelling(began, &[1, 2]);
    assert!(travels.cancel(&1), "the control: it was travelling");
    assert!(!travels.cancel(&1), "it was cancelled twice");
    assert_eq!(travels.how_many(), 1);
    assert_eq!(
        travels
            .move_on(began + THE_TRAVEL_LASTS / 2, every_window_has_a_card)
            .len(),
        1
    );
}

/// **A window put aside twice travels once**, from the second time.
#[test]
fn a_window_put_aside_again_travels_once() {
    let began = Instant::now();
    let travels = travelling(began, &[4, 4]);
    assert_eq!(travels.how_many(), 1);
}

/// **Less motion asked for ends every travel at once.**
#[test]
fn less_motion_ends_every_travel() {
    let mut travels = travelling(Instant::now(), &[1, 2, 3]);
    travels.end_all();
    assert_eq!(travels.how_many(), 0);
}
