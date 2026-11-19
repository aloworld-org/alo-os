//! Independent transport-boundary cases; no running portal or recording claim.

use super::*;

/// Accept the inclusive byte limit while refusing code/path syntax and Unicode.
#[test]
fn private_node_and_application_names_are_nonempty_bounded_ascii_identities() {
    for accepted in [
        "A",
        "z",
        "0",
        ".",
        "-",
        "_",
        "org.aloworld.Studio",
        "capture-123_45",
    ] {
        assert!(
            valid(accepted),
            "valid opaque identity refused: {accepted:?}"
        );
    }
    assert!(valid(&"n".repeat(255)));
    for refused in [
        "",
        "two words",
        "a/b",
        "a\\b",
        "a:b",
        "a\nb",
        "a\tb",
        "a\0b",
        "é",
    ] {
        assert!(!valid(refused), "invalid identity accepted: {refused:?}");
    }
    assert!(!valid(&"n".repeat(256)));
}

/// Node kind and exact name must both agree before retaining an actual identity.
#[test]
fn only_the_named_registry_node_can_supply_the_media_identity() {
    let expected = Some(MediaNode { id: 42, serial: 71 });
    assert_eq!(
        selected_node(
            &ObjectType::Node,
            42,
            Some("capture-7"),
            Some("71"),
            "capture-7"
        ),
        expected
    );
    for kind in [ObjectType::Client, ObjectType::Port] {
        assert_eq!(
            selected_node(&kind, 42, Some("capture-7"), Some("71"), "capture-7"),
            None
        );
    }
    for node_name in [None, Some("capture-70"), Some("other"), Some("")] {
        assert_eq!(
            selected_node(&ObjectType::Node, 42, node_name, Some("71"), "capture-7"),
            None
        );
    }
}

/// A reusable global ID cannot stand in for the registry's own u64 serial.
#[test]
fn a_node_id_without_its_own_parseable_serial_cannot_satisfy_selection() {
    for serial in [
        None,
        Some(""),
        Some("-1"),
        Some("1.5"),
        Some(" 71"),
        Some("71x"),
        Some("18446744073709551616"),
    ] {
        assert_eq!(
            selected_node(
                &ObjectType::Node,
                42,
                Some("capture-7"),
                serial,
                "capture-7"
            ),
            None
        );
    }
    assert_eq!(
        selected_node(
            &ObjectType::Node,
            6,
            Some("capture-7"),
            Some("0"),
            "capture-7"
        ),
        Some(MediaNode { id: 6, serial: 0 })
    );
    assert_eq!(
        selected_node(
            &ObjectType::Node,
            9,
            Some("capture-7"),
            Some("18446744073709551615"),
            "capture-7"
        ),
        Some(MediaNode {
            id: 9,
            serial: u64::MAX
        })
    );
}
