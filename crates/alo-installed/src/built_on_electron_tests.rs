//! Packages laid out as flatpak lays them out, each with the manifest its
//! build left in it, read back for whether it is built on Electron.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use super::*;

/// The start of a manifest as `flatpak-builder` writes it into a package,
/// for an application built on Electron's base. **Written in the shape of one,
/// not copied from a machine.**
const ON_ELECTRON: &str = "{\n  \"id\" : \"org.example.Chat\",\n  \"runtime\" : \"org.freedesktop.Platform\",\n  \"base\" : \"org.electronjs.Electron2.BaseApp\",\n  \"base-version\" : \"26.08\",\n  \"command\" : \"chat\"\n}\n";

/// One built on another base, which is not Electron's.
const ON_ANOTHER_BASE: &str = "{\n  \"id\" : \"org.example.Game\",\n  \"base\" : \"org.godotengine.godot.BaseApp\",\n  \"base-version\" : \"org.electronjs.Electron2.BaseApp\"\n}\n";

/// One built on no base at all, which names Electron only in passing.
const ON_NO_BASE: &str = "{\n  \"id\" : \"org.gnome.TextEditor\",\n  \"x-note\" : \"not org.electronjs.Electron2.BaseApp\"\n}\n";

/// A directory of the test's own, emptied.
fn a_place(named: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!(
        "alo-installed-electron-{}-{named}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    at
}

/// Install a package's manifest where flatpak keeps the package's files.
fn a_package(installation: &Path, identifier: &str, manifest: &str) {
    let at = installation
        .join("app")
        .join(identifier)
        .join("current")
        .join("active")
        .join("files");
    std::fs::create_dir_all(&at).unwrap();
    std::fs::write(at.join("manifest.json"), manifest).unwrap();
}

/// **A package whose manifest names Electron's base is built on Electron**,
/// in the machine's installation or the person's own.
#[test]
fn a_package_on_electrons_base_is_built_on_electron() {
    let machines = a_place("machines");
    let persons = a_place("persons");
    a_package(&machines, "org.example.Chat", ON_ELECTRON);
    a_package(&persons, "org.example.Notes", ON_ELECTRON);
    assert!(is_built_on_electron(&machines, Some(&persons), "org.example.Chat"));
    assert!(is_built_on_electron(&machines, Some(&persons), "org.example.Notes"));
}

/// **Nothing that is not Electron's is read as Electron**: another base, no
/// base, a name only in passing or in another key, no manifest, and no package.
#[test]
fn nothing_that_is_not_on_electrons_base_is_read_as_electron() {
    let machines = a_place("not-electron");
    a_package(&machines, "org.example.Game", ON_ANOTHER_BASE);
    a_package(&machines, "org.gnome.TextEditor", ON_NO_BASE);
    std::fs::create_dir_all(
        machines
            .join("app")
            .join("org.kde.dolphin")
            .join("current")
            .join("active")
            .join("files"),
    )
    .unwrap();
    for identifier in [
        "org.example.Game",
        "org.gnome.TextEditor",
        "org.kde.dolphin",
        "org.example.NotInstalled",
    ] {
        assert!(
            !is_built_on_electron(&machines, None, identifier),
            "{identifier} was read as built on Electron"
        );
    }
}

/// **An identifier that could name another place is not looked for at all**,
/// even where that place holds a manifest naming Electron.
#[test]
fn an_identifier_that_names_another_place_is_never_read() {
    let machines = a_place("escape");
    a_package(&machines, "org.example.Chat", ON_ELECTRON);
    a_package(&machines, ".hidden", ON_ELECTRON);
    // **Where each escape would land, a manifest naming Electron**, so a guard
    // that let one through would answer yes: `""` reads `app/current/…`, and
    // `".."` reads `current/…` beside `app`.
    for landing in [machines.join("app"), machines.clone()] {
        let files = landing.join("current").join("active").join("files");
        std::fs::create_dir_all(&files).unwrap();
        std::fs::write(files.join("manifest.json"), ON_ELECTRON).unwrap();
    }
    for identifier in [
        "",
        ".",
        "..",
        "../app/org.example.Chat",
        "org.example.Chat/../org.example.Chat",
        ".hidden",
    ] {
        assert!(
            !is_built_on_electron(&machines, None, identifier),
            "{identifier:?} was read"
        );
    }
}

/// **The words are read as a key and its value, not as a phrase anywhere**:
/// spacing as `flatpak-builder` writes it and as a compact writer does both
/// read, and `base-version` holding the name is not a base.
#[test]
fn only_the_base_key_with_electrons_name_says_electron() {
    assert!(says_its_base_is_electron(ON_ELECTRON));
    assert!(says_its_base_is_electron(
        "{\"base\":\"org.electronjs.Electron2.BaseApp\"}"
    ));
    assert!(!says_its_base_is_electron(ON_ANOTHER_BASE));
    assert!(!says_its_base_is_electron(ON_NO_BASE));
    assert!(!says_its_base_is_electron(
        "{\"base\":\"org.electronjs.Electron2.BaseAppX\"}"
    ));
    assert!(!says_its_base_is_electron(""));
}

/// **The closed list stays as the fallback**: an application on it is drawn
/// by Chromium with no package to read, as one installed some other way is —
/// and an application neither on it nor on Electron's base never is, so it is
/// never handed the switch.
#[test]
fn the_list_is_the_fallback_and_nothing_else_is_drawn_by_chromium() {
    let machines = a_place("fallback");
    a_package(&machines, "org.example.Chat", ON_ELECTRON);
    a_package(&machines, "org.gnome.TextEditor", ON_NO_BASE);
    for identifier in alo_launching::DRAWN_BY_CHROMIUM {
        assert!(is_drawn_by_chromium_under(&machines, None, identifier), "{identifier}");
    }
    assert!(is_drawn_by_chromium_under(&machines, None, "org.example.Chat"));
    for identifier in ["org.gnome.TextEditor", "org.mozilla.firefox", "org.kde.dolphin"] {
        assert!(
            !is_drawn_by_chromium_under(&machines, None, identifier),
            "{identifier} would be handed the switch"
        );
        for fully in [false, true] {
            assert!(
                alo_launching::what_its_command_gains(
                    is_drawn_by_chromium_under(&machines, None, identifier),
                    true,
                    fully
                )
                .is_empty(),
                "{identifier} gained a switch"
            );
        }
    }
}

/// **On this machine, as this process runs**: an application on the closed
/// list is drawn by Chromium whatever is installed here, and one that is
/// neither on it nor ever built on Electron is not.
#[test]
fn on_this_machine_the_list_answers_and_a_text_editor_is_not_chromium() {
    assert!(is_drawn_by_chromium("com.vscodium.codium"));
    assert!(!is_drawn_by_chromium("org.gnome.TextEditor"));
}
