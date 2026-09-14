//! **The measurement, put to weights a person brought** — task 4 of
//! `docs/autonomy/v0-5-the-models-measured-plan.md`.
//!
//! The same ten, the same bar and the same door as
//! `against_a_model_on_this_machine.rs` — literally: both call
//! `tests/measuring/mod.rs`. What differs is the road to the weights. A file on
//! this disk is taken as brought (`alo_models::Weights::at`, its id the file's
//! own name), handed to the pinned runtime the way it accepts one
//! (`alo_models::ModelRuntime::bring`), and measured by that id. The grade is
//! then written **into the person's own settings beside the file**, through
//! `alo_choosing::Choosing::measuring`, with the machine and date beside it —
//! never into the shipped catalogue, which is not theirs.
//!
//! `#[ignore]`d for the reason its sibling is. Run it with:
//!
//! ```text
//! ALO_DRIVING_FILE=/absolute/path/to/weights.gguf \
//! ALO_DRIVING_MACHINE="Apple M3, 8 GB unified memory" \
//! ALO_DRIVING_DATE=2026-09-14 \
//! ALO_DRIVING_SETTINGS=/path/to/alo/settings.toml \
//!   cargo test -p alo-driving --test against_a_file_brought_to_this_machine \
//!   -- --ignored --nocapture
//! ```
//!
//! The machine and the date are given rather than guessed: the machine because
//! a harness that read a model name off the operating system would be writing a
//! description nobody checked, and the date because this workspace keeps no
//! clock library for one line. They are checked before anything is run — a run
//! whose grade could not be written would be a run wasted.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

// This harness writes a person's grade into their settings, which hold the grade
// for the way a turn asks today — freely — and no envelope grade; so the shared
// loop's `Asked::InTheEnvelope` is never built here. The other harness builds it.
#[allow(
    dead_code,
    reason = "the envelope road is measured by against_a_model_on_this_machine.rs"
)]
mod measuring;

use std::path::PathBuf;

use alo_choosing::Choosing;
use alo_models::{Catalogue, MeasuredOn, ModelRuntime, Ollama, THE_PINNED_RUNTIME, Weights};

#[test]
#[ignore = "needs the pinned runtime and a weights file on this machine — see this file's header"]
fn the_fixed_set_put_to_a_file_somebody_brought() {
    let file = measuring::said("ALO_DRIVING_FILE")
        .map(PathBuf::from)
        .expect("set ALO_DRIVING_FILE to an absolute path of a weights file on this disk");
    let on = MeasuredOn {
        machine: measuring::said("ALO_DRIVING_MACHINE")
            .expect("set ALO_DRIVING_MACHINE to the machine, with its memory in GB"),
        date: measuring::said("ALO_DRIVING_DATE").expect("set ALO_DRIVING_DATE to YYYY-MM-DD"),
        runtime: format!("Ollama {THE_PINNED_RUNTIME}"),
        drove: None,
        of: None,
    };
    let settings = measuring::said("ALO_DRIVING_SETTINGS").map(PathBuf::from);
    let endpoint = alo_models::ollama::DEFAULT_ENDPOINT;

    let weights = Weights::at(&file).expect("the file is a file on this disk");
    // Checked before the run, through the same rule the settings write holds
    // it to, so an hour of measurement cannot end in a grade nobody can place.
    assert!(
        weights
            .clone()
            .measured(alo_models::Driving::Rarely, on.clone())
            .grade_is_placed(),
        "{on:?} does not say, checkably, which machine this is — the grade could not be written"
    );

    let runtime = Ollama::at(
        endpoint,
        Catalogue::built_in().expect("the catalogue loads"),
    );
    runtime
        .bring(&weights)
        .expect("the pinned runtime accepts the brought file");
    println!("brought {} as {}", file.display(), weights.id);

    let measured =
        measuring::the_fixed_set_put_to(&weights.id, endpoint, 1, measuring::Asked::Freely);

    if let Some(at) = settings {
        let mut choosing = Choosing::at(&at).expect("the settings are readable");
        if choosing.settings().brought().get(&weights.id).is_none() {
            choosing
                .bringing_a_file(&file)
                .expect("the file goes on the person's list");
        }
        // The counts go beside the grade, so it can be re-derived from the file.
        let counted = MeasuredOn {
            drove: u32::try_from(measured.drove()).ok(),
            of: u32::try_from(measured.how_many()).ok(),
            ..on
        };
        choosing
            .measuring(&weights.id, measured.grade(), counted)
            .expect("the grade is written beside the file");
        let theirs = choosing
            .settings()
            .brought()
            .get(&weights.id)
            .expect("the weights are on the list")
            .clone();
        println!(
            "written to {}: drives-verbs = \"{}\", can be the agent: {}",
            at.display(),
            measuring::as_it_is_written(theirs.drives_verbs),
            theirs.can_be_the_agent()
        );
    }

    runtime
        .unload(&weights.id)
        .expect("the brought model is let go of");
    runtime
        .remove(&weights.id)
        .expect("the brought model is not left in the runtime");
}
