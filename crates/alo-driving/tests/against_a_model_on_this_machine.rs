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
//! # The variables, and only the first is required
//!
//! | | |
//! |---|---|
//! | `ALO_DRIVING_MODEL` | What the runtime calls the model. Not the catalogue's `id` — the two are different namespaces, and matching them is the job of whoever writes the entry down |
//! | `ALO_DRIVING_ENDPOINT` | Where the runtime is, defaulting to `alo_models::ollama::DEFAULT_ENDPOINT` |
//! | `ALO_DRIVING_ROUNDS` | How many times to put the whole set, defaulting to one. A model is not deterministic, and `measured.rs` says repeats are a bigger sample rather than a different method |
//! | `ALO_DRIVING_ASKED` | `in-the-envelope` to hold the answer to the protocol's envelope (ADR 0032), which earns `drives_verbs_in_the_envelope`; unset to ask freely, which earns `drives_verbs` |
//! | `ALO_DRIVING_INSTRUCTIONS` | `one-example-per-door` to open every prompt with `alo_driving::ONE_EXAMPLE_PER_DOOR` (ADR 0034); unset for the instructions every grade before it was earned under. The run prints their digest, which the grade records |
//!
//! The loop itself is `tests/measuring/mod.rs`, shared with
//! `against_a_file_brought_to_this_machine.rs`, so a catalogue entry and a file
//! somebody brought are measured by one piece of code.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod measuring;

/// What the runtime calls the model to be measured. Without it there is nothing
/// to measure and the test says so.
const WHICH_MODEL: &str = "ALO_DRIVING_MODEL";

/// Where the runtime is listening, if it is not where Ollama listens by default.
const WHERE_THE_RUNTIME_IS: &str = "ALO_DRIVING_ENDPOINT";

/// How many times to put the whole fixed set.
const HOW_MANY_ROUNDS: &str = "ALO_DRIVING_ROUNDS";

/// How to ask: unset for freely, `in-the-envelope` for ADR 0032's way.
const HOW_IT_IS_ASKED: &str = "ALO_DRIVING_ASKED";

/// Which instructions open every prompt.
const WHICH_INSTRUCTIONS: &str = "ALO_DRIVING_INSTRUCTIONS";

/// **Put the fixed ten to a model on this machine, and print the grade.**
#[test]
#[ignore = "the measurement needs a model on this machine — run it with ALO_DRIVING_MODEL set"]
fn the_fixed_set_put_to_a_model_that_exists() {
    let model = measuring::said(WHICH_MODEL).unwrap_or_default();
    assert!(
        !model.is_empty(),
        "set {WHICH_MODEL} to what the runtime calls the model — there is nothing to measure \
         without one, and a measurement that passed green with no model would be worth nothing"
    );
    let endpoint = measuring::said(WHERE_THE_RUNTIME_IS)
        .unwrap_or_else(|| alo_models::ollama::DEFAULT_ENDPOINT.to_owned());
    let rounds: usize =
        measuring::said(HOW_MANY_ROUNDS).map_or(1, |value| value.parse().unwrap_or(0));
    assert!(
        rounds > 0,
        "{HOW_MANY_ROUNDS} must be a whole number of rounds, and at least one"
    );
    let asked = match measuring::said(HOW_IT_IS_ASKED).as_deref() {
        None => measuring::Asked::Freely,
        Some("in-the-envelope") => measuring::Asked::InTheEnvelope,
        Some(other) => panic!("{HOW_IT_IS_ASKED} is `in-the-envelope` or unset, not `{other}`"),
    };
    let instructions = match measuring::said(WHICH_INSTRUCTIONS).as_deref() {
        None => alo_driving::Instructions::AsFirstWritten,
        Some("one-example-per-door") => alo_driving::Instructions::OneExamplePerDoor,
        Some(other) => {
            panic!("{WHICH_INSTRUCTIONS} is `one-example-per-door` or unset, not `{other}`")
        }
    };
    measuring::the_fixed_set_put_to(&model, &endpoint, rounds, asked, instructions);
}
