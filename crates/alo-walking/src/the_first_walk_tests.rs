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
    // The every-application gate is named too, because every variable a walk
    // reads is redacted from what it keeps; its value is a word of its own
    // (walk.yml), so redacting it hides nothing else.
    assert_eq!(
        walk.secrets(),
        vec![
            "ALO_WALK_PRIVATE_STUDIO",
            crate::in_every_shipped_application::EVERY_APPLICATION,
            THE_PROVIDER_KEY
        ]
    );
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

/// **The terminal's check reads only what was said after Ctrl+Alt+T**, so a
/// terminal opened before the mark cannot answer it, and it waits for the
/// shell's own words (`alo-shell`'s `the_walk_waits_for_the_words_this_says`
/// holds that those are the words the shell says).
#[test]
fn the_terminal_check_reads_from_the_chord() {
    let steps = the_first_walk().steps().to_vec();
    let Some(check) = steps.iter().position(|step| {
        matches!(step, Step::ExpectLine { phrase, since: Since::Mark, .. }
            if phrase == super::THE_TERMINAL_AT_REST)
    }) else {
        panic!("the walk does not check the terminal at rest");
    };
    let before: Vec<&Step> = steps.iter().take(check).collect();
    let Some(mark) = before.iter().rposition(|step| matches!(step, Step::Mark)) else {
        panic!("no mark before the terminal check");
    };
    assert!(
        matches!(before.get(mark + 1), Some(Step::Press(chord))
            if chord.the_key() == "t" && chord.held() == ["ctrl", "alt"]),
        "the step after the mark is not Ctrl+Alt+T"
    );
    let gate = steps
        .iter()
        .position(|step| matches!(step, Step::OnlyWith(_)))
        .unwrap_or(steps.len());
    assert!(
        check < gate,
        "the terminal check is skipped without a provider key"
    );
}

/// **The terminal's maximise check reads only what was said after the panel
/// was opened with Alt+F3**, and waits for the shell's own words
/// (`alo-shell`'s `the_walk_waits_for_the_words_the_panel_says` holds that).
#[test]
fn the_maximise_check_reads_from_alt_f3() {
    let steps = the_first_walk().steps().to_vec();
    let Some(check) = steps.iter().position(|step| {
        matches!(step, Step::ExpectLine { phrase, since: Since::Mark, .. }
            if phrase == super::THE_TERMINAL_MAXIMISED)
    }) else {
        panic!("the walk does not check the terminal maximised");
    };
    let before: Vec<&Step> = steps.iter().take(check).collect();
    let Some(mark) = before.iter().rposition(|step| matches!(step, Step::Mark)) else {
        panic!("no mark before the maximise check");
    };
    assert!(
        matches!(before.get(mark + 1), Some(Step::Press(chord))
            if chord.the_key() == "f3" && chord.held() == ["alt"]),
        "the step after the mark is not Alt+F3"
    );
    let gate = steps
        .iter()
        .position(|step| matches!(step, Step::OnlyWith(_)))
        .unwrap_or(steps.len());
    assert!(
        check < gate,
        "the maximise check is skipped without a provider key"
    );
}

/// **The four sections alo is asked in**: each starts from its own Escape and
/// sets its own level and grants, so a failure in one does not stop the next —
/// full-access VSCodium, every application, ask-each-time VSCodium and the
/// Papers sheet, in that order — **except the Papers sheet, which needs
/// ask-each-time VSCodium.** A failure there leaves its turn waiting on a card,
/// and `alo-agentd` serves one turn at a time (`serving.rs`: *a second agent
/// arriving while a turn is under way is refused in words and closed*), so the
/// Papers ask then fails on a broken pipe that says nothing about Papers (runs
/// 44069105568 and 44161055698). Nothing a person can press ends a waiting
/// turn from any state; a Stop shortcut would be a product promise, and is not
/// being built here.
#[test]
fn the_sections_alo_is_asked_in_are_independent() {
    let sections: Vec<(String, Vec<String>)> = the_first_walk()
        .steps()
        .iter()
        .filter_map(|step| match step {
            Step::Section { name, needs } => Some((
                name.clone(),
                needs.iter().map(|it| (*it).to_owned()).collect(),
            )),
            _ => None,
        })
        .collect();
    let names: Vec<&str> = sections.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "VSCodium at full access",
            "every application at full access",
            "VSCodium at ask each time",
            "the Papers sheet at ask each time",
        ]
    );
    let needs: Vec<Vec<String>> = sections.into_iter().map(|(_, needs)| needs).collect();
    assert_eq!(
        needs,
        vec![
            Vec::<String>::new(),
            Vec::new(),
            Vec::new(),
            vec!["VSCodium at ask each time".to_owned()],
        ]
    );
}
