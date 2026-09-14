//! A model that was measured and does not drive the verbs, offered for what it
//! is.
//!
//! `docs/features.md` says a model that cannot drive the verbs is offered for
//! what it is and never the agent. Until 2026-09-13 every entry that proved it
//! was one nobody had measured, so the sentence a person read was *nobody has
//! measured this* — true, and not the sentence the promise is about. Now the
//! catalogue carries measured `rarely` grades on 7B entries, and this file holds
//! the three things that follow to the vocabulary the whole machine loads:
//!
//! - **choosing such a model still stands.** This crate holds what somebody
//!   chose and never gates it on a grade; the catalogue recommends.
//! - **the agent is refused with the sentence for a measurement that was made**
//!   — `alo-models`' `NONE_CLEARS_THE_BAR`, which this crate shows rather than
//!   repeats — and not the one for a measurement that was not.
//! - **an entry with no grade says why**, in words `alo-saying` collects.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_choosing::{Chosen, Which};
use alo_models::{
    AskedTheWay, Catalogue, Driving, InferenceSource, NoAgentHere, WhyUnmeasured, words,
};
use alo_strings::Strings;

/// The vocabulary a machine actually holds.
fn what_this_machine_can_say() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// An entry the catalogue ships, measured on a machine and graded `rarely`.
const MEASURED_RARELY: &str = "qwen2.5-7b-instruct";

/// A machine with no graphics card and the memory the refusal tests ask about.
const SIXTEEN_GIGABYTES: f32 = 16.0;

/// **Choosing a model that rarely drives the verbs is still a choice.** It
/// answers questions on this machine; the grade takes away the agent, not the
/// model.
#[test]
fn a_model_measured_rarely_can_still_be_chosen_to_answer_questions() {
    let shipped = Catalogue::built_in().unwrap();
    let entry = shipped.get(MEASURED_RARELY).unwrap();
    assert_eq!(entry.drives_verbs, Driving::Rarely);
    assert!(
        entry.measured.is_some(),
        "the grade is shown, so the machine it was earned on must be too"
    );
    assert!(!entry.can_be_the_agent());

    let chosen = Chosen::of(Which::Catalogue, MEASURED_RARELY).unwrap();
    assert_eq!(chosen.model(), MEASURED_RARELY);
    assert_eq!(chosen.source(), InferenceSource::ThisMachine);
}

/// **The agent is refused with the sentence for a measurement that was made.**
/// Nine entries run here and may be used; all nine were measured, and a
/// machine saying *nobody has measured* would be claiming the opposite of what
/// happened.
#[test]
fn the_refusal_is_the_one_for_a_measurement_that_was_made() {
    let shipped = Catalogue::built_in().unwrap();
    let refused = shipped.agent_for_cpu(SIXTEEN_GIGABYTES).unwrap_err();
    assert!(
        matches!(refused, NoAgentHere::NoneClearsTheBar { measured, .. } if measured > 0),
        "{refused:?}"
    );
    assert_eq!(refused.word(), words::NONE_CLEARS_THE_BAR);

    let strings = what_this_machine_can_say();
    let [why, brought, elsewhere] = refused.lines(&strings);
    assert!(
        !why.text().contains("models.agent"),
        "reached a person as a key: {why}"
    );
    assert_eq!(
        why.text(),
        strings
            .say(
                &words::NONE_CLEARS_THE_BAR.key(),
                &alo_strings::Filling::nothing()
            )
            .text()
    );
    assert_ne!(
        why.text(),
        strings
            .say(
                &words::NONE_MEASURED.key(),
                &alo_strings::Filling::nothing()
            )
            .text(),
        "a measured catalogue was refused with the sentence for an unmeasured one"
    );
    assert!(
        brought.text().contains("weights you already have"),
        "{brought}"
    );
    assert!(
        elsewhere.text().contains("will not choose for you"),
        "{elsewhere}"
    );
}

/// **Every entry with no grade says why, in the machine's vocabulary**, and
/// none of those sentences reaches a person as a key.
#[test]
fn an_entry_with_no_grade_says_why_in_words_the_machine_holds() {
    let strings = what_this_machine_can_say();
    let shipped = Catalogue::built_in().unwrap();
    let unmeasured: Vec<_> = shipped
        .models
        .iter()
        .filter(|entry| !entry.drives_verbs.has_been_measured())
        .collect();
    assert!(!unmeasured.is_empty());
    for entry in unmeasured {
        let reason = entry.unmeasured.as_ref().unwrap();
        let said = reason.said(&strings);
        assert!(
            said.text().starts_with("not measured yet"),
            "{}: {said}",
            entry.id
        );
        assert!(
            !said.text().contains("models.unmeasured"),
            "{}: {said}",
            entry.id
        );
        assert_eq!(
            reason.because,
            WhyUnmeasured::TooLargeForTheMeasuringMachine,
            "{} — every reason the catalogue ships today is this one; a new one is a new \
             finding and belongs in the report that made it",
            entry.id
        );
    }
}

/// **The offer names which way the grade that decides was earned** (task 15).
///
/// An agent turn asks a local model in the envelope, so an entry measured that
/// way is offered by that grade, and says so; an entry measured only freely is
/// offered by its free grade, and says that nobody measured it the way a turn
/// asks. Both grades stay on the entry.
#[test]
fn an_offer_says_which_way_the_grade_that_decides_was_earned() {
    let strings = what_this_machine_can_say();
    let shipped = Catalogue::built_in().unwrap();

    let entry = shipped.get(MEASURED_RARELY).unwrap();
    assert_eq!(
        entry.drives_verbs,
        Driving::Rarely,
        "the free grade is kept"
    );
    let (grade, asked) = entry.grade_for_the_turn();
    assert_eq!(
        (grade, asked),
        (Driving::Sometimes, AskedTheWay::InTheEnvelope)
    );
    assert!(!entry.can_be_the_agent());
    let said = asked.said(&strings);
    assert!(said.text().contains("the way an agent turn asks"), "{said}");
    assert!(!said.text().contains("models.graded"), "{said}");

    // Every measured entry the catalogue ships today was also measured in the
    // envelope, so none is offered by its free grade; the sentence for one that
    // is must still reach a person as words.
    assert!(
        shipped
            .models
            .iter()
            .filter(|model| model.drives_verbs.has_been_measured())
            .all(|model| model.grade_for_the_turn().1 == AskedTheWay::InTheEnvelope)
    );
    let freely = AskedTheWay::Freely;
    let said = freely.said(&strings);
    assert!(said.text().contains("nobody has measured it"), "{said}");
    assert!(!said.text().contains("models.graded"), "{said}");
}
