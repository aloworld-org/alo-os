//! **The shape a provider holds an agent turn's answer to, naming this
//! machine's verbs and each one's arguments.**
//!
//! `alo_models::in_the_envelope::the_envelope_a_provider_accepts` holds a
//! hosted model to the doors and to each door's two fields, and leaves the
//! verb's name and every argument's name a free string — its own doc says so,
//! records `gpt-4o-mini` putting a path in `named`, and names enumerating the
//! verbs as the next thing. **This is that next thing**, for the read and
//! propose doors: each holds one of the verbs of its own kind, by name, and
//! each verb's `given` holds only arguments that verb takes, each by its exact
//! name, with a value of the type that argument declared.
//!
//! # Why, walked rather than supposed
//!
//! On a booted machine on 2026-11-14 (image from `main` `3959dd65`,
//! `gpt-4o-mini` at OpenAI, VSCodium open), asked to type into the open file,
//! the model sent an argument named `Untitled-1` to `accessible.type_text`,
//! then `Untitled`, then `name` and `window` to `accessible.read_window`, which
//! takes only `application`. Every one was refused correctly, and the schema it
//! was held to admitted every one. Under this one none of them can be written.
//!
//! # What it still leaves to the model, and to the capability model
//!
//! - **Leaving an argument out.** A strict schema cannot say *every one of
//!   these, once each* about an array without keywords a provider's strict
//!   mode is not known here to accept, so it says *only these*.
//!   `alo_capability::Call::of` refuses a missing one, in words that say what
//!   it is for, and the model is told every argument is required.
//! - **The value.** A path the grants do not cover, a name that is not on the
//!   screen: refused by the machine, as ever. This shape lets nothing through
//!   that was not let through before; it only removes ways to be misunderstood.
//!
//! The other doors — `ask`, `run`, the answer — are the envelope's own,
//! unchanged: they name no verb.
//!
//! **A hosted provider only.** The pinned runtime is held to the envelope alone
//! (ADR 0032), because holding a local model to the whole call was measured to
//! make its answers worse; nothing here changes that road.

use alo_capability::{Arg, Effect, Offered, Takes, Verb, Verbs};
use alo_models::in_the_envelope::the_envelope_a_provider_accepts;
use serde_json::{Value, json};

/// **The provider's envelope, with the read and propose doors naming this
/// registry's verbs.** A door with no verb of its kind is left out, because a
/// door with nothing behind it is a line nothing could act on — and a provider
/// refuses an `anyOf` with no branches.
#[must_use]
pub fn the_envelope_naming_the_verbs(verbs: &Verbs) -> Value {
    let mut envelope = the_envelope_a_provider_accepts();
    let Some(doors) = envelope
        .pointer_mut("/properties/asks/anyOf")
        .and_then(Value::as_array_mut)
    else {
        // The envelope always has its doors; were it ever written without
        // them, it is sent as it is rather than half-rewritten.
        return envelope;
    };
    let mut named = Vec::with_capacity(doors.len());
    for door in doors.drain(..) {
        match door_of(&door) {
            Some(effect) => {
                if let Some(narrowed) = a_door_of(effect, verbs) {
                    named.push(narrowed);
                }
            }
            None => named.push(door),
        }
    }
    *doors = named;
    envelope
}

/// Which kind of verb a branch of the envelope is the door for, if any.
fn door_of(door: &Value) -> Option<Effect> {
    let properties = door.get("properties")?;
    if properties.get("read").is_some() {
        Some(Effect::Read)
    } else if properties.get("propose").is_some() {
        Some(Effect::Change)
    } else {
        None
    }
}

/// One door, holding one call of any of this registry's verbs of its kind.
fn a_door_of(effect: Effect, verbs: &Verbs) -> Option<Value> {
    let door = match effect {
        Effect::Read => "read",
        Effect::Change => "propose",
    };
    let calls: Vec<Value> = verbs
        .all()
        .filter(|verb| verb.effect() == effect)
        .map(a_call_of)
        .collect();
    if calls.is_empty() {
        return None;
    }
    Some(json!({
        "type": "object",
        "additionalProperties": false,
        "required": [door],
        "properties": { door: { "anyOf": calls } },
    }))
}

/// One call of one verb: its name, and a `given` holding only its arguments.
///
/// A verb that takes nothing is given the envelope's own free argument, rather
/// than an `anyOf` with no branches, which a provider refuses — and a model
/// sends it an empty list, which is what it is told to send.
fn a_call_of(verb: &Verb) -> Value {
    let arguments: Vec<Value> = verb.args().iter().map(an_argument_of).collect();
    let names: Vec<&str> = verb.args().iter().map(Arg::name).collect();
    // What a provider's strict mode cannot say about an array — *every one of
    // these, once each* — said in words beside it, as the model is told.
    let every_one = if names.is_empty() {
        "nothing: an empty list".to_owned()
    } else {
        format!("every one of these, once each: {}", names.join(", "))
    };
    let items = if arguments.is_empty() {
        json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["named", "is"],
            "properties": {
                "named": { "type": "string" },
                "is": { "type": ["string", "integer"] },
            },
        })
    } else {
        json!({ "anyOf": arguments })
    };
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["verb", "given"],
        "properties": {
            "verb": { "type": "string", "enum": [verb.name()] },
            "given": {
                "type": "array",
                "description": every_one,
                "items": items,
            },
        },
    })
}

/// One argument, by its exact name, with a value of the type it declared.
fn an_argument_of(arg: &Arg) -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["named", "is"],
        "properties": {
            "named": {
                "type": "string",
                "enum": [arg.name()],
                "description": arg.purpose_as_written(),
            },
            "is": a_value_of(arg.takes()),
        },
    })
}

/// The value an argument's type allows, as far as a strict schema says it: a
/// whole number, one of a choice's names, or text. The bounds — a range, a
/// longest name, a full path — are the capability model's to refuse.
fn a_value_of(takes: &Takes) -> Value {
    match takes {
        Takes::Count { .. } => json!({ "type": "integer" }),
        Takes::Choice(options) => {
            let names: Vec<&str> = options.iter().map(Offered::name).collect();
            json!({ "type": "string", "enum": names })
        }
        Takes::Path | Takes::Application | Takes::Name { .. } | Takes::Text => {
            json!({ "type": "string" })
        }
    }
}
