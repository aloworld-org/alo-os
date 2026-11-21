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
//! - **Nothing about which arguments, since 2026-11-14.** `given` is one
//!   object with a required property per argument, so under a strict schema an
//!   argument cannot be left out or written twice — the two things the list it
//!   replaced could not rule out, and the second of which failed every try of
//!   the walk's own request (`a_call_of` has the runs).
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

use alo_capability::{Effect, Offered, Takes, Verb, Verbs};
use alo_models::in_the_envelope::the_envelope_a_provider_accepts;
use alo_software::Enabled;
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

/// One call of one verb: its name, and a `given` naming **each of its
/// arguments once**, as one object with a property per argument.
///
/// **An object, since 2026-11-14, and no longer a list of `{named, is}`.**
/// Held to the list, `gpt-4o-mini` named an argument twice in every one of
/// ten tries of the walk's own request — *write in VSCodium* — twice with two
/// different values (the every-try check, CI runs 43682925362 and
/// 43683523750), and the walk's runs failed the same way. A strict schema
/// cannot say *once each* about a list; about an object it says nothing else,
/// because a property is written once and every one is required. The daemon
/// reads this form key by key and refuses a key written twice
/// (`alo_protocol::argument`), so nothing is settled by a JSON reader.
fn a_call_of(verb: &Verb) -> Value {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();
    for arg in verb.args() {
        let mut value = a_value_of(arg.takes());
        if let Some(said) = value.as_object_mut() {
            said.insert(
                "description".to_owned(),
                Value::String(arg.purpose_as_written().to_owned()),
            );
        }
        properties.insert(arg.name().to_owned(), value);
        required.push(Value::String(arg.name().to_owned()));
    }
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["verb", "given"],
        "properties": {
            "verb": { "type": "string", "enum": [verb.name()] },
            "given": {
                "type": "object",
                "additionalProperties": false,
                "required": required,
                "properties": properties,
            },
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

/// **The places a model may name as an installation's source**: every place
/// this machine is set up with that `alo_software::Enabled::usable` would let
/// an installation use — the rented tool's own list, under the rule an
/// organisation set, the same `Enabled::read` the person's own installs go
/// through (`alo-shell`'s `what_a_person_installs`) — by name, in its order.
///
/// **Only usable places, on purpose**: offering one the machine would refuse
/// invites a proposal certain to be refused. What that costs: a person who
/// names a place outside the rule is answered with a place inside it, and the
/// sentence they approve names it, so nothing is changed silently.
///
/// A place switched off, outside the rule, unsigned or with nowhere to reach is
/// left out, because the machine would refuse an installation from it.
#[must_use]
pub fn the_places_a_model_may_name(enabled: &Enabled) -> Vec<String> {
    enabled
        .all()
        .map(|source| source.name().as_str())
        .filter(|name| enabled.usable(name).is_ok())
        .map(str::to_owned)
        .collect()
}

/// **[`the_envelope_naming_the_verbs`], with an installation's source held to
/// `places`**: the one argument whose right values only this machine knows.
///
/// `install_application`'s `source` becomes one of `places`, and its
/// description names them, so a hosted model is shown where this machine
/// installs from and cannot send anything else. Every other argument of every
/// verb is held exactly as before.
///
/// **No places, no narrowing**: a provider refuses an empty `enum`, and a
/// machine that installs from nowhere refuses every installation itself
/// (`alo_software::NotDone::NotEnabled`), so the envelope is the one
/// [`the_envelope_naming_the_verbs`] gives.
///
/// This narrows to what already worked: a source outside `places` was refused
/// by the machine before (`Enabled::usable`), only later, and after the person
/// had been asked.
#[must_use]
pub fn the_envelope_naming_the_verbs_and_places(verbs: &Verbs, places: &[String]) -> Value {
    let mut envelope = the_envelope_naming_the_verbs(verbs);
    if places.is_empty() {
        return envelope;
    }
    let Some(doors) = envelope
        .pointer_mut("/properties/asks/anyOf")
        .and_then(Value::as_array_mut)
    else {
        return envelope;
    };
    for call in doors
        .iter_mut()
        .filter_map(|door| door.pointer_mut("/properties/propose/anyOf"))
        .filter_map(Value::as_array_mut)
        .flatten()
    {
        let names_it = call
            .pointer("/properties/verb/enum/0")
            .and_then(Value::as_str)
            == Some(alo_software::INSTALL_APPLICATION);
        if !names_it {
            continue;
        }
        let Some(source) = call.pointer_mut(&format!(
            "/properties/given/properties/{}",
            alo_software::words::SOURCE
        )) else {
            continue;
        };
        let was = source
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        *source = json!({
            "type": "string",
            "enum": places,
            "description": format!(
                "{was}; the places this machine installs from: {}",
                places.join(", ")
            ),
        });
    }
    envelope
}

/// **The envelope a turn's question on `machine` is held to**: its verbs,
/// with an installation's source held to the places it may install from now.
/// The one place `asking.rs` takes the schema from, so the test that holds
/// this holds what a provider is sent.
pub(crate) fn the_envelope_a_turn_on(
    machine: &crate::machine::Machine<'_>,
    offered: &Verbs,
) -> Value {
    the_envelope_naming_the_verbs_and_places(offered, &machine.the_places_a_model_may_name())
}
