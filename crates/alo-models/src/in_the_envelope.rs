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

/// **The door a model answers the person through**, beside the three above and
/// not one of them: it is what a model says to the person, not what an agent
/// asks the machine, so `alo-protocol` does not read it and the daemon never
/// sees it. Until 2026-10-27 the envelope had no such door, and a question put
/// through it could not be answered at all
/// ([ADR 0133](../../../docs/decisions/0133-a-question-is-answered-inside-the-envelope.md)).
/// `alo-driving`'s tests hold it to `alo_instructing::THE_ANSWER_DOOR`, the
/// crate that reads it.
pub const THE_ANSWER: &str = "answer";

/// **The schema an agent turn's question carries**: an object with the
/// protocol's version and an `asks` holding exactly one door — one of
/// [`THE_DOORS`], or [`THE_ANSWER`] — whose contents are any object.
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
            "asks": { "oneOf": THE_DOORS.iter().chain([&THE_ANSWER]).map(|door| one_door(door)).collect::<Vec<_>>() },
        },
        "required": ["format", "asks"],
        "additionalProperties": false,
    })
}

/// **The same envelope, in the subset a hosted provider's strict mode
/// accepts.**
///
/// Two renderings of one piece of knowledge, which this repository normally
/// refuses — so here is why there are two, measured against OpenAI on
/// 2026-10-13 rather than supposed.
///
/// [`the_envelope`] says *an `asks` holding exactly one door, whose contents
/// are any object*. A provider's strict schema cannot say that:
///
/// - `oneOf` is refused outright — *in context=('properties', 'asks'),
///   'oneOf' is not permitted*. `anyOf` is accepted, and means the same thing
///   here because the branches forbid each other's keys.
/// - every object must be fully specified — *'additionalProperties' is
///   required to be supplied and to be false*. So *any object* is
///   inexpressible: an object with no properties and `additionalProperties:
///   false` admits only `{}`, which would refuse every real request.
///
/// **So a provider cannot be held to the envelope alone.** The least it will
/// accept is the envelope *and* each door's own two fields, which is what this
/// is. It is strictly more than [`the_envelope`] and strictly less than the
/// whole call: the verb's name and the argument names are still the model's to
/// get wrong, and the capability model still refuses them.
///
/// # What it fixes and what it does not
///
/// It makes the measured failure impossible. Every failure of `gpt-4o-mini`
/// over three rounds of the fixed ten was the verb's name put where the door
/// belongs — `{"format":1,"asks":{"find_in_folder":{…}}}` — and under this
/// schema the only keys `asks` may hold are the three doors.
///
/// **It can introduce a content error in exchange.** Asked under it to list a
/// folder, `gpt-4o-mini` answered
/// `{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"/home/anna/Invoices","is":"string"}]}}}`
/// — the path in `named` and the schema's own word `string` in `is`. A correct
/// shape carrying nonsense, which is the placeholder-copying
/// `data/catalogue.toml` already records for local models. That is refused as
/// `ArgumentRefused` rather than as an unreadable message, which is a better
/// refusal to be given: it names the argument, and an agent can act on it.
///
/// Enumerating the verbs and their arguments would close that too, and is the
/// next thing rather than this one — `alo_driving::grammar_for` already builds
/// exactly that knowledge in GBNF for the local road.
#[must_use]
pub fn the_envelope_a_provider_accepts() -> Value {
    let argument = json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["named", "is"],
        "properties": {
            "named": { "type": "string" },
            "is": { "type": ["string", "integer"] },
        },
    });
    let one_door = |door: &str| {
        json!({
            "type": "object",
            "additionalProperties": false,
            "required": [door],
            "properties": {
                door: {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["verb", "given"],
                    "properties": {
                        "verb": { "type": "string" },
                        "given": { "type": "array", "items": argument },
                    },
                },
            },
        })
    };
    // **The answer, specified as fully as a provider requires**: one field,
    // the words. A door whose contents were any object cannot be said to a
    // provider, for the reason above.
    let the_answer = json!({
        "type": "object",
        "additionalProperties": false,
        "required": [THE_ANSWER],
        "properties": {
            THE_ANSWER: {
                "type": "object",
                "additionalProperties": false,
                "required": ["text"],
                "properties": { "text": { "type": "string" } },
            },
        },
    });
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["format", "asks"],
        "properties": {
            "format": { "type": "integer", "enum": [THE_PROTOCOL_FORMAT] },
            // `anyOf` and not `oneOf`: a provider refuses `oneOf`, and the
            // branches forbid each other's keys so the two mean the same.
            "asks": { "anyOf": THE_DOORS.iter().map(|door| one_door(door)).chain([the_answer]).collect::<Vec<_>>() },
        },
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    /// **The two renderings name the same three doors and the same version.**
    ///
    /// Not that they are equal — they are deliberately not, and the provider's
    /// is stricter. What this holds is that the difference is only in how much
    /// of the *inside* of a door is specified: a rendering that quietly admitted
    /// a fourth door, or a different protocol version, would be two opinions
    /// about what the protocol is rather than two renderings of one.
    #[test]
    fn both_renderings_name_the_same_doors_and_the_same_version() {
        for schema in [
            super::the_envelope(),
            super::the_envelope_a_provider_accepts(),
        ] {
            let text = schema.to_string();
            for door in super::THE_DOORS.iter().chain([&super::THE_ANSWER]) {
                assert!(text.contains(door), "{door} is missing from {text}");
            }
            assert!(
                text.contains(&super::THE_PROTOCOL_FORMAT.to_string()),
                "the protocol's version is missing from {text}"
            );
        }
        // And the provider's refuses `oneOf`, which is the whole reason it
        // exists — asserted so a later tidy-up cannot put it back.
        assert!(
            !super::the_envelope_a_provider_accepts()
                .to_string()
                .contains("oneOf"),
            "the provider's rendering uses oneOf, which a provider refuses"
        );
    }

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
        assert_eq!(doors.len(), THE_DOORS.len() + 1);
        for (schema, door) in doors.iter().zip(THE_DOORS.iter().chain([&THE_ANSWER])) {
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
