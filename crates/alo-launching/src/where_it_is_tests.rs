//! Finding an entry in real folders, and a program on a search path.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::*;

/// A folder of this test's own, holding these files.
fn a_place(called: &str, files: &[&str]) -> PathBuf {
    let at = std::env::temp_dir().join(format!("alo-launching-{}-{called}", std::process::id()));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    for file in files {
        let path = at.join(file);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, "[Desktop Entry]\n").unwrap();
    }
    at
}

#[test]
fn the_first_place_with_the_entry_wins() {
    let persons = a_place("persons", &["org.kde.dolphin.desktop"]);
    let systems = a_place("systems", &["org.kde.dolphin.desktop", "firefox.desktop"]);
    let places = [persons.clone(), systems.clone()];
    assert_eq!(
        where_the_entry_is("org.kde.dolphin", &places),
        Some(persons.join("org.kde.dolphin.desktop"))
    );
    assert_eq!(
        where_the_entry_is("firefox", &places),
        Some(systems.join("firefox.desktop"))
    );
    assert_eq!(where_the_entry_is("not-there", &places), None);
}

#[test]
fn a_dash_may_be_a_folder() {
    let place = a_place("folders", &["kde4/konsole.desktop"]);
    assert_eq!(
        where_the_entry_is("kde4-konsole", std::slice::from_ref(&place)),
        Some(place.join("kde4").join("konsole.desktop"))
    );
}

/// `..-..-etc-passwd` read as folders would be `../../etc/passwd.desktop`.
#[test]
fn an_identifier_that_would_climb_out_is_never_looked_for() {
    let place = a_place("climbing", &[]);
    for identifier in ["..-..-x", ".-x", "a/b", "", r"a\b"] {
        assert_eq!(
            where_the_entry_is(identifier, std::slice::from_ref(&place)),
            None,
            "{identifier}"
        );
    }
}

/// What the test machine can run.
fn runs(paths: &[&str]) -> impl Fn(&Path) -> bool {
    let paths: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
    move |path| paths.iter().any(|it| it == path)
}

#[test]
fn a_bare_program_is_found_on_the_search_path_in_order() {
    let can = runs(&["/usr/bin/dolphin", "/opt/bin/dolphin"]);
    assert_eq!(
        the_program("dolphin", Some(OsStr::new("/usr/local/bin:/usr/bin:/opt/bin")), &can),
        Some(PathBuf::from("/usr/bin/dolphin"))
    );
    assert_eq!(
        the_program("dolphin", None, &can),
        Some(PathBuf::from("/usr/bin/dolphin")),
        "no PATH is the usual folders"
    );
    assert_eq!(the_program("nautilus", None, &can), None);
}

#[test]
fn an_absolute_program_is_used_as_written_or_not_at_all() {
    let can = runs(&["/usr/bin/flatpak"]);
    assert_eq!(
        the_program("/usr/bin/flatpak", None, &can),
        Some(PathBuf::from("/usr/bin/flatpak"))
    );
    assert_eq!(the_program("/usr/bin/gone", None, &can), None);
}

#[test]
fn a_relative_program_and_a_relative_search_folder_are_refused() {
    let can = runs(&["bin/app", "./app"]);
    assert_eq!(the_program("bin/app", None, &can), None);
    assert_eq!(the_program("./app", None, &can), None);
    assert_eq!(the_program("app", Some(OsStr::new(".:bin")), &can), None);
}
