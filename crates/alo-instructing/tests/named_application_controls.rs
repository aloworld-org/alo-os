//! Named-control instructions are derived from offered application declarations.
//!
//! These synthetic families deliberately are not Studio. The real product entry
//! point must generate guidance, beyond merely repeating the verb catalogue,
//! from each offered `.status` read and its family's named changes. This checks
//! instruction construction, not repeated model choice or installed app state.

#![expect(
    clippy::unwrap_used,
    reason = "invalid fixture declarations or missing promised guidance fail the test"
)]

use alo_capability::{Effect, Requires, Verb, Verbs};
use alo_instructing::{the_products_words, Instructions};
use alo_strings::Word;

/// A separate paragraph makes actionable advice distinguishable from the catalogue.
const HEADING: &str = "Named application controls:\n";
/// Selection must follow a fresh observation, rather than a guessed state.
const READ_FIRST: &str = "Read the live status before choosing a change";
/// A matching semantic operation takes precedence over incidental mechanisms.
const PREFERENCE: &str = "Prefer the matching named control";
/// Having some named controls must not pretend they cover every possible request.
const FALLBACK: &str = "Use the generic application guidance when no offered named control matches";

/// Test-only application declarations with no machine access or implementations.
fn registry(declarations: &[(&str, Effect, &'static str)]) -> Verbs {
    let mut verbs = Verbs::default();
    for (name, effect, purpose) in declarations {
        verbs
            .declare(
                Verb::checked(
                    name,
                    Word::saying("testing.named-control.purpose", purpose),
                    *effect,
                    vec![],
                    Requires::nothing_because("these synthetic declarations reach no machine"),
                    Word::saying("testing.named-control.sentence", purpose),
                )
                .unwrap(),
            )
            .unwrap();
    }
    verbs
}

/// Inspect only the generated advice, so ordinary catalogue text cannot pass.
fn guidance(text: &str) -> &str {
    assert_eq!(text.matches(HEADING).count(), 1, "{text}");
    text.split_once(HEADING)
        .unwrap()
        .1
        .split("\n\n")
        .next()
        .unwrap()
}

/// Each enumerated control retains its own declared meaning on the same line.
fn has_entry(guidance: &str, name: &str, purpose: &str) {
    assert!(
        guidance
            .lines()
            .any(|line| line.contains(name) && line.contains(purpose)),
        "guidance has no declared name/purpose entry for {name}: {guidance}"
    );
}

/// Check the sequencing, preference and limited scope of the selection rule.
fn has_choice_rule(guidance: &str) {
    for required in [
        READ_FIRST,
        PREFERENCE,
        "file action",
        "accessibility gesture",
        "another setting",
        FALLBACK,
    ] {
        assert!(
            guidance.contains(required),
            "missing {required}: {guidance}"
        );
    }
}

#[test]
/// Every instruction variant receives registry-derived application advice.
fn product_guidance_reads_live_state_and_prefers_every_matching_declared_change() {
    let verbs = registry(&[
        (
            "fixture.status",
            Effect::Read,
            "inspect the fixture's current capture state",
        ),
        (
            "fixture.enable_microphone",
            Effect::Change,
            "enable this recording's microphone",
        ),
        (
            "fixture.countdown_3",
            Effect::Change,
            "set this recording's countdown to three seconds",
        ),
        (
            "fixture.countdown_5",
            Effect::Change,
            "set this recording's countdown to five seconds",
        ),
        (
            "fixture.pause",
            Effect::Change,
            "pause the current recording",
        ),
        (
            "fixture.other_read",
            Effect::Read,
            "read unrelated fixture metadata",
        ),
        (
            "fixture_extra.erase",
            Effect::Change,
            "change a different application",
        ),
        (
            "orphan.countdown_3",
            Effect::Change,
            "change an application with no status read",
        ),
    ]);
    for instructions in Instructions::ALL {
        let text = the_products_words(instructions, &verbs);
        assert!(
            text.starts_with(instructions.text()),
            "generic instructions moved: {text}"
        );
        let named = guidance(&text);
        has_choice_rule(named);
        has_entry(
            named,
            "fixture.status",
            "inspect the fixture's current capture state",
        );
        for (name, purpose) in [
            (
                "fixture.enable_microphone",
                "enable this recording's microphone",
            ),
            (
                "fixture.countdown_3",
                "set this recording's countdown to three seconds",
            ),
            (
                "fixture.countdown_5",
                "set this recording's countdown to five seconds",
            ),
            ("fixture.pause", "pause the current recording"),
        ] {
            has_entry(named, name, purpose);
        }
        for excluded in [
            "fixture.other_read",
            "fixture_extra.erase",
            "orphan.countdown_3",
        ] {
            assert!(
                !named.contains(excluded),
                "{excluded} was presented as a matching change: {named}"
            );
            assert!(
                text.contains(excluded),
                "the ordinary registry catalogue disappeared: {text}"
            );
        }
        assert!(
            !text.contains("studio."),
            "synthetic registries must not invent Studio: {text}"
        );
    }
}

#[test]
/// Guidance cannot be a Studio-specific list or mix up two applications' reads.
fn two_arbitrary_application_families_keep_their_own_status_and_declared_purposes() {
    let verbs = registry(&[
        (
            "orchard.status",
            Effect::Read,
            "read the orchard's live watering state",
        ),
        ("orchard.water", Effect::Change, "water the selected bed"),
        (
            "loom.status",
            Effect::Read,
            "read the loom's live weaving state",
        ),
        ("loom.pause", Effect::Change, "pause the current weave"),
    ]);
    let text = the_products_words(Instructions::SHOWN_TO_A_TURN, &verbs);
    let named = guidance(&text);
    has_choice_rule(named);
    for verb in verbs.all() {
        has_entry(named, verb.name(), verb.purpose_as_written());
    }
    assert!(
        named.find("orchard.status").unwrap() < named.find("orchard.water").unwrap(),
        "{named}"
    );
    assert!(
        named.find("loom.status").unwrap() < named.find("loom.pause").unwrap(),
        "{named}"
    );
    assert!(
        !named.contains("fixture."),
        "the previous fixture is not a product control: {named}"
    );
}

#[test]
/// Registry insertion order cannot hide changes declared before the status read.
fn a_status_read_declared_last_still_guides_every_offered_change() {
    let verbs = registry(&[
        (
            "weaver.tighten",
            Effect::Change,
            "tighten the selected thread",
        ),
        (
            "weaver.release",
            Effect::Change,
            "release the selected thread",
        ),
        (
            "weaver.status",
            Effect::Read,
            "read the current thread tension",
        ),
    ]);
    let text = the_products_words(Instructions::SHOWN_TO_A_TURN, &verbs);
    let named = guidance(&text);
    has_choice_rule(named);
    for verb in verbs.all() {
        has_entry(named, verb.name(), verb.purpose_as_written());
    }
    for change in ["weaver.tighten", "weaver.release"] {
        assert!(
            named.find("weaver.status").unwrap() < named.find(change).unwrap(),
            "live observation must still come before the change: {named}"
        );
    }
}

#[test]
/// Removing a declaration removes both its catalogue entry and advice to use it.
fn withdrawn_or_unoffered_controls_are_never_added_by_instruction_guidance() {
    let full = registry(&[
        ("fixture.status", Effect::Read, "read current fixture state"),
        (
            "fixture.keep",
            Effect::Change,
            "keep the chosen configuration",
        ),
        (
            "fixture.withdrawn",
            Effect::Change,
            "change the withdrawn configuration",
        ),
    ]);
    let offered = full.selecting(|verb| verb.name() != "fixture.withdrawn");
    let text = the_products_words(Instructions::SHOWN_TO_A_TURN, &offered);
    let named = guidance(&text);
    has_entry(named, "fixture.keep", "keep the chosen configuration");
    assert!(
        !text.contains("fixture.withdrawn"),
        "withdrawal must also remove advice: {text}"
    );
    assert!(
        !text.contains("change the withdrawn configuration"),
        "{text}"
    );
    assert!(!text.contains("fixture.enable_microphone"), "{text}");
}

#[test]
/// The registry remains the sole maintained description of a named operation.
fn changing_a_declarations_purpose_changes_the_guidance_without_a_second_description() {
    for purpose in [
        "set the fixture's selected input",
        "choose its presentation background",
    ] {
        let verbs = registry(&[
            ("fixture.status", Effect::Read, "read current fixture state"),
            ("fixture.configure", Effect::Change, purpose),
        ]);
        let text = the_products_words(Instructions::SHOWN_TO_A_TURN, &verbs);
        has_entry(guidance(&text), "fixture.configure", purpose);
    }
}

#[test]
/// Both the exact family boundary and the status operation's read effect matter.
fn a_change_named_status_and_a_different_familys_status_are_not_a_status_read() {
    let verbs = registry(&[
        (
            "impostor.status",
            Effect::Change,
            "change the impostor's state",
        ),
        ("impostor.enable", Effect::Change, "enable the impostor"),
        (
            "real.status",
            Effect::Read,
            "read the real application state",
        ),
        ("real.enable", Effect::Change, "enable the real application"),
        (
            "realish.enable",
            Effect::Change,
            "enable a lookalike application",
        ),
    ]);
    let text = the_products_words(Instructions::SHOWN_TO_A_TURN, &verbs);
    let named = guidance(&text);
    has_entry(named, "real.status", "read the real application state");
    has_entry(named, "real.enable", "enable the real application");
    for excluded in ["impostor.status", "impostor.enable", "realish.enable"] {
        assert!(!named.contains(excluded), "{excluded}: {named}");
    }
}

#[test]
/// No matching declared operation means the existing generic guidance still applies.
fn applications_without_matching_named_changes_keep_the_generic_product_instructions() {
    for verbs in [
        Verbs::default(),
        registry(&[(
            "orphan.enable",
            Effect::Change,
            "enable an application without a status read",
        )]),
        registry(&[(
            "readonly.status",
            Effect::Read,
            "read state without any offered change",
        )]),
    ] {
        for instructions in Instructions::ALL {
            let text = the_products_words(instructions, &verbs);
            assert!(
                text.starts_with(instructions.text()),
                "generic fallback disappeared: {text}"
            );
            // A status-only family may receive read advice, but never an invented
            // mutation or a categorical ban on the generic application road.
            assert!(!text.contains("readonly.enable"), "{text}");
            assert!(!text.contains("orphan.status"), "{text}");
            if text.contains(HEADING) {
                assert!(guidance(&text).contains(FALLBACK), "{text}");
            }
        }
    }
}
