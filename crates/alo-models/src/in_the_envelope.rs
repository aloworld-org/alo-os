//! The shape an agent turn asks a model on this machine to answer in —
//! [ADR 0032](../../../docs/decisions/0032-a-local-model-is-held-to-the-envelope-not-the-call.md).
//!
//! **The envelope and the door, never the call.** The pinned runtime can hold a
//! model's answer to a JSON schema. Holding it to the whole call made answers
//! worse, because the runtime orders a schema's keys alphabetically and the
//! protocol's argument is `named` before `is`; holding it to the envelope alone —
//! the protocol's version, and exactly one of the three doors an agent has —
//! took Qwen 2.5 7B from 8 of 20 to 19 of 20. The ADR has the table.
//!
//! What is inside the door is left to the model and read, as ever, by
//! `alo-protocol` and validated by `alo-capability`. This schema removes a way to
//! fail to be understood; it lets nothing through that was not let through
//! before.

use serde_json::{Value, json};

/// The version of the protocol an agent's line carries. `alo-driving`'s tests
/// hold it to what `alo-protocol` reads.
pub const THE_PROTOCOL_FORMAT: u64 = 1;

/// The doors an agent has, in the order ADR 0001 names them: a read, a change to
/// propose, and a question for a model. `alo-driving`'s tests hold these to what
/// `alo-protocol` accepts from an agent, so the schema cannot admit a door the
/// protocol refuses or refuse one it reads.
pub const THE_DOORS: [&str; 3] = ["read", "propose", "ask"];

/// **The schema an agent turn's question carries**: an object with the
/// protocol's version and an `asks` holding exactly one door, whose contents are
/// any object.
#[must_use]
pub fn the_envelope() -> Value {
    let one_door = |door: &str| {
        json!({
            "type": "object",
            "properties": { door: { "type": "object" } },
            "required": [door],
            "additionalProperties": false,
        })
    };
    json!({
        "type": "object",
        "properties": {
            "format": { "type": "integer", "enum": [THE_PROTOCOL_FORMAT] },
            "asks": { "oneOf": THE_DOORS.iter().map(|door| one_door(door)).collect::<Vec<_>>() },
        },
        "required": ["format", "asks"],
        "additionalProperties": false,
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **The envelope names every door once, and nothing inside any of them** —
    /// the half of ADR 0032 that is measured: a schema reaching into the call
    /// makes the runtime order its keys.
    #[test]
    fn the_envelope_holds_the_door_and_leaves_the_call_to_the_model() {
        let envelope = the_envelope();
        let doors = envelope
            .pointer("/properties/asks/oneOf")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(doors.len(), THE_DOORS.len());
        for (schema, door) in doors.iter().zip(THE_DOORS) {
            let inside = schema.pointer(&format!("/properties/{door}")).unwrap();
            assert_eq!(inside, &json!({ "type": "object" }), "{door}");
            assert_eq!(schema.get("required").unwrap(), &json!([door]));
            assert_eq!(schema.get("additionalProperties").unwrap(), false);
        }
        for into_the_call in ["verb", "given", "named", "is", "question"] {
            assert!(
                !envelope
                    .to_string()
                    .contains(&format!("\"{into_the_call}\"")),
                "the envelope reaches into the call through `{into_the_call}`: {envelope}"
            );
        }
        assert_eq!(
            envelope.pointer("/properties/format/enum").unwrap(),
            &json!([THE_PROTOCOL_FORMAT])
        );
    }
}
