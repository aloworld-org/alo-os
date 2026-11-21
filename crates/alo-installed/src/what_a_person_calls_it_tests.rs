//! What a person calls an application, read from entries shaped like the real
//! ones: by name, in their language, and never by its identifier where an
//! entry names it.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::path::PathBuf;

use alo_strings::{Language, Strings};

use super::*;

/// A folder of this test's own, holding these entries.
fn a_place(called: &str, entries: &[(&str, &str)]) -> PathBuf {
    let at = std::env::temp_dir().join(format!(
        "alo-installed-a-person-calls-{}-{called}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    for (named, text) in entries {
        std::fs::write(at.join(named), text).unwrap();
    }
    at
}

/// VSCodium's entry, abridged, with no translation of its name.
const VSCODIUM: &str = "[Desktop Entry]
Type=Application
Name=VSCodium
Exec=codium %F
";

/// GNOME Text Editor's entry, abridged, with a French name.
const TEXT_EDITOR: &str = "[Desktop Entry]
Type=Application
Name=Text Editor
Name[fr]=Éditeur de texte
Exec=gnome-text-editor %U
";

/// A person reading these languages, best first, with the words a list of
/// applications is shown in — so a name said as a list says it is caught.
fn reading(languages: &[&str]) -> Strings {
    let mut strings = Strings::of(alo_applications::words::application_words().unwrap());
    let languages: Vec<Language> = languages
        .iter()
        .map(|tag| Language::written(tag).unwrap())
        .collect();
    strings.prefers(&languages);
    strings
}

/// **The name its entry gives, and never the identifier** — the corner of
/// 2026-11-20 read *VSCodium (com.vscodium.codium)*, which is how a list shows
/// an application and not how a sentence names one.
#[test]
fn an_application_is_called_by_its_name_and_never_by_its_identifier() {
    let at = a_place("vscodium", &[("com.vscodium.codium.desktop", VSCODIUM)]);
    let called = what_a_person_calls_it(&[at], &reading(&["en"]), "com.vscodium.codium");
    assert_eq!(called.as_deref(), Some("VSCodium"));
    assert!(
        !called.unwrap_or_default().contains("com.vscodium.codium"),
        "the identifier was in the name"
    );
}

/// **In the person's language where the entry speaks it**, and in the entry's
/// own words where it does not.
#[test]
fn a_person_reading_french_is_given_the_french_name() {
    let at = a_place("french", &[("org.gnome.TextEditor.desktop", TEXT_EDITOR)]);
    let places = [at];
    assert_eq!(
        what_a_person_calls_it(&places, &reading(&["fr-FR"]), "org.gnome.TextEditor").as_deref(),
        Some("Éditeur de texte")
    );
    assert_eq!(
        what_a_person_calls_it(&places, &reading(&["fi"]), "org.gnome.TextEditor").as_deref(),
        Some("Text Editor")
    );
}

/// **An application no entry names is nothing here**, so the caller says its
/// identifier, which is all there is — rather than a name invented for it.
#[test]
fn an_application_no_entry_names_is_called_nothing() {
    let at = a_place("nothing", &[("com.vscodium.codium.desktop", VSCODIUM)]);
    assert_eq!(
        what_a_person_calls_it(&[at], &reading(&["en"]), "org.example.Nothing"),
        None
    );
}
