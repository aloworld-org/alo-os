//! **Every application a fresh machine ships is seen as installed** once
//! `alo-shipping` has put it there — by the list the agent's verbs are held to,
//! and by every name a person or a model says it by.
//!
//! The walk of 2026-11-14 is why: the first boot installed VSCodium as
//! `com.vscodium.codium`, `alo-app list` showed it, and alo said *vscodium is
//! not installed* and then installed something from Flathub. This reads the
//! decided list (`shipped.toml`, through `Shipped::decided`), writes each
//! application's desktop entry **where flatpak's machine-wide installation
//! exports it** — `alo-shipping` installs with `--system`, into
//! `/var/lib/flatpak` — and reads the list back the way `alo-agentd` does:
//! `alo_installed::with_what_flatpak_exports` and `where_entries_live`, then
//! `alo_installed::read`.
//!
//! **What this does not show.** The names in the entries below are this test's
//! own, except VSCodium's, which is the name its Flathub entry gives. Whether a
//! booted machine's agent reaches them is a walk's to say.
//!
//! Unix only: a search path is joined with `:`, which a Windows path holds.

#![cfg(unix)]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::Path;

use alo_applications::short_name::short_name;
use alo_applications::{AnsweringTo, Installed};
use alo_software::Shipped;
use tempfile::TempDir;

/// The name an entry gives: VSCodium's as Flathub's entry gives it, and the
/// identifier's own last part for the rest.
fn named(identifier: &str) -> String {
    if identifier == "com.vscodium.codium" {
        "VSCodium".to_owned()
    } else {
        short_name(identifier)
    }
}

/// Write one desktop entry, as flatpak exports it, under `share`.
fn exported(share: &Path, identifier: &str) {
    let applications = share.join("applications");
    fs::create_dir_all(&applications).unwrap();
    fs::write(
        applications.join(format!("{identifier}.desktop")),
        format!(
            "[Desktop Entry]\nType=Application\nName={}\nExec=/usr/bin/flatpak run {identifier}\n",
            named(identifier)
        ),
    )
    .unwrap();
}

/// What `alo-agentd` reads on a machine whose installations are under `root`:
/// both flatpak installations, and nothing of the machine running this test.
fn as_the_daemon_reads_it(root: &Path) -> Installed {
    let machines = root.join("var/lib/flatpak");
    let home = root.join("home/alo");
    let nothing_else = root.join("usr/share");
    let data_dirs = alo_installed::with_what_flatpak_exports(
        &machines,
        Some(home.as_path()),
        None,
        Some(nothing_else.as_os_str()),
    );
    let places =
        alo_installed::where_entries_live(Some(home.as_path()), None, Some(data_dirs.as_os_str()));
    alo_installed::read(&places)
}

/// Every identifier the decided list names.
fn shipped() -> Vec<String> {
    Shipped::decided()
        .unwrap()
        .every()
        .iter()
        .map(|pinned| pinned.application().identifier().to_owned())
        .collect()
}

/// The identifier a name answers to, or what went wrong.
fn answered(installed: &Installed, said: &str) -> Result<String, String> {
    match installed.answering_to(said) {
        AnsweringTo::One(application) => Ok(application.identifier().to_owned()),
        AnsweringTo::Several(several) => Err(format!("{said} named {}", several.len())),
        AnsweringTo::Nothing => Err(format!("{said} named nothing")),
    }
}

/// **Every shipped application, installed machine-wide, is on the list** and
/// answers to its identifier in any case and to its short name.
#[test]
fn every_shipped_application_installed_machine_wide_is_seen() {
    let root = TempDir::new().unwrap();
    let identifiers = shipped();
    assert_eq!(identifiers.len(), 8, "{identifiers:?}");
    for identifier in &identifiers {
        exported(
            &root.path().join("var/lib/flatpak/exports/share"),
            identifier,
        );
    }
    let installed = as_the_daemon_reads_it(root.path());
    assert_eq!(
        installed.how_many(),
        8,
        "{:?}",
        installed.all().collect::<Vec<_>>()
    );
    for identifier in &identifiers {
        assert!(installed.has(identifier), "{identifier} is not installed");
        for said in [
            identifier.clone(),
            identifier.to_lowercase(),
            identifier.to_uppercase(),
            short_name(identifier),
        ] {
            assert_eq!(
                answered(&installed, &said),
                Ok(identifier.clone()),
                "{said}"
            );
        }
    }
}

/// **The walk's two requests**: VSCodium by the name the person said.
#[test]
fn vscodium_answers_to_the_name_the_person_said() {
    let root = TempDir::new().unwrap();
    for identifier in shipped() {
        exported(
            &root.path().join("var/lib/flatpak/exports/share"),
            &identifier,
        );
    }
    let installed = as_the_daemon_reads_it(root.path());
    for said in ["VSCodium", "vscodium", "codium", "com.vscodium.codium"] {
        assert_eq!(
            answered(&installed, said),
            Ok("com.vscodium.codium".to_owned()),
            "{said}"
        );
    }
}

/// **The person's own installation is read too**, beside the machine's: an
/// application installed into either is on the list.
#[test]
fn both_flatpak_installations_are_read() {
    let root = TempDir::new().unwrap();
    exported(
        &root.path().join("var/lib/flatpak/exports/share"),
        "org.gnome.TextEditor",
    );
    exported(
        &root
            .path()
            .join("home/alo/.local/share/flatpak/exports/share"),
        "com.vscodium.codium",
    );
    let installed = as_the_daemon_reads_it(root.path());
    assert!(installed.has("org.gnome.TextEditor"));
    assert!(installed.has("com.vscodium.codium"));
}
