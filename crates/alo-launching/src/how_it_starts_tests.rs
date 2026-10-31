//! Choosing the road, for the entries a fresh alo machine has.

use super::*;

/// An entry with these keys.
fn entry(exec: Option<&str>, bus: bool) -> TheEntry {
    TheEntry {
        kind: Some("Application".to_owned()),
        name: Some("It".to_owned()),
        exec: exec.map(str::to_owned),
        bus_activatable: bus,
        ..TheEntry::default()
    }
}

/// Text Editor, Loupe, Papers and Ptyxis say `DBusActivatable=true`.
#[test]
fn an_activatable_entry_goes_over_the_bus_and_keeps_its_command_for_after() {
    let editor = entry(
        Some(
            "/usr/bin/flatpak run --branch=stable --command=gnome-text-editor org.gnome.TextEditor %U",
        ),
        true,
    );
    assert_eq!(
        how_it_starts("org.gnome.TextEditor", &editor, None),
        Ok(HowItStarts::OverTheBus {
            or_else: Some(
                [
                    "/usr/bin/flatpak",
                    "run",
                    "--branch=stable",
                    "--command=gnome-text-editor",
                    "org.gnome.TextEditor",
                ]
                .map(str::to_owned)
                .to_vec()
            )
        })
    );
}

#[test]
fn an_activatable_entry_with_no_usable_command_still_goes_over_the_bus() {
    assert_eq!(
        how_it_starts("org.example.Thing", &entry(None, true), None),
        Ok(HowItStarts::OverTheBus { or_else: None })
    );
    assert_eq!(
        how_it_starts("org.example.Thing", &entry(Some("app ;"), true), None),
        Ok(HowItStarts::OverTheBus { or_else: None })
    );
}

/// Dolphin, Firefox and LibreOffice say nothing about the bus.
#[test]
fn an_entry_that_is_not_activatable_starts_from_its_command() {
    assert_eq!(
        how_it_starts("org.kde.dolphin", &entry(Some("dolphin %u"), false), None),
        Ok(HowItStarts::FromItsCommand(vec!["dolphin".to_owned()]))
    );
    assert_eq!(
        how_it_starts(
            "libreoffice-writer",
            &entry(Some("libreoffice --writer %U"), false),
            None
        ),
        Ok(HowItStarts::FromItsCommand(vec![
            "libreoffice".to_owned(),
            "--writer".to_owned()
        ]))
    );
}

/// `libreoffice-writer` is not a bus name, so whatever its entry claimed it
/// could not be activated.
#[test]
fn an_identifier_that_is_not_a_bus_name_is_never_sent_to_the_bus() {
    assert_eq!(
        how_it_starts("libreoffice-writer", &entry(Some("lowriter"), true), None),
        Ok(HowItStarts::FromItsCommand(vec!["lowriter".to_owned()]))
    );
}

#[test]
fn an_entry_with_no_command_and_no_bus_is_refused() {
    assert_eq!(
        how_it_starts("org.example.Nothing", &entry(None, false), None),
        Err(WhyItCannotStart::NothingToStartItWith)
    );
}

#[test]
fn a_malformed_command_is_refused_by_what_is_wrong_with_it() {
    assert_eq!(
        how_it_starts("org.example.Bad", &entry(Some(r#"app "open"#), false), None),
        Err(WhyItCannotStart::ItsCommandIsMalformed(
            WhyNotACommand::UnterminatedQuote
        ))
    );
}

#[test]
fn a_deleted_entry_a_link_and_a_terminal_program_are_refused() {
    let mut deleted = entry(Some("app"), false);
    deleted.hidden = true;
    assert_eq!(
        how_it_starts("org.example.Gone", &deleted, None),
        Err(WhyItCannotStart::Deleted)
    );
    let mut link = entry(Some("app"), false);
    link.kind = Some("Link".to_owned());
    assert_eq!(
        how_it_starts("org.example.Link", &link, None),
        Err(WhyItCannotStart::NotAnApplication)
    );
    let mut in_a_terminal = entry(Some("htop"), false);
    in_a_terminal.terminal = true;
    assert_eq!(
        how_it_starts("htop", &in_a_terminal, None),
        Err(WhyItCannotStart::NeedsATerminal)
    );
}

#[test]
fn a_bus_name_is_held_to_the_specifications_shape() {
    for name in [
        "org.kde.dolphin",
        "org.gnome.TextEditor",
        "a.b",
        "org.x-y._z",
    ] {
        assert!(is_a_bus_name(name), "{name}");
    }
    for name in [
        "firefox",
        "libreoffice-writer",
        "org..x",
        "org.1x",
        ".org.x",
        "",
    ] {
        assert!(!is_a_bus_name(name), "{name}");
    }
}
