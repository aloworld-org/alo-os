//! **One question on each road, and the two answers held to each other.**
//!
//! Task 22 of `docs/autonomy/v0-5-the-models-measured-plan.md`, and the
//! instrument task 23 runs.
//! [ADR 0007](../../../docs/decisions/0007-the-cpu-is-the-default.md) says *a
//! GPU changes speed, not capability*, and until something puts one question to
//! both roads of one machine and compares what comes back, that is a sentence
//! rather than a measurement.
//!
//! # What this file can show here, and what it cannot
//!
//! The machine this was written on — the third PC, `AGAI01`, x86_64, no VGA or
//! 3D device on the bus, no `/dev/dri`, no `nvidia-smi` — **has no card**, so
//! the card road cannot be walked on it at all. That is not a gap in the work;
//! it is the work. What runs here is
//! [`the_card_road_cannot_be_walked_on_a_machine_with_no_card`], which reads
//! this machine, states the road it takes, and shows that the comparison is
//! **refused** rather than passing green on two processor runs — the failure
//! `docs/autonomy/LOOP.md` describes, where a suite is the same colour as
//! success and worth nothing.
//!
//! [`one_question_on_each_road_of_a_machine_that_has_both`] is the run itself,
//! and it is `#[ignore]`d for `against_a_model_on_this_machine.rs`'s reason: it
//! needs a model on a disk, a runtime serving it, **and** a card the runtime
//! can use, and `cargo test --workspace` runs on machines with none of the
//! three. Like that file's, it asserts its preconditions before it does
//! anything, so asking for it on a machine that cannot do it fails and says
//! which of the three is missing — which is the right way round.
//!
//! # The two runs are asked through the runtime, and that is a deliberate
//! difference from the graded measurement
//!
//! `measuring/mod.rs` puts the fixed set through `alo-asking`'s local door, so
//! that a grade is earned the way a turn asks. There is no road in that door,
//! and putting one there is a change to a crate this plan does not own. What
//! this file compares is **two runs against each other**, and for that the
//! thing that matters is that the two are asked identically: both go through
//! [`alo_models::Ollama::answers_in_the_envelope_on`], on this machine, with
//! the road as the only difference between them — held by
//! `which_road_this_machine_takes.rs`, which reads the bytes of both requests.
//!
//! # The variables
//!
//! | | |
//! |---|---|
//! | `ALO_DRIVING_MODEL` | What the runtime calls the model. Required |
//! | `ALO_DRIVING_ENDPOINT` | Where the runtime is, defaulting to `alo_models::ollama::DEFAULT_ENDPOINT` |
//! | `ALO_DRIVING_ROUNDS` | How many times to put the whole set on each road, defaulting to one |

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

/// The shared harness. This file uses three things from it — the real verb
/// registry, the environment reader and the spelling of a grade — and leaves
/// the rest, which is the loop that earns a catalogue grade through
/// `alo-asking`'s door. Rust counts what one test binary does not call as dead,
/// so the expectation is stated here rather than in the module, where it would
/// hide genuinely dead code from the two files that use the loop.
#[expect(
    dead_code,
    reason = "a shared test module is only ever partly used by each test binary"
)]
mod measuring;

use alo_driving::{Attempt, BothRoads, Exercises, Instructions, Measured, NotAComparison, OnARoad};
use alo_models::card::WhatDrawsHere;
use alo_models::{Catalogue, ModelRuntime, Ollama, Road, WhichRoad};

/// What the runtime calls the model to be measured.
const WHICH_MODEL: &str = "ALO_DRIVING_MODEL";

/// Where the runtime is listening, if it is not where Ollama listens by default.
const WHERE_THE_RUNTIME_IS: &str = "ALO_DRIVING_ENDPOINT";

/// How many times to put the whole fixed set on each road.
const HOW_MANY_ROUNDS: &str = "ALO_DRIVING_ROUNDS";

/// **This machine's own road, and the comparison it cannot make.**
///
/// It asserts nothing about *which* road, because the hardware is the hardware
/// and a test that expected *no card* would fail on the machine task 23 waits
/// for. What it holds is the pair of facts that matter either way: the machine
/// says which road it takes and why, and a machine with no usable card cannot
/// produce a [`BothRoads`] at all — it is refused, naming what is missing,
/// rather than agreeing with itself.
#[test]
fn the_card_road_cannot_be_walked_on_a_machine_with_no_card() {
    let runtime = Ollama::new(Catalogue::built_in().expect("the built-in catalogue loads"));
    let draws = WhatDrawsHere::on_this_machine();
    let weights = Catalogue::built_in()
        .expect("the built-in catalogue loads")
        .get("phi-3-mini-instruct")
        .expect("the entry the image pins")
        .download_bytes;
    // Nothing is loaded here, so this is the road the machine's own bus says it
    // would take rather than one a runtime reported.
    let road = Road::of(None, &draws, runtime.cards_it_can_use(), weights);
    println!("this machine draws with: {draws:?}\nits road: {road:?}");

    assert!(
        !road.is_the_card(),
        "nothing was loaded, so no road can be the card's: {road:?}"
    );

    // Two runs this machine could actually make are two runs on one road, and
    // that is refused rather than reported as agreement.
    let refused = BothRoads::of(
        OnARoad::of(road.clone(), a_run_of_the_set()),
        OnARoad::of(road, a_run_of_the_set()),
    )
    .expect_err("a machine with no card has no second road");
    assert_eq!(refused, NotAComparison::NeitherWasOnACard);
    println!(
        "the card road is not shown from here, and this is why: {refused}\n\
         it waits on a named machine with a card the pinned runtime can use (task 23)"
    );
}

/// **The whole comparison, on a machine that has both roads.**
///
/// One question of each road, the same weights, the same way of asking, and
/// the two held to each other — ADR 0007's claim as a measurement. Everything
/// is printed, because a comparison a reader cannot re-derive is one they have
/// to take on trust.
#[test]
#[ignore = "this needs a model, a runtime and a graphics card the runtime can use — run it with ALO_DRIVING_MODEL set on such a machine"]
fn one_question_on_each_road_of_a_machine_that_has_both() {
    let model = measuring::said(WHICH_MODEL).unwrap_or_default();
    assert!(
        !model.is_empty(),
        "set {WHICH_MODEL} to what the runtime calls the model — a comparison that passed green \
         with no model would be worth nothing"
    );
    let endpoint = measuring::said(WHERE_THE_RUNTIME_IS)
        .unwrap_or_else(|| alo_models::ollama::DEFAULT_ENDPOINT.to_owned());
    let rounds: usize =
        measuring::said(HOW_MANY_ROUNDS).map_or(1, |value| value.parse().unwrap_or(0));
    assert!(
        rounds > 0,
        "{HOW_MANY_ROUNDS} must be a whole number of rounds, and at least one"
    );

    let runtime = Ollama::at(
        &endpoint,
        Catalogue::built_in().expect("the built-in catalogue loads"),
    );
    let draws = WhatDrawsHere::on_this_machine();
    println!("this machine draws with: {draws:?}");
    assert!(
        draws
            .cards()
            .iter()
            .any(|card| runtime.cards_it_can_use().contains(&card.made_by())),
        "this machine has no card the pinned runtime can use, so there is no card road to \
         compare — that is task 23's hardware, and it is not here: {draws:?}"
    );

    let as_the_machine_is = a_run_on(WhichRoad::AsTheMachineIs, &model, &runtime, rounds, &draws);
    let the_processor = a_run_on(WhichRoad::TheProcessor, &model, &runtime, rounds, &draws);

    println!(
        "\nas the machine is: {:?}\n  {} of {} drove\non the processor: {:?}\n  {} of {} drove",
        as_the_machine_is.road(),
        as_the_machine_is.measured().drove(),
        as_the_machine_is.measured().how_many(),
        the_processor.road(),
        the_processor.measured().drove(),
        the_processor.measured().how_many(),
    );

    let both = BothRoads::of(as_the_machine_is, the_processor).unwrap_or_else(|why| {
        panic!(
            "these two runs are not a comparison of the two roads: {why} — a machine whose \
             runtime would not use its card cannot show ADR 0007's claim"
        )
    });
    let (on_the_card, on_the_processor) = both.grades();
    println!(
        "\non the card: {}\non the processor: {}\nthey differed about: {:?}",
        measuring::as_it_is_written(on_the_card),
        measuring::as_it_is_written(on_the_processor),
        both.where_they_differed(),
    );
    assert!(
        both.agreed(),
        "the card changed what this model could do, which ADR 0007 says it must not: {} on the \
         card and {} on the processor, differing about {:?}",
        measuring::as_it_is_written(on_the_card),
        measuring::as_it_is_written(on_the_processor),
        both.where_they_differed(),
    );
}

/// The fixed set put to `model` on one named road, with the road the machine
/// actually took read back from the runtime afterwards.
///
/// **The road is read, never assumed.** Asking for the processor is not the
/// same as having run there, and what says which happened is the residency the
/// runtime reports for the weights it is holding — which is exactly the rule
/// [`Road::of`] is built on.
fn a_run_on(
    asked_on: WhichRoad,
    model: &str,
    runtime: &Ollama,
    rounds: usize,
    draws: &WhatDrawsHere,
) -> OnARoad {
    let verbs = measuring::the_verbs();
    let exercises = Exercises::over(&verbs).expect("the fixed set is built over alo OS's verbs");
    let instructions = Instructions::AsFirstWritten;

    // The first question to a runtime loads the weights, which is a disk read
    // rather than a model thinking — `measuring/mod.rs` has why that is never
    // scored. It also puts the weights where this road puts them, which is what
    // the residency below is read from.
    let warmed =
        runtime.answers_in_the_envelope_on("Answer with the single word: ready.", model, asked_on);
    assert!(
        warmed.is_ok(),
        "{model} did not answer at all on {asked_on:?}, so nothing was measured: {warmed:?}"
    );

    let held = runtime
        .loaded()
        .expect("a runtime that answered a question will say what it is holding");
    // One model is loaded during a run, so the first entry is these weights.
    // Matching by name would mean spelling the runtime's naming convention
    // outside the one file allowed to know it (ADR 0006).
    let road = Road::of(
        held.first(),
        draws,
        runtime.cards_it_can_use(),
        held.first().map_or(0, |loaded| loaded.loaded_bytes),
    );
    println!("asked on {asked_on:?}, the machine took: {road:?}");

    let mut attempts: Vec<Attempt> = Vec::new();
    for round in 1..=rounds {
        for exercise in exercises.all() {
            let said = runtime
                .answers_in_the_envelope_on(
                    &exercises.prompt_under(instructions, exercise),
                    model,
                    asked_on,
                )
                .unwrap_or_else(|why| {
                    panic!(
                        "{model} did not answer the {} exercise on {asked_on:?}: {why:?} — a \
                         machine that failed is never scored as a model that failed",
                        exercise.named()
                    )
                });
            let attempt = exercises.attempt(exercise, &said);
            println!(
                "round {round}  {:<8}  {:?}",
                exercise.named(),
                attempt.outcome()
            );
            attempts.push(attempt);
        }
    }
    let measured = Measured::of(&exercises, attempts)
        .expect("every exercise was asked, which is what makes a run a measurement");
    OnARoad::of(road, measured)
}

/// A complete run of the fixed set, answered the way a model that lost the
/// structure answers — enough to be a measurement, which is all the refusal
/// above needs.
fn a_run_of_the_set() -> Measured {
    let verbs = measuring::the_verbs();
    let exercises = Exercises::over(&verbs).expect("the fixed set is built over alo OS's verbs");
    let attempts = exercises
        .all()
        .map(|exercise| exercises.attempt(exercise, "Of course — I'll take care of that for you."))
        .collect();
    Measured::of(&exercises, attempts).expect("every exercise was asked")
}
