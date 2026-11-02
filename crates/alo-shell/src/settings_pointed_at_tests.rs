//! What a press on Settings does to where the keyboard is: a category shows its
//! page, a row and an entry are then chosen as Enter chooses them, and a press
//! beside an open list closes it.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::{PointedAt, ThenChoose};
use crate::settings_categories::SettingsCategory;
use crate::settings_picker::SettingsPicker;
use crate::settings_testing::{CountingDoor, a_persons_machine, doors, noon, settings_words};
use crate::{SettingsKey, SettingsPress, SettingsRow, SettingsWindow};

/// Enter, as a key gives it.
fn enter() -> SettingsPress {
    SettingsPress {
        key: SettingsKey::Choose,
        chord: None,
        filling: crate::a_key_fills_in_a_provider::FillingInKey::Nothing,
    }
}

/// **A category shows its page, with the keyboard on the category**, as Up and
/// Down choose a page — and nothing is chosen.
#[test]
fn a_category_shows_its_page() {
    let machine = a_persons_machine("pointed-category");
    let mut window = machine.opened();
    assert_eq!(
        window.pointed_at(&PointedAt::ACategory(SettingsCategory::Appearance), noon()),
        ThenChoose::No
    );
    assert_eq!(window.category(), Some(SettingsCategory::Appearance));
    assert!(!window.is_in_the_page());
}

/// **A row takes the keyboard and is then chosen**: the keyboard is on that
/// very row of the page.
#[test]
fn a_row_takes_the_keyboard_and_is_chosen() {
    let machine = a_persons_machine("pointed-row");
    let mut window = machine.opened();
    let _ = window.pointed_at(&PointedAt::ACategory(SettingsCategory::RunningCode), noon());
    let last = window
        .rows_on_the_page(noon())
        .last()
        .cloned()
        .expect("the page has rows");
    assert_eq!(
        window.pointed_at(&PointedAt::ARow(last.clone()), noon()),
        ThenChoose::Yes
    );
    assert!(window.is_in_the_page());
    assert_eq!(window.focused(noon()), Some(last));
}

/// **The refusal: a row not on the page shown is not chosen**, and the
/// keyboard stays where it was.
#[test]
fn a_row_from_another_page_is_not_chosen() {
    let machine = a_persons_machine("pointed-elsewhere");
    let mut window = machine.opened();
    let _ = window.pointed_at(&PointedAt::ACategory(SettingsCategory::RunningCode), noon());
    assert_eq!(
        window.pointed_at(&PointedAt::ARow(SettingsRow::GrantAFolder), noon()),
        ThenChoose::No
    );
    assert!(!window.is_in_the_page(), "the keyboard moved to nothing");
}

/// **A press beside an open list closes it**, as Escape does, and Settings
/// stays open.
#[test]
fn a_press_beside_an_open_list_closes_it() {
    let machine = a_persons_machine("pointed-list");
    let mut window = machine.opened();
    let _ = window.pointed_at(
        &PointedAt::ACategory(SettingsCategory::RegionAndTime),
        noon(),
    );
    let picker = SettingsRow::OpenThePicker(SettingsPicker::Region);
    assert_eq!(
        window.pointed_at(&PointedAt::ARow(picker), noon()),
        ThenChoose::Yes
    );
    let strings = settings_words();
    let door = CountingDoor::revoking();
    let _ = window.pressed(enter(), doors(&strings, &door));
    assert!(
        window.open().is_some_and(|open| open.picker.is_some()),
        "the control: choosing the row opened its list"
    );
    let _ = window.pointed_at(&PointedAt::OutsideTheList, noon());
    assert!(window.open().is_some_and(|open| open.picker.is_none()));
    assert!(window.is_open());
}

/// **A shut window does nothing.**
#[test]
fn a_shut_window_does_nothing() {
    let mut window = SettingsWindow::closed();
    assert_eq!(
        window.pointed_at(&PointedAt::ACategory(SettingsCategory::Terminal), noon()),
        ThenChoose::No
    );
    assert!(!window.is_open());
}
