//! **What a person is told, in the order they meet it** — task 5 of
//! `docs/autonomy/v0-5-the-models-measured-plan.md`.
//!
//! The measurement work ends in sentences, and the sentences are the product.
//! This walks `alo-choosing` and `alo-telling` from a fresh settings file, on a
//! machine of 16 GB where no catalogued model clears the bar, through the five
//! situations a person meets, and records every line they read — word for word,
//! in the vocabulary the whole machine loads.
//!
//! [`THE_TABLE`] is that record, and it is the same table the task's report
//! prints. [`what_a_person_reads_is_the_table`] fails if any sentence in the
//! sequence changes without the table changing, so a sentence cannot drift away
//! from what somebody signed off as reading well and being true.
//!
//! Three things every sentence in it is held to: it is in the machine's
//! vocabulary, it carries a note for whoever translates it, and it claims no
//! measurement on the reader's machine — a grade travels with the weights, and
//! the machine it was earned on is shown beside the sentence, never inside it.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::PathBuf;

use alo_choosing::Choosing;
use alo_models::costing::GIGABYTE;
use alo_models::{Catalogue, Driving, MeasuredOn, Weights, words as models};
use alo_strings::{Said, Strings, Word};
use alo_telling::{Warning, WhoAsked, words as telling};

/// The memory of the machine this walk is on.
const SIXTEEN_GIGABYTES: f32 = 16.0;

/// One situation, and the lines a person reads in it, in order: the word each
/// line is, and its English exactly.
type Situation = (&'static str, &'static [(Word, &'static str)]);

/// **Every line a person reads, in the order they meet it.**
const THE_TABLE: [Situation; 5] = [
    (
        "The model the catalogue recommends",
        &[
            (
                models::NONE_CLEARS_THE_BAR,
                "the models that run on this machine do not produce a workable instruction often \
                 enough to be given the agent",
            ),
            (
                models::WEIGHTS_YOU_ALREADY_HAVE,
                "you can point alo OS at weights you already have on this machine, and it will \
                 run them — this catalogue is what alo OS offers, not everything it can run",
            ),
            (
                models::THE_OTHER_PLACES,
                "you can use a model on a machine you have paired with on your network, or a \
                 provider you add — whichever you prefer, and alo OS will not choose for you",
            ),
        ],
    ),
    (
        "A model that runs, measured sometimes driving the verbs",
        &[
            (
                models::WEIGHTS_FIT,
                "these weights fit in the memory this machine has",
            ),
            (
                models::LICENCE_IS_YOURS,
                "these weights are yours, and so are their terms — alo OS states the licence of what it \
                 offers and has not read the licence of a model it did not offer you",
            ),
            (
                models::WEIGHTS_MEASURED_NOT_THE_AGENT,
                "these weights have been measured driving the agent's verbs, not often enough to be \
                 given agent turns — they still answer your questions",
            ),
        ],
    ),
    (
        "A model that runs, measured rarely driving the verbs",
        &[
            (
                models::WEIGHTS_FIT,
                "these weights fit in the memory this machine has",
            ),
            (
                models::LICENCE_IS_YOURS,
                "these weights are yours, and so are their terms — alo OS states the licence of what it \
                 offers and has not read the licence of a model it did not offer you",
            ),
            (
                models::WEIGHTS_MEASURED_NOT_THE_AGENT,
                "these weights have been measured driving the agent's verbs, not often enough to be \
                 given agent turns — they still answer your questions",
            ),
        ],
    ),
    (
        "A model too large for this machine",
        &[
            (
                models::WEIGHTS_LARGER_THAN_MEMORY,
                "these weights are larger than the memory this machine has — alo OS will still run \
                 them, and this machine will be slow",
            ),
            (
                telling::RUNS_THEM_ANYWAY,
                "That is said once: alo OS will run these weights whenever you choose them, and will \
                 not mention their size again by itself",
            ),
        ],
    ),
    (
        "A file they brought",
        &[
            (
                models::WEIGHTS_FIT,
                "these weights fit in the memory this machine has",
            ),
            (
                models::LICENCE_IS_YOURS,
                "these weights are yours, and so are their terms — alo OS states the licence of what it \
                 offers and has not read the licence of a model it did not offer you",
            ),
            (
                models::WEIGHTS_NOT_MEASURED,
                "nobody has measured whether these weights can drive the agent's verbs, and alo OS has \
                 not guessed — they answer your questions now, and get an agent turn once a \
                 measurement says they can",
            ),
        ],
    ),
];

/// The vocabulary a machine actually holds.
fn what_this_machine_can_say() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// A fresh settings file, in a folder only this walk uses.
fn a_fresh_settings_file(what: &str) -> (PathBuf, PathBuf) {
    let folder =
        std::env::temp_dir().join(format!("alo-telling-walk-{what}-{}", std::process::id()));
    if folder.exists() {
        fs::remove_dir_all(&folder).unwrap();
    }
    fs::create_dir_all(&folder).unwrap();
    let at = folder
        .join(alo_choosing::THE_FOLDER)
        .join(alo_choosing::THE_SETTINGS);
    (folder, at)
}

/// The machine every grade in this walk is written as earned on — the one the
/// measurements behind this task were made on.
fn the_measuring_machine() -> MeasuredOn {
    MeasuredOn {
        machine: "Apple M3, 8 GB unified memory".to_owned(),
        date: "2026-09-14".to_owned(),
        runtime: "Ollama 0.34.0".to_owned(),
        drove: None,
        of: None,
        loaded_bytes: None,
        on_the_gpu_bytes: None,
        held_to: None,
        instructions: None,
    }
}

/// A file somebody brings, of this size, and — where given — the grade a
/// measurement wrote beside it, read back off the disk.
fn a_file_brought(what: &str, bytes: u64, grade: Option<Driving>) -> Weights {
    let (folder, at) = a_fresh_settings_file(what);
    let file = folder.join(format!("{what}.gguf"));
    // A sparse file of the stated size: the size is what the disk reports, and
    // nothing here reads what is in it.
    fs::File::create(&file).unwrap().set_len(bytes).unwrap();
    let mut choosing = Choosing::at(&at).unwrap();
    choosing.bringing_a_file(&file).unwrap();
    let id = format!("{what}.gguf");
    if let Some(grade) = grade {
        choosing
            .measuring(&id, grade, the_measuring_machine())
            .unwrap();
    }
    alo_choosing::Settings::at(&at)
        .unwrap()
        .brought()
        .get(&id)
        .unwrap()
        .clone()
}

/// **The walk itself**: every line, in order, as the machine's own code says it.
fn what_a_person_reads(strings: &Strings) -> Vec<(&'static str, Vec<Said>)> {
    let shipped = Catalogue::built_in().unwrap();
    let recommended = shipped
        .agent_for_cpu(SIXTEEN_GIGABYTES)
        .map(|_| Vec::new())
        .unwrap_or_else(|refused| refused.lines(strings).to_vec());

    let sometimes = a_file_brought("sometimes", 4 * GIGABYTE, Some(Driving::Sometimes));
    let rarely = a_file_brought("rarely", 4 * GIGABYTE, Some(Driving::Rarely));
    let enormous = a_file_brought("enormous", 40 * GIGABYTE, None);
    let theirs = a_file_brought("theirs", 4 * GIGABYTE, None);

    let mut warning = Warning::nothing_said_yet();
    let warned = warning.about(&enormous, SIXTEEN_GIGABYTES, WhoAsked::ThePerson);
    let too_large = warned
        .to_say()
        .map(|once| once.lines(strings).to_vec())
        .unwrap_or_default();

    vec![
        (THE_TABLE[0].0, recommended),
        (
            THE_TABLE[1].0,
            sometimes.lines(strings, SIXTEEN_GIGABYTES).to_vec(),
        ),
        (
            THE_TABLE[2].0,
            rarely.lines(strings, SIXTEEN_GIGABYTES).to_vec(),
        ),
        (THE_TABLE[3].0, too_large),
        (
            THE_TABLE[4].0,
            theirs.lines(strings, SIXTEEN_GIGABYTES).to_vec(),
        ),
    ]
}

/// **The sequence a person reads is the table**, word for word and in order.
#[test]
fn what_a_person_reads_is_the_table() {
    let strings = what_this_machine_can_say();
    let walked = what_a_person_reads(&strings);
    for ((situation, lines), (expected_situation, expected)) in walked.iter().zip(THE_TABLE) {
        println!("\n## {situation}");
        for line in lines {
            println!("{}", line.text());
        }
        assert_eq!(situation, &expected_situation);
        assert_eq!(lines.len(), expected.len(), "{situation}");
        for (line, (word, english)) in lines.iter().zip(expected) {
            assert_eq!(
                line.text(),
                strings
                    .say(&word.key(), &alo_strings::Filling::nothing())
                    .text(),
                "{situation}: this line is not {}",
                word.named()
            );
            assert_eq!(
                line.text(),
                *english,
                "{situation}: a sentence changed and the table did not"
            );
        }
    }
}

/// **Every sentence in the table is in the machine's vocabulary, and carries a
/// note for whoever translates it.**
#[test]
fn every_sentence_in_the_table_is_the_machines_and_has_a_note() {
    let vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    for (situation, lines) in THE_TABLE {
        for (word, _) in lines {
            let phrase = vocabulary
                .phrase(&word.key())
                .unwrap_or_else(|| panic!("{situation}: the machine cannot say {}", word.named()));
            assert!(
                phrase.note().is_some_and(|note| !note.trim().is_empty()),
                "{situation}: {} has no note for a translator",
                word.named()
            );
        }
    }
}

/// **No sentence in the table claims a measurement made on this machine.** A
/// grade is a fact about the weights and was earned on the machine written
/// beside it; a sentence that said *measured on this machine* would be true on
/// that machine and false on every other.
#[test]
fn no_sentence_claims_a_measurement_made_on_the_readers_machine() {
    for (situation, lines) in THE_TABLE {
        for (word, english) in lines {
            assert!(
                !claims_a_measurement_here(english),
                "{situation}: {} claims a measurement on the reader's machine: {english}",
                word.named()
            );
        }
    }
}

/// Whether a sentence says something was measured, and on this machine.
fn claims_a_measurement_here(english: &str) -> bool {
    english.contains("measured") && english.contains("this machine")
}

/// **And the check catches the sentence it exists for** — the wording the two
/// measured lines had before this task, when they said *on this machine*.
#[test]
fn the_check_catches_a_measurement_claimed_on_this_machine() {
    assert!(claims_a_measurement_here(
        "these weights have been measured driving the agent's verbs dependably on this machine, \
         so they can be given agent turns"
    ));
    assert!(!claims_a_measurement_here(
        "these weights fit in the memory this machine has"
    ));
}
