//! Where a window alo operates says it is sending, on its border, and in what
//! words — the owner, 2026-11-15: *it should be at the edges of the window*.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use alo_egress::HowItIsShown;
use cosmic_text::FontSystem;
use smithay::utils::{Logical, Point, Rectangle, Size};

use super::*;

/// VSCodium's window in the walk's screenshot of 2026-11-15, as a card.
fn codium() -> Rectangle<i32, Logical> {
    Rectangle::new(Point::from((14, 28)), Size::from((1140, 634)))
}

/// The fonts every drawing shapes with.
fn fonts() -> FontSystem {
    let mut fonts = FontSystem::new();
    fonts
        .db_mut()
        .load_font_data(alo_drawing::fonts::MANROPE.to_vec());
    fonts
}

/// **On the border's foot, outside the window, 16 in from its left-hand
/// corner**: nothing of the application's is covered, and the mark's tab on
/// the top edge keeps its own corner.
#[test]
fn it_stands_on_the_foot_outside_the_window() {
    assert_eq!(
        where_sending_is_said(codium(), 70, (1280, 800), 100),
        Rectangle::new(Point::from((14 + 16, 28 + 634)), Size::from((70, 20)))
    );
}

/// **Where the screen has no room under the window it stands on the top edge,
/// and where it has none there either, inside the foot's left-hand corner** —
/// the same room, read in the display's logical size at any density.
#[test]
fn it_finds_room_and_reads_it_in_logical_units() {
    let low = Rectangle::new(Point::from((100, 300)), Size::from((600, 490)));
    assert_eq!(
        where_sending_is_said(low, 70, (1280, 800), 100),
        Rectangle::new(Point::from((116, 280)), Size::from((70, 20)))
    );
    let whole = Rectangle::new(Point::from((0, 0)), Size::from((1280, 800)));
    let inside = Rectangle::new(Point::from((16, 800 - 16 - 20)), Size::from((70, 20)));
    assert_eq!(where_sending_is_said(whole, 70, (1280, 800), 100), inside);
    assert_eq!(where_sending_is_said(whole, 70, (2560, 1600), 200), inside);
    assert_eq!(
        where_sending_is_said(codium(), 70, (2560, 1600), 200),
        where_sending_is_said(codium(), 70, (1280, 800), 100)
    );
}

/// **The words are the indicator's two truths**: *sending* when alo's copy
/// was seen sending, *may be sending* where nothing can count it.
#[test]
fn the_words_say_what_is_known() {
    assert_eq!(
        what_the_border_says(HowItIsShown::Counted).says(),
        "sending"
    );
    assert_eq!(
        what_the_border_says(HowItIsShown::Uncounted).says(),
        "may be sending"
    );
    assert_eq!(
        what_the_border_says(HowItIsShown::TheirWindow).says(),
        "may be sending"
    );
}

/// **It is drawn as words on a tab of the border's ink, outside the window**,
/// and nothing of it lies over the window's own content.
#[test]
fn it_is_written_outside_the_window() {
    let ink = crate::window_menu_paint::rgb(alo_appearance::Role::AccentDefault);
    let (rasters, words) = drawn(
        HowItIsShown::Counted,
        codium(),
        ink,
        &mut fonts(),
        100,
        (1280, 800),
    );
    assert!(!words.is_empty(), "nothing was written");
    assert!(!rasters.is_empty(), "no tab was drawn under the words");
    let inside = crate::window_menu_paint::in_pixels(
        Rectangle::<i32, Logical>::new(Point::from((17, 31)), Size::from((1134, 628))),
        100,
    );
    for raster in &rasters {
        let area = crate::painted::on_the_frame(raster.area);
        assert!(!inside.overlaps(area), "{area:?} lies over the window");
    }
    let first = rasters.first().expect("the tab");
    assert!(
        crate::painted::on_the_frame(first.area).loc.y >= 28 + 634,
        "the tab is not under the foot: {:?}",
        first.area
    );
}
