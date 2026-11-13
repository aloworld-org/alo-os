//! Packages and overrides laid out as flatpak lays them out, read back.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use super::*;

/// VSCodium's package as Flathub's own answer listed its permissions on
/// 2026-11-12 (`/api/v2/summary/com.vscodium.codium`), written in the shape a
/// `metadata` file has. **Reconstructed from that answer, not copied from a
/// file on a machine.**
const VSCODIUM: &str = "[Application]\nname=com.vscodium.codium\n\
     runtime=org.freedesktop.Sdk/x86_64/26.08\n\n\
     [Context]\nshared=network;ipc;\nsockets=fallback-x11;pulseaudio;ssh-auth;wayland;\n\
     devices=all;\nfilesystems=xdg-run/gnupg:ro;xdg-config/kdeglobals:ro;host;\n\n\
     [Session Bus Policy]\norg.freedesktop.Flatpak=talk\norg.freedesktop.secrets=talk\n";

/// Firefox's, the same way: folders of the home, never the whole of it.
const FIREFOX: &str = "[Application]\nname=org.mozilla.firefox\n\n\
     [Context]\nshared=network;ipc;\nsockets=x11;wayland;pulseaudio;\n\
     filesystems=xdg-config/gtk-3.0:ro;xdg-download;/run/udev:ro;\n\n\
     [Session Bus Policy]\norg.a11y.Bus=talk\norg.freedesktop.FileManager1=talk\n";

/// Papers', which reads the home and changes none of it.
const PAPERS: &str = "[Application]\nname=org.gnome.Papers\n\n\
     [Context]\nfilesystems=home:ro;/media:ro;xdg-run/gvfs;\n";

/// A directory of the test's own, emptied.
fn a_place(named: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!(
        "alo-installed-reach-{}-{named}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    at
}

/// Install a package's metadata where flatpak keeps it.
fn a_package(installation: &Path, identifier: &str, metadata: &str) {
    let at = installation
        .join("app")
        .join(identifier)
        .join("current")
        .join("active");
    std::fs::create_dir_all(&at).unwrap();
    std::fs::write(at.join("metadata"), metadata).unwrap();
}

/// Write an override where flatpak keeps it.
fn an_override(installation: &Path, named: &str, text: &str) {
    let at = installation.join("overrides");
    std::fs::create_dir_all(&at).unwrap();
    std::fs::write(at.join(named), text).unwrap();
}

/// **VSCodium is said as fact 4 says it**: every file in the home, to read and
/// change, and programs started outside its sandbox.
#[test]
fn a_package_asking_for_the_whole_home_and_for_leaving_is_said_so() {
    let machines = a_place("vscodium");
    a_package(&machines, "com.vscodium.codium", VSCODIUM);
    let read = what_they_may_reach(&machines, None, ["com.vscodium.codium"]);
    let it = read.of("com.vscodium.codium").unwrap();
    assert_eq!(it.home(), TheirHome::ReadAndChanged);
    assert!(it.starts_programs_outside());
    assert!(it.is_worth_saying());
}

/// **A sealed package is not said to be open**: Firefox reaches folders of
/// the home and nothing outside, so there is nothing to say.
#[test]
fn a_sealed_package_is_not_said_to_be_open() {
    let machines = a_place("firefox");
    a_package(&machines, "org.mozilla.firefox", FIREFOX);
    let it = what_they_may_reach(&machines, None, ["org.mozilla.firefox"])
        .of("org.mozilla.firefox")
        .unwrap();
    assert_eq!(it.home(), TheirHome::NotAsAWhole);
    assert!(!it.starts_programs_outside());
    assert!(!it.is_worth_saying());
}

/// **Read-only is said as read-only.**
#[test]
fn a_home_given_read_only_is_read_and_not_changed() {
    let machines = a_place("papers");
    a_package(&machines, "org.gnome.Papers", PAPERS);
    let it = what_they_may_reach(&machines, None, ["org.gnome.Papers"])
        .of("org.gnome.Papers")
        .unwrap();
    assert_eq!(it.home(), TheirHome::Read);
    assert!(!it.starts_programs_outside());
}

/// **An application with no package here is left out**, which a caller reads
/// as *nothing to say* — never as *sealed*. And an identifier that could name
/// another directory is not looked for.
#[test]
fn nothing_is_said_where_no_package_was_read() {
    let machines = a_place("absent");
    a_package(&machines, "com.vscodium.codium", VSCODIUM);
    let read = what_they_may_reach(
        &machines,
        None,
        [
            "org.libreoffice.LibreOffice",
            "../app/com.vscodium.codium",
            "..",
        ],
    );
    assert_eq!(read, WhatTheyMayReach::nothing());
}

/// **A person's override narrows what is said**: Flatseal's road, a file
/// under the person's own installation, takes the home and the way out away.
#[test]
fn an_override_that_narrows_a_package_is_believed() {
    let machines = a_place("narrowed-machine");
    let persons = a_place("narrowed-person");
    a_package(&machines, "com.vscodium.codium", VSCODIUM);
    an_override(
        &persons,
        "com.vscodium.codium",
        "[Context]\nfilesystems=!host;xdg-documents;\n\n\
         [Session Bus Policy]\norg.freedesktop.Flatpak=none\n",
    );
    let it = what_they_may_reach(&machines, Some(&persons), ["com.vscodium.codium"])
        .of("com.vscodium.codium")
        .unwrap();
    assert_eq!(it.home(), TheirHome::NotAsAWhole);
    assert!(!it.starts_programs_outside());
}

/// **And one that widens a package is said too** — the machine's `global`
/// override giving every application the whole session bus, which is a way
/// out as surely as naming Flatpak is.
#[test]
fn an_override_that_widens_a_package_is_said() {
    let machines = a_place("widened");
    a_package(&machines, "org.mozilla.firefox", FIREFOX);
    an_override(&machines, "global", "[Context]\nsockets=session-bus;\n");
    an_override(
        &machines,
        "org.mozilla.firefox",
        "[Context]\nfilesystems=~:ro;\n",
    );
    let it = what_they_may_reach(&machines, None, ["org.mozilla.firefox"])
        .of("org.mozilla.firefox")
        .unwrap();
    assert_eq!(it.home(), TheirHome::Read);
    assert!(it.starts_programs_outside());
}

/// **The person's package is the one read** where both installations hold
/// one, and a wildcard over Flatpak's name is a way out.
#[test]
fn the_persons_package_is_read_first_and_a_wildcard_counts() {
    let machines = a_place("both-machine");
    let persons = a_place("both-person");
    a_package(&machines, "org.example.Editor", FIREFOX);
    a_package(
        &persons,
        "org.example.Editor",
        "[Session Bus Policy]\norg.freedesktop.*=talk\n",
    );
    let it = what_they_may_reach(&machines, Some(&persons), ["org.example.Editor"])
        .of("org.example.Editor")
        .unwrap();
    assert!(it.starts_programs_outside());
    assert_eq!(it.home(), TheirHome::NotAsAWhole);
}

/// **`!host-reset` takes every filesystem away**, as flatpak's own reset does.
#[test]
fn a_reset_takes_every_filesystem_away() {
    let machines = a_place("reset");
    a_package(&machines, "com.vscodium.codium", VSCODIUM);
    an_override(
        &machines,
        "com.vscodium.codium",
        "[Context]\nfilesystems=!host-reset;\n",
    );
    let it = what_they_may_reach(&machines, None, ["com.vscodium.codium"])
        .of("com.vscodium.codium")
        .unwrap();
    assert_eq!(it.home(), TheirHome::NotAsAWhole);
    assert!(it.starts_programs_outside(), "the bus is not a filesystem");
}
