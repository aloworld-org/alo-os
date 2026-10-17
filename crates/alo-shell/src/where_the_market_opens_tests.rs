//! Where the window of what a person can install ends up.

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_appearance::TextScale;
use alo_software::{Catalogue, Category};
use smithay::utils::{Physical, Rectangle, Size};

use super::*;
use crate::market_keys::MarketKey;
use crate::market_window::MarketWindow;

/// A window open on the decided catalogue.
fn an_open_window() -> MarketWindow {
    let mut window = MarketWindow::closed();
    window.opened(Catalogue::decided().unwrap(), Vec::new());
    window
}

fn a_display(w: i32, h: i32) -> Rectangle<i32, Physical> {
    Rectangle::new((0, 0).into(), Size::from((w, h)))
}

/// **A shut window has no place on any display**, which is what the frame reads
/// as *do not draw one*.
#[test]
fn a_shut_window_is_nowhere() {
    let shut = MarketWindow::closed();
    assert!(
        where_the_market_opens(a_display(1920, 1080), &shut.shows(), TextScale::ordinary())
            .is_none()
    );
}

/// **It is centred**, with the leftover pixel of an odd display below and to
/// the right rather than rounded away.
#[test]
fn it_is_centred_on_the_display() {
    let window = an_open_window();
    let view = a_display(1920, 1080);
    let at = where_the_market_opens(view, &window.shows(), TextScale::ordinary()).unwrap();

    assert_eq!(at.loc.x, (view.size.w - at.size.w) / 2);
    assert_eq!(at.loc.y, (view.size.h - at.size.h) / 2);
    // The same margin each side, to the pixel the integer division leaves.
    let left = at.loc.x;
    let right = view.size.w - (at.loc.x + at.size.w);
    assert!(
        (left - right).abs() <= 1,
        "{left} pixels one side and {right} the other"
    );
}

/// **It is the width the design draws**, which is the figure every other
/// measure in `market_measures` is derived from.
#[test]
fn it_is_the_width_the_design_draws() {
    let window = an_open_window();
    let at = where_the_market_opens(
        a_display(1920, 1080),
        &window.shows(),
        TextScale::ordinary(),
    )
    .unwrap();

    assert_eq!(at.size.w, i32::try_from(m::THE_WINDOW_IS_WIDE).unwrap());
}

/// **Narrowing the list makes the window shorter**, which is the whole reason
/// the height is a function of the rows rather than a constant.
///
/// *Pictures* is the kind of work the catalogue has fewest of, and all of the
/// eight are on show before anything is narrowed.
#[test]
fn narrowing_the_list_shortens_the_window() {
    let mut window = an_open_window();
    let whole = where_the_market_opens(
        a_display(1920, 1080),
        &window.shows(),
        TextScale::ordinary(),
    )
    .unwrap();

    // Right once from *everything* narrows to the first kind of work.
    assert_eq!(
        window.pressed(MarketKey::After),
        crate::MarketPressed::Narrowed
    );
    let narrowed = where_the_market_opens(
        a_display(1920, 1080),
        &window.shows(),
        TextScale::ordinary(),
    )
    .unwrap();

    assert!(
        narrowed.size.h < whole.size.h,
        "narrowed to one kind of work the window is {} tall and the whole list is {}",
        narrowed.size.h,
        whole.size.h
    );
    assert_eq!(
        narrowed.size.w, whole.size.w,
        "narrowing changed the window's width"
    );
}

/// **The last row is not lost**, which is what `div_ceil` is there for: an odd
/// number of applications needs a row of its own for the last one.
#[test]
fn an_odd_number_of_applications_still_gets_its_last_row() {
    let mut window = an_open_window();
    // Walk every kind of work and check each one's window against the rows its
    // own count needs. One of them is bound to be odd, and the arithmetic is
    // asserted against the measures rather than against a figure typed here.
    let mut odd_ones = 0;
    for _ in 0..=Category::EVERY.len() {
        // **The borrow of the window ends before the next press**, which the
        // block is for: `shows` borrows it immutably and `pressed` wants it
        // mutably.
        let how_many = {
            let showing = window.shows();
            let crate::MarketShows::Browsing(browsing) = &showing else {
                unreachable!("an open window is browsing");
            };
            let how_many = browsing.how_many();
            let rows = u32::try_from(how_many.div_ceil(m::CARDS_ACROSS as usize)).unwrap();
            let at = where_the_market_opens(a_display(1920, 1080), &showing, TextScale::ordinary())
                .unwrap();
            assert_eq!(
                at.size.h,
                i32::try_from(m::the_window_is_tall(rows)).unwrap(),
                "{how_many} applications were given {} pixels",
                at.size.h
            );
            how_many
        };
        if how_many % 2 == 1 {
            odd_ones += 1;
        }
        window.pressed(MarketKey::After);
    }
    assert!(
        odd_ones > 0,
        "no kind of work in the catalogue has an odd number of applications, so this test \
         measured nothing it was written for"
    );
}

/// **A display too small to hold the window gets none**, rather than one laid
/// out past its own edges.
#[test]
fn a_display_too_small_holds_no_window() {
    let window = an_open_window();
    for (w, h) in [(320, 240), (632, 100), (100, 1080)] {
        assert!(
            where_the_market_opens(a_display(w, h), &window.shows(), TextScale::ordinary())
                .is_none(),
            "a {w}x{h} display was given a window"
        );
    }
}

/// **A larger text size makes a larger window**, because every measure goes
/// through the person's scale rather than being a pixel count.
#[test]
fn a_person_who_reads_larger_gets_a_larger_window() {
    let window = an_open_window();
    let view = a_display(3840, 2160);
    let ordinary = where_the_market_opens(view, &window.shows(), TextScale::ordinary()).unwrap();
    let largest = TextScale::percent(TextScale::range().1).unwrap();
    let larger = where_the_market_opens(view, &window.shows(), largest).unwrap();

    assert!(
        larger.size.w > ordinary.size.w && larger.size.h > ordinary.size.h,
        "largest is {}x{} and ordinary is {}x{}",
        larger.size.w,
        larger.size.h,
        ordinary.size.w,
        ordinary.size.h
    );
}

/// **A window too tall for the display is cut, not refused.**
///
/// The whole catalogue needs 755 logical pixels and the largest text size a
/// person may choose is 300 per cent, which is 2265 — taller than a 4K
/// display. Refusing it, which this file did first, left a person who reads
/// large with no market at all while everybody else had one.
///
/// `crate::market_raster` draws a cut window correctly: it stops at the last
/// whole card. What it cannot yet do is scroll to the rest, which
/// `super`'s header states as the next change rather than half-building it.
#[test]
fn a_window_too_tall_for_the_display_is_cut_rather_than_refused() {
    let window = an_open_window();
    let largest = TextScale::percent(TextScale::range().1).unwrap();
    let view = a_display(3840, 2160);

    let at = where_the_market_opens(view, &window.shows(), largest)
        .expect("a person who reads at 300 per cent has no market at all");

    assert_eq!(
        at.size.h, view.size.h,
        "the window is {} tall on a {} display, so it was neither cut nor fitted",
        at.size.h, view.size.h
    );
    assert_eq!(at.loc.y, 0, "a cut window is not centred vertically");
    // And it is still the design's width, because cutting the width would
    // narrow the cards past what `market_measures` asserts about them.
    assert_eq!(
        at.size.w,
        i32::try_from(m::THE_WINDOW_IS_WIDE).unwrap() * 3,
        "the width was cut as well as the height"
    );
}

/// **A screen with no room for even the window's furniture gets none.**
///
/// The title bar, the search field, the chips and the closing line come before
/// any card. A window shorter than those would draw a title and nothing else,
/// which reads as *there is nothing to install*.
#[test]
fn a_screen_with_no_room_for_the_furniture_holds_no_window() {
    let window = an_open_window();
    let bare = i32::try_from(m::the_window_is_tall(0)).unwrap();

    assert!(
        where_the_market_opens(
            a_display(1920, bare - 1),
            &window.shows(),
            TextScale::ordinary()
        )
        .is_none(),
        "a screen one pixel short of the furniture was given a window"
    );
    assert!(
        where_the_market_opens(
            a_display(1920, bare),
            &window.shows(),
            TextScale::ordinary()
        )
        .is_some(),
        "a screen with exactly room for the furniture was refused one"
    );
}
