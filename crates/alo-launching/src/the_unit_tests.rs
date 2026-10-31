//! The names `systemctl --user list-units 'app-alo-*'` will show.

use super::*;

/// The name, owned, for an assertion.
fn named(it: &str) -> Option<String> {
    Some(it.to_owned())
}

#[test]
fn a_bus_style_identifier_keeps_its_dots() {
    assert_eq!(
        the_unit_for("org.kde.dolphin", 0x2a),
        named("app-alo-org.kde.dolphin@2a.service")
    );
}

#[test]
fn a_dash_in_the_identifier_is_escaped_so_it_cannot_split_the_name() {
    assert_eq!(
        the_unit_for("libreoffice-writer", 7),
        named(r"app-alo-libreoffice\x2dwriter@7.service")
    );
}

#[test]
fn anything_a_unit_name_cannot_hold_is_escaped() {
    assert_eq!(
        the_unit_for("a b/é", 1),
        named(r"app-alo-a\x20b\x2f\xc3\xa9@1.service")
    );
    assert_eq!(
        the_unit_for(".hidden", 1),
        named(r"app-alo-\x2ehidden@1.service")
    );
}

#[test]
fn two_starts_of_one_application_are_two_units() {
    assert_ne!(the_unit_for("firefox", 1), the_unit_for("firefox", 2));
}

/// systemd's own rule for a valid name: only these characters, at most 255
/// bytes, and the `@` once.
#[test]
fn every_name_is_one_systemd_accepts() {
    for identifier in [
        "org.mozilla.firefox",
        "libreoffice-calc",
        "kde4-konsole",
        "a;b$c",
    ] {
        let name = the_unit_for(identifier, u64::MAX).unwrap_or_default();
        assert!(!name.is_empty() && name.len() <= 255, "{name}");
        assert_eq!(name.matches('@').count(), 1, "{name}");
        assert!(
            name.chars()
                .all(|it| it.is_ascii_alphanumeric() || ":_.\\-@".contains(it)),
            "{name}"
        );
    }
}

#[test]
fn a_name_systemd_would_refuse_for_its_length_is_not_made() {
    assert_eq!(the_unit_for(&"-".repeat(70), 1), None);
    assert!(the_unit_for(&"a".repeat(200), 1).is_some());
}
