//! The pinned names are read only as identifier, tab, name, for applications a
//! fresh machine ships, once each; and what the file should hold is made from
//! the entries read, in `shipped.toml`'s order, or refused for one missing.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_applications::Installed;
use alo_applications::application::Application;

use super::{NotNamed, ShippedNames, THE_HEADER, as_their_entries_name_them};
use crate::shipped::Shipped;

fn shipped() -> Shipped {
    Shipped::decided().unwrap()
}

fn read(text: &str) -> Result<ShippedNames, NotNamed> {
    ShippedNames::read(text, &shipped())
}

#[test]
fn a_line_is_an_identifier_a_tab_and_a_name() {
    let names =
        read("# a comment\n\ncom.vscodium.codium\tVSCodium\norg.gnome.TextEditor\tText Editor\n")
            .unwrap();
    assert_eq!(names.name_of("com.vscodium.codium"), Some("VSCodium"));
    assert_eq!(names.name_of("org.gnome.TextEditor"), Some("Text Editor"));
    assert_eq!(names.name_of("org.mozilla.firefox"), None);
    assert_eq!(names.every().len(), 2);
}

#[test]
fn a_line_with_no_tab_is_refused() {
    assert_eq!(
        read("com.vscodium.codium VSCodium\n"),
        Err(NotNamed::NotALine {
            line: "com.vscodium.codium VSCodium".to_owned()
        })
    );
}

#[test]
fn a_line_with_no_name_is_refused() {
    assert_eq!(
        read("com.vscodium.codium\t  \n"),
        Err(NotNamed::NotALine {
            line: "com.vscodium.codium\t  ".to_owned()
        })
    );
}

#[test]
fn an_identifier_named_twice_is_refused() {
    assert_eq!(
        read("com.vscodium.codium\tVSCodium\ncom.vscodium.codium\tCodium\n"),
        Err(NotNamed::Twice {
            identifier: "com.vscodium.codium".to_owned()
        })
    );
}

#[test]
fn an_identifier_no_fresh_machine_ships_is_refused() {
    assert_eq!(
        read("org.gimp.GIMP\tGIMP\n"),
        Err(NotNamed::NotShipped {
            identifier: "org.gimp.GIMP".to_owned()
        })
    );
}

/// Every shipped application's entry, named after its identifier's last part.
fn every_entry() -> Installed {
    Installed::holding(shipped().every().iter().map(|pinned| {
        let identifier = pinned.application().identifier();
        let last = identifier.rsplit('.').next().unwrap();
        Application::called(identifier, &format!("{last} app")).unwrap()
    }))
}

#[test]
fn what_the_file_should_hold_is_its_header_and_a_line_each_in_the_lists_order() {
    let made = as_their_entries_name_them(&shipped(), &every_entry()).unwrap();
    let lines: Vec<String> = shipped()
        .every()
        .iter()
        .map(|pinned| {
            let identifier = pinned.application().identifier();
            format!(
                "{identifier}\t{} app",
                identifier.rsplit('.').next().unwrap()
            )
        })
        .collect();
    assert_eq!(made, format!("{THE_HEADER}{}\n", lines.join("\n")));
    // And what it made reads back as the same names.
    let back = read(&made).unwrap();
    assert_eq!(back.every().len(), shipped().every().len());
}

#[test]
fn a_shipped_application_with_no_entry_is_refused_by_its_identifier() {
    let mut fewer = Installed::nothing();
    for application in every_entry().all() {
        if application.identifier() != "com.vscodium.codium" {
            fewer.add(application.clone());
        }
    }
    assert_eq!(
        as_their_entries_name_them(&shipped(), &fewer),
        Err(NotNamed::NoEntry {
            identifier: "com.vscodium.codium".to_owned()
        })
    );
}

/// **An entry with no name of its packager's is no entry**: an application
/// known only by its identifier has no name to pin.
#[test]
fn an_entry_that_gives_no_name_is_refused_as_none() {
    let mut unnamed = Installed::nothing();
    for application in every_entry().all() {
        if application.identifier() == "com.vscodium.codium" {
            unnamed.add(Application::identified("com.vscodium.codium").unwrap());
        } else {
            unnamed.add(application.clone());
        }
    }
    assert_eq!(
        as_their_entries_name_them(&shipped(), &unnamed),
        Err(NotNamed::NoEntry {
            identifier: "com.vscodium.codium".to_owned()
        })
    );
}

/// **The header is the file's**: the file that ships begins with it, so what
/// the reader prints and what is committed differ only by the names.
#[test]
fn the_file_that_ships_begins_with_the_header() {
    assert!(super::THE_NAMES.starts_with(THE_HEADER));
}

#[test]
fn a_line_with_no_identifier_is_refused() {
    assert_eq!(
        read("\tVSCodium\n"),
        Err(NotNamed::NotALine {
            line: "\tVSCodium".to_owned()
        })
    );
}

/// **A file written on another system reads the same**: a line ending in a
/// carriage return names VSCodium, not `VSCodium\r`.
#[test]
fn a_carriage_return_is_not_part_of_a_name() {
    let names = read("com.vscodium.codium\tVSCodium\r\n").unwrap();
    assert_eq!(names.name_of("com.vscodium.codium"), Some("VSCodium"));
}
