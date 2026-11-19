//! **A reader's press on Settings goes where a click on it goes**: a page in
//! the side column is shown, a row on the page shown waits to be carried out
//! as Enter carries it out, and a row heard on a page the person has since
//! left — or anything, while Settings is shut — does nothing.

use std::os::unix::fs::PermissionsExt as _;
use std::time::{Duration, SystemTime};

use alo_capability::{Ask, Grant, GrantId, Grantee, Grants, Reach};
use alo_changing::Row;
use smithay::input::keyboard::XkbConfig;

use super::*;
use crate::SettingsRow;
use crate::access_action::Pressed;
use crate::settings_categories::SettingsCategory;
use crate::settings_testing::{Machine, a_browser_and_a_mail_app, a_persons_machine};

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
    assert_eq!(
        server.settings.category(),
        Some(SettingsCategory::DefaultApps)
    );
    assert!(
        server.a_click_waits_for_settings.is_none(),
        "a page is not a row to carry out"
    );
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
    assert_eq!(
        server.settings.focused(std::time::SystemTime::now()),
        Some(row)
    );
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
        server.a_reader_pressed(Pressed::InSettings(InSettings::ARow(
            SettingsRow::DefaultFor(
                alo_applications::LinkKind::Web,
                "org.mozilla.firefox".to_owned(),
            )
        ))),
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

/// The separately installed application the private recording walk grants.
const STUDIO: &str = "org.aloworld.Studio";

/// A real timed application grant, heard on the page before the reader presses.
/// The clock difference is explicit; this fixture never sleeps to age a row.
fn with_a_timed_application_heard(
    named: &str,
    heard_at: SystemTime,
) -> (tempfile::TempDir, Server, Machine, SettingsRow, GrantId) {
    let (runtime, mut server, machine) = with_settings_open(named);
    let mut grants = Grants::default();
    let id = grants.grant(
        Grant::checked(
            "alo",
            Reach::Application(STUDIO.to_owned()),
            heard_at - Duration::from_secs(2),
            Duration::from_secs(60 * 60),
        )
        .expect("a bounded application grant"),
    );
    alo_remembering::kept(&machine.grants(), &grants, heard_at)
        .expect("the fixture's real grants file");
    server.settings = crate::SettingsWindow::closed();
    let opened = server.settings.opened_by_hand(&machine.places, heard_at);
    assert!(opened.not_remembered.is_empty(), "{opened:?}");
    let _ = server.a_reader_pressed(Pressed::InSettings(InSettings::APage(
        SettingsCategory::FoldersAloCanUse,
    )));
    let heard = server
        .settings
        .rows_on_the_page(heard_at)
        .into_iter()
        .find(|row| matches!(row, SettingsRow::Granted(Row::Grant(seen)) if seen.id() == id))
        .expect("the live grant is a row the reader can hear");
    (runtime, server, machine, heard, id)
}

/// **An hour grant stays the same target as its remaining time decreases.**
/// The real reader dispatch and deferred click must revoke it on disk once.
#[test]
fn a_reader_revokes_the_same_timed_grant_heard_earlier() {
    let heard_at = SystemTime::now() - Duration::from_secs(1);
    let (_runtime, mut server, machine, heard, _) =
        with_a_timed_application_heard("reader-timed-application", heard_at);
    let before = alo_remembering::remembered(&machine.grants(), SystemTime::now())
        .expect("the live application grant");
    assert!(before.permits(
        &Grantee::named("alo"),
        &Ask::application(STUDIO),
        SystemTime::now()
    ));

    assert_eq!(
        server.a_reader_pressed(Pressed::InSettings(InSettings::ARow(heard))),
        WhatAReaderPressCameTo::SettingsDidIt,
        "a live grant does not become another target as its clock counts down"
    );
    assert!(server.a_click_waits_for_settings.is_some());
    crate::a_key_reaches_settings::a_click_is_carried_out(
        &mut server,
        &crate::settings_testing::settings_words(),
    );
    assert!(server.a_click_waits_for_settings.is_none());
    let after = alo_remembering::remembered(&machine.grants(), SystemTime::now())
        .expect("the grants file after the reader's actual revocation");
    assert!(!after.permits(
        &Grantee::named("alo"),
        &Ask::application(STUDIO),
        SystemTime::now()
    ));
    assert!(!server.settings.rows_on_the_page(SystemTime::now()).iter().any(
        |row| matches!(row, SettingsRow::Granted(Row::Grant(seen)) if seen.over() == &Reach::Application(STUDIO.to_owned()))
    ));
}

/// **An expired row cannot be activated**, even though it was live when heard.
#[test]
fn a_reader_cannot_choose_a_timed_grant_that_expired_after_it_was_heard() {
    let heard_at = SystemTime::now() - Duration::from_secs(2 * 60 * 60);
    let (_runtime, mut server, machine, heard, _) =
        with_a_timed_application_heard("reader-expired-application", heard_at);
    let before = std::fs::read(machine.grants()).expect("the expired grant file");
    assert_eq!(
        server.a_reader_pressed(Pressed::InSettings(InSettings::ARow(heard))),
        WhatAReaderPressCameTo::NothingInSettings
    );
    assert!(server.a_click_waits_for_settings.is_none());
    crate::a_key_reaches_settings::a_click_is_carried_out(
        &mut server,
        &crate::settings_testing::settings_words(),
    );
    assert_eq!(
        std::fs::read(machine.grants()).expect("the unchanged grant file"),
        before
    );
}

/// **The same application name is not the same grant.** A late reader press
/// must not revoke a newly granted replacement after the original is gone.
#[test]
fn a_reader_cannot_revoke_a_replacement_for_the_timed_grant_it_heard() {
    let heard_at = SystemTime::now() - Duration::from_secs(1);
    let (_runtime, mut server, machine, heard, original) =
        with_a_timed_application_heard("reader-replaced-application", heard_at);
    let now = SystemTime::now();
    let mut grants = alo_remembering::remembered(&machine.grants(), now)
        .expect("the original application grant");
    assert!(grants.revoke(original));
    let replacement = grants.grant(
        Grant::checked(
            "alo",
            Reach::Application(STUDIO.to_owned()),
            now,
            Duration::from_secs(60 * 60),
        )
        .expect("the replacement application grant"),
    );
    assert_ne!(replacement, original, "grant handles must not be reused");
    alo_remembering::kept(&machine.grants(), &grants, now).expect("the replacement on disk");
    server
        .settings
        .open_mut()
        .expect("Settings is open")
        .granted
        .the_grants_are_read_again(now);
    let before = std::fs::read(machine.grants()).expect("the replacement grant file");

    assert_eq!(
        server.a_reader_pressed(Pressed::InSettings(InSettings::ARow(heard))),
        WhatAReaderPressCameTo::NothingInSettings
    );
    assert!(server.a_click_waits_for_settings.is_none());
    crate::a_key_reaches_settings::a_click_is_carried_out(
        &mut server,
        &crate::settings_testing::settings_words(),
    );
    assert_eq!(
        std::fs::read(machine.grants()).expect("the preserved replacement"),
        before
    );
    let after = alo_remembering::remembered(&machine.grants(), SystemTime::now())
        .expect("the replacement is still usable");
    assert!(after.permits(
        &Grantee::named("alo"),
        &Ask::application(STUDIO),
        SystemTime::now()
    ));
}
