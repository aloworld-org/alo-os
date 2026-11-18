//! Real-file application permission choices, compiled only as unit-test support.

use super::*;
use crate::application_permissions::{ApplicationBeingPicked, ApplicationPermissionRow};
use crate::settings_testing::{CountingDoor, noon, words};
use alo_capability::{Ask, Grantee};
use std::time::Duration;

#[test]
fn a_person_selects_an_app_and_expiry_then_can_revoke_the_persisted_permission() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("grants.toml");
    let (mut section, refused) = GrantedSection::at_sign_in(
        &path,
        &folder.path().join("pairings.toml"),
        Some(folder.path()),
        Some("@alo"),
        noon(),
    );
    assert!(refused.is_empty());
    let installed = alo_applications::Installed::holding([alo_applications::Application::called(
        alo_adapters::studio::APPLICATION,
        "alo Studio",
    )
    .unwrap()]);
    let door = CountingDoor::revoking();
    let strings = words();
    let choose =
        ApplicationPermissionRow::Application(alo_adapters::studio::APPLICATION.to_owned());
    let duration = ApplicationPermissionRow::For(HowLong::AnHour);
    section.application_picking = Some(ApplicationBeingPicked::from_installed(&installed));
    assert_eq!(
        section.act_on_application(&duration, &door, noon(), &strings),
        SettingsGave::Nothing
    );
    assert_eq!(
        section.act_on_application(&choose, &door, noon(), &strings),
        SettingsGave::ItChanged
    );
    assert!(!path.exists());
    assert_eq!(door.knocks.get(), 0);
    assert_eq!(
        section.act_on_application(&ApplicationPermissionRow::Cancel, &door, noon(), &strings),
        SettingsGave::ItChanged
    );
    assert!(!path.exists());
    section.application_picking = Some(ApplicationBeingPicked::from_installed(&installed));
    section.act_on_application(&choose, &door, noon(), &strings);
    assert_eq!(
        section.act_on_application(&duration, &door, noon(), &strings),
        SettingsGave::ItWasGiven
    );
    assert_eq!(
        section.act_on_application(&duration, &door, noon(), &strings),
        SettingsGave::Nothing
    );
    assert_eq!(door.knocks.get(), 1);
    let agent = Grantee::named("@alo");
    let app = Ask::Application(alo_adapters::studio::APPLICATION.to_owned());
    let grants = alo_remembering::remembered(&path, noon()).unwrap();
    assert!(grants.permits(&agent, &app, noon()));
    assert!(!grants.permits(&agent, &app, noon() + Duration::from_secs(3601)));
    assert!(!grants.permits(
        &agent,
        &Ask::Application("org.example.Other".to_owned()),
        noon()
    ));
    let row = section.rows(noon()).into_iter().next().unwrap();
    section.revoked(&row, &door, noon(), &strings);
    assert!(
        !alo_remembering::remembered(&path, noon())
            .unwrap()
            .permits(&agent, &app, noon())
    );
    assert_eq!(door.knocks.get(), 2);
}
