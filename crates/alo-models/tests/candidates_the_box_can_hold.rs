//! Candidates the measuring box can actually hold, and what they did with the
//! fixed set.
//!
//! Task 9 found that the catalogue's unmeasured half is exactly the half this
//! lane's box cannot load, and that trying anyway took the guest down. The way
//! to a grade was therefore never a bigger model run harder: it was **entries
//! small enough to be measured here that have a reason to clear the bar**. The
//! five that were already measured are general chat models that failed at the
//! *shape* — the envelope right and the argument list wrong, a fence round the
//! answer, the prompt's own placeholders copied back — and nobody had put a
//! model trained for tool calls and constrained output to the same ten
//! requests.
//!
//! This holds the outcome of doing that to the catalogue it changed, the way
//! `the_carry_or_fetch_measurement.rs` and `the_grade_the_weights_wait_on.rs`
//! hold theirs. Four things, and the first is the one that matters:
//!
//! - **Every candidate was measured.** A curator who adds a name to the
//!   catalogue and leaves it `not-measured` has widened the catalogue and moved
//!   nothing, which is precisely what ADR 0007 refuses: a grade is a run
//!   somebody made.
//! - **Every candidate is one this box can hold.** A model that needs more
//!   memory than the measuring guest has is not a candidate for this work; it
//!   is task 9's finding again with a new name.
//! - **The account names each one and the grade it earned**, spelled the way
//!   `data/catalogue.toml` spells it, so the prose and the catalogue cannot
//!   drift apart — the failure mode every documentation check in this crate
//!   exists for.
//! - **Nothing in the method moved to help a model.** The prompt is
//!   `alo-driving`'s, the scoring is `alo-driving`'s and the wait is
//!   `alo-models`', and the account says all three were left alone.
//!
//! Like its siblings it needs no model and no socket: it reads this
//! repository's own files and the catalogue compiled into this crate.

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

/// The heading of the entry in `docs/quirks.md` that carries the run.
const THE_RUN: &str = "### Two models trained for tool calls, put to the same ten requests";

/// Where findings about models are written.
const THE_QUIRKS: &str = "docs/quirks.md";

/// The catalogue itself, whose own rules the next curator reads.
const THE_CATALOGUE: &str = "crates/alo-models/data/catalogue.toml";

/// The plan this task belongs to.
const THE_PLAN: &str = "docs/autonomy/v0-01-lane-b-plan.md";

/// The candidates this task added, by the ids the catalogue uses.
///
/// Two, which is what the task asked for, and both were chosen because their
/// publishers train them for tool calls and constrained output rather than
/// because they are small — the distinction the whole task turns on.
const THE_CANDIDATES: [&str; 2] = ["qwen3-1.7b", "granite-3.2-2b-instruct"];

/// What the measuring guest has, in gigabytes of system memory.
///
/// `C:\Users\SBW\.wslconfig` gives it `memory=6GB`, which is 5,926 MB once the
/// kernel has taken its share — the number `docs/quirks.md` records. A
/// candidate whose `min_ram_gb` is above this is one the box cannot hold, and
/// task 9 already measured what happens then.
const WHAT_THE_BOX_HAS: f32 = 5.9;

/// The three parts of the method that may not move to get a grade.
const LEFT_ALONE: [&str; 3] = ["the prompt", "the scoring", "five-minute wait"];

/// How much of an account the entry has to be, in bytes.
///
/// Not a judge of one — nothing mechanical can be — only the floor beneath it,
/// so that *both were measured* cannot pass for what they wrote.
const AN_ANSWER: usize = 600;

/// One candidate as the catalogue states it.
#[derive(Debug, Clone, PartialEq)]
struct Candidate {
    /// The catalogue id.
    id: String,
    /// The artefact the entry names, which is what the run was put to.
    artefact: Option<String>,
    /// The grade, spelled the way the catalogue spells it.
    grade: String,
    /// Whether that grade is a measurement somebody ran.
    measured: bool,
    /// The system memory the entry says it needs on the CPU.
    min_ram_gb: f32,
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

/// The candidates, off the catalogue rather than from memory.
fn off_the_catalogue() -> Vec<Candidate> {
    let catalogue = Catalogue::built_in().expect("the built-in catalogue loads");
    THE_CANDIDATES
        .iter()
        .map(|id| {
            let model = catalogue.get(id).unwrap_or_else(|| {
                panic!(
                    "the catalogue has no `{id}`, and it is one of the entries this task added. \
                     If it was renamed, this test and the account in {THE_QUIRKS} move with it"
                )
            });
            Candidate {
                id: model.id.clone(),
                artefact: model.quantised_at().map(|(_, at)| at.to_owned()),
                grade: spelled(model.drives_verbs).to_owned(),
                measured: model.drives_verbs.has_been_measured(),
                min_ram_gb: model.min_ram_gb,
            }
        })
        .collect()
}

/// One entry of `docs/quirks.md`: from its heading to the next one.
fn the_entry_in(document: &str, heading: &str) -> String {
    let mut kept = Vec::new();
    let mut inside = false;
    for line in document.lines() {
        if line.starts_with(heading) {
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

/// Whether the candidates and the account of them still agree.
///
/// Separate from the files it reads so that every refusal below can be shown
/// happening: a documentation check nobody has seen fail is one nobody should
/// believe.
///
/// # Errors
/// A sentence naming what stopped being true and where to go about it.
fn whether_the_candidates_hold(candidates: &[Candidate], entry: &str) -> Result<(), String> {
    if candidates.len() < 2 {
        return Err(
            "fewer than two candidates were compared against the account. The task asked for at \
             least two, because one entry that happens to work is an anecdote"
                .to_owned(),
        );
    }
    for candidate in candidates {
        if !candidate.measured {
            return Err(format!(
                "`{}` is in the catalogue and says `{}`. A candidate added and not measured has \
                 widened the catalogue and moved nothing: ADR 0007 says the grade is a run \
                 somebody made",
                candidate.id, candidate.grade
            ));
        }
        let Some(artefact) = candidate.artefact.as_deref() else {
            return Err(format!(
                "`{}` names no artefact, so the grade beside it was earned against a file the \
                 entry cannot point at — which is rule 4 of {THE_CATALOGUE}",
                candidate.id
            ));
        };
        if candidate.min_ram_gb > WHAT_THE_BOX_HAS {
            return Err(format!(
                "`{}` wants {} GB and the measuring box has {WHAT_THE_BOX_HAS}. A model this box \
                 cannot hold is not a candidate for this task, and a grade beside one would have \
                 come from somewhere else",
                candidate.id, candidate.min_ram_gb
            ));
        }
        for named in [candidate.id.as_str(), artefact, candidate.grade.as_str()] {
            if !entry.contains(named) {
                return Err(format!(
                    "the account does not carry `{named}`. What a candidate is, what it was run \
                     against and what it earned are one statement, and an account missing a part \
                     of it sends the next curator to the catalogue to guess"
                ));
            }
        }
    }
    for left_alone in LEFT_ALONE {
        if !entry.contains(left_alone) {
            return Err(format!(
                "the account does not say that {left_alone} was left alone. A grade that needed \
                 the method moved to exist is a measurement of the move"
            ));
        }
    }
    if entry.len() < AN_ANSWER {
        return Err(
            "the account says almost nothing. This is the floor under one, not a judge of it: \
             what each model wrote goes beside the earlier runs or it is not recorded"
                .to_owned(),
        );
    }
    Ok(())
}

/// **Both candidates were measured, and the account says what they did.**
#[test]
fn the_candidates_were_measured_and_the_account_says_what_they_did() {
    let candidates = off_the_catalogue();
    let entry = the_entry_in(&reading(THE_QUIRKS), THE_RUN);
    assert!(
        !entry.is_empty(),
        "there is no entry under `{THE_RUN}` in {THE_QUIRKS}, so two grades arrived with nothing \
         to say what the models actually wrote"
    );
    if let Err(why) = whether_the_candidates_hold(&candidates, &entry) {
        panic!("{why}");
    }
}

/// **The candidates were chosen for how they were trained, not for their
/// size**, and the account says which training that was.
///
/// The task's own reasoning, as a check: a small model added because it fits is
/// the catalogue getting longer, and the five entries already measured are the
/// evidence that fitting is not the property that matters.
#[test]
fn the_account_says_why_these_two_rather_than_two_more_small_models() {
    let entry = the_entry_in(&reading(THE_QUIRKS), THE_RUN);
    for why in ["tool call", "structured output"] {
        assert!(
            entry.to_lowercase().contains(why),
            "the account does not say `{why}`. These two were chosen because their publishers \
             train them for it, and an account that leaves the reason out reads as two more small \
             models tried on a hunch"
        );
    }
}

/// **Rule 4 is in the catalogue's own rules**, which is where the next curator
/// reads it — the half of the Teuken question that outlives this run.
///
/// `data/catalogue.toml` stated a `quantisation` for every entry, and for
/// Teuken its publisher ships no such artefact at all. The answer chosen is
/// that a quantisation is paired with the artefact it names, or it is not
/// claimed; the loader refuses the half-statement, and the rule is written
/// where a person editing the file will see it rather than in a plan they will
/// never open.
#[test]
fn the_rule_a_quantisation_is_paired_with_its_artefact_is_in_the_catalogue_s_own_rules() {
    let catalogue = reading(THE_CATALOGUE);
    let (rules, _) = catalogue
        .split_once("[[model]]")
        .expect("the catalogue has entries, and its rules come before the first of them");
    for said in ["artefact", "quantisation", "teuken-7b-instruct"] {
        assert!(
            rules.contains(said),
            "the rules at the top of {THE_CATALOGUE} do not mention `{said}`. The next curator \
             reads those and not the lane's plan, so the rule lives there or it does not exist"
        );
    }
    let entries = Catalogue::built_in().expect("the built-in catalogue loads");
    assert!(
        entries
            .models
            .iter()
            .any(|m| m.id == "teuken-7b-instruct" && m.quantised_at().is_none()),
        "`teuken-7b-instruct` still claims a quantisation. Its publisher ships no GGUF, so every \
         Q4_K_M of it is a stranger's requantisation and the entry is the one the rule was \
         written from"
    );
}

/// **And the check catches each way the account can stop being true**, which
/// is the half a green run cannot show anybody.
#[test]
fn the_check_catches_an_account_that_has_stopped_being_true() {
    let candidate = |id: &str, artefact: &str, grade: &str, measured: bool, ram: f32| Candidate {
        id: id.to_owned(),
        artefact: Some(artefact.to_owned()),
        grade: grade.to_owned(),
        measured,
        min_ram_gb: ram,
    };
    let candidates = vec![
        candidate("first-candidate", "runtime:first", "rarely", true, 3.0),
        candidate("second-candidate", "runtime:second", "sometimes", true, 4.0),
    ];
    let sound = format!(
        "**Version:** `runtime:first` and `runtime:second`, trained for tool calls and \
         structured output.\n\
         **Behaviour:** `first-candidate` graded `rarely` and `second-candidate` graded \
         `sometimes`. {}\n\
         **Our response:** {} was left alone, and so were {} and the {}.",
        "a".repeat(AN_ANSWER),
        LEFT_ALONE[0],
        LEFT_ALONE[1],
        LEFT_ALONE[2],
    );
    assert_eq!(
        whether_the_candidates_hold(&candidates, &sound),
        Ok(()),
        "a sound account was refused, so the refusals below say nothing"
    );

    // The same pair with one of them changed, named rather than indexed so
    // that a reordering of the list cannot quietly doctor a different entry.
    let but = |id: &str, change: &dyn Fn(&mut Candidate)| -> Vec<Candidate> {
        candidates
            .iter()
            .cloned()
            .map(|mut it| {
                if it.id == id {
                    change(&mut it);
                }
                it
            })
            .collect()
    };

    // The one this test exists for: a name added to the catalogue with no run
    // behind it.
    let unmeasured = but("second-candidate", &|it| {
        it.measured = false;
        it.grade = "not-measured".to_owned();
    });
    assert!(
        whether_the_candidates_hold(&unmeasured, &sound)
            .is_err_and(|why| why.contains("widened the catalogue and moved nothing")),
        "a candidate that was never measured was accepted"
    );

    // A grade earned against a file the entry cannot point at.
    let unpointed = but("first-candidate", &|it| it.artefact = None);
    assert!(
        whether_the_candidates_hold(&unpointed, &sound)
            .is_err_and(|why| why.contains("names no artefact")),
        "a candidate with no artefact behind its grade was accepted"
    );

    // A model this box could not have held, which is where the grade would
    // have come from somewhere other than this run.
    let too_big = but("second-candidate", &|it| it.min_ram_gb = 10.0);
    assert!(
        whether_the_candidates_hold(&too_big, &sound).is_err_and(|why| why.contains("cannot hold")),
        "a candidate the measuring box cannot load was accepted"
    );

    // A grade the account and the catalogue disagree about.
    let regraded = but("first-candidate", &|it| it.grade = "reliably".to_owned());
    assert!(
        whether_the_candidates_hold(&regraded, &sound)
            .is_err_and(|why| why.contains("does not carry")),
        "a grade the account does not state was accepted"
    );

    // An account about one of the two.
    let partial = sound.replace("second-candidate", "something else");
    assert!(
        whether_the_candidates_hold(&candidates, &partial)
            .is_err_and(|why| why.contains("does not carry")),
        "an account that dropped a candidate was accepted"
    );

    // An account of a method that moved to get a number.
    let loosened = sound.replace(LEFT_ALONE[0], "a shorter prompt");
    assert!(
        whether_the_candidates_hold(&candidates, &loosened)
            .is_err_and(|why| why.contains("measurement of the move")),
        "an account that does not say the method was left alone was accepted"
    );

    // An account too short to be one.
    let brief = "Both were measured.".to_owned();
    assert!(
        whether_the_candidates_hold(&candidates, &brief).is_err(),
        "an account of one line was accepted"
    );

    // And the one that means the check is looking at almost nothing.
    let alone: Vec<Candidate> = candidates.iter().take(1).cloned().collect();
    assert!(
        whether_the_candidates_hold(&alone, &sound).is_err_and(|why| why.contains("at least two")),
        "a single candidate was accepted as two"
    );
}

/// **The entry is the one under that heading and no other**, so a heading that
/// moved fails loudly rather than checking a neighbour's text.
#[test]
fn the_entry_is_the_one_under_that_heading_and_no_other() {
    let document = format!(
        "## Models\n\n{THE_RUN} on the box that could hold them\nthe account\n\n\
         ### A different entry entirely\nnot the account\n"
    );
    assert_eq!(the_entry_in(&document, THE_RUN), "the account\n");
    assert!(
        the_entry_in("### A heading nothing is under\ntext", THE_RUN).is_empty(),
        "the parser found an entry under a heading that is not in the document"
    );
}

/// **The task that comes after this one is in the plan**, so the loop sends the
/// next worker at what this run leaves open rather than at this run again.
#[test]
fn the_task_written_from_this_outcome_is_in_the_plan() {
    let plan = reading(THE_PLAN);
    assert!(
        plan.contains("### 12. ") && plan.contains("**Done, 2026-09-11.**"),
        "{THE_PLAN} does not mark this task done; a task finished and not marked is a task the \
         loop selects again"
    );
    let (_, after) = plan
        .split_once("### 13. ")
        .expect("the plan carries a task 13, written from what this run found");
    assert!(
        after.contains("catalogue"),
        "task 13 is not written from this outcome: what this run leaves open is the catalogue's, \
         and the next worker needs to be told which part"
    );
}
