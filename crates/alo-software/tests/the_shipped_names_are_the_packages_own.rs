//! **The names pinned for the shipped applications are their packages' own.**
//!
//! `the-shipped-names.txt` is made by `alo-shipped-names` from the desktop
//! entries each package exports, read by the machine's own reader
//! (`alo_installed::read`). This holds two things about it: the file that
//! ships names every shipped application once, and the name made from an entry
//! is its untranslated `Name` — never a `Name[xx]`, before or after it, and
//! never an action's.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;

use alo_software::{NotNamed, Shipped, ShippedNames, as_their_entries_name_them};
use tempfile::TempDir;

/// **Every application a fresh machine ships is named, and named once.** The
/// file is filled from the packages, from what their own job prints
/// (`.github/workflows/shipped-names.yml`); a file missing one fails here.
#[test]
fn every_shipped_application_is_named_once_in_the_file_that_ships() {
    let shipped = Shipped::decided().unwrap();
    let names = ShippedNames::pinned().unwrap();
    for pinned in shipped.every() {
        let identifier = pinned.application().identifier();
        assert!(
            names.name_of(identifier).is_some(),
            "{identifier} is not named in the-shipped-names.txt"
        );
    }
    assert_eq!(names.every().len(), shipped.every().len());
}

/// **The untranslated name, whatever the entry translates.** Every shipped
/// application's entry here gives `Name[fr]` before its `Name` and `Name[de]`
/// after, and an action with a `Name` of its own; what is pinned is `Name`.
#[test]
fn a_translated_name_and_an_actions_name_are_left_aside() {
    let shipped = Shipped::decided().unwrap();
    let folder = TempDir::new().unwrap();
    let applications = folder.path().join("applications");
    fs::create_dir_all(&applications).unwrap();
    for pinned in shipped.every() {
        let identifier = pinned.application().identifier();
        fs::write(
            applications.join(format!("{identifier}.desktop")),
            format!(
                "[Desktop Entry]\nType=Application\nName[fr]=Le {identifier}\n\
                 Name=The {identifier}\nName[de]=Das {identifier}\n\
                 Exec=/usr/bin/flatpak run {identifier}\nActions=new-window;\n\n\
                 [Desktop Action new-window]\nName=New Window\nExec=true\n"
            ),
        )
        .unwrap();
    }
    let installed = alo_installed::read(&[applications]);
    let made = as_their_entries_name_them(&shipped, &installed).unwrap();
    let names = ShippedNames::read(&made, &shipped).unwrap();
    for pinned in shipped.every() {
        let identifier = pinned.application().identifier();
        assert_eq!(
            names.name_of(identifier),
            Some(format!("The {identifier}").as_str()),
            "{made}"
        );
    }
}

/// **A translation is never the name**: an entry that gives only `Name[fr]`
/// has no untranslated name, so nothing is pinned for it and making the file
/// refuses, by its identifier.
#[test]
fn an_entry_with_only_a_translated_name_is_refused() {
    let shipped = Shipped::decided().unwrap();
    let folder = TempDir::new().unwrap();
    let applications = folder.path().join("applications");
    fs::create_dir_all(&applications).unwrap();
    for pinned in shipped.every() {
        let identifier = pinned.application().identifier();
        let named = if identifier == "com.vscodium.codium" {
            format!("Name[fr]=Le {identifier}\n")
        } else {
            format!("Name=The {identifier}\n")
        };
        fs::write(
            applications.join(format!("{identifier}.desktop")),
            format!("[Desktop Entry]\nType=Application\n{named}Exec=true\n"),
        )
        .unwrap();
    }
    let installed = alo_installed::read(&[applications]);
    assert_eq!(
        as_their_entries_name_them(&shipped, &installed),
        Err(NotNamed::NoEntry {
            identifier: "com.vscodium.codium".to_owned()
        })
    );
}
