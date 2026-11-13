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
    for (name, value) in alo_launching::TYPED_AS_IT_IS {
        assert!(
            serde_json::from_str::<Value>(value).is_ok(),
            "{name}: {value}"
        );
    }
}

#[test]
fn a_copy_with_no_settings_is_given_every_one() {
    let written = read(&with_typing_left_alone(None).unwrap());
    assert_eq!(written.len(), alo_launching::TYPED_AS_IT_IS.len());
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
    assert_eq!(written.len(), alo_launching::TYPED_AS_IT_IS.len());
    let mode = std::fs::metadata(&file).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
    assert!(!file.with_file_name(".settings.json.alo").exists());

    let commented = "// the person's own note\n{}\n";
    std::fs::write(&file, commented).unwrap();
    assert!(leave_typing_alone_in(&file).is_err());
    assert_eq!(std::fs::read_to_string(&file).unwrap(), commented);
}
