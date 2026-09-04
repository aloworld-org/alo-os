//! **The measurement itself, put to a model that really exists.**
//!
//! Everything else in this crate is the measurement's method: the fixed ten,
//! the daemon's door, the six outcomes and the bar. None of it had ever been
//! put to a model, on any machine, so every entry in `data/catalogue.toml` said
//! `not-measured` and no machine offered anybody a local agent. This file is
//! what closes that, and what it produces is a grade somebody copies into the
//! catalogue.
//!
//! # It is ignored, and that is not the gate being weakened
//!
//! A measurement needs a model — gigabytes on a disk and a runtime serving it —
//! and `cargo test --workspace` runs on machines that have neither. So this test
//! is `#[ignore]`d and is run deliberately, by whoever is adding or correcting a
//! catalogue entry:
//!
//! ```text
//! ALO_DRIVING_MODEL=phi3:3.8b-mini-4k-instruct-q4_K_M \
//!   cargo test -p alo-driving --test against_a_model_on_this_machine \
//!   -- --ignored --nocapture
//! ```
//!
//! **Unlike the ignored tests in `alo-bounding`, this one is not harmless under
//! `--include-ignored`.** Those are the child halves of tests that run them by
//! name, and they pass on their own. This one asserts that a model was named
//! before it does anything, so asking for it on a machine with nothing to
//! measure fails and says so — which is the right way round. A measurement that
//! passed green because there was no model would be the failure `LOOP.md`
//! describes for Windows: the same colour as success, worth nothing.
//!
//! # What it asserts, and what it deliberately does not
//!
//! It asserts three things, and a grade is not one of them. **Every answer came
//! from this machine**, so the measurement causes no egress and is run under the
//! rule that would forbid any — `alo_models::SourcePolicy::ThisMachineOnly`, the
//! strongest one there is. **Every exercise was asked**, which is
//! [`Measured::of`]'s refusal and is what makes a run a measurement. And **the
//! run produced a measured grade**, which on any `Measured` that exists is one
//! of the three.
//!
//! What it does not assert is *which* grade. A test that expected `reliably`
//! would be an assertion about somebody else's model, failing on the day they
//! improved it; and one that expected `rarely` would be worse, because it would
//! pass for a machine whose runtime was answering nothing at all. The grade is
//! **output**, printed for whoever is writing the catalogue entry, and the
//! catalogue is where a claim about a model is made.
//!
//! # A runtime that fails is not a model that failed
//!
//! If the runtime cannot answer — it is not running, the model is not pulled,
//! the answer took longer than `alo-models` waits — this stops and says so
//! rather than scoring the exercise as one the model did not drive. Blaming a
//! model for a machine's failure is the one way this measurement could produce a
//! grade that is worse than no grade, since `alo_models::Driving` carries the
//! authority of having been checked.
//!
//! # Three variables, and only the first is required
//!
//! | | |
//! |---|---|
//! | `ALO_DRIVING_MODEL` | What the runtime calls the model. Not the catalogue's `id` — the two are different namespaces, and matching them is the job of whoever writes the entry down |
//! | `ALO_DRIVING_ENDPOINT` | Where the runtime is, defaulting to `alo_models::ollama::DEFAULT_ENDPOINT` |
//! | `ALO_DRIVING_ROUNDS` | How many times to put the whole set, defaulting to one. A model is not deterministic, and `measured.rs` says repeats are a bigger sample rather than a different method |

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_answering::Answering;
use alo_asking::{Answer, Asking, NotAnswered, Question};
use alo_capability::{Grantee, Verbs};
use alo_driving::{Attempt, Exercises, Measured};
use alo_models::{Catalogue, Driving, InferenceSource, Ollama, SourcePolicy};

/// What the runtime calls the model to be measured. Without it there is nothing
/// to measure and the test says so.
const WHICH_MODEL: &str = "ALO_DRIVING_MODEL";

/// Where the runtime is listening, if it is not where Ollama listens by default.
const WHERE_THE_RUNTIME_IS: &str = "ALO_DRIVING_ENDPOINT";

/// How many times to put the whole fixed set.
const HOW_MANY_ROUNDS: &str = "ALO_DRIVING_ROUNDS";

/// The verbs alo OS itself offers, as `alo-agentd` would put them on a registry
/// — `alo-files`' six and `alo-applications`' four.
///
/// The same fixture the unit tests use, for the reason `src/testing.rs` gives:
/// a measurement against verbs invented here would be a measurement of a
/// machine nobody ships.
fn the_verbs() -> Verbs {
    let mut verbs = Verbs::default();
    alo_files::declare_into(&mut verbs).unwrap();
    alo_applications::declare_into(&mut verbs).unwrap();
    verbs
}

/// One variable, trimmed, or `None` if it was not set or was set to nothing.
fn said(variable: &str) -> Option<String> {
    std::env::var(variable)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

/// One question put to the model, through the door that leaves this machine
/// alone.
///
/// The same three steps for the warm-up and for every exercise, because a
/// warm-up that reached the runtime by a shorter road would be warming
/// something the measurement does not use.
fn put(
    text: &str,
    model: &str,
    runtime: &Ollama,
    agent: &Grantee,
    policy: &SourcePolicy,
) -> Result<Answer, NotAnswered> {
    let question = Question::asked(text, model).unwrap();
    let answering = Answering::chosen(InferenceSource::ThisMachine, policy).unwrap();
    Asking::by(agent, answering, &[], policy).to_this_machine(&question, runtime)
}

/// What the model is asked before anything is scored.
///
/// **The first question to a runtime loads the weights**, and a load is a disk
/// read rather than a model thinking: measured on this machine it took 220
/// seconds against 2 for the answer itself. Scoring the exercise that happened
/// to pay for it would grade a disk, and on a slow one it would grade it as
/// `alo_models::RuntimeError::TookTooLong` — the model blamed for the machine,
/// which is the one thing this file refuses to do. So one throwaway question
/// goes first, its answer is dropped, and the exercises that count are put to a
/// model that is already loaded. That is also the state a real turn finds it in.
const TO_WARM_IT_UP: &str = "Answer with the single word: ready.";

/// How a catalogue entry writes the grade this crate produced.
///
/// The same four spellings `driving.rs` asserts, written out here rather than
/// serialised, because what this prints is a line somebody types into a TOML
/// file by hand.
fn as_the_catalogue_writes_it(grade: Driving) -> &'static str {
    match grade {
        Driving::Reliably => "reliably",
        Driving::Sometimes => "sometimes",
        Driving::Rarely => "rarely",
        Driving::NotMeasured => "not-measured",
    }
}

/// The start of what came back, on one line, for an answer that did not drive.
///
/// The six outcomes are kept apart because they are different problems with
/// different answers, and for the first of them — *the door would not read it* —
/// the outcome alone does not say which problem it is. *It wrote a paragraph*
/// and *it wrote exactly the right call and put a code fence round it* are the
/// same `NotAMessage`, and they are not the same finding about a model. So the
/// line itself is printed, bounded, because a model that ran on for a page would
/// otherwise bury the report it belongs to.
fn as_far_as_it_helps(said: &str) -> String {
    let one_line: String = said.split_whitespace().collect::<Vec<&str>>().join(" ");
    let shown: String = one_line.chars().take(160).collect();
    if shown.chars().count() < one_line.chars().count() {
        format!("{shown}…")
    } else {
        shown
    }
}

/// **Put the fixed ten to a model on this machine, and print the grade.**
///
/// The whole journey a real turn's question takes: the prompt is built from the
/// registry, the question goes through `alo-asking`'s door that does not leave
/// the machine, and what comes back is scored through `alo-protocol`'s reader
/// and `alo-capability`'s validation.
#[test]
#[ignore = "the measurement needs a model on this machine — run it with ALO_DRIVING_MODEL set"]
fn the_fixed_set_put_to_a_model_that_exists() {
    let model = said(WHICH_MODEL).unwrap_or_default();
    assert!(
        !model.is_empty(),
        "set {WHICH_MODEL} to what the runtime calls the model — there is nothing to measure \
         without one, and a measurement that passed green with no model would be worth nothing"
    );
    let endpoint = said(WHERE_THE_RUNTIME_IS)
        .unwrap_or_else(|| alo_models::ollama::DEFAULT_ENDPOINT.to_owned());
    let rounds: usize = said(HOW_MANY_ROUNDS).map_or(1, |value| value.parse().unwrap_or(0));
    assert!(
        rounds > 0,
        "{HOW_MANY_ROUNDS} must be a whole number of rounds, and at least one"
    );

    let verbs = the_verbs();
    let exercises = Exercises::over(&verbs).unwrap();
    let runtime = Ollama::at(&endpoint, Catalogue::built_in().unwrap());

    // The rule this run is under forbids everything that would leave, which is
    // what makes "the measurement caused no egress" a fact about the run rather
    // than an observation about it.
    let policy = SourcePolicy::ThisMachineOnly;
    let measuring = Grantee::named("@measuring");

    println!("loading {model} at {endpoint}");
    let warmed = put(TO_WARM_IT_UP, &model, &runtime, &measuring, &policy);
    assert!(
        warmed.is_ok(),
        "{model} at {endpoint} did not answer at all, so nothing was measured: {warmed:?}"
    );

    println!("putting the fixed set to {model}, {rounds} round(s)");
    let mut attempts: Vec<Attempt> = Vec::new();
    for round in 1..=rounds {
        for exercise in exercises.all() {
            let answered = put(
                &exercises.prompt(exercise),
                &model,
                &runtime,
                &measuring,
                &policy,
            );
            // A runtime that could not answer is this machine failing, and
            // scoring it would blame the model for it.
            assert!(
                answered.is_ok(),
                "{model} at {endpoint} did not answer the {} exercise: {answered:?}",
                exercise.named()
            );
            let answer = answered.unwrap();
            assert_eq!(
                answer.source(),
                &InferenceSource::ThisMachine,
                "a measurement whose answers came from anywhere else is not this measurement"
            );
            let attempt = exercises.attempt(exercise, answer.text());
            println!(
                "round {round}  {:<8}  {:?}",
                exercise.named(),
                attempt.outcome()
            );
            if !attempt.drove() {
                println!("                    {}", as_far_as_it_helps(answer.text()));
            }
            attempts.push(attempt);
        }
    }

    let measured = Measured::of(&exercises, attempts).unwrap();
    let grade = measured.grade();
    println!(
        "\n{model}: {} of {} drove the verbs — drives_verbs = \"{}\"",
        measured.drove(),
        measured.how_many(),
        as_the_catalogue_writes_it(grade)
    );
    assert!(
        grade.has_been_measured(),
        "a run that happened never grades as one that did not"
    );
}
