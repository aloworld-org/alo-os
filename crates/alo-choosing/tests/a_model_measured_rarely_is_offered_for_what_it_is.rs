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
//!
//! **And since task 19 a machine with room is no longer refused at all.** Two
//! entries clear the bar in the words a turn shows, so the refusal below is
//! asked of a machine that cannot hold either — which is what the sentence was
//! always about, and is now a statement about memory rather than about the state
//! of the catalogue.

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

/// An entry the catalogue ships, measured on a machine and graded `rarely`
/// every way it was asked — including the way a turn asks (task 20: 3 of 20).
const MEASURED_RARELY: &str = "llama-3.2-3b-instruct";

/// The entry a machine with room is given, measured 20 of 20 the way a turn
/// asks.
const CLEARS_THE_BAR: &str = "qwen3-8b";

/// A machine with no graphics card, with room for the entries that clear the
/// bar — both state `min_ram_gb = 10`.
const SIXTEEN_GIGABYTES: f32 = 16.0;

/// A machine that can hold none of them, where every entry that fits was
/// measured and none clears the bar.
const EIGHT_GIGABYTES: f32 = 8.0;

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
    assert_eq!(
        entry.grade_for_the_turn(),
        (Driving::Rarely, AskedTheWay::AsATurnAsks),
        "it was measured the way a turn asks, and that is the grade that decides"
    );
    assert!(!entry.can_be_the_agent());

    let chosen = Chosen::of(Which::Catalogue, MEASURED_RARELY).unwrap();
    assert_eq!(chosen.model(), MEASURED_RARELY);
    assert_eq!(chosen.source(), InferenceSource::ThisMachine);
}

/// **A machine with room is given the agent** (task 19). Two entries clear the
/// bar in the words a turn shows; the ordering — comfortable before workable,
/// then larger — picks between them, and the grade read is the one the turn's
/// own words earned.
#[test]
fn a_machine_with_room_is_given_the_model_that_clears_the_bar() {
    let shipped = Catalogue::built_in().unwrap();
    let given = shipped.agent_for_cpu(SIXTEEN_GIGABYTES).unwrap();
    assert_eq!(given.id, CLEARS_THE_BAR);
    assert_eq!(
        given.grade_for_the_turn(),
        (Driving::Reliably, AskedTheWay::AsATurnAsks)
    );
    assert!(given.can_be_the_agent());
    assert_eq!(
        given.drives_verbs_in_the_envelope,
        Some(Driving::Sometimes),
        "the grade under the first instructions is kept and is not what decided"
    );
}

/// **The agent is refused with the sentence for a measurement that was made.**
/// Every entry an 8 GB machine can hold was measured, and a machine saying
/// *nobody has measured* would be claiming the opposite of what happened.
#[test]
fn the_refusal_is_the_one_for_a_measurement_that_was_made() {
    let shipped = Catalogue::built_in().unwrap();
    let refused = shipped.agent_for_cpu(EIGHT_GIGABYTES).unwrap_err();
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

/// **The offer names which way the grade that decides was earned** (tasks 15
/// and 19).
///
/// An agent turn asks a local model in the envelope and shows it the words this
/// machine wrote, so an entry measured **that** way is offered by that grade and
/// says so; an entry measured only another way has no grade for the turn, and
/// the sentence says that rather than offering a grade about a different
/// question. Every grade stays on the entry either way.
#[test]
fn an_offer_says_which_way_the_grade_that_decides_was_earned() {
    let strings = what_this_machine_can_say();
    let shipped = Catalogue::built_in().unwrap();

    let entry = shipped.get(CLEARS_THE_BAR).unwrap();
    assert_eq!(
        entry.drives_verbs,
        Driving::Sometimes,
        "the free grade is kept"
    );
    assert_eq!(
        entry.drives_verbs_in_the_envelope,
        Some(Driving::Sometimes),
        "and so is the grade under the first instructions"
    );
    let (grade, asked) = entry.grade_for_the_turn();
    assert_eq!(
        (grade, asked),
        (Driving::Reliably, AskedTheWay::AsATurnAsks),
        "task 20 measured this entry the way a turn asks, and that is the grade read"
    );
    assert!(entry.can_be_the_agent());
    let said = asked.said(&strings);
    assert!(said.text().contains("the way an agent turn asks"), "{said}");
    assert!(!said.text().contains("models.graded"), "{said}");

    // Every entry this machine could hold was measured the way a turn asks
    // (task 20); the three it could not hold have no grade at all, and the
    // sentence for an entry with no grade for the turn must still reach a
    // person as words.
    let (asked_that_way, another_way): (Vec<_>, Vec<_>) = shipped
        .models
        .iter()
        .partition(|model| model.grade_for_the_turn().1 == AskedTheWay::AsATurnAsks);
    assert!(asked_that_way.len() >= 13, "{}", asked_that_way.len());
    assert!(
        another_way
            .iter()
            .all(|model| model.unmeasured.is_some() || model.drives_verbs.has_been_measured())
    );
    let said = AskedTheWay::NotTheWayATurnAsks.said(&strings);
    assert!(
        said.text()
            .contains("not measured the way an agent turn asks"),
        "{said}"
    );
    assert!(!said.text().contains("models.graded"), "{said}");
}
