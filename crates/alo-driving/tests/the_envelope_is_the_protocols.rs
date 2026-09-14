//! **The envelope a local model is held to is exactly the protocol's** —
//! [ADR 0032](../../../docs/decisions/0032-a-local-model-is-held-to-the-envelope-not-the-call.md).
//!
//! `alo-models` names the version and the three doors in its schema, and cannot
//! depend on `alo-protocol` to read them. This crate depends on both, so this is
//! where the two lists are held to each other: every door the schema admits is a
//! door `alo-protocol` reads from an agent, and the version is the one it reads.
//! A schema admitting a door the protocol refuses would make a model fail in a
//! new way; one refusing a door the protocol reads would take a request away
//! from the agent.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_models::in_the_envelope::{THE_DOORS, THE_PROTOCOL_FORMAT, the_envelope};
use alo_protocol::{FORMAT, FromAnAgent};

/// A line through each door, as an agent would write it.
fn a_line_through(door: &str) -> String {
    let inside = if door == "ask" {
        r#"{"question":"how many?"}"#
    } else {
        r#"{"verb":"list_folder","given":[]}"#
    };
    format!(r#"{{"format":{THE_PROTOCOL_FORMAT},"asks":{{"{door}":{inside}}}}}"#)
}

#[test]
fn every_door_the_envelope_admits_is_one_the_protocol_reads_from_an_agent() {
    assert_eq!(u64::from(FORMAT), THE_PROTOCOL_FORMAT);
    for door in THE_DOORS {
        assert!(
            FromAnAgent::read(&a_line_through(door)).is_ok(),
            "the envelope admits `{door}` and the protocol does not read it from an agent"
        );
    }
    let admitted = the_envelope()
        .pointer("/properties/asks/oneOf")
        .unwrap()
        .as_array()
        .unwrap()
        .len();
    assert_eq!(admitted, THE_DOORS.len());
}

/// **And a door the protocol refuses from an agent is not one the envelope
/// names** — the person's doors, checked by name.
#[test]
fn no_door_a_person_uses_is_in_the_envelope() {
    for persons in ["approve", "decline", "waiting", "granted"] {
        assert!(!THE_DOORS.contains(&persons), "{persons}");
        assert!(
            !the_envelope()
                .to_string()
                .contains(&format!("\"{persons}\""))
        );
    }
}
