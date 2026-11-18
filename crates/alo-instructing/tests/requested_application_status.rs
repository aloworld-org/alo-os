//! Status advice is conditional on the person's requested app and operation.
//!
//! Synthetic declarations keep this generic. These inspect actual product
//! instructions, not model compliance; every-try cases remain independently
//! responsible for catching unrelated reads and wrong setting changes.

#![expect(
    clippy::expect_used,
    reason = "invalid test declarations or missing guidance must fail"
)]

use alo_capability::{Effect, Requires, Verb, Verbs};
use alo_instructing::{Instructions, the_products_words};
use alo_strings::Word;

const HEADING: &str = "Named application controls:\n";
const STATUS_SCOPE: &str = "Use a status read only for the application in the person's request.";
const NO_STATUS: &str =
    "If that application has no offered status read, use the generic application guidance.";
const MATCH_FIRST: &str =
    "Match the requested application and operation to a declared purpose before reading status.";
const PRESERVE_OPERATION: &str =
    "After reading status, keep the requested operation and leave unrelated settings unchanged.";

fn offered() -> Verbs {
    let mut verbs = Verbs::default();
    for (name, effect, purpose) in [
        (
            "weave.status",
            Effect::Read,
            "read the loom's current recording settings",
        ),
        (
            "weave.countdown_five",
            Effect::Change,
            "set a five-second countdown",
        ),
        ("weave.quality", Effect::Change, "set recording quality"),
        (
            "garden.status",
            Effect::Read,
            "read the garden's live watering state",
        ),
        ("garden.water", Effect::Change, "water the selected bed"),
        (
            "adjust_slider",
            Effect::Change,
            "adjust the person's selected slider",
        ),
        (
            "close_application",
            Effect::Change,
            "close the application the person requested",
        ),
    ] {
        verbs
            .declare(
                Verb::checked(
                    name,
                    Word::saying("test.requested.purpose", purpose),
                    effect,
                    vec![],
                    Requires::nothing_because(
                        "synthetic instruction fixture has no machine access",
                    ),
                    Word::saying("test.requested.sentence", purpose),
                )
                .expect("valid declaration"),
            )
            .expect("distinct declaration");
    }
    verbs
}

fn advice(text: &str) -> (&str, &str) {
    let (before, named) = text
        .split_once(HEADING)
        .expect("actual named-control instruction paragraph");
    let status = before
        .split_once("Read the live status before answering")
        .expect("actual current-state instruction paragraph")
        .1;
    let named = named.split("\n\n").next().expect("named paragraph");
    (status, named)
}

#[test]
fn current_state_questions_use_only_the_requested_applications_status_with_a_real_fallback() {
    let verbs = offered();
    for instructions in Instructions::ALL {
        let text = the_products_words(instructions, &verbs);
        let (status, _) = advice(&text);
        assert!(
            status.contains(STATUS_SCOPE),
            "an offered status read is not a prerequisite for an unrelated slider/app task: {status}"
        );
        assert!(
            status.contains(NO_STATUS),
            "an app without a status read must not borrow another family's: {status}"
        );
        assert!(status.contains("weave.status") && status.contains("garden.status"));
        assert!(
            status.contains("verb's description is not a live"),
            "scope must preserve genuine state-before-answer guidance"
        );
    }
}

#[test]
fn named_change_guidance_matches_the_requested_app_and_operation_before_reading_state() {
    let verbs = offered();
    for instructions in Instructions::ALL {
        let text = the_products_words(instructions, &verbs);
        let (_, named) = advice(&text);
        assert!(
            named.contains(STATUS_SCOPE),
            "named-control advice also needs the requested-app boundary: {named}"
        );
        let matched = named
            .find(MATCH_FIRST)
            .expect("match requested purpose before reading state");
        let read = named
            .find("Read the live status before choosing a change")
            .expect("retain state-before-change guidance");
        assert!(
            matched < read,
            "matching request scope must precede the live-read instruction: {named}"
        );
        assert!(
            named.contains(PRESERVE_OPERATION),
            "current quality must not turn a countdown request into a quality change: {named}"
        );
        for retained in [
            "Prefer the matching named control",
            "file action",
            "accessibility gesture",
            "another setting",
            "Use the generic application guidance when no offered named control matches",
        ] {
            assert!(
                named.contains(retained),
                "missing prior requirement {retained}: {named}"
            );
        }
    }
}

#[test]
fn scope_guidance_preserves_live_registry_purposes_and_unrelated_generic_operations() {
    let verbs = offered();
    let text = the_products_words(Instructions::SHOWN_TO_A_TURN, &verbs);
    let (_, named) = advice(&text);
    for (name, purpose) in [
        ("weave.status", "read the loom's current recording settings"),
        ("weave.countdown_five", "set a five-second countdown"),
        ("weave.quality", "set recording quality"),
        ("garden.status", "read the garden's live watering state"),
        ("garden.water", "water the selected bed"),
    ] {
        assert!(
            named
                .lines()
                .any(|line| line.contains(name) && line.contains(purpose)),
            "lost registry purpose: {name}"
        );
    }
    for generic in ["adjust_slider", "close_application"] {
        assert!(text.contains(generic), "generic operation remains offered");
        assert!(
            !named.contains(generic),
            "generic operation must not be assigned to a status family"
        );
    }
    assert!(
        !text.contains("studio."),
        "generic guidance must not invent Studio controls"
    );
}

#[test]
fn a_completed_generic_application_operation_does_not_start_a_named_status_read() {
    let verbs = offered();
    for instructions in Instructions::ALL {
        let text = the_products_words(instructions, &verbs);
        let (_, named) = advice(&text);
        assert!(named.contains("After a generic application operation completes the person's whole request, answer the person"));
        assert!(named.contains("do not append a named status read"));
        assert!(named.contains("If requested steps remain, continue those steps"));
    }
}
