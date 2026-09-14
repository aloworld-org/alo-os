//! The measurement itself, shared by the two files that put it to something
//! real: a model the runtime already has by name
//! (`against_a_model_on_this_machine.rs`) and a file a person brought
//! (`against_a_file_brought_to_this_machine.rs`).
//!
//! One loop for both, because *the same ten, the same bar, the same door* is
//! the whole of task 4's promise: a brought file measured by a second copy of
//! this code would be measured by whatever that copy had drifted into.

use alo_answering::Answering;
use alo_asking::{Answer, Asking, NotAnswered, Question};
use std::net::SocketAddr;

use alo_asking::Served;
use alo_capability::{Grantee, Verbs};
use alo_driving::{Attempt, Exercises, Instructions, Measured};
use alo_models::{Driving, InferenceSource, Ollama, Provider, Region, SourcePolicy};

/// **How the model is asked**, which is part of what a grade is a grade of
/// ([ADR 0032](../../../../docs/decisions/0032-a-local-model-is-held-to-the-envelope-not-the-call.md)).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asked<'a> {
    /// As the catalogue's `drives_verbs` was earned: the question and nothing
    /// else, through `alo-asking`'s door that does not leave the machine.
    Freely,
    /// As `drives_verbs_in_the_envelope` is earned: the same question, with the
    /// runtime holding the answer to the protocol's envelope — through
    /// `alo-asking`'s `Asking::to_this_machine_in_the_envelope`, the door an
    /// agent turn takes, so the measurement and the product ask in one way.
    InTheEnvelope,
    /// As [ADR 0035](../../../../docs/decisions/0035-the-wrapper-or-the-engine.md)
    /// puts on trial: every token of the answer held to this grammar, by a
    /// service on this machine that takes one — `llama.cpp`'s server — through
    /// `alo-asking`'s `to_a_service_on_this_machine_held_to`. The envelope, the
    /// door, the verb's spelling and its argument names are held by
    /// construction; which verb, and the values, are the model's.
    HeldToTheWholeCall(&'a str),
}

impl Asked<'_> {
    /// How a report and a catalogue comment name it.
    pub fn named(self) -> &'static str {
        match self {
            Self::Freely => "freely",
            Self::InTheEnvelope => "in the envelope",
            Self::HeldToTheWholeCall(_) => "held to the whole call",
        }
    }
}

/// The verbs alo OS itself offers, as `alo-agentd` would put them on a registry
/// — `alo-files`' six and `alo-applications`' four.
///
/// The same fixture the unit tests use, for the reason `src/testing.rs` gives:
/// a measurement against verbs invented here would be a measurement of a
/// machine nobody ships.
pub fn the_verbs() -> Verbs {
    let mut verbs = Verbs::default();
    alo_files::declare_into(&mut verbs).expect("alo-files declares its verbs");
    alo_applications::declare_into(&mut verbs).expect("alo-applications declares its verbs");
    verbs
}

/// One variable, trimmed, or `None` if it was not set or was set to nothing.
pub fn said(variable: &str) -> Option<String> {
    std::env::var(variable)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

/// What the model is asked before anything is scored.
///
/// **The first question to a runtime loads the weights**, and a load is a disk
/// read rather than a model thinking: measured on the development PC it took
/// 220 seconds against 2 for the answer itself. Scoring the exercise that
/// happened to pay for it would grade a disk, and on a slow one it would grade
/// it as `alo_models::RuntimeError::TookTooLong` — the model blamed for the
/// machine, which is the one thing this measurement refuses to do. So one
/// throwaway question goes first, its answer is dropped, and the exercises that
/// count are put to a model that is already loaded. That is also the state a
/// real turn finds it in.
const TO_WARM_IT_UP: &str = "Answer with the single word: ready.";

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
    let question = Question::asked(text, model).expect("a question the harness wrote");
    let answering = Answering::chosen(InferenceSource::ThisMachine, policy)
        .expect("no policy forbids this machine answering");
    Asking::by(agent, answering, &[], policy).to_this_machine(&question, runtime)
}

/// One question, asked the way `asked` says, answered with the model's text.
fn put_as(
    asked: Asked<'_>,
    text: &str,
    model: &str,
    runtime: &Ollama,
    agent: &Grantee,
    policy: &SourcePolicy,
) -> Result<String, String> {
    match asked {
        Asked::Freely => put(text, model, runtime, agent, policy)
            .map(|answer| {
                assert_eq!(
                    answer.source(),
                    &InferenceSource::ThisMachine,
                    "a measurement whose answers came from anywhere else is not this measurement"
                );
                answer.text().to_owned()
            })
            .map_err(|why| format!("{why:?}")),
        Asked::InTheEnvelope => {
            let question = Question::asked(text, model).expect("a question the harness wrote");
            let answering = Answering::chosen(InferenceSource::ThisMachine, policy)
                .expect("no policy forbids this machine answering");
            Asking::by(agent, answering, &[], policy)
                .to_this_machine_in_the_envelope(&question, runtime)
                .map(|answer| {
                    assert_eq!(
                        answer.source(),
                        &InferenceSource::ThisMachine,
                        "a measurement whose answers came from anywhere else is not this measurement"
                    );
                    answer.text().to_owned()
                })
                .map_err(|why| format!("{why:?}"))
        }
        Asked::HeldToTheWholeCall(grammar) => {
            let question = Question::asked(text, model).expect("a question the harness wrote");
            // The engine's server is a service on this machine's address, which
            // is what `Served` refuses every other address to be. It is asked
            // through the door for one, with the answer held to the grammar.
            let service =
                Provider::checked("the engine's server", ENDPOINT_HERE, Region::Unknown, None)
                    .expect("a provider on loopback");
            let served = Served::at(&service, None).expect("a service on this machine");
            let answering =
                Answering::chosen(InferenceSource::AServiceAtThisMachinesAddress, policy)
                    .expect("no policy forbids a service on this machine answering");
            let to: Vec<SocketAddr> =
                vec![THE_ENGINES_PORT.parse().expect("an address on loopback")];
            Asking::by(agent, answering, &[], policy)
                .to_a_service_on_this_machine_held_to(&question, &served, &to, grammar)
                .map(|answer| {
                    assert_eq!(
                        answer.source(),
                        &InferenceSource::AServiceAtThisMachinesAddress,
                        "a measurement whose answers came from anywhere else is not this measurement"
                    );
                    answer.text().to_owned()
                })
                .map_err(|why| format!("{why:?}"))
        }
    }
}

/// Where the engine's server listens for the trial (task 17), as an address and
/// as the endpoint a provider is checked against. Loopback, and never a name:
/// this crate's measurement resolves nothing.
const THE_ENGINES_PORT: &str = "127.0.0.1:8081";

/// The same, as a provider's endpoint.
const ENDPOINT_HERE: &str = "http://127.0.0.1:8081";

/// How a catalogue entry, or a person's settings, writes the grade.
///
/// The same four spellings `driving.rs` asserts, written out here rather than
/// serialised, because what this prints is a line somebody may read and type.
pub fn as_it_is_written(grade: Driving) -> &'static str {
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

/// **Put the fixed ten to `model` at `endpoint`, `rounds` times, and hand back
/// the measurement.**
///
/// The whole journey a real turn's question takes: the prompt is built from the
/// registry, the question goes through `alo-asking`'s door that does not leave
/// the machine, and what comes back is scored through `alo-protocol`'s reader
/// and `alo-capability`'s validation. Every answer is printed whole after the
/// run, so a grade can be re-derived by a reader who disagrees with it.
///
/// Every prompt opens with `instructions` (ADR 0034), and the run prints their
/// digest, which is what a grade earned here records beside itself.
///
/// # Panics
/// When the runtime cannot answer — the machine failing, which is never scored
/// as the model failing — and when an answer came from anywhere but this
/// machine.
pub fn the_fixed_set_put_to(
    model: &str,
    endpoint: &str,
    rounds: usize,
    asked: Asked<'_>,
    instructions: Instructions,
) -> Measured {
    let verbs = the_verbs();
    let exercises = Exercises::over(&verbs).expect("the fixed set is built over alo OS's verbs");
    let runtime = Ollama::at(
        endpoint,
        alo_models::Catalogue::built_in().expect("the built-in catalogue loads"),
    );

    // The rule this run is under forbids everything that would leave, which is
    // what makes "the measurement caused no egress" a fact about the run rather
    // than an observation about it.
    let policy = SourcePolicy::ThisMachineOnly;
    let measuring = Grantee::named("@measuring");

    println!("loading {model} at {endpoint}, asked {}", asked.named());
    println!(
        "under the instructions {}, sha256 {}",
        instructions.named(),
        instructions.digest()
    );
    let warmed = put_as(asked, TO_WARM_IT_UP, model, &runtime, &measuring, &policy);
    assert!(
        warmed.is_ok(),
        "{model} at {endpoint} did not answer at all, so nothing was measured: {warmed:?}"
    );

    println!("putting the fixed set to {model}, {rounds} round(s)");
    let mut attempts: Vec<Attempt> = Vec::new();
    // Every answer whole, in the order it was given. A grade a reader cannot
    // re-derive is a grade they have to take on trust, and the bounded line
    // below is for reading the run, not for checking it.
    let mut verbatim: Vec<String> = Vec::new();
    for round in 1..=rounds {
        for exercise in exercises.all() {
            let answered = put_as(
                asked,
                &exercises.prompt_under(instructions, exercise),
                model,
                &runtime,
                &measuring,
                &policy,
            );
            // A runtime that could not answer is this machine failing, and
            // scoring it would blame the model for it.
            let said = answered.unwrap_or_else(|why| {
                panic!(
                    "{model} at {endpoint} did not answer the {} exercise: {why}",
                    exercise.named()
                )
            });
            let attempt = exercises.attempt(exercise, &said);
            println!(
                "round {round}  {:<8}  {:?}",
                exercise.named(),
                attempt.outcome()
            );
            if !attempt.drove() {
                println!("                    {}", as_far_as_it_helps(&said));
            }
            verbatim.push(format!(
                "round {round}, {}: {:?}\n{said}",
                exercise.named(),
                attempt.outcome(),
            ));
            attempts.push(attempt);
        }
    }

    println!("\nevery answer, verbatim:");
    for said in &verbatim {
        println!("\n----- {said}\n----- end");
    }

    let measured = Measured::of(&exercises, attempts)
        .expect("every exercise was asked, which is what makes a run a measurement");
    println!(
        "\n{model}, asked {} under {}: {} of {} drove the verbs — \"{}\"",
        asked.named(),
        instructions.named(),
        measured.drove(),
        measured.how_many(),
        as_it_is_written(measured.grade())
    );
    assert!(
        measured.grade().has_been_measured(),
        "a run that happened never grades as one that did not"
    );
    measured
}
