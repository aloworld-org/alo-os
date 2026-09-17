//! **No step a person walks names anything we rented.**
//!
//! `docs/features.md`: *a person never learns the name of anything we rented*,
//! and this plan's task 4 says it exactly: no step names LoRA, QLoRA, a rank, a
//! learning rate, an epoch or a checkpoint.
//!
//! # If this test has stopped you
//!
//! You have put a rented name, or a training setting, into something a person
//! reads. The fix is not to rename it more gently — it is that **the value is
//! not a step**. Advanced values live in `docs/contracts/fine-tuning-values.md`
//! and in a file for people who want them; the flow asks five questions whose
//! answers a person already has.
//!
//! What this reads: every sentence this crate declares, and the flow itself.
//! Not the engine — `engine.rs` is the one file allowed to name the stack, and
//! naming it there is how nothing else has to.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_adapting::{Step, words};

/// Names of things we rent, and settings only a toolchain asks about.
const NOTHING_A_PERSON_READS_MAY_SAY: [&str; 14] = [
    "lora",
    "qlora",
    "rank",
    "learning rate",
    "epoch",
    "checkpoint",
    "gradient",
    "optimizer",
    "optimiser",
    "peft",
    "transformers",
    "safetensors",
    "gguf",
    "tensor",
];

#[test]
fn no_sentence_a_person_reads_names_a_rented_thing_or_a_training_setting() {
    let mut said: Vec<String> = Vec::new();
    for word in words::EVERY_WORD {
        let sentence = word.says().to_lowercase();
        for never in NOTHING_A_PERSON_READS_MAY_SAY {
            if sentence.contains(never) {
                said.push(format!("{} says `{never}`", word.key()));
            }
        }
    }
    assert!(
        said.is_empty(),
        "a person reads the name of something we rented, or a setting only a toolchain asks \
         about:\n{said:#?}\n\nThe value is not a step. Put it in \
         docs/contracts/fine-tuning-values.md and leave the flow to the five questions a person \
         already has answers to."
    );
}

/// **And every step has a sentence**, so none of the five is a screen somebody
/// wrote words for in one surface and not another.
#[test]
fn every_step_of_the_flow_has_a_sentence_this_crate_declares() {
    for step in Step::ALL {
        let word = step.word();
        assert!(
            words::EVERY_WORD
                .iter()
                .any(|declared| declared.key() == word.key()),
            "{step:?} says something this crate does not declare"
        );
        assert!(
            !word.says().is_empty(),
            "{step:?} has a sentence that says nothing"
        );
        word.note()
            .expect("every step tells a translator what it is for");
    }
}

/// **The notes may name what a translator needs**, but a note is not a
/// sentence: this holds the *said* text only, and says so here so that nobody
/// later widens the test into forbidding the explanation as well.
#[test]
fn a_translators_note_may_explain_what_the_sentence_may_not_say() {
    let with_notes = words::EVERY_WORD
        .iter()
        .filter(|word| word.note().is_some())
        .count();
    assert_eq!(with_notes, words::EVERY_WORD.len());
}
