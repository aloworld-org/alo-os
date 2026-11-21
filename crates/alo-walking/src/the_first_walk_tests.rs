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

/// **A provider that needs no key is added on every walk**, before the gate
/// that skips everything without a real provider's key: the switch is pressed
/// with Space, and the walk waits — since a mark set just before Enter — for
/// the line `alo-desktop` says when such a provider was saved
/// (`alo-desktop`'s `the_walk_waits_for_the_line_said_when_a_keyless_provider_is_saved`
/// holds the two together).
#[test]
fn a_provider_that_needs_no_key_is_added_before_the_gate() {
    let steps = the_first_walk().steps().to_vec();
    let Some(check) = steps.iter().position(|step| {
        matches!(step, Step::ExpectLine { phrase, since: Since::Mark, .. }
            if phrase == super::A_PROVIDER_THAT_NEEDS_NO_KEY_SAVED)
    }) else {
        panic!("the walk does not check that a keyless provider was saved");
    };
    let gate = steps
        .iter()
        .position(|step| *step == Step::OnlyWith(THE_PROVIDER_KEY))
        .unwrap_or(steps.len());
    assert!(
        check < gate,
        "the keyless provider is skipped without a key"
    );
    let before: Vec<&Step> = steps.iter().take(check).collect();
    let Some(mark) = before.iter().rposition(|step| matches!(step, Step::Mark)) else {
        panic!("no mark before the keyless provider check");
    };
    assert!(
        matches!(before.get(mark + 1), Some(Step::Press(chord)) if chord.the_key() == "ret"),
        "the step after the mark is not Enter"
    );
    let Some(form) = before.iter().rposition(
        |step| matches!(step, Step::Look { name, .. } if name == "the-keyless-provider-form"),
    ) else {
        panic!("no picture of the keyless form before it was saved");
    };
    assert!(
        before.get(..form).is_some_and(|it| it
            .iter()
            .rev()
            .take(4)
            .any(|step| matches!(step, Step::Press(chord) if chord.the_key() == "spc"))),
        "the switch was not pressed with Space just before the form was pictured"
    );
    assert!(
        !before
            .iter()
            .any(|step| matches!(step, Step::TypeSecret(_))),
        "a key was typed for a service that needs none"
    );
}

/// The keys a step presses, or what it types, as one word each.
fn said_as_keys(steps: &[Step]) -> Vec<String> {
    steps
        .iter()
        .filter_map(|step| match step {
            Step::Press(chord) => Some(chord.the_key().to_owned()),
            Step::Type(text) => Some(format!("type {text}")),
            Step::TypeSecret(_) => Some("type the secret".to_owned()),
            Step::PressUntilTheKeyboardIsOn { name, .. } => Some(format!("until {name}")),
            _ => None,
        })
        .collect()
}

/// The keys pressed from the step that typed `from`, for `count` keys.
fn the_keys_after(from: &str, count: usize) -> Vec<String> {
    let keys = said_as_keys(the_first_walk().steps());
    let Some(at) = keys.iter().position(|key| key == from) else {
        panic!("the walk never does {from:?}");
    };
    keys.into_iter().skip(at).take(count).collect()
}

/// **The keyless provider's form is gone through field by field, by name**:
/// past the empty key to the switch, Space, then the model. Space anywhere but
/// the switch would leave it off, and Enter would be refused on the machine —
/// so the switch is reached by its name, not by a count of Tabs.
#[test]
fn the_keyless_form_is_tabbed_through_field_by_field() {
    use alo_acting::said_in_settings as said;
    assert_eq!(
        the_keys_after("type http://127.0.0.1:11434", 5),
        vec![
            "type http://127.0.0.1:11434".to_owned(),
            format!("until {}", said::NEEDS_NO_KEY),
            "spc".to_owned(),
            format!("until {}", said::MODEL),
            "type llama3.2".to_owned(),
        ]
    );
    assert_eq!(
        the_keys_after("type llama3.2", 2),
        vec!["type llama3.2", "ret"]
    );
}

/// **And the provider with a key is found by its row's name**, wherever the
/// keyless one put it, and the switch is passed, left off, to its model.
#[test]
fn the_provider_with_a_key_is_one_row_further_down_and_past_the_switch() {
    use alo_acting::said_in_settings as said;
    let keys = said_as_keys(the_first_walk().steps());
    let Some(typed) = keys.iter().position(|key| key == "type openai") else {
        panic!("the walk never types openai");
    };
    let Some(before) = keys.get(..typed) else {
        panic!("no keys before openai");
    };
    assert_eq!(
        before.iter().rev().take(2).cloned().collect::<Vec<_>>(),
        vec!["ret".to_owned(), format!("until {}", said::ADD_A_PROVIDER)],
        "Add a provider is not found by its name before the form"
    );
    assert_eq!(
        the_keys_after("type the secret", 3),
        vec![
            "type the secret".to_owned(),
            format!("until {}", said::MODEL),
            "type gpt-4o-mini".to_owned(),
        ]
    );
}

/// **The provider is added by names, never by counting rows or Tabs**: Down
/// until the keyboard is on *Add a provider*, and Tab until it is on each
/// field. Main's walk at 9ced5c40 (run 44851000278) counted five Downs, landed
/// on *alo* (picture 54), and every question afterwards went nowhere.
#[test]
fn the_provider_is_added_by_names() {
    use alo_acting::said_in_settings as said;
    let steps = the_first_walk().steps().to_vec();
    let found: Vec<&str> = steps
        .iter()
        .filter_map(|step| match step {
            Step::PressUntilTheKeyboardIsOn { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect();
    for name in [said::ADD_A_PROVIDER, said::ADDRESS, said::KEY, said::MODEL] {
        assert!(
            found.contains(&name),
            "{name} is not found by name: {found:?}"
        );
    }
    let at = |name: &str| {
        steps
            .iter()
            .position(|it| matches!(it, Step::Look { name: n, .. } if n == name))
    };
    // Both providers the walk adds: the one that needs no key (#1410), and
    // the one with a key.
    for (opened, saved) in [
        (
            "settings-for-a-provider-that-needs-no-key",
            "the-keyless-provider-saved",
        ),
        ("settings-again", "the-provider-saved"),
    ] {
        let (Some(from), Some(to)) = (at(opened), at(saved)) else {
            unreachable!("the walk does not add the provider {saved}");
        };
        let Some(adding) = steps.get(from..to) else {
            unreachable!("{saved} comes before {opened}");
        };
        let pressed = |key: &'static str| {
            adding
                .iter()
                .filter(|it| **it == Step::Press(crate::keys::Chord::key(key)))
                .count()
        };
        // One Tab goes into the page from the side column; no other is counted.
        assert_eq!(pressed("down"), 0, "{saved}: still counts Downs");
        assert!(pressed("tab") <= 1, "{saved}: still counts Tabs");
    }
}
