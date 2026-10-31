//! The flatpak export directories, added by flatpak's own rule.

use super::*;

/// The answer as text a test can compare.
fn said(it: &OsString) -> String {
    it.to_string_lossy().into_owned()
}

/// **The desktop's own case**: nothing in the environment but a home. Both
/// installations come first, then the specification's default.
#[test]
fn a_process_given_nothing_still_reads_both_installations() {
    let it = with_what_flatpak_exports(
        Path::new("/var/lib/flatpak"),
        Some(Path::new("/home/someone")),
        None,
        None,
    );
    assert_eq!(
        said(&it),
        "/home/someone/.local/share/flatpak/exports/share:/var/lib/flatpak/exports/share:\
         /usr/local/share:/usr/share"
    );
}

/// **The terminal's case**: a login shell already put the machine's
/// installation on, spelled with a trailing separator. It is not added twice.
#[test]
fn a_directory_already_given_is_not_given_twice() {
    let it = with_what_flatpak_exports(
        Path::new("/var/lib/flatpak"),
        Some(Path::new("/home/someone")),
        None,
        Some(OsStr::new(
            "/var/lib/flatpak/exports/share/:/usr/local/share/:/usr/share/",
        )),
    );
    assert_eq!(
        said(&it),
        "/home/someone/.local/share/flatpak/exports/share:\
         /var/lib/flatpak/exports/share/:/usr/local/share/:/usr/share/"
    );
}

/// An empty variable means unset, here as everywhere in this crate: `""` is
/// not a directory.
#[test]
fn an_empty_variable_is_the_default() {
    let it = with_what_flatpak_exports(
        Path::new("/var/lib/flatpak"),
        None,
        Some(OsStr::new("")),
        Some(OsStr::new("")),
    );
    assert_eq!(
        said(&it),
        "/var/lib/flatpak/exports/share:/usr/local/share:/usr/share"
    );
}

/// A person who set their own data directory has their flatpaks under it.
#[test]
fn the_persons_data_directory_is_where_their_installation_is() {
    let it = with_what_flatpak_exports(
        Path::new("/var/lib/flatpak"),
        Some(Path::new("/home/someone")),
        Some(OsStr::new("/data/someone")),
        Some(OsStr::new("/usr/share")),
    );
    assert_eq!(
        said(&it),
        "/data/someone/flatpak/exports/share:/var/lib/flatpak/exports/share:/usr/share"
    );
}
