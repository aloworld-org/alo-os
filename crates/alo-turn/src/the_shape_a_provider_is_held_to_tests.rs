//! Tests for `crate::the_shape_a_provider_is_held_to`, and the regression from
//! the walk of 2026-11-14 that it and `alo_capability::CallError` answer.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_capability::{CallError, Effect, Given, Verbs};
use serde_json::Value;

use crate::machine::the_verbs_a_whole_machine_offers;
use crate::testing::in_english;
use crate::the_shape_a_provider_is_held_to::the_envelope_naming_the_verbs;

/// Every verb a whole machine offers — the list a booted machine's model is
/// shown, not one written for a test.
fn the_whole_machine() -> Verbs {
    the_verbs_a_whole_machine_offers().unwrap()
}

/// The calls a door admits, by verb name.
fn the_calls_through<'v>(schema: &'v Value, door: &str) -> Vec<&'v Value> {
    let doors = schema
        .pointer("/properties/asks/anyOf")
        .and_then(Value::as_array)
        .unwrap();
    doors
        .iter()
        .find_map(|branch| branch.pointer(&format!("/properties/{door}/anyOf")))
        .and_then(Value::as_array)
        .map(|calls| calls.iter().collect())
        .unwrap_or_default()
}

/// The one call of `verb` the schema admits.
fn the_call_of<'v>(schema: &'v Value, door: &str, verb: &str) -> &'v Value {
    the_calls_through(schema, door)
        .into_iter()
        .find(|call| {
            call.pointer("/properties/verb/enum/0")
                .and_then(Value::as_str)
                == Some(verb)
        })
        .unwrap_or_else(|| panic!("{verb} is not behind the {door} door"))
}

/// **The argument names a call of this verb may carry**, read off the schema.
fn the_names_admitted(call: &Value) -> Vec<String> {
    call.pointer("/properties/given/items/anyOf")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .map(|argument| {
            argument
                .pointer("/properties/named/enum")
                .and_then(Value::as_array)
                .unwrap()
                .iter()
                .map(|name| name.as_str().unwrap().to_owned())
                .collect::<Vec<_>>()
        })
        .inspect(|names| assert_eq!(names.len(), 1, "an argument admits more than one name"))
        .flatten()
        .collect()
}

/// **Every verb a machine offers is behind its own door, by name, and admits
/// exactly the arguments it declares** — by their exact names, in its order.
#[test]
fn the_schema_sent_for_each_verb_lists_its_arguments() {
    let verbs = the_whole_machine();
    let schema = the_envelope_naming_the_verbs(&verbs);
    for verb in verbs.all() {
        let door = match verb.effect() {
            Effect::Read => "read",
            Effect::Change => "propose",
        };
        let call = the_call_of(&schema, door, verb.name());
        if verb.args().is_empty() {
            continue;
        }
        let declared: Vec<String> = verb
            .args()
            .iter()
            .map(|arg| arg.name().to_owned())
            .collect();
        assert_eq!(the_names_admitted(call), declared, "{}", verb.name());
    }
    let offered =
        the_calls_through(&schema, "read").len() + the_calls_through(&schema, "propose").len();
    assert_eq!(
        offered,
        verbs.len(),
        "a verb is missing, or one is there twice"
    );
}

/// **Every object in it is fully specified**, as a provider's strict mode
/// requires: `additionalProperties` false, and every property required. One
/// that is not is refused by the provider outright, and every turn fails.
#[test]
fn every_object_is_closed_and_every_property_required() {
    fn walk(at: &Value, path: &str) {
        match at {
            Value::Object(object) => {
                if let Some(Value::Object(properties)) = object.get("properties") {
                    assert_eq!(
                        object.get("additionalProperties"),
                        Some(&Value::Bool(false)),
                        "{path} is open"
                    );
                    let mut required: Vec<&str> = object
                        .get("required")
                        .and_then(Value::as_array)
                        .unwrap_or_else(|| panic!("{path} requires nothing"))
                        .iter()
                        .map(|name| name.as_str().unwrap())
                        .collect();
                    let mut named: Vec<&str> = properties.keys().map(String::as_str).collect();
                    required.sort_unstable();
                    named.sort_unstable();
                    assert_eq!(required, named, "{path}");
                }
                for (key, inside) in object {
                    walk(inside, &format!("{path}/{key}"));
                }
            }
            Value::Array(items) => {
                assert!(
                    !items.is_empty() || !path.ends_with("anyOf"),
                    "{path} is an empty anyOf"
                );
                for (nth, inside) in items.iter().enumerate() {
                    walk(inside, &format!("{path}/{nth}"));
                }
            }
            _ => {}
        }
    }
    walk(&the_envelope_naming_the_verbs(&the_whole_machine()), "");
    assert!(
        !the_envelope_naming_the_verbs(&the_whole_machine())
            .to_string()
            .contains("oneOf"),
        "a provider refuses oneOf"
    );
}

/// **The doors that name no verb are the envelope's own, unchanged** — the
/// command, the question and the answer.
#[test]
fn the_doors_that_name_no_verb_are_unchanged() {
    let alone = alo_models::in_the_envelope::the_envelope_a_provider_accepts();
    let named = the_envelope_naming_the_verbs(&the_whole_machine());
    let others = |schema: &Value| -> Vec<Value> {
        schema
            .pointer("/properties/asks/anyOf")
            .and_then(Value::as_array)
            .unwrap()
            .iter()
            .filter(|branch| {
                let properties = branch.get("properties").unwrap();
                properties.get("read").is_none() && properties.get("propose").is_none()
            })
            .cloned()
            .collect()
    };
    assert_eq!(others(&named), others(&alone));
    assert_eq!(named.get("required"), alone.get("required"));
    assert_eq!(
        named.pointer("/properties/format"),
        alone.pointer("/properties/format")
    );
}

/// **A registry with no verb of a kind writes no door for it**, rather than
/// an `anyOf` with no branches, which a provider refuses.
#[test]
fn a_door_with_no_verb_behind_it_is_left_out() {
    let mut reads_only = Verbs::default();
    let whole = the_whole_machine();
    for verb in whole.all().filter(|verb| verb.effect() == Effect::Read) {
        reads_only.declare(verb.clone()).unwrap();
    }
    let schema = the_envelope_naming_the_verbs(&reads_only);
    assert!(!the_calls_through(&schema, "read").is_empty());
    assert!(the_calls_through(&schema, "propose").is_empty());
    assert!(
        !schema.to_string().contains("\"propose\""),
        "a propose door with nothing behind it: {schema}"
    );
}

/// **The regression, from the walk's own log.** `gpt-4o-mini` sent
/// `accessible.type_text` an argument named `Untitled-1`, then `Untitled`, and
/// `accessible.read_window` arguments named `name` and `window`.
///
/// Two answers to each, asserted together. **The schema does not admit the
/// name**, so a provider holding the model to it cannot emit the line at all;
/// and **were it sent anyway** — by a service with no strict mode, or a paired
/// machine — the refusal names what the verb does take, so the next step can
/// be the right one.
#[test]
fn the_walks_argument_names_are_not_admitted_and_their_refusal_says_what_is() {
    let verbs = the_whole_machine();
    let schema = the_envelope_naming_the_verbs(&verbs);
    let strings = in_english();
    for (verb, door, sent, takes) in [
        (
            alo_adapters::TYPE_TEXT,
            "propose",
            "Untitled-1",
            "application, name, text",
        ),
        (
            alo_adapters::TYPE_TEXT,
            "propose",
            "Untitled",
            "application, name, text",
        ),
        (alo_adapters::READ_WINDOW, "read", "name", "application"),
        (alo_adapters::READ_WINDOW, "read", "window", "application"),
    ] {
        let admitted = the_names_admitted(the_call_of(&schema, door, verb));
        assert!(
            !admitted.iter().any(|name| name == sent),
            "{verb} admits an argument named {sent}: {admitted:?}"
        );

        let refused = verbs
            .call(verb, &[(sent, Given::text("hello from alo"))])
            .unwrap_err();
        assert!(
            matches!(&refused, CallError::NoSuchArgument { argument, .. } if argument == sent),
            "{refused:?}"
        );
        let said = refused.said(&strings);
        assert_eq!(
            said.text(),
            format!("{verb} takes {takes} — it does not take {sent}"),
        );
        assert!(said.unfilled().is_empty(), "{said:?}");
        // **Nothing the call carried is repeated**: the value is in no
        // refusal, only the name the model put where a name goes.
        assert!(!said.text().contains("hello from alo"), "{said:?}");
    }
}

/// **A verb's own declared names, sent right, are what the schema admits** —
/// the half of the pair above that stops it being a schema that admits
/// nothing.
#[test]
fn the_right_names_are_admitted_and_make_a_call() {
    let verbs = the_whole_machine();
    let schema = the_envelope_naming_the_verbs(&verbs);
    assert_eq!(
        the_names_admitted(the_call_of(&schema, "propose", alo_adapters::TYPE_TEXT)),
        ["application", "name", "text"]
    );
    assert_eq!(
        the_names_admitted(the_call_of(&schema, "read", alo_adapters::READ_WINDOW)),
        ["application"]
    );
    assert!(
        verbs
            .call(
                alo_adapters::TYPE_TEXT,
                &[
                    ("application", Given::text("com.vscodium.codium")),
                    ("name", Given::text("Untitled-1")),
                    ("text", Given::text("hello from alo")),
                ],
            )
            .is_ok()
    );
}

/// **The model is told the same names it is held to**: the paragraph about
/// working in a window names the two verbs this machine declares by their
/// constants in `alo-adapters`, and is shown after the verbs.
#[test]
fn the_steering_names_the_verbs_the_machine_really_has() {
    assert_eq!(
        alo_instructing::working_in_an_application::READ_WINDOW,
        alo_adapters::READ_WINDOW
    );
    assert_eq!(
        alo_instructing::working_in_an_application::TYPE_TEXT,
        alo_adapters::TYPE_TEXT
    );
    assert_eq!(
        alo_instructing::working_in_an_application::ACTIVATE_CONTROL,
        alo_adapters::ACTIVATE_CONTROL
    );
    let verbs = the_whole_machine();
    let told = alo_instructing::working_in_an_application(&verbs);
    assert!(told.contains("Never send a run line"), "{told}");
    let words =
        alo_instructing::the_products_words(alo_instructing::Instructions::SHOWN_TO_A_TURN, &verbs);
    assert!(words.ends_with(&told), "{words}");
}
