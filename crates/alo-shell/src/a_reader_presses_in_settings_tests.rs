//! **A reader's press on Settings goes where a click on it goes**: a page in
//! the side column is shown, a row on the page shown waits to be carried out
//! as Enter carries it out, and a row heard on a page the person has since
//! left — or anything, while Settings is shut — does nothing.

use std::os::unix::fs::PermissionsExt as _;

use smithay::input::keyboard::XkbConfig;

use super::*;
use crate::settings_categories::SettingsCategory;
use crate::settings_testing::{Machine, a_browser_and_a_mail_app, a_persons_machine};
use crate::SettingsRow;
use crate::access_action::Pressed;

/// A session with Settings open on it, in front.
fn with_settings_open(named: &str) -> (tempfile::TempDir, Server, Machine) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind_keyboard(
        runtime.path(),
        named,
        XkbConfig {
            layout: "us",
            ..XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    let machine = a_persons_machine(named);
    server.the_settings_places_are(machine.places.clone());
    server.settings = machine.opened();
    server
        .settings
        .the_applications_are(a_browser_and_a_mail_app());
    server.the_person_went_to_settings();
    (runtime, server, machine)
}

/// **A page pressed in the side column is shown**, as a click shows it.
#[test]
fn a_page_pressed_is_shown() {
    let (_runtime, mut server, _machine) = with_settings_open("reader-presses-a-page");
    assert_eq!(
        server.a_reader_pressed(Pressed::InSettings(InSettings::APage(
            SettingsCategory::DefaultApps
        ))),
        WhatAReaderPressCameTo::SettingsDidIt
    );
    assert_eq!(server.settings.category(), Some(SettingsCategory::DefaultApps));
    assert!(server.a_click_waits_for_settings.is_none(), "a page is not a row to carry out");
}

/// **A row on the page shown has the keyboard moved to it and waits to be
/// carried out**, through the press a click leaves.
#[test]
fn a_row_on_the_page_shown_waits_to_be_carried_out() {
    let (_runtime, mut server, _machine) = with_settings_open("reader-presses-a-row");
    let _ = server.a_reader_pressed(Pressed::InSettings(InSettings::APage(
        SettingsCategory::DefaultApps,
    )));
    let row = server
        .settings
        .rows_on_the_page(std::time::SystemTime::now())
        .into_iter()
        .next()
        .expect("Default apps offers a row");
    assert_eq!(
        server.a_reader_pressed(Pressed::InSettings(InSettings::ARow(row.clone()))),
        WhatAReaderPressCameTo::SettingsDidIt
    );
    assert_eq!(server.settings.focused(std::time::SystemTime::now()), Some(row));
    assert!(server.a_click_waits_for_settings.is_some());
}

/// **A row heard on another page does nothing**, and nothing waits.
#[test]
fn a_row_not_on_the_page_shown_does_nothing() {
    let (_runtime, mut server, _machine) = with_settings_open("reader-presses-elsewhere");
    let _ = server.a_reader_pressed(Pressed::InSettings(InSettings::APage(
        SettingsCategory::Accessibility,
    )));
    assert_eq!(
        server.a_reader_pressed(Pressed::InSettings(InSettings::ARow(SettingsRow::DefaultFor(
            alo_applications::LinkKind::Web,
            "org.mozilla.firefox".to_owned(),
        )))),
        WhatAReaderPressCameTo::NothingInSettings
    );
    assert!(server.a_click_waits_for_settings.is_none());
}

/// **Nothing reaches a Settings that is shut.**
#[test]
fn nothing_reaches_a_settings_that_is_shut() {
    let (_runtime, mut server, _machine) = with_settings_open("reader-presses-shut");
    server.settings.close();
    assert_eq!(
        server.a_reader_pressed(Pressed::InSettings(InSettings::APage(
            SettingsCategory::DefaultApps
        ))),
        WhatAReaderPressCameTo::NothingInSettings
    );
    assert!(server.a_click_waits_for_settings.is_none());
}
