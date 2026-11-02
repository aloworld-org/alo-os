//! Settings as drawn: a press is carried back through the camera into the
//! picture and found on what it is seen on, at a pan and a zoom.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use smithay::utils::{Physical, Point, Rectangle};

use super::SettingsAsDrawn;
use crate::display_scale::DisplayScale;
use crate::settings_categories::SettingsCategory;
use crate::settings_pointed_at::PointedAt;
use crate::settings_testing::{a_persons_machine, light, noon, settings_words};

/// Settings opened on a fresh machine, laid out in the room the design's
/// 1440 × 960 display gives it, and drawn with that room at `room_on_screen`.
fn drawn(room_on_screen: Rectangle<i32, Physical>) -> SettingsAsDrawn {
    let machine = a_persons_machine("as-drawn");
    let window = machine.opened();
    let sizes = crate::settings_on_the_plane::how_big_settings_is(
        (1440, 960),
        alo_appearance::TextScale::ordinary(),
        DisplayScale::ONE,
    )
    .expect("the design's display has room for Settings");
    let mut labels = crate::WindowControlLabels::new().expect("the bundled faces");
    let picture = crate::settings_raster::picture(
        &window,
        &settings_words(),
        &mut labels,
        sizes.room,
        light(),
        noon(),
        None,
        DisplayScale::ONE,
    )
    .expect("Settings is drawn in its room");
    SettingsAsDrawn::of(&picture, room_on_screen).expect("a window was drawn")
}

/// The middle of a rectangle.
fn middle(of: Rectangle<i32, Physical>) -> Point<i32, Physical> {
    Point::from((of.loc.x + of.size.w / 2, of.loc.y + of.size.h / 2))
}

/// **At life size and unmoved, the window is where the rule puts it in its
/// room**: 1200 × 816 at the design's margin of 64 and 56.
#[test]
fn at_life_size_the_window_is_the_designs() {
    let it = drawn(Rectangle::new((0, 0).into(), (1328, 928).into()));
    assert_eq!(
        it.the_window_on_the_screen(),
        Rectangle::new((64, 56).into(), (1200, 816).into())
    );
}

/// **Zoomed to half and moved, a press finds a category where it is seen**,
/// and not where it would be at life size; a press beside the window finds
/// nothing.
#[test]
fn a_press_finds_a_category_where_the_camera_drew_it() {
    let room = Rectangle::new((100, 40).into(), (664, 464).into());
    let it = drawn(room);
    let seen = it
        .where_the_category_is(SettingsCategory::Terminal)
        .expect("the category is drawn");
    assert_eq!(
        it.what_is_under(middle(seen)),
        Some(PointedAt::ACategory(SettingsCategory::Terminal))
    );
    let life_size = drawn(Rectangle::new((0, 0).into(), (1328, 928).into()))
        .where_the_category_is(SettingsCategory::Terminal)
        .expect("the category is drawn");
    assert_ne!(
        it.what_is_under(middle(life_size)),
        Some(PointedAt::ACategory(SettingsCategory::Terminal)),
        "a press found the category where it would be unzoomed"
    );
    assert_eq!(it.what_is_under(Point::from((99, 39))), None);
    let window = it.the_window_on_the_screen();
    assert_eq!(
        (window.size.w, window.size.h),
        (600, 408),
        "half of 1200 × 816"
    );
}

/// **A row of the page is found where it is seen**, and what it is is the row
/// the page drew there.
#[test]
fn a_press_finds_a_row_of_the_page() {
    let it = drawn(Rectangle::new((0, 0).into(), (1328, 928).into()));
    let first = it
        .the_rows_in_view()
        .first()
        .cloned()
        .expect("the first page has rows in view");
    let seen = it.where_the_row_is(&first).expect("drawn");
    assert_eq!(it.what_is_under(middle(seen)), Some(PointedAt::ARow(first)));
}
