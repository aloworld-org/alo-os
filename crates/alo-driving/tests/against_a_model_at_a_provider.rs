//! **The measurement put to a model at a provider, over the network.**
//!
//! Every grade in `data/catalogue.toml` was earned by a model on the machine.
//! Nothing had ever put the fixed ten to a hosted model, so there was no number
//! to compare the local ones against and no answer to *how far is a local model
//! from what a provider does*. This file produces that number, on the same ten
//! exercises, scored by the same code, against the same bar — which is the only
//! way the two are comparable at all.
//!
//! # It is ignored, and it causes egress
//!
//! Two reasons it is `#[ignore]`d, and the second is the important one:
//!
//! 1. It needs a provider, a model name and a key, which a machine running
//!    `cargo test --workspace` does not have.
//! 2. **It leaves the machine.** Every exercise is carried to the endpoint
//!    named below. A suite that quietly called out to a paid service would be
//!    the exact thing law 1 exists to prevent, so this one is never reached by
//!    a plain test run and must be asked for by name.
//!
//! What makes it safe to have in the tree at all is that it cannot cheat: the
//! only road to a provider is `Asking::to_a_provider`, which will not move
//! without an `alo_egress::Departing`. The run is on the indicator while it
//! happens and in the record afterwards, exactly as a person's turn would be.
//!
//! # Running it
//!
//! The key is read from the environment and **never from a file in this
//! repository**. Put it in the shell that runs the test and nowhere else:
//!
//! ```text
//! export ALO_PROVIDER_KEY=...              # never committed, never pasted
//! ALO_PROVIDER_NAME=anthropic \
//! ALO_PROVIDER_ENDPOINT=https://api.anthropic.com \
//! ALO_PROVIDER_REGION=us \
//! ALO_PROVIDER_MODEL=claude-haiku-5-5 \
//!   cargo test -p alo-driving --test against_a_model_at_a_provider \
//!   -- --ignored --nocapture
//! ```
//!
//! `ALO_PROVIDER_REGION` is what *you state* about where it runs, not something
//! this test can verify — `alo_models::Region` is a claim with a source, and
//! here the source is you.
#![cfg(not(target_os = "windows"))]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

#[allow(
    dead_code,
    reason = "the local roads are measured by the files beside this one"
)]
mod measuring;

// Declared here and not from `measuring/mod.rs` because it carries its own
// tests: shared, they would run once in each of the five binaries that include
// that module, and five identical passes read as five checks.
#[path = "measuring/a_provider_to_measure.rs"]
mod a_provider_to_measure;

use alo_models::Secret;

/// Which instruction set the model is given. **Defaults to what alo OS ships**,
/// because a grade earned under instructions production does not use is a grade
/// of something nobody runs.
const WHICH_INSTRUCTIONS: &str = "ALO_PROVIDER_INSTRUCTIONS";

/// What the provider is called, which is what the record will say answered.
const WHICH_PROVIDER: &str = "ALO_PROVIDER_NAME";
/// The base address of its API.
const WHERE_IT_IS: &str = "ALO_PROVIDER_ENDPOINT";
/// Where it runs, as whoever runs this test states it.
const WHICH_REGION: &str = "ALO_PROVIDER_REGION";
/// The model, as the provider names it.
const WHICH_MODEL: &str = "ALO_PROVIDER_MODEL";
/// The key. **Read here and nowhere else**, and never printed.
const THE_KEY: &str = "ALO_PROVIDER_KEY";
/// How many times to put the whole set. One is a measurement; more is a
/// measurement of how steady it is.
const HOW_MANY_ROUNDS: &str = "ALO_PROVIDER_ROUNDS";

/// Put the fixed ten to a model at a provider, and say what it scored.
///
/// # Panics
/// When a name, an address, a region, a model or a key is missing — a run with
/// any of them absent would measure nothing and must not read as a pass.
#[test]
#[ignore = "needs a provider, a model and a key, and it leaves the machine"]
fn the_fixed_set_is_put_to_a_model_at_a_provider() {
    let ready = a_provider_to_measure::from_what_was_said(
        measuring::said(WHICH_PROVIDER).as_deref(),
        measuring::said(WHERE_IT_IS).as_deref(),
        measuring::said(WHICH_REGION).as_deref(),
        measuring::said(WHICH_MODEL).as_deref(),
        measuring::said(HOW_MANY_ROUNDS).as_deref(),
    )
    .unwrap_or_else(|nothing| panic!("nothing was measured: {}", nothing.what_to_do()));

    // **Not `said`**, because `said` is for things that may be printed and this
    // may not be. It is read straight into a `Secret`, which has no `Display`
    // and whose `Debug` does not show it.
    let key = std::env::var(THE_KEY)
        .ok()
        .filter(|key| !key.trim().is_empty())
        .unwrap_or_else(|| {
            panic!(
                "set {THE_KEY} in the shell that runs this test. It is never read from a file in \
                 this repository and never appears in its output"
            )
        });

    let instructions = match measuring::said(WHICH_INSTRUCTIONS).as_deref() {
        None | Some("one-example-per-door") => alo_driving::Instructions::OneExamplePerDoor,
        Some("as-first-written") => alo_driving::Instructions::AsFirstWritten,
        Some("several-steps") => alo_driving::Instructions::SeveralSteps,
        Some("answers-in-words") => alo_driving::Instructions::AnswersInWords,
        Some(other) => panic!(
            "{WHICH_INSTRUCTIONS} is `one-example-per-door`, `several-steps`, \
             `answers-in-words`, `as-first-written` or unset, not `{other}`"
        ),
    };
    let key = Secret::typed(&key).expect("the key read from the environment is a sendable one");

    // **Held to the protocol's envelope, or not.** Unset means held, because
    // that is what an agent turn really asks for and what the local runtime has
    // always done; `ALO_PROVIDER_UNHELD=1` is how the two are compared.
    let in_the_envelope = std::env::var("ALO_PROVIDER_UNHELD").is_err();
    println!(
        "held to the protocol's envelope: {}",
        if in_the_envelope { "yes" } else { "no" }
    );
    let measured = measuring::to_a_provider::the_fixed_set_put_to_a_provider(
        &ready.model,
        &ready.provider,
        &key,
        ready.rounds,
        instructions,
        in_the_envelope,
    );

    // What this test asserts is that a measurement *happened*, never that it
    // reached a grade. A provider that drives nothing is a result, and a test
    // that failed on it would be asserting an outcome rather than measuring
    // one.
    assert!(
        measured.grade().has_been_measured(),
        "a run that happened never grades as one that did not"
    );
    println!(
        "\ncopy into the catalogue beside the local grades: {} at {} — {}",
        ready.model,
        ready.provider.name,
        measuring::as_it_is_written(measured.grade())
    );
}
