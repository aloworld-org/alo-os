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

/// **Firefox's own entry takes the link where its `%u` is**, between
/// Flatpak's forwarding marks, as one argument.
#[test]
fn a_link_goes_where_the_entry_takes_one() {
    let firefox = entry(
        Some(
            "/usr/bin/flatpak run --branch=stable --arch=x86_64 --command=firefox \
             --file-forwarding org.mozilla.firefox @@u %u @@",
        ),
        false,
    );
    assert_eq!(
        how_it_starts_with_a_link(
            "org.mozilla.firefox",
            &firefox,
            None,
            "https://example.com/invoice"
        ),
        Ok(HowItStarts::FromItsCommand(
            [
                "/usr/bin/flatpak",
                "run",
                "--branch=stable",
                "--arch=x86_64",
                "--command=firefox",
                "--file-forwarding",
                "org.mozilla.firefox",
                "@@u",
                "https://example.com/invoice",
                "@@",
            ]
            .map(str::to_owned)
            .to_vec()
        ))
    );
    let list = entry(Some("thunderbird %U"), false);
    assert_eq!(
        how_it_starts_with_a_link("thunderbird", &list, None, "mailto:anna@example.com"),
        Ok(HowItStarts::FromItsCommand(
            ["thunderbird", "mailto:anna@example.com"]
                .map(str::to_owned)
                .to_vec()
        ))
    );
}

/// **An entry with nowhere to put a link is refused**, rather than started
/// without it — and one that cannot start at all says why.
#[test]
fn an_entry_that_takes_no_link_is_refused() {
    for exec in ["gimp", "gimp %f", "gimp %F"] {
        assert_eq!(
            how_it_starts_with_a_link("org.gimp.GIMP", &entry(Some(exec), false), None, "https://a.example/"),
            Err(WhyNotWithALink::TakesNoLink),
            "{exec}"
        );
    }
    assert_eq!(
        how_it_starts_with_a_link("org.example.None", &entry(None, false), None, "https://a.example/"),
        Err(WhyNotWithALink::CannotStart(WhyItCannotStart::NothingToStartItWith))
    );
}

/// **Over the bus the link travels in `Open`**, and a command to fall back on
/// is kept only where it would carry the link too.
#[test]
fn over_the_bus_the_fallback_keeps_the_link_or_is_dropped() {
    let carries = entry(Some("epiphany %u"), true);
    assert_eq!(
        how_it_starts_with_a_link("org.gnome.Epiphany", &carries, None, "https://a.example/"),
        Ok(HowItStarts::OverTheBus {
            or_else: Some(["epiphany", "https://a.example/"].map(str::to_owned).to_vec())
        })
    );
    let does_not = entry(Some("geary"), true);
    assert_eq!(
        how_it_starts_with_a_link("org.gnome.Geary", &does_not, None, "mailto:a@example.com"),
        Ok(HowItStarts::OverTheBus { or_else: None })
    );
}
