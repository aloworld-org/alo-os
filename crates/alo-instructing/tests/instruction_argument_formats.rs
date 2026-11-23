//! The owning instruction crate must hold its catalogue to each instruction set's
//! argument format. These declarations are test-only and reach no application.
//! This checks actual public prompt rendering, not model choice or hosted transport.

#![expect(
    clippy::expect_used,
    reason = "invalid independent declarations or missing catalogue lines fail the test"
)]

use alo_capability::{Arg, Effect, Requires, Takes, Verb, Verbs};
use alo_instructing::{Instructions, the_products_words, verb_as_told};
use alo_strings::Word;

/// Declarations with no arguments in both doors and unfamiliar named arguments.
fn offered() -> Verbs {
    let mut verbs = Verbs::default();
    for (name, effect, args, sentence) in [
        (
            "sample.inspect",
            Effect::Read,
            vec![],
            "inspect this fixture",
        ),
        (
            "sample.refresh",
            Effect::Change,
            vec![],
            "refresh this fixture",
        ),
        (
            "sample.exchange",
            Effect::Change,
            vec![
                Arg::taking(
                    "destination",
                    Word::saying(
                        "testing.argument-formats.destination",
                        "the destination text",
                    ),
                    Takes::Text,
                ),
                Arg::taking(
                    "payload",
                    Word::saying("testing.argument-formats.payload", "the payload text"),
                    Takes::Text,
                ),
            ],
            "exchange {payload} with {destination}",
        ),
    ] {
        let verb = Verb::checked(
            name,
            Word::saying(
                "testing.argument-formats.purpose",
                "inspect this test-only fixture",
            ),
            effect,
            args,
            Requires::nothing_because("these declarations have no executor or authority"),
            Word::saying("testing.argument-formats.sentence", sentence),
        )
        .expect("the independent declaration is valid");
        verbs.declare(verb).expect("fixture names are distinct");
    }
    verbs
}

/// Read this verb's catalogue line rather than a matching generic example.
fn given<'a>(prompt: &'a str, name: &str) -> &'a str {
    let start = format!("\n\n- {name} (");
    let (_, following) = prompt
        .split_once(&start)
        .expect("the offered verb is present");
    let block = following.split("\n\n").next().expect("a catalogue block");
    block
        .lines()
        .find_map(|line| line.strip_prefix("  given: "))
        .expect("this verb declares its given format")
}

/// Current no-argument reads and changes use the hosted object's empty shape.
#[test]
fn current_no_argument_reads_and_changes_are_empty_objects() {
    assert_eq!(Instructions::SHOWN_TO_A_TURN, Instructions::NamedArguments);
    let prompt = the_products_words(Instructions::SHOWN_TO_A_TURN, &offered());
    assert_eq!(given(&prompt, "sample.inspect"), "{}");
    assert_eq!(given(&prompt, "sample.refresh"), "{}");
}

/// Object keys are the offered argument names, not generic named/is wrappers.
#[test]
fn current_arguments_are_an_object_with_the_actual_declared_names() {
    let prompt = the_products_words(Instructions::SHOWN_TO_A_TURN, &offered());
    assert_eq!(
        given(&prompt, "sample.exchange"),
        r#"{"destination":VALUE,"payload":VALUE}"#
    );
    assert!(prompt.contains("\n  - destination (required; "));
    assert!(prompt.contains("\n  - payload (required; "));
}

/// Historical grades retain list arguments, including empty lists at both doors.
#[test]
fn every_historical_set_keeps_its_list_catalogue() {
    for instructions in [
        Instructions::AsFirstWritten,
        Instructions::OneExamplePerDoor,
        Instructions::SeveralSteps,
        Instructions::EveryDoorAndAnAnswer,
        Instructions::EveryDoorAnAnswerAndACommand,
    ] {
        let prompt = the_products_words(instructions, &offered());
        assert_eq!(
            given(&prompt, "sample.inspect"),
            "[]",
            "{}",
            instructions.named()
        );
        assert_eq!(
            given(&prompt, "sample.refresh"),
            "[]",
            "{}",
            instructions.named()
        );
        assert_eq!(
            given(&prompt, "sample.exchange"),
            r#"[{"named":"destination","is":VALUE},{"named":"payload","is":VALUE}]"#,
            "{}",
            instructions.named()
        );
    }
}

/// The explicitly legacy public verb renderer remains a list-based API.
#[test]
fn legacy_public_renderers_keep_their_existing_argument_representation() {
    let verbs = offered();
    let empty = verbs
        .of("sample.inspect")
        .expect("the fixture read is offered");
    let change = verbs
        .of("sample.exchange")
        .expect("the fixture change is offered");
    assert_eq!(verb_as_told::given_as_told(empty), "[]");
    assert!(verb_as_told::as_told(empty).ends_with("\n  given: []"));
    let expected = r#"[{"named":"destination","is":VALUE},{"named":"payload","is":VALUE}]"#;
    assert_eq!(verb_as_told::given_as_told(change), expected);
    assert!(verb_as_told::as_told(change).ends_with(&format!("\n  given: {expected}")));
}
