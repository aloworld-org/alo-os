#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use std::path::Path;

use super::{TYPED_AS_IT_IS, WITH_AN_EDITOR_THAT_REWRITES, its_settings_file};
use crate::alos_copy::{WITH_A_COPY_OF_THEIR_OWN, alos_copy};

#[test]
fn the_file_is_inside_alos_copys_own_data_folder_and_nowhere_of_the_persons() {
    let home = Path::new("/home/anna");
    let copy = alos_copy("com.vscodium.codium", home).expect("VSCodium has a copy of its own");
    let file = its_settings_file("com.vscodium.codium", &copy).expect("its editor rewrites");
    assert_eq!(
        file,
        Path::new("/home/anna/.var/app/com.vscodium.codium/alos-copy/data/User/settings.json")
    );
    assert!(file.starts_with(copy.profile()));
    // The person's own VSCodium keeps its settings under `config`, and alo's
    // copy is told its data folder is `<profile>/data`.
    assert!(!file.starts_with("/home/anna/.var/app/com.vscodium.codium/config"));
    assert!(
        copy.arguments()
            .iter()
            .any(|word| Path::new(word) == copy.profile().join("data"))
    );
}

#[test]
fn an_application_whose_editor_does_not_rewrite_is_given_no_file() {
    let home = Path::new("/home/anna");
    let firefox = alos_copy("org.mozilla.firefox", home).expect("Firefox has a copy of its own");
    assert_eq!(its_settings_file("org.mozilla.firefox", &firefox), None);
}

#[test]
fn every_editor_set_this_way_has_a_copy_of_its_own() {
    for editor in WITH_AN_EDITOR_THAT_REWRITES {
        assert!(WITH_A_COPY_OF_THEIR_OWN.contains(&editor), "{editor}");
    }
}

#[test]
fn each_setting_is_named_once_and_given_a_value() {
    let mut names: Vec<&str> = TYPED_AS_IT_IS.iter().map(|(name, _)| *name).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), TYPED_AS_IT_IS.len());
    for (name, value) in TYPED_AS_IT_IS {
        assert!(!value.trim().is_empty(), "{name}");
    }
}

#[test]
fn the_three_that_changed_the_measured_file_are_on_the_list() {
    // Indenting after a new line, closing a bracket, and a tab becoming
    // spaces were each seen in the measurement of 2026-11-13.
    for name in [
        "editor.autoIndent",
        "editor.autoClosingBrackets",
        "editor.insertSpaces",
    ] {
        assert!(
            TYPED_AS_IT_IS.iter().any(|(named, _)| *named == name),
            "{name}"
        );
    }
}
