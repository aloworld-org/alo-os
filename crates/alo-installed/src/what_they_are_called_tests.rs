//! What each application is called, read from entries shaped like the real ones.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::path::PathBuf;

use super::*;

/// A folder of this test's own, holding these entries.
fn a_place(called: &str, entries: &[(&str, &str)]) -> PathBuf {
    let at = std::env::temp_dir().join(format!(
        "alo-installed-names-{}-{called}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    for (named, text) in entries {
        std::fs::write(at.join(named), text).unwrap();
    }
    at
}

/// GNOME Text Editor's entry, abridged, with three translations and an action
/// whose own translated name must never be read.
const TEXT_EDITOR: &str = "[Desktop Entry]
Type=Application
Name=Text Editor
Name[fr]=Éditeur de texte
Name[pt_BR]=Editor de texto
Name[pt]=Editor de Texto
Exec=gnome-text-editor %U

[Desktop Action new-window]
Name=New Window
Name[de]=Neues Fenster
";

#[test]
fn a_person_reading_french_is_given_the_french_name() {
    let at = a_place("french", &[("org.gnome.TextEditor.desktop", TEXT_EDITOR)]);
    let called = what_they_are_called(&[at], &["fr-FR"]);
    assert_eq!(
        called.get("org.gnome.TextEditor").map(String::as_str),
        Some("Éditeur de texte")
    );
}

#[test]
fn the_most_specific_translation_wins() {
    let at = a_place("brazil", &[("org.gnome.TextEditor.desktop", TEXT_EDITOR)]);
    let called = what_they_are_called(&[at], &["pt-BR"]);
    assert_eq!(
        called.get("org.gnome.TextEditor").map(String::as_str),
        Some("Editor de texto"),
        "pt_BR is read before pt"
    );
}

#[test]
fn a_language_the_entry_does_not_speak_leaves_the_application_out() {
    let at = a_place("finnish", &[("org.gnome.TextEditor.desktop", TEXT_EDITOR)]);
    let called = what_they_are_called(&[at], &["fi"]);
    assert_eq!(
        called.get("org.gnome.TextEditor"),
        None,
        "with no translation the walk's unlocalised Name is the answer"
    );
}

#[test]
fn an_actions_translated_name_is_never_the_applications() {
    let at = a_place("action", &[("org.gnome.TextEditor.desktop", TEXT_EDITOR)]);
    let called = what_they_are_called(&[at], &["de"]);
    assert_eq!(
        called.get("org.gnome.TextEditor"),
        None,
        "Neues Fenster belongs to the right-click action, not the application"
    );
}

#[test]
fn an_entry_asking_not_to_be_listed_has_no_name_here_either() {
    let hidden =
        "[Desktop Entry]\nType=Application\nName=Secret\nName[fr]=Secret\nNoDisplay=true\n";
    let at = a_place("hidden", &[("secret.desktop", hidden)]);
    assert!(what_they_are_called(&[at], &["fr"]).is_empty());
}

#[test]
fn no_language_reads_nothing() {
    let at = a_place("none", &[("org.gnome.TextEditor.desktop", TEXT_EDITOR)]);
    assert!(what_they_are_called(&[at], &[]).is_empty());
}
