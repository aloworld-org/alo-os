//! Which applications open folders, read from entries shaped like the real ones.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::path::PathBuf;

use super::*;

/// A folder of this test's own, holding these entries.
fn a_place(called: &str, entries: &[(&str, &str)]) -> PathBuf {
    let at = std::env::temp_dir().join(format!(
        "alo-installed-folders-{}-{called}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    for (named, text) in entries {
        std::fs::write(at.join(named), text).unwrap();
    }
    at
}

/// Dolphin's entry, abridged: it opens folders.
const DOLPHIN: &str = "[Desktop Entry]
Type=Application
Name=Dolphin
Exec=dolphin %u
MimeType=inode/directory;
Categories=Qt;KDE;System;FileManager;
";

/// Papers' entry, abridged: it opens documents, and no folder.
const PAPERS: &str = "[Desktop Entry]
Type=Application
Name=Papers
Exec=papers %U
MimeType=application/pdf;
";

/// **What opens folders is what says so**, and nothing else is.
#[test]
fn an_application_that_opens_folders_is_one_and_others_are_not() {
    let place = a_place(
        "says-so",
        &[
            ("org.kde.dolphin.desktop", DOLPHIN),
            ("org.gnome.Papers.desktop", PAPERS),
        ],
    );
    let opening = what_opens_folders(std::slice::from_ref(&place));
    assert!(opening.contains("org.kde.dolphin"), "{opening:?}");
    assert!(!opening.contains("org.gnome.Papers"), "{opening:?}");
}

/// **The refusal cases**: a category alone does not make a file manager, a
/// person's own copy that stopped declaring folders wins over the system's,
/// and a place that is not there answers nothing.
#[test]
fn only_the_declaration_counts_and_the_first_entry_wins() {
    let only_a_category = "[Desktop Entry]
Type=Application
Name=Something
Categories=FileManager;
";
    let place = a_place(
        "category",
        &[("org.example.Something.desktop", only_a_category)],
    );
    assert!(what_opens_folders(std::slice::from_ref(&place)).is_empty());

    let theirs = a_place(
        "theirs",
        &[(
            "org.kde.dolphin.desktop",
            "[Desktop Entry]\nType=Application\nName=Dolphin\nMimeType=text/plain;\n",
        )],
    );
    let system = a_place("system", &[("org.kde.dolphin.desktop", DOLPHIN)]);
    assert!(what_opens_folders(&[theirs, system]).is_empty());

    assert!(what_opens_folders(&[PathBuf::from("/no/such/place/at/all")]).is_empty());
}
