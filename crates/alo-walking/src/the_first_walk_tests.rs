//! What the first walk will do, read without a machine.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on a missing step is the failure being reported"
)]

use std::collections::HashSet;

use super::{THE_PROVIDER_KEY, the_first_walk};
use crate::keys;
use crate::step::{Since, Step};

/// Every text it types has a key for every character, so no run of it stops
/// on a character halfway through a field.
#[test]
fn every_text_it_types_can_be_typed() {
    for step in the_first_walk().steps() {
        if let Step::Type(text) = step {
            assert!(keys::typing(text).is_ok(), "{text:?} cannot be typed");
        }
    }
}

/// Each picture has its own name, so none overwrites another.
#[test]
fn every_picture_has_its_own_name() {
    let mut seen = HashSet::new();
    for step in the_first_walk().steps() {
        if let Step::Look { name, .. } = step {
            assert!(seen.insert(name.clone()), "{name} twice");
        }
    }
}

/// **The screen reader's checks read only what was said after the switch was
/// pressed**: the log is marked, then Space is pressed, then both checks read
/// since the mark — so a reader that started at sign-in, or a Traceback from
/// before, cannot answer them.
#[test]
fn the_screen_reader_checks_read_from_the_press() {
    let steps = the_first_walk().steps().to_vec();
    let Some(started) = steps.iter().position(|step| {
        matches!(step, Step::ExpectLine { phrase, since: Since::Mark, .. }
            if phrase == "Started alo-screen-reader.service")
    }) else {
        panic!("the walk does not check that the screen reader started");
    };
    let before: Vec<&Step> = steps.iter().take(started).collect();
    let Some(mark) = before.iter().rposition(|step| matches!(step, Step::Mark)) else {
        panic!("no mark before the screen reader check");
    };
    let pressed = before.get(mark + 1);
    assert!(
        matches!(pressed, Some(Step::Press(chord)) if chord.the_key() == "spc"),
        "the step after the mark is {pressed:?}, not Space"
    );
    assert!(
        steps.iter().any(|step| matches!(step,
            Step::ExpectNoLine { phrases, since: Since::Mark, .. }
                if phrases.iter().any(|it| it == "strange signature")
                    && phrases.iter().any(|it| it == "Traceback"))),
        "no watch for strange signature and Traceback since the switch"
    );
}

/// **The provider's key is only ever typed from its secret, after the gate
/// that skips everything without it.**
#[test]
fn the_key_is_typed_only_from_its_secret_after_its_gate() {
    let walk = the_first_walk();
    assert_eq!(walk.secrets(), vec![THE_PROVIDER_KEY]);
    let steps = walk.steps();
    let gate = steps
        .iter()
        .position(|step| *step == Step::OnlyWith(THE_PROVIDER_KEY));
    let typed = steps
        .iter()
        .position(|step| *step == Step::TypeSecret(THE_PROVIDER_KEY));
    assert!(
        matches!((gate, typed), (Some(gate), Some(typed)) if gate < typed),
        "gate {gate:?}, typed {typed:?}"
    );
}
