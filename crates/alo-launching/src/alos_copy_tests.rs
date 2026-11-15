//! alo's own copy: which applications have one, and how it is started.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use std::path::Path;

use super::*;
use crate::is_a_unit_of;

const HOME: &str = "/home/disan";

#[test]
fn firefox_is_started_with_a_profile_of_its_own_and_hands_over_to_nothing() {
    let copy = alos_copy("org.mozilla.firefox", Path::new(HOME)).unwrap();
    assert_eq!(
        copy.profile(),
        Path::new("/home/disan/.var/app/org.mozilla.firefox/alos-copy")
    );
    assert_eq!(
        copy.arguments(),
        [
            "--no-remote",
            "--profile",
            "/home/disan/.var/app/org.mozilla.firefox/alos-copy"
        ]
    );
}

#[test]
fn each_editor_has_a_data_folder_of_its_own() {
    for editor in ["com.vscodium.codium", "com.visualstudio.code"] {
        let copy = alos_copy(editor, Path::new(HOME)).unwrap();
        let folder = format!("/home/disan/.var/app/{editor}/alos-copy");
        assert_eq!(copy.profile(), Path::new(&folder));
        assert_eq!(
            copy.arguments(),
            [
                "--new-window".to_owned(),
                "--user-data-dir".to_owned(),
                format!("{folder}/data"),
                "--extensions-dir".to_owned(),
                format!("{folder}/extensions"),
            ]
        );
    }
}

/// **Refusal A** (the owner, 2026-11-10): an application nobody has written a
/// separate copy down for has none, so the default refuses it.
#[test]
fn an_application_with_no_way_to_a_copy_of_its_own_has_none() {
    for none in [
        "org.gnome.TextEditor",
        "org.kde.dolphin",
        "org.mozilla.Thunderbird",
        "org.mozilla.firefoxx",
        "",
    ] {
        assert_eq!(alos_copy(none, Path::new(HOME)), None, "{none}");
    }
}

/// The list is the table: every name on it has a copy.
#[test]
fn every_application_on_the_list_has_a_copy() {
    for listed in WITH_A_COPY_OF_THEIR_OWN {
        assert!(alos_copy(listed, Path::new(HOME)).is_some(), "{listed}");
    }
}

#[test]
fn alos_copy_runs_in_a_unit_of_its_own_kind() {
    assert_eq!(
        the_unit_for_alos_copy("org.mozilla.firefox", 0x2a).as_deref(),
        Some("app-alocopy-org.mozilla.firefox@2a.scope")
    );
    assert_ne!(
        the_unit_for_alos_copy("org.mozilla.firefox", 1),
        the_unit_for_alos_copy("org.mozilla.firefox", 2)
    );
}

/// **The person's own windows are not counted as alo's, nor alo's as
/// theirs**: a count of the person's windows of Firefox (ADR 0148) never takes
/// in the unit alo's copy runs in by its name.
#[test]
fn a_count_of_the_persons_windows_never_takes_in_alos_copy() {
    let unit = the_unit_for_alos_copy("org.mozilla.firefox", 0x2a).unwrap_or_default();
    assert!(!unit.is_empty());
    assert!(!is_a_unit_of(&unit, "org.mozilla.firefox"), "{unit}");
}

/// **A file is handed to a copy already running over the same profile**, and
/// nothing that would refuse the hand-over or open a second window is in it.
#[test]
fn a_running_copy_is_handed_a_file_over_its_own_profile() {
    let firefox = alos_copy("org.mozilla.firefox", Path::new(HOME)).unwrap();
    assert_eq!(
        firefox.handing_over(),
        [
            "--profile",
            "/home/disan/.var/app/org.mozilla.firefox/alos-copy"
        ]
    );
    for editor in ["com.vscodium.codium", "com.visualstudio.code"] {
        let copy = alos_copy(editor, Path::new(HOME)).unwrap();
        let folder = format!("/home/disan/.var/app/{editor}/alos-copy");
        assert_eq!(
            copy.handing_over(),
            [
                "--user-data-dir".to_owned(),
                format!("{folder}/data"),
                "--extensions-dir".to_owned(),
                format!("{folder}/extensions"),
            ]
        );
        assert!(!copy.handing_over().contains(&"--new-window".to_owned()));
    }
}
