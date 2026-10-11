//! **Does asking alo a question get an answer?** Put to a real model.
//!
//! Every test in this crate asks the decision logic and stubs the runtime, so
//! until this existed nothing in the repository had put a question to a model
//! that answers. That is the gap the owner asked about, and it is the same
//! gap this crate's own header names one level up: *thirteen crates decide
//! correctly about a question and none of them sends one.*
//!
//! # What it proves and what it cannot
//!
//! It proves the local road end to end: a question, the policy that permits
//! this machine, the runtime that holds the model, an answer with text in it,
//! and the sentence saying where the answer came from. It proves **zero
//! egress**, which is law 1's measurable promise — the indicator is asked
//! afterwards and must have nothing in it.
//!
//! It cannot prove the provider road, because that needs a key and a network,
//! and it cannot prove the answer is any *good* — a 1.7B model is here because
//! it is small, not because it is the one a person would be given.
//!
//! # Run it
//!
//! ```text
//! ollama serve &
//! ollama pull smollm2:1.7b-instruct-q4_K_M
//! cargo run -p alo-asking --example does_ask_alo_answer
//! ```
//!
//! `ALO_ASK` sets the question and `ALO_MODEL` the catalogue id. The default
//! model is the smallest of the sixteen in `alo-models`' catalogue, so that
//! this asks a model the project actually ships rather than whatever was to
//! hand.
//!
//! Exits non-zero when the road does not work, so it can be a check rather
//! than only a demonstration.

use std::process::ExitCode;

use alo_answering::Answering;
use alo_asking::{Asking, Question};
use alo_capability::Grantee;
use alo_egress::Indicator;
use alo_models::{Catalogue, InferenceSource, SourcePolicy, ollama};

fn main() -> ExitCode {
    let asked = std::env::var("ALO_ASK")
        .unwrap_or_else(|_| "In one short sentence: what is an operating system?".to_owned());
    let of = std::env::var("ALO_MODEL").unwrap_or_else(|_| "smollm2-1.7b-instruct".to_owned());

    // **The runtime is found rather than assumed.** `found_on_this_machine`
    // answers None when nothing is listening, which is a different failure
    // from a model that is not installed and is worth telling apart: the first
    // means start the runtime, the second means pull the model.
    let catalogue = match Catalogue::built_in() {
        Ok(catalogue) => catalogue,
        Err(why) => {
            eprintln!("the built-in catalogue would not read: {why:?}");
            return ExitCode::FAILURE;
        }
    };
    let Some(runtime) = ollama::found_on_this_machine(catalogue) else {
        eprintln!("no model runtime is listening on this machine, so there is nothing to ask");
        eprintln!("  start one:  ollama serve &");
        return ExitCode::FAILURE;
    };
    println!("the runtime answered, so something is listening");

    // Where an answer may come from is decided before anything is asked, and
    // against the rule this machine is under — not as a formality, since
    // `Answering::chosen` is what refuses a source the policy forbids.
    let policy = SourcePolicy::Anywhere;
    let answering = match Answering::chosen(InferenceSource::ThisMachine, &policy) {
        Ok(answering) => answering,
        Err(refusal) => {
            eprintln!("this machine's own model was refused by the policy: {refusal:?}");
            return ExitCode::FAILURE;
        }
    };

    let question = match Question::asked(&asked, &of) {
        Ok(question) => question,
        Err(why) => {
            eprintln!("that is not a question this machine can put: {why:?}");
            return ExitCode::FAILURE;
        }
    };
    println!("asking {of}: {asked}");

    let indicator = Indicator::default();
    // A let binding rather than a temporary: `Asking::by` borrows the grantee
    // for the life of the asking, so an inline `&Grantee::named(..)` is freed
    // at the end of its own statement.
    let owner = Grantee::named("@the-owner");
    let asking = Asking::by(&owner, answering, &[], &policy);
    let answer = match asking.to_this_machine(&question, &runtime) {
        Ok(answer) => answer,
        Err(not_answered) => {
            eprintln!("it did not answer: {not_answered:?}");
            return ExitCode::FAILURE;
        }
    };

    let text = answer.text();
    println!("\n--- the answer ---\n{text}\n------------------");
    println!("it came from: {:?}, model {}", answer.source(), answer.model());

    // **Law 1, measured rather than promised.** A question answered on this
    // machine causes no egress, so the indicator must hold nothing. This is
    // the assertion that makes the whole run worth more than a transcript.
    let quiet = indicator.is_quiet();
    println!("the egress indicator is quiet: {quiet}  (showing {} things)", indicator.showing().len());
    if !quiet {
        eprintln!("REFUSED: a question answered on this machine caused egress");
        return ExitCode::FAILURE;
    }

    if text.trim().is_empty() {
        eprintln!("REFUSED: it answered with nothing, which is not an answer");
        return ExitCode::FAILURE;
    }
    println!("\nask alo works on this machine, with no egress.");
    ExitCode::SUCCESS
}
