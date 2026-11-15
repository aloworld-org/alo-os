#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use serde_json::Value;

use super::{LeftAsItIs, leave_typing_alone_in, with_typing_left_alone};

fn read(text: &str) -> serde_json::Map<String, Value> {
    match serde_json::from_str::<Value>(text).expect("JSON") {
        Value::Object(settings) => settings,
        other => panic!("not an object: {other}"),
    }
}

#[test]
fn every_value_alo_sets_reads_as_json() {
    for (name, value) in alo_launching::every_setting_in_alos_copy() {
        assert!(
            serde_json::from_str::<Value>(value).is_ok(),
            "{name}: {value}"
        );
    }
}

#[test]
fn a_copy_with_no_settings_is_given_every_one() {
    let written = read(&with_typing_left_alone(None).unwrap());
    assert_eq!(
        written.len(),
        alo_launching::every_setting_in_alos_copy().count()
    );
    assert_eq!(
        written.get("editor.autoIndent").cloned(),
        Some(Value::from("none"))
    );
    assert_eq!(
        written.get("editor.insertSpaces").cloned(),
        Some(Value::from(false))
    );
}

#[test]
fn what_the_person_set_in_alos_copy_is_kept_and_alos_own_are_put_back() {
    let theirs = r#"{"workbench.colorTheme": "Solarized Light", "editor.autoIndent": "full"}"#;
    let written = read(&with_typing_left_alone(Some(theirs)).unwrap());
    assert_eq!(
        written.get("workbench.colorTheme").cloned(),
        Some(Value::from("Solarized Light"))
    );
    assert_eq!(
        written.get("editor.autoIndent").cloned(),
        Some(Value::from("none"))
    );
}

#[test]
fn a_file_with_comments_or_not_an_object_is_left_as_it_is() {
    for theirs in [
        "// mine\n{\"editor.fontSize\": 14}",
        "[1, 2]",
        "\"just text\"",
    ] {
        assert_eq!(
            with_typing_left_alone(Some(theirs)),
            Err(LeftAsItIs::NotAnObject),
            "{theirs}"
        );
    }
}

#[test]
fn the_file_is_written_whole_and_private_and_a_file_left_alone_is_unchanged() {
    use std::os::unix::fs::PermissionsExt as _;
    let home = tempfile::tempdir().unwrap();
    let file = home.path().join("alos-copy/data/User/settings.json");
    leave_typing_alone_in(&file).unwrap();
    let written = read(&std::fs::read_to_string(&file).unwrap());
    assert_eq!(
        written.len(),
        alo_launching::every_setting_in_alos_copy().count()
    );
    let mode = std::fs::metadata(&file).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
    assert!(!file.with_file_name(".settings.json.alo").exists());

    let commented = "// the person's own note\n{}\n";
    std::fs::write(&file, commented).unwrap();
    assert!(leave_typing_alone_in(&file).is_err());
    assert_eq!(std::fs::read_to_string(&file).unwrap(), commented);
}

/// **alo's copy of VSCodium is told not to look for updates and not to send
/// telemetry** — the owner's of 2026-11-15, after a walk where the copy was
/// seen sending. Flatpak updates the application, so its own check costs the
/// person nothing; neither was measured sending in VSCodium 1.143.17232, and
/// these keep it that way if a later build adds an address for either.
#[test]
fn alos_copy_is_told_not_to_check_for_updates_or_send_telemetry() {
    let written = read(&with_typing_left_alone(None).unwrap());
    assert_eq!(
        written.get("update.mode").cloned(),
        Some(Value::from("none"))
    );
    assert_eq!(
        written.get("telemetry.telemetryLevel").cloned(),
        Some(Value::from("off"))
    );
}

/// **alo's copy shows no Restricted Mode banner, and its protection stays** —
/// the owner's of 2026-11-15. The banner covered the top of the editor alo
/// works in; *Restricted Mode* stays in the status bar, and a folder nobody
/// trusted still runs none of its code, because trust itself is not touched.
#[test]
fn alos_copy_hides_the_restricted_mode_banner_and_keeps_workspace_trust() {
    let written = read(&with_typing_left_alone(None).unwrap());
    assert_eq!(
        written.get("security.workspace.trust.banner").cloned(),
        Some(Value::from("never"))
    );
    for kept in [
        "security.workspace.trust.enabled",
        "security.workspace.trust.startupPrompt",
        "security.workspace.trust.untrustedFiles",
        "security.workspace.trust.emptyWindow",
    ] {
        assert!(written.get(kept).is_none(), "{kept} is left as VSCodium has it");
    }
}
