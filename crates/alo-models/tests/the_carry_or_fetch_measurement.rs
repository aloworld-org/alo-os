//! The carry-or-fetch measurement ADR 0025 owes, held to the catalogue it was
//! read off.
//!
//! `docs/quirks.md` carries the measurement: every catalogued entry with its
//! size and its verb-driving grade, what the update channel can honestly
//! carry, and the sentence — today, *no weights go aboard*, because no
//! catalogued entry clears the bar. That entry is the deliverable ADR 0025
//! named, and it is the kind of document that rots in the worst direction: the
//! day somebody measures a model `reliably`, nothing about that run changes a
//! paragraph in `docs/quirks.md`, and a plan made from the stale sentence
//! ships a machine with no model on a recommendation whose numbers moved.
//!
//! So the entry is parsed rather than admired, and held to three things:
//!
//! - **Every number is the catalogue's.** Each table row names a catalogued
//!   entry, its `download_bytes` and its `drives_verbs` grade exactly, and
//!   every catalogued entry is in the table — a measurement over some of the
//!   catalogue would carry the authority of one over all of it.
//! - **The verdict is still true.** While nothing clears the bar the entry
//!   must say so; the day something does, the entry must name the smallest
//!   entry that clears it, and saying both at once is refused.
//! - **The answer is beside the numbers**, as ADR 0025 requires: the channel's
//!   reasoning and the carry-or-fetch sentence live in the same entry as the
//!   table, not somewhere a reader of the numbers would miss.
//!
//! It needs no model and no socket: it reads this repository's own files and
//! the catalogue compiled into this crate, which is deliberate — the
//! measurement has to be checkable on a machine that cannot load a single
//! catalogued entry, which is the measuring box itself.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use alo_models::{Catalogue, Driving};

/// The heading of the entry in `docs/quirks.md` that carries the measurement.
const THE_ENTRY: &str = "### The carry-or-fetch measurement ADR 0025 owes";

/// Where the measurement is written, relative to the repository.
const THE_MEASUREMENT: &str = "docs/quirks.md";

/// The plan whose next task is the grade the weights wait on.
const THE_PLAN: &str = "docs/autonomy/v0-01-lane-b-plan.md";

/// What the entry must say while no catalogued entry clears the bar.
const NOTHING_CLEARS: &str = "none of them clears the bar";

/// The sentence's verdict while there is nothing to carry.
const NOTHING_ABOARD: &str = "no weights go aboard";

/// The honest bound on the channel half: reasoning, not infrastructure.
const HONESTLY_BOUNDED: &str = "not been measured on real infrastructure";

/// How much of an answer the channel and sentence paragraphs have to be.
///
/// Not a judge of the reasoning — nothing mechanical can be — only the floor
/// beneath one, so that `**The channel:** fine.` cannot pass as a measurement.
const AN_ANSWER: usize = 200;

/// One model as the measurement states it: a name, a size, a grade.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Stated {
    /// The catalogue id.
    id: String,
    /// `download_bytes`, what the disk actually loses.
    bytes: u64,
    /// The grade, spelled the way the catalogue spells it.
    grade: String,
}

/// This repository, from the crate this test is in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the repository is where this crate says it is")
}

/// A file of this repository, read.
fn reading(named: &str) -> String {
    let at = the_repository().join(named);
    fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// A grade as `data/catalogue.toml` writes it.
fn spelled(grade: Driving) -> &'static str {
    match grade {
        Driving::Reliably => "reliably",
        Driving::Sometimes => "sometimes",
        Driving::Rarely => "rarely",
        Driving::NotMeasured => "not-measured",
    }
}

/// Every catalogued entry, off the catalogue rather than from memory.
fn off_the_catalogue() -> Vec<(Stated, bool)> {
    Catalogue::built_in()
        .expect("the built-in catalogue loads; its own test says so first")
        .models
        .iter()
        .map(|model| {
            (
                Stated {
                    id: model.id.clone(),
                    bytes: model.download_bytes,
                    grade: spelled(model.drives_verbs).to_owned(),
                },
                model.drives_verbs.clears_the_bar(),
            )
        })
        .collect()
}

/// The measurement's own entry: from its heading to the next one.
fn the_entry_in(document: &str) -> String {
    let mut kept = Vec::new();
    let mut inside = false;
    for line in document.lines() {
        if line.starts_with(THE_ENTRY) {
            inside = true;
            continue;
        }
        if inside && (line.starts_with("## ") || line.starts_with("### ")) {
            break;
        }
        if inside {
            kept.push(line);
        }
    }
    kept.join("\n")
}

/// The table's rows, read the way a person reads them.
///
/// A row is a line beginning with a pipe whose first cell is a backticked id;
/// the header and the rule beneath it are neither, so they fall away without
/// being special-cased. Bytes may be written with the underscores the
/// catalogue itself uses.
fn the_table_in(entry: &str) -> Vec<Stated> {
    let mut rows = Vec::new();
    for line in entry.lines() {
        if !line.trim_start().starts_with('|') {
            continue;
        }
        let cells: Vec<&str> = line
            .trim()
            .trim_start_matches('|')
            .trim_end_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        let [id, bytes, grade] = cells.as_slice() else {
            continue;
        };
        let Some(id) = id.strip_prefix('`').and_then(|it| it.strip_suffix('`')) else {
            continue;
        };
        let Ok(bytes) = bytes.replace(['_', ','], "").parse::<u64>() else {
            continue;
        };
        rows.push(Stated {
            id: id.to_owned(),
            bytes,
            grade: grade.trim_matches('`').to_owned(),
        });
    }
    rows
}

/// One paragraph of the entry, from its bold marker to the next blank line
/// before another marker — enough to hold a floor to.
fn the_part(entry: &str, marker: &str) -> Option<String> {
    let (_, from) = entry.split_once(marker)?;
    let until = from.find("\n\n**").unwrap_or(from.len());
    from.get(..until).map(str::to_owned)
}

/// Whether the measurement still holds, against what the catalogue says now.
///
/// Separate from the files it reads so that each refusal below can be shown
/// happening: a documentation test that has never been seen to fail is a
/// documentation test nobody should believe.
///
/// # Errors
/// A sentence naming what stopped being true and where to go about it.
fn whether_it_still_holds(catalogue: &[(Stated, bool)], entry: &str) -> Result<(), String> {
    let rows = the_table_in(entry);
    if rows.is_empty() {
        return Err(format!(
            "there is no table of catalogued entries under `{THE_ENTRY}` in {THE_MEASUREMENT}, \
             so this test is checking nothing. The measurement is the numbers; if the entry \
             moved, this test moves with it"
        ));
    }
    if catalogue.is_empty() {
        return Err(
            "the catalogue answered with no entries at all, so nothing below is being compared \
             against anything"
                .to_owned(),
        );
    }

    for row in &rows {
        let Some((stated, _)) = catalogue.iter().find(|(it, _)| it.id == row.id) else {
            return Err(format!(
                "the measurement names `{}`, and the catalogue has no such entry. A number about \
                 a model nobody offers is not a measurement of this catalogue",
                row.id
            ));
        };
        if stated.bytes != row.bytes {
            return Err(format!(
                "the measurement says `{}` is {} bytes and the catalogue says {}. The numbers \
                 are the catalogue's or they are from memory",
                row.id, row.bytes, stated.bytes
            ));
        }
        if stated.grade != row.grade {
            return Err(format!(
                "the measurement grades `{}` as `{}` and the catalogue says `{}`. A grade is a \
                 measurement somebody ran, and this entry may only repeat it",
                row.id, row.grade, stated.grade
            ));
        }
    }
    for (stated, _) in catalogue {
        if !rows.iter().any(|row| row.id == stated.id) {
            return Err(format!(
                "the catalogue offers `{}` and the measurement's table does not carry it. A \
                 measurement over part of the catalogue reads as one over all of it",
                stated.id
            ));
        }
    }

    let smallest_clearing = catalogue
        .iter()
        .filter(|(_, clears)| *clears)
        .min_by_key(|(stated, _)| stated.bytes);
    match smallest_clearing {
        None => {
            if !entry.contains(NOTHING_CLEARS) {
                return Err(format!(
                    "no catalogued entry clears the bar and the entry does not say \
                     `{NOTHING_CLEARS}`. That finding is the whole of the first number, and it \
                     is stated or the measurement is not one"
                ));
            }
            if !entry.contains(NOTHING_ABOARD) {
                return Err(format!(
                    "nothing clears the bar, so the sentence is `{NOTHING_ABOARD}` — an answer \
                     of carry or fetch over an empty candidate list would be a recommendation \
                     with no numbers under it, which is what ADR 0025 refused to be"
                ));
            }
        }
        Some((stated, _)) => {
            if entry.contains(NOTHING_CLEARS) {
                return Err(format!(
                    "`{}` now clears the bar at {} bytes, and the entry still says \
                     `{NOTHING_CLEARS}`. The measurement must be made again: name it, its size \
                     and its grade, and turn the sentence into carry or fetch for real",
                    stated.id, stated.bytes
                ));
            }
            if !the_part(entry, "**The sentence:**").is_some_and(|it| it.contains(&stated.id)) {
                return Err(format!(
                    "the smallest entry that clears the bar is `{}` and the sentence does \
                     not name it. The first number is that model's, off the catalogue, and \
                     the answer is made about it by name",
                    stated.id
                ));
            }
        }
    }

    for marker in ["**The channel:**", "**The sentence:**"] {
        let Some(part) = the_part(entry, marker) else {
            return Err(format!(
                "the entry has no `{marker}` — the measurement is two numbers and a sentence, \
                 and ADR 0025 says the answer goes beside the numbers rather than somewhere a \
                 reader of the table would miss it"
            ));
        };
        if part.len() < AN_ANSWER {
            return Err(format!(
                "`{marker}` says almost nothing. This is the floor under an answer, not a judge \
                 of one; what the channel can carry is stated with its reasoning or it is not \
                 stated"
            ));
        }
    }

    Ok(())
}

/// **The first number is read off the catalogue, and today it is nobody:**
/// no catalogued entry clears the verb-driving bar, the entry says so, and
/// every size and grade it states is the catalogue's own.
#[test]
fn the_smallest_entry_that_clears_the_bar_is_read_off_the_catalogue() {
    let catalogue = off_the_catalogue();
    assert!(
        catalogue.iter().all(|(_, clears)| !clears),
        "a catalogued entry now clears the bar — the carry-or-fetch measurement in \
         {THE_MEASUREMENT} was made over a catalogue where none did, and it must be made again \
         rather than left standing"
    );

    let entry = the_entry_in(&reading(THE_MEASUREMENT));
    assert!(
        entry.contains(NOTHING_CLEARS),
        "the finding is `{NOTHING_CLEARS}`, and the entry does not say it"
    );
    if let Err(why) = whether_it_still_holds(&catalogue, &entry) {
        panic!("{why}");
    }
}

/// **What the update channel can carry is stated with its reasoning, and the
/// bound is honest:** nothing of ours runs yet, and the entry says so in as
/// many words rather than dressing an argument as a measurement.
#[test]
fn what_the_channel_can_carry_is_stated_and_honestly_bounded() {
    let entry = the_entry_in(&reading(THE_MEASUREMENT));
    let channel =
        the_part(&entry, "**The channel:**").expect("the entry states what the channel can carry");
    assert!(
        channel.len() >= AN_ANSWER,
        "the channel half is not an answer"
    );
    assert!(
        entry.contains(HONESTLY_BOUNDED),
        "the channel half claims more than anybody measured: nothing of ours is running, and \
         `{HONESTLY_BOUNDED}` is the sentence that keeps the reasoning honest"
    );
}

/// **The answer is beside the numbers**, in the one entry: the table, the
/// channel and the sentence all live under the same heading, which is where
/// ADR 0025 says the answer goes.
#[test]
fn the_answer_is_beside_the_numbers() {
    let entry = the_entry_in(&reading(THE_MEASUREMENT));
    assert!(
        !the_table_in(&entry).is_empty(),
        "the numbers are not in the entry"
    );
    assert!(
        the_part(&entry, "**The sentence:**").is_some_and(|it| it.contains(NOTHING_ABOARD)),
        "the sentence is not beside the numbers, or it is not `{NOTHING_ABOARD}` on a catalogue \
         where nothing clears the bar"
    );
}

/// **The task the answer waits on is written in the plan**, in the same
/// change: the weights wait on the catalogue, so the plan's next task is the
/// grade — measured, never regraded from memory.
#[test]
fn the_weights_task_is_written_and_waits_on_the_catalogue() {
    let plan = reading(THE_PLAN);
    assert!(
        plan.contains("### 8. The carry-or-fetch measurement ADR 0025 owes")
            && plan.contains("### 9. "),
        "{THE_PLAN} does not carry a task after the measurement, so the loop would send the \
         next worker at work that is already done"
    );
    let (_, after) = plan
        .split_once("### 9. ")
        .expect("the plan has a task 9; the assertion above said so");
    assert!(
        after.contains("catalogue") && after.contains("alo-driving"),
        "task 9 is not the grade the weights wait on: the finding was that the weights task \
         waits on the catalogue, and the next task is measuring it"
    );
}

/// **And the check would catch each way the measurement can stop being
/// true**, which is the half a green run cannot tell anybody.
///
/// Every refusal it exists for is put in front of it here, starting from a
/// sound pair so the refusals are about the fault and not about a checker
/// that never accepted anything.
#[test]
fn the_check_catches_a_measurement_that_has_stopped_being_true() {
    let stated = |id: &str, bytes: u64, grade: &str| Stated {
        id: id.to_owned(),
        bytes,
        grade: grade.to_owned(),
    };
    let catalogue = vec![
        (stated("small-but-rarely", 1_000, "rarely"), false),
        (stated("unmeasured-7b", 4_000, "not-measured"), false),
    ];
    let answer = "a".repeat(AN_ANSWER);
    let sound = format!(
        "| Entry | Download bytes | Drives the verbs |\n\
         |---|---|---|\n\
         | `small-but-rarely` | 1_000 | `rarely` |\n\
         | `unmeasured-7b` | 4_000 | `not-measured` |\n\n\
         The finding: {NOTHING_CLEARS}.\n\n\
         **The channel:** {answer}, and it has {HONESTLY_BOUNDED}.\n\n\
         **The sentence:** {NOTHING_ABOARD}; {answer}.\n\n\
         **Date:** today."
    );
    assert_eq!(
        whether_it_still_holds(&catalogue, &sound),
        Ok(()),
        "a sound measurement was refused, so the refusals below say nothing"
    );

    // The one this test exists for: a model now clears the bar and the entry
    // still answers for a catalogue where nothing did.
    let cleared = vec![
        (stated("small-but-rarely", 1_000, "rarely"), false),
        (stated("now-reliable", 4_000, "reliably"), true),
    ];
    let with_it = sound.replace(
        "| `unmeasured-7b` | 4_000 | `not-measured` |",
        "| `now-reliable` | 4_000 | `reliably` |",
    );
    assert!(
        whether_it_still_holds(&cleared, &with_it)
            .is_err_and(|why| why.contains("must be made again")),
        "a catalogue with a clearing entry was accepted against a measurement that says nothing \
         clears"
    );

    // The same future done half-right: the stale verdict removed, and the
    // model that clears still not named.
    let unnamed = with_it.replace(NOTHING_CLEARS, "one of them clears it now");
    assert!(
        whether_it_still_holds(&cleared, &unnamed)
            .is_err_and(|why| why.contains("does not name it")),
        "a measurement that names no clearing model was accepted for a catalogue with one"
    );

    // A number from memory rather than from the catalogue.
    let resized = sound.replace(
        "| `small-but-rarely` | 1_000 |",
        "| `small-but-rarely` | 999 |",
    );
    assert!(
        whether_it_still_holds(&catalogue, &resized).is_err_and(|why| why.contains("from memory")),
        "a size the catalogue does not state was accepted"
    );

    // A grade the entry awarded itself.
    let regraded = sound.replace("| 1_000 | `rarely` |", "| 1_000 | `reliably` |");
    assert!(
        whether_it_still_holds(&catalogue, &regraded)
            .is_err_and(|why| why.contains("may only repeat it")),
        "a grade differing from the catalogue's was accepted"
    );

    // A model the catalogue does not offer.
    let stranger = sound.replace("`small-but-rarely`", "`somebody-elses-model`");
    assert!(
        whether_it_still_holds(&catalogue, &stranger)
            .is_err_and(|why| why.contains("no such entry")),
        "a row about a model outside the catalogue was accepted"
    );

    // A catalogued entry the table quietly dropped.
    let partial = sound.replace("| `unmeasured-7b` | 4_000 | `not-measured` |\n", "");
    assert!(
        whether_it_still_holds(&catalogue, &partial)
            .is_err_and(|why| why.contains("does not carry it")),
        "a table missing a catalogued entry was accepted"
    );

    // A sentence of carry or fetch over an empty candidate list.
    let decided = sound.replace(NOTHING_ABOARD, "carried, obviously");
    assert!(
        whether_it_still_holds(&catalogue, &decided)
            .is_err_and(|why| why.contains("no numbers under it")),
        "an answer with no candidate under it was accepted"
    );

    // A channel half that says almost nothing.
    let shrugged = sound.replace(
        &format!("**The channel:** {answer}, and it has {HONESTLY_BOUNDED}."),
        "**The channel:** fine.",
    );
    assert!(
        whether_it_still_holds(&catalogue, &shrugged)
            .is_err_and(|why| why.contains("floor under an answer")),
        "a channel half with nothing in it was accepted"
    );

    // And the two that mean the check is looking at nothing at all.
    assert!(
        whether_it_still_holds(&catalogue, "no table here")
            .is_err_and(|why| why.contains("checking nothing")),
        "an entry with no table was accepted, so a heading that moved would pass in silence"
    );
    assert!(
        whether_it_still_holds(&[], &sound).is_err_and(|why| why.contains("no entries at all")),
        "an empty catalogue was accepted as something to compare against"
    );
}

/// **The entry is read the way a person reads it**: the parser finds the table
/// under this heading and not one belonging to another entry, and the bytes
/// keep the catalogue's own underscores.
#[test]
fn the_entry_is_the_one_under_that_heading_and_no_other() {
    let document = format!(
        "## Models\n\n{THE_ENTRY}: the catalogue has nothing to weigh\n\
         | Entry | Download bytes | Drives the verbs |\n\
         |---|---|---|\n\
         | `smollm2-1.7b-instruct` | 1_060_000_000 | `rarely` |\n\n\
         ### A different entry entirely\n\
         | `not-this-one` | 1 | `reliably` |\n"
    );
    let rows = the_table_in(&the_entry_in(&document));
    assert_eq!(
        rows,
        vec![Stated {
            id: "smollm2-1.7b-instruct".to_owned(),
            bytes: 1_060_000_000,
            grade: "rarely".to_owned(),
        }],
        "the parser read a table that is not the measurement's, or lost the underscored bytes"
    );
    assert!(
        the_entry_in("### A heading nothing is under\ntext").is_empty(),
        "the parser found an entry under a heading that is not in the document"
    );
}
