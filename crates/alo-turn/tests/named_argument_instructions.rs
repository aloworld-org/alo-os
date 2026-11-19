//! Rendered argument examples must agree with the real hosted call schema.
//!
//! These checks use existing product entrypoints, the actual machine registry
//! and the actual hosted schema. Historical instruction sets keep their bytes,
//! digests and list-form catalogues. The current set must be a separately named
//! measurement with object arguments on initial and continued prompts. No model
//! request, accepted road, policy, provider constraint or retry is changed here.

#![expect(
    clippy::expect_used,
    reason = "missing public declarations, schema fields or rendered examples fail the regression"
)]

use std::collections::BTreeSet;

use alo_capability::{Arg, Effect, Requires, Takes, Verb, Verbs};
use alo_instructing::{
    AStep, ATurnInProgress, Instructions, WhatItMayReach, shown_in_progress, shown_to_a_model,
    shown_to_a_turn, the_products_words,
};
use alo_strings::Word;
use alo_turn::the_shape_a_provider_is_held_to::the_envelope_naming_the_verbs;
use serde_json::{Value, json};

/// The five published instruction sets and independently hashed literal bytes.
const HISTORICAL: [(Instructions, &str, &str); 5] = [
    (
        Instructions::AsFirstWritten,
        "as-first-written",
        "d468e469651d778ae369c53e37816fce62c80f703de729a074bcf8ff44a5adce",
    ),
    (
        Instructions::OneExamplePerDoor,
        "one-example-per-door",
        "93a7f458ce9d017d6d12759a281e5f0b347a0d6963c04eae03b4c30581aebd02",
    ),
    (
        Instructions::SeveralSteps,
        "several-steps",
        "dbe41854f379d4b0bbb3e0dc9fd55b39be6907c6092f0d0571226e0bb2c43f75",
    ),
    (
        Instructions::EveryDoorAndAnAnswer,
        "every-door-and-an-answer",
        "cf522fc50729caf13ce79d9fcb3fd44d48b95cc531316b9e4dac4a67c84ff406",
    ),
    (
        Instructions::EveryDoorAnAnswerAndACommand,
        "every-door-an-answer-and-a-command",
        "1dd7243b1984c2a710b0da702edbad2652e150caba7029641da271c8de0f641e",
    ),
];

/// Hash the actual text independently of the API that identifies a grade.
fn digest(text: &str) -> String {
    ring::digest::digest(&ring::digest::SHA256, text.as_bytes())
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Find one actual declared verb's given line, not advice elsewhere in the text.
fn catalogue_given<'a>(text: &'a str, name: &str) -> &'a str {
    let beginning = format!("\n\n- {name} (");
    let (_, rest) = text
        .split_once(&beginning)
        .expect("the actual offered verb remains in the catalogue");
    let block = rest
        .split("\n\n-")
        .next()
        .expect("the declared verb has a catalogue block");
    block
        .lines()
        .find_map(|line| line.strip_prefix("  given: "))
        .expect("the actual verb retains an explicit given example")
}

/// VALUE is an unfilled value, not a property name or a wire representation.
fn parse_given(text: &str) -> Value {
    serde_json::from_str(&text.replace("VALUE", "null"))
        .expect("the argument example is JSON apart from its documented VALUE placeholder")
}

/// Locate the independently generated hosted schema branch for this actual verb.
fn schema_call<'a>(schema: &'a Value, verb: &Verb) -> &'a Value {
    let door = match verb.effect() {
        Effect::Read => "read",
        Effect::Change => "propose",
    };
    schema
        .pointer("/properties/asks/anyOf")
        .and_then(Value::as_array)
        .expect("the hosted schema offers its real doors")
        .iter()
        .find_map(|branch| branch.pointer(&format!("/properties/{door}/anyOf")))
        .and_then(Value::as_array)
        .expect("the verb's actual read or propose door exists")
        .iter()
        .find(|branch| {
            branch
                .pointer("/properties/verb/enum/0")
                .and_then(Value::as_str)
                == Some(verb.name())
        })
        .expect("the schema offers the same declared verb as the prompt")
}

/// Match every rendered argument name and empty-object shape to the real schema.
fn matches_current_schema(text: &str, verbs: &Verbs) {
    let schema = the_envelope_naming_the_verbs(verbs);
    for verb in verbs.all() {
        let rendered = parse_given(catalogue_given(text, verb.name()));
        let object = rendered.as_object().expect(
            "the current catalogue must teach object arguments, matching the strict hosted schema",
        );
        let call = schema_call(&schema, verb);
        assert_eq!(
            call.pointer("/properties/given/type"),
            Some(&json!("object"))
        );
        assert_eq!(
            call.pointer("/properties/given/additionalProperties"),
            Some(&json!(false))
        );
        let offered = call
            .pointer("/properties/given/properties")
            .and_then(Value::as_object)
            .expect("the strict schema lists its actual argument properties");
        let required = call
            .pointer("/properties/given/required")
            .and_then(Value::as_array)
            .expect("every actual argument remains required");
        let names: BTreeSet<_> = object.keys().map(String::as_str).collect();
        let schema_names: BTreeSet<_> = offered.keys().map(String::as_str).collect();
        let required_names: BTreeSet<_> = required
            .iter()
            .map(|name| name.as_str().expect("an actual required argument name"))
            .collect();
        assert_eq!(names, schema_names, "{} argument names differ", verb.name());
        assert_eq!(names, required_names, "{} omits an argument", verb.name());
        if verb.args().is_empty() {
            assert_eq!(rendered, json!({}), "{} takes no arguments", verb.name());
        }
    }
}

/// Another application family and a plain read prevent Studio-only formatting.
fn independent_registry() -> Verbs {
    let mut verbs = Verbs::default();
    for (name, effect, args, sentence) in [
        (
            "other.inspect",
            Effect::Read,
            vec![],
            "inspect this test-only fixture",
        ),
        (
            "other.refresh",
            Effect::Change,
            vec![],
            "refresh this test-only fixture",
        ),
        (
            "different.exchange",
            Effect::Change,
            vec![
                Arg::taking(
                    "destination",
                    Word::saying(
                        "testing.named-arguments.destination",
                        "the destination text",
                    ),
                    Takes::Text,
                ),
                Arg::taking(
                    "payload",
                    Word::saying("testing.named-arguments.payload", "the payload text"),
                    Takes::Text,
                ),
            ],
            "exchange {payload} with {destination}",
        ),
    ] {
        verbs
            .declare(
                Verb::checked(
                    name,
                    Word::saying(
                        "testing.named-arguments.purpose",
                        "inspect this test-only fixture",
                    ),
                    effect,
                    args,
                    Requires::nothing_because("these declarations have no executor or authority"),
                    Word::saying("testing.named-arguments.sentence", sentence),
                )
                .expect("the independent declaration is valid"),
            )
            .expect("every independent declaration has a distinct name");
    }
    verbs
}

#[test]
/// The changed current instructions must not silently reuse an old grade's name.
fn current_object_arguments_have_a_separately_named_instruction_set() {
    let current = Instructions::SHOWN_TO_A_TURN;
    for (old, name, fingerprint) in HISTORICAL {
        assert_ne!(
            current, old,
            "object arguments need a new instruction variant"
        );
        assert_ne!(current.named(), name);
        assert_ne!(current.digest(), fingerprint);
    }
    assert_eq!(Instructions::of_digest(&current.digest()), Some(current));
    assert!(Instructions::ALL.contains(&current));
}

#[test]
/// Generic read and propose examples use the same object representation as calls.
fn current_generic_door_examples_do_not_teach_the_legacy_list() {
    let mut doors = BTreeSet::new();
    for line in Instructions::SHOWN_TO_A_TURN.text().lines() {
        if !line.starts_with("{\"format\":") || !line.contains("\"verb\":\"NAME\"") {
            continue;
        }
        let example: Value = serde_json::from_str(&line.replace("VALUE", "null"))
            .expect("generic protocol examples remain valid JSON with a value supplied");
        for door in ["read", "propose"] {
            if let Some(given) = example.pointer(&format!("/asks/{door}/given")) {
                assert!(
                    given.is_object(),
                    "the {door} example conflicts with the hosted schema"
                );
                doors.insert(door);
            }
        }
    }
    assert_eq!(doors, BTreeSet::from(["read", "propose"]));
}

#[test]
/// The real machine's whole catalogue matches its real strict provider schema.
fn current_catalogue_matches_every_real_hosted_argument_property() {
    let verbs = alo_turn::the_verbs_with_application_controls().expect("the real offered registry");
    let text = the_products_words(Instructions::SHOWN_TO_A_TURN, &verbs);
    matches_current_schema(&text, &verbs);
}

#[test]
/// Empty argument objects apply to every no-argument read and change, not one name.
fn current_no_argument_calls_are_objects_on_both_doors() {
    let verbs = independent_registry();
    let text = the_products_words(Instructions::SHOWN_TO_A_TURN, &verbs);
    for name in ["other.inspect", "other.refresh"] {
        assert_eq!(parse_given(catalogue_given(&text, name)), json!({}));
    }
    matches_current_schema(&text, &verbs);
}

#[test]
/// Actual unfamiliar argument names are properties, never named/is list entries.
fn current_multi_argument_call_uses_its_own_declared_property_names() {
    let verbs = independent_registry();
    let text = the_products_words(Instructions::SHOWN_TO_A_TURN, &verbs);
    assert_eq!(
        parse_given(catalogue_given(&text, "different.exchange")),
        json!({"destination": null, "payload": null})
    );
}

#[test]
/// Every published literal set remains identifiable and byte-for-byte unchanged.
fn historical_instruction_bytes_names_and_digests_remain_published() {
    for (old, name, fingerprint) in HISTORICAL {
        assert_eq!(old.named(), name);
        assert_eq!(
            digest(old.text()),
            fingerprint,
            "{name} changed its historical bytes"
        );
        assert_eq!(old.digest(), fingerprint);
        assert_eq!(Instructions::of_digest(fingerprint), Some(old));
        assert!(Instructions::ALL.contains(&old));
    }
}

#[test]
/// Reproducing an old measurement still renders its original list-form catalogue.
fn historical_catalogues_keep_their_list_argument_representation() {
    let verbs = alo_turn::the_verbs_with_application_controls().expect("the real offered registry");
    for (old, _, _) in HISTORICAL {
        let text = the_products_words(old, &verbs);
        for verb in verbs.all() {
            let expected: Vec<Value> = verb
                .args()
                .iter()
                .map(|argument| json!({"named": argument.name(), "is": null}))
                .collect();
            assert_eq!(
                parse_given(catalogue_given(&text, verb.name())),
                json!(expected),
                "{} changed the old {} catalogue",
                verb.name(),
                old.named()
            );
        }
    }
}

#[test]
/// The actual turn entrypoint renders exactly the current shared product set.
fn initial_turn_and_explicit_current_rendering_are_identical() {
    let verbs = independent_registry();
    let reach = WhatItMayReach::nothing();
    let request = "Tell me this fixture's state.";
    let explicit = shown_to_a_model(Instructions::SHOWN_TO_A_TURN, &verbs, &reach, request);
    assert_eq!(shown_to_a_turn(&verbs, &reach, request), explicit);
    assert!(explicit.starts_with(Instructions::SHOWN_TO_A_TURN.text()));
    assert!(explicit.ends_with(&format!("The request: {request}")));
}

#[test]
/// A continuation retains the current schema-aligned catalogue and exact wire data.
fn continued_turn_uses_current_object_examples_without_rewriting_observed_lines() {
    let verbs = alo_turn::the_verbs_with_application_controls().expect("the real offered registry");
    let reach = WhatItMayReach::nothing();
    let asked = r#"{"format":1,"asks":{"read":{"verb":"studio.status","given":{}}}}"#;
    let reply = r#"{"format":1,"tells":{"did":{"application_controlled":{"application":"org.aloworld.Studio","action":"studio.status","reply":"{\"version\":1,\"phase\":\"paused\"}","unanswered":false}}}}"#;
    let steps = [AStep::of(asked, reply)];
    let request = "Tell me whether the recording is paused.";
    let text = shown_in_progress(
        Instructions::SHOWN_TO_A_TURN,
        &verbs,
        ATurnInProgress::of(request, &steps, &reach),
    );
    assert!(text.starts_with(&the_products_words(Instructions::SHOWN_TO_A_TURN, &verbs)));
    matches_current_schema(&text, &verbs);
    assert!(text.contains(&format!("You asked: {asked}")));
    assert!(text.contains(&format!("The answer: {reply}")));
    assert!(text.contains(request));
}
