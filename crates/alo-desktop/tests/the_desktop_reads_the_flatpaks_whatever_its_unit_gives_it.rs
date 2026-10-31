//! **The desktop finds the machine's flatpaks with the environment its shipped
//! unit gives it.**
//!
//! On 2026-10-31 a booted machine's terminal printed
//! `XDG_DATA_DIRS=/var/lib/flatpak/exports/share:/usr/local/share/:/usr/share/`
//! — the login shell's `flatpak.sh` adding it — while the desktop, a system
//! unit run as the person, is given no such line: its unit sets none and its
//! PAM stack has no `pam_env`. Every flatpak's desktop entry lives under that
//! directory, so a desktop that trusted its environment could find none of them.
//!
//! This reads the shipped unit's own `Environment=` lines, builds the
//! environment they give, and asks the readers `alo-desktop` uses where they
//! would look. **The unit is read, not restated**, so a unit that one day sets
//! `XDG_DATA_DIRS` to something without the export directory still has it
//! looked in — and a test that hand-typed the environment would go on passing
//! against a unit nobody ships.
//!
//! What this cannot see is a variable systemd or PAM adds that the unit does
//! not name. That is measured on a machine, not here; the next walk is where.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The desktop's shipped unit, from this repository.
fn the_shipped_unit() -> String {
    let at = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../image/usr/lib/systemd/system/alo-desktop@.service");
    std::fs::read_to_string(&at).expect("the desktop's unit is where the image keeps it")
}

/// Every variable the unit's `Environment=` lines set.
fn what_the_unit_sets(unit: &str) -> BTreeMap<String, String> {
    unit.lines()
        .filter_map(|line| line.trim().strip_prefix("Environment="))
        .filter_map(|set| set.split_once('='))
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect()
}

/// The entries and the pictures are looked for under both flatpak
/// installations and under `/usr/share`, from the unit's own environment.
#[test]
fn the_desktop_looks_where_flatpak_and_the_image_put_applications() {
    let set = what_the_unit_sets(&the_shipped_unit());
    assert!(
        set.contains_key("ALO_KEYBOARD"),
        "the control: the unit's Environment= lines were read at all ({set:?})"
    );
    let home = PathBuf::from("/home/someone");
    let data_home = set.get("XDG_DATA_HOME").map(OsStr::new);
    let data_dirs = set.get("XDG_DATA_DIRS").map(OsStr::new);

    let with = alo_installed::with_what_flatpak_exports(
        Path::new(alo_installed::THE_MACHINES_INSTALLATION),
        Some(&home),
        data_home,
        data_dirs,
    );
    let entries = alo_installed::where_entries_live(Some(&home), data_home, Some(&with));
    for wanted in [
        "/var/lib/flatpak/exports/share/applications",
        "/home/someone/.local/share/flatpak/exports/share/applications",
        "/usr/share/applications",
    ] {
        assert!(
            entries.iter().any(|it| it == Path::new(wanted)),
            "the desktop would not look in {wanted}: {entries:?}"
        );
    }
    let icons = alo_icons::where_icons_live(Some(&home), data_home, Some(&with));
    assert!(
        icons
            .iter()
            .any(|it| it == Path::new("/var/lib/flatpak/exports/share/icons")),
        "the desktop would find a flatpak's entry and not its picture: {icons:?}"
    );

    // **And the reason this exists, held as a fact**: the unit gives none of
    // it. A unit that started setting `XDG_DATA_DIRS` would make this a
    // belt-and-braces rule rather than the only one, and is worth knowing.
    let bare = alo_installed::where_entries_live(Some(&home), data_home, data_dirs);
    assert!(
        !bare
            .iter()
            .any(|it| it == Path::new("/var/lib/flatpak/exports/share/applications")),
        "the shipped unit now gives the flatpak directory itself; this file's header is stale"
    );
}
