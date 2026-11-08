//! Left and Right along every row of choices on the Appearance page, and
//! nowhere else: the owner's *"Yes, also Left/Right"* of 2026-11-08.

use crate::settings_categories::SettingsCategory;
use crate::settings_pointed_at::PointedAt;
use crate::settings_testing::{CountingDoor, a_persons_machine, bytes, doors, noon, words};
use crate::{SettingsKey, SettingsPress, SettingsRow, SettingsWindow};

/// Left or Right, as the seat makes it.
fn press(window: &mut SettingsWindow, key: SettingsKey, door: &CountingDoor) {
    let strings = words();
    let did = window.pressed(
        SettingsPress {
            key,
            chord: None,
            filling: crate::a_key_fills_in_a_provider::FillingInKey::Nothing,
        },
        doors(&strings, door),
    );
    assert_eq!(did, None, "{key:?} did something besides moving");
}

/// The keyboard on `row`, as a press would put it there, without choosing it.
fn on(window: &mut SettingsWindow, row: &SettingsRow) {
    let _ = window.pointed_at(&PointedAt::ARow(row.clone()), noon());
    assert_eq!(window.focused(noon()).as_ref(), Some(row), "the control");
}

/// Every file the Appearance page keeps, as it is now.
fn the_files(machine: &crate::settings_testing::Machine) -> Vec<Option<Vec<u8>>> {
    [alo_appearance::keeping::THE_FILE, alo_dock::keeping::THE_FILE]
        .into_iter()
        .map(|file| bytes(&machine.kept(file)))
        .collect()
}

/// **Each row of choices on Appearance is walked by Right to its end and by
/// Left back to its start**, and no further either way — light and dark, the
/// accent's colours and where the Dock sits — writing nothing.
#[test]
fn right_and_left_walk_each_row_of_choices_on_appearance_and_stop_at_its_ends() {
    let machine = a_persons_machine("along-appearance");
    let door = CountingDoor::revoking();
    let mut window = machine.opened();
    let _ = window.pointed_at(&PointedAt::ACategory(SettingsCategory::Appearance), noon());
    let before = the_files(&machine);
    let rows: [Vec<SettingsRow>; 3] = [
        crate::settings_window::LightOrDark::ALL
            .into_iter()
            .map(SettingsRow::LightOrDark)
            .collect(),
        alo_appearance::Accent::ALL
            .into_iter()
            .map(SettingsRow::Accent)
            .collect(),
        alo_dock::Edge::EVERY
            .into_iter()
            .map(SettingsRow::Edge)
            .collect(),
    ];
    for row in &rows {
        assert!(row.len() > 1, "a row of one choice cannot show this");
        let Some(first) = row.first() else {
            continue;
        };
        on(&mut window, first);
        press(&mut window, SettingsKey::Less, &door);
        assert_eq!(window.focused(noon()).as_ref(), Some(first), "Left at the start");
        for part in row.iter().skip(1) {
            press(&mut window, SettingsKey::More, &door);
            assert_eq!(window.focused(noon()).as_ref(), Some(part), "Right");
        }
        press(&mut window, SettingsKey::More, &door);
        assert_eq!(
            window.focused(noon()).as_ref(),
            row.last(),
            "Right left the row at its end"
        );
        for part in row.iter().rev().skip(1) {
            press(&mut window, SettingsKey::Less, &door);
            assert_eq!(window.focused(noon()).as_ref(), Some(part), "Left");
        }
    }
    assert_eq!(the_files(&machine), before, "walking a row wrote a file");
    assert_eq!(door.knocks.get(), 0);
}

/// **The refusal path: Left and Right do nothing on a row that is not a row
/// of choices** — the title switch, a ring of the no-room choice, a ring of
/// the Dock's hiding and Reduced motion — and write nothing.
#[test]
fn left_and_right_do_nothing_on_a_row_that_is_not_a_row_of_choices() {
    let machine = a_persons_machine("along-nowhere");
    let door = CountingDoor::revoking();
    let mut window = machine.opened();
    let before = the_files(&machine);
    let access = machine.kept(alo_access::keeping::THE_FILE);
    let access_before = bytes(&access);
    let mut elsewhere = vec![SettingsRow::WindowTitles];
    elsewhere.extend(
        alo_appearance::WhenThereIsNoRoom::ALL
            .into_iter()
            .map(SettingsRow::WhenThereIsNoRoom),
    );
    elsewhere.extend(
        alo_dock::Hiding::ALL
            .into_iter()
            .map(SettingsRow::Hiding),
    );
    let _ = window.pointed_at(&PointedAt::ACategory(SettingsCategory::Appearance), noon());
    for row in &elsewhere {
        on(&mut window, row);
        for key in [SettingsKey::More, SettingsKey::Less] {
            press(&mut window, key, &door);
            assert_eq!(window.focused(noon()).as_ref(), Some(row), "{key:?} moved");
        }
    }
    let _ = window.pointed_at(
        &PointedAt::ACategory(SettingsCategory::Accessibility),
        noon(),
    );
    on(&mut window, &SettingsRow::ReducedMotion);
    for key in [SettingsKey::More, SettingsKey::Less] {
        press(&mut window, key, &door);
        assert_eq!(
            window.focused(noon()),
            Some(SettingsRow::ReducedMotion),
            "{key:?} moved"
        );
    }
    assert_eq!(window.motion_is_reduced(), Some(false));
    assert_eq!(the_files(&machine), before);
    assert_eq!(bytes(&access), access_before);
}
