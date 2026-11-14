//! **The side column scrolls where it does not fit, and nothing in it is
//! hidden or made smaller** — the owner's *"The side column scrolls"* of
//! 2026-11-13, held at the design's two sizes, at a larger text size and at
//! twice the display's density.
#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_appearance::TextScale;

use super::*;
use crate::display_scale::DisplayScale;
use crate::settings_raster::{SettingsLook, picture};
use crate::settings_side_column_scrolls::TheSideColumnsScroll;
use crate::settings_testing::{
    CountingDoor, a_persons_machine, doors, light, noon, settings_words,
};
use crate::{SettingsKey, SettingsPress, SettingsWindow};

/// Settings as it stands on an output of `size`, at `display`.
fn drawn(
    window: &SettingsWindow,
    size: (i32, i32),
    look: SettingsLook,
    display: DisplayScale,
) -> SettingsPicture {
    let mut labels = WindowControlLabels::new().unwrap();
    picture(
        window,
        &settings_words(),
        &mut labels,
        size,
        look,
        noon(),
        None,
        display,
    )
    .unwrap()
}

/// The ordinary look at a text size of `percent`.
fn at_text_size(percent: u16) -> SettingsLook {
    SettingsLook {
        scale: TextScale::percent(percent).unwrap(),
        ..light()
    }
}

/// Press `key` once.
fn press(window: &mut SettingsWindow, key: SettingsKey) {
    let strings = settings_words();
    let door = CountingDoor::revoking();
    window.pressed(
        SettingsPress {
            key,
            chord: None,
            filling: crate::a_key_fills_in_a_provider::FillingInKey::Nothing,
        },
        doors(&strings, &door),
    );
}

/// Whether every category is recorded, each whole in the column's view.
fn the_whole_of(drawn: &SettingsPicture, which: SettingsCategory) -> bool {
    let side = drawn.side_column.unwrap().view;
    drawn.categories.iter().any(|(category, area)| {
        *category == which
            && area.size.h == drawn.categories[0].1.size.h.max(area.size.h)
            && side.contains(area.loc)
            && area.loc.y + area.size.h <= side.loc.y + side.size.h
    })
}

/// **Where it all fits, nothing scrolls**: the design's 1440 × 960 at the
/// ordinary text size, at one and at twice the density.
#[test]
fn where_everything_fits_nothing_scrolls() {
    let machine = a_persons_machine("side-fits");
    let window = machine.opened();
    let one = drawn(&window, (1440, 960), light(), DisplayScale::ONE);
    let side = one.side_column.unwrap();
    assert_eq!((side.scrolled, side.most), (0, 0));
    assert!(!side.scrolls());
    assert_eq!(one.categories.len(), SettingsCategory::ALL.len());
    let two = drawn(
        &window,
        (2880, 1920),
        light(),
        DisplayScale::of_per_cent(200),
    );
    let side = two.side_column.unwrap();
    assert_eq!((side.scrolled, side.most), (0, 0));
}

/// **Where it does not fit, it scrolls, and End reaches the last category
/// whole** — on 1280 × 800 at a text size that makes the column taller than
/// the window, at one and at twice the density. Nothing is drawn smaller:
/// every category in view is still a whole target tall.
#[test]
fn where_it_does_not_fit_the_keyboard_scrolls_it_and_nothing_shrinks() {
    for (size, display) in [
        ((1280, 800), DisplayScale::ONE),
        ((2560, 1600), DisplayScale::of_per_cent(200)),
    ] {
        let machine = a_persons_machine("side-scrolls");
        let mut window = machine.opened();
        let look = at_text_size(150);
        let before = drawn(&window, size, look, display);
        let side = before.side_column.unwrap();
        assert!(
            side.scrolls(),
            "{size:?}: the column was expected to run over"
        );
        assert_eq!(side.scrolled, 0);
        assert!(the_whole_of(&before, SettingsCategory::AloAndAi));
        press(&mut window, SettingsKey::Last);
        let after = drawn(&window, size, look, display);
        let side = after.side_column.unwrap();
        assert!(side.scrolled > 0, "{size:?}");
        assert!(
            the_whole_of(&after, SettingsCategory::Accessibility),
            "{size:?}"
        );
        let tall = after
            .categories
            .iter()
            .map(|(_, area)| area.size.h)
            .max()
            .unwrap();
        assert_eq!(tall, before.categories[0].1.size.h, "{size:?}");
        // **Nothing recorded outside the view**, so a press can never reach a
        // category through the window's edge.
        assert!(after.categories.iter().all(|(_, area)| {
            area.loc.y >= side.view.loc.y
                && area.loc.y + area.size.h <= side.view.loc.y + side.view.size.h
        }));
        press(&mut window, SettingsKey::First);
        let back = drawn(&window, size, look, display);
        assert_eq!(back.side_column.unwrap().scrolled, 0, "{size:?}");
    }
}

/// **With Accounts as the tenth entry, 1280 × 800 at the ordinary text size
/// scrolls** — the case the owner ruled on, *"The side column scrolls"* — at
/// one and at twice the density, and the last entry is reached whole.
#[test]
fn ten_entries_scroll_at_1280_by_800_and_the_last_is_reached() {
    for (size, display) in [
        ((1280, 800), DisplayScale::ONE),
        ((2560, 1600), DisplayScale::of_per_cent(200)),
    ] {
        let machine = a_persons_machine("side-ten");
        let mut window = machine.opened();
        let before = drawn(&window, size, light(), display);
        let side = before.side_column.unwrap();
        assert!(
            side.scrolls(),
            "{size:?}: ten entries were expected to run over"
        );
        press(&mut window, SettingsKey::Last);
        let after = drawn(&window, size, light(), display);
        assert!(after.side_column.unwrap().scrolled > 0, "{size:?}");
        assert!(
            the_whole_of(&after, SettingsCategory::Accessibility),
            "{size:?}"
        );
    }
}

/// **The arithmetic**: where it fits, nothing; while following, as little as
/// brings the chosen whole into view; after a wheel, where the wheel left it;
/// and never past either end.
#[test]
fn it_scrolls_as_little_as_it_must_and_never_past_an_end() {
    let following = TheSideColumnsScroll::at_the_top();
    assert_eq!(scrolled_for(following, 500, 600, Some((450, 498))), 0);
    assert_eq!(scrolled_for(following, 900, 600, Some((700, 748))), 148);
    assert_eq!(scrolled_for(following, 900, 600, Some((100, 148))), 0);
    let mut wheeled = TheSideColumnsScroll::at_the_top();
    assert!(wheeled.wheeled(250, 300));
    // Left where the wheel put it, with the chosen out of view.
    assert_eq!(scrolled_for(wheeled, 900, 600, Some((0, 48))), 250);
    // A choice brings it back, from there, as little as it must.
    wheeled.a_category_was_chosen();
    assert_eq!(scrolled_for(wheeled, 900, 600, Some((100, 148))), 100);
    let mut far = TheSideColumnsScroll::at_the_top();
    far.was_drawn_at(10_000);
    assert_eq!(scrolled_for(far, 900, 600, None), 300);
    far.was_drawn_at(-50);
    assert_eq!(scrolled_for(far, 900, 600, None), 0);
}
