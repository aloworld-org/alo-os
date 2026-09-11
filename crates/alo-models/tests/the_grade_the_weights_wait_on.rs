//! The grade the weights wait on, and the finding that it was not earned here.
//!
//! The lane's plan asked for three 7B-class entries to be measured with
//! `alo-driving` against the pinned runtime, on the reasoning that the machine
//! had room for them. It has not: the runtime and every grade in
//! `data/catalogue.toml` live inside a 6 GB WSL guest, the model loads more
//! slowly than `alo_models` waits for an answer, and the guest went down under
//! the attempt. So the three still say `not-measured`, and **that is a finding
//! rather than an omission** — it was tried, with numbers, and
//! `docs/quirks.md` carries them.
//!
//! A finding written in prose rots in one direction: the day somebody measures
//! one of the three, nothing about that run rewrites the paragraph that says
//! none of them has been measured, and a reader who trusts it plans around a
//! gap that has closed. So the entry is parsed rather than admired, and held to
//! three things:
//!
//! - **The three are the catalogue's**, by the ids the catalogue uses, and each
//!   of them is still `not-measured`. The day one is not, this fails and sends
//!   whoever sees it back to the entry to write what was measured.
//! - **The finding names all three**, because a finding about some of them
//!   reads as one about all of them — the same rule
//!   `the_carry_or_fetch_measurement.rs` holds its table to.
//! - **It says what the box could not do, with the numbers**, so that *it did
//!   not work* cannot stand in for a measurement of why.
//!
//! And one rule that is not about this run at all, found while looking for the
//! weights: **no entry whose licence permits commercial use may name a release
//! its publisher licensed for research.** openGPT-X publishes Teuken twice
//! under two licences, and this catalogue named the research release while
//! stating the commercial licence. That is the catalogue's own first rule being
//! broken by the catalogue, and one line of it is worth more than the paragraph
//! about it in `docs/quirks.md`.
//!
//! Like its sibling it needs no model and no socket: it reads this
//! repository's own files and the catalogue compiled into this crate, which is
//! the point — the finding is about a machine that cannot run a model, and a
//! check that needed one could never run there either.

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

/// The heading of the entry in `docs/quirks.md` that carries the finding.
const THE_FINDING: &str = "### A 7B-class entry cannot be measured on the box";

/// Where findings about models are written.
const THE_QUIRKS: &str = "docs/quirks.md";

/// The plan this task belongs to.
const THE_PLAN: &str = "docs/autonomy/v0-01-lane-b-plan.md";

/// The three entries the plan named, by the ids the catalogue uses.
const THE_THREE: [&str; 3] = [
    "teuken-7b-instruct",
    "mistral-7b-instruct",
    "qwen2.5-7b-instruct",
];

/// How a catalogue entry spells a grade nobody has run.
const UNMEASURED: &str = "not-measured";

/// What the box could not do, in the numbers the run produced. A finding that
/// kept the sentence and dropped these would be an opinion about a machine.
const THE_NUMBERS: [&str; 5] = [
    "Ollama 0.33.3",
    "5,926 MB",
    "715",
    "0.25 tokens per second",
    "five minutes",
];

/// How much of an answer the finding has to be, in bytes.
///
/// Not a judge of one — nothing mechanical can be — only the floor beneath it,
/// so that *we tried and it was slow* cannot pass for a measurement of why.
const AN_ANSWER: usize = 600;

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

/// What the catalogue says about the three, as a grade each.
fn what_the_catalogue_says() -> Vec<(String, Driving)> {
    let catalogue = Catalogue::built_in().expect("the built-in catalogue loads");
    THE_THREE
        .iter()
        .map(|id| {
            let model = catalogue.get(id).unwrap_or_else(|| {
                panic!(
                    "the catalogue has no `{id}`, and the grade the weights wait on is that \
                     entry's. If it was renamed, this test and the finding in {THE_QUIRKS} move \
                     with it"
                )
            });
            (model.id.clone(), model.drives_verbs)
        })
        .collect()
}

/// Whether the finding still says what is true, against the catalogue as it is
/// now.
///
/// Separate from the files it reads so that every refusal below can be shown
/// happening: a documentation check nobody has seen fail is one nobody should
/// believe.
///
/// # Errors
/// A sentence naming what stopped being true and where to go about it.
fn whether_the_finding_still_holds(said: &[(String, Driving)], entry: &str) -> Result<(), String> {
    if said.is_empty() {
        return Err("nothing was compared against the finding at all".to_owned());
    }
    for (id, grade) in said {
        if *grade != Driving::NotMeasured {
            return Err(format!(
                "`{id}` is now graded and the finding still says the three are unmeasured. A \
                 grade is a run somebody made: write which machine made it, and what it earned, \
                 rather than leaving this entry standing"
            ));
        }
        if !entry.contains(id) {
            return Err(format!(
                "the finding does not name `{id}`. It is one of the three the grade waits on, \
                 and a finding about some of them reads as one about all of them"
            ));
        }
    }
    if !entry.contains(UNMEASURED) {
        return Err(format!(
            "the finding does not say the three stay `{UNMEASURED}`, which is the whole of what \
             the run concluded — a machine without the memory for a run does not guess"
        ));
    }
    if entry.len() < AN_ANSWER {
        return Err(
            "the finding says almost nothing. This is the floor under an answer, not a judge of \
             one: what the box could not do is stated with its numbers or it is not stated"
                .to_owned(),
        );
    }
    for number in THE_NUMBERS {
        if !entry.contains(number) {
            return Err(format!(
                "the finding does not carry `{number}`. Without the numbers it is an opinion \
                 about a machine, and the next worker cannot tell whether their own box is \
                 different"
            ));
        }
    }
    Ok(())
}

/// **The three entries the grade waits on are still `not-measured`, and the
/// finding says so with the numbers behind it.**
#[test]
fn the_three_entries_the_grade_waits_on_are_unmeasured_and_the_finding_says_why() {
    let said = what_the_catalogue_says();
    let entry = the_entry_in(&reading(THE_QUIRKS), THE_FINDING);
    assert!(
        !entry.is_empty(),
        "there is no entry under `{THE_FINDING}` in {THE_QUIRKS}, so a run that was made and \
         produced no grade is indistinguishable from a run nobody attempted"
    );
    if let Err(why) = whether_the_finding_still_holds(&said, &entry) {
        panic!("{why}");
    }
}

/// **Nothing was loosened to get a number out of a box that could not carry
/// the run**, and the finding says which three things were left alone.
///
/// The constraint the task was given, as a check rather than as a promise: the
/// prompt and the scoring are `alo-driving`'s, and the wait is `alo-models`'.
/// A worker who moved any of them would have measured the loosening.
#[test]
fn the_finding_says_what_was_left_alone_rather_than_what_was_adjusted() {
    let entry = the_entry_in(&reading(THE_QUIRKS), THE_FINDING);
    for left_alone in ["the prompt", "the scoring", "five-minute wait"] {
        assert!(
            entry.contains(left_alone),
            "the finding does not say that {left_alone} was left alone. A grade that needed the \
             method moved to exist is a measurement of the move"
        );
    }
    assert!(
        the_entry_in(&reading(THE_QUIRKS), THE_FINDING).contains("WHILE_A_MODEL_THINKS"),
        "the finding does not name the constant that stopped the run, so nobody reading it can \
         check the arithmetic against the code"
    );
}

/// **No entry that permits commercial use names a release its publisher
/// licensed for research.**
///
/// Found on `teuken-7b-instruct`, whose `upstream` was openGPT-X's
/// `-research-v0.4` while its licence line said Apache-2.0 with commercial use
/// permitted — two releases of one model, licensed differently, and the
/// catalogue naming the wrong one. `data/catalogue.toml`'s first rule calls a
/// licence stated wrongly worse than a model omitted, and this is that rule
/// with teeth.
///
/// Deliberately a test over the catalogue rather than a refusal in the loader:
/// a publisher may legitimately have `research` in its name, and a loader that
/// refused those would be a rule about spelling. Here, where a person curates
/// twelve entries, it is a question worth being asked about every one of them.
#[test]
fn a_licence_that_permits_commercial_use_never_names_a_research_release() {
    let catalogue = Catalogue::built_in().expect("the built-in catalogue loads");
    for model in &catalogue.models {
        if !model.safe_default_for_business() {
            continue;
        }
        let names_research = model
            .upstream
            .rsplit('/')
            .next()
            .is_some_and(|release| release.to_lowercase().contains("research"));
        assert!(
            !names_research,
            "`{}` says commercial use is permitted and names {}, which is a release licensed for \
             research. Name the release the licence is true of, or state the licence the release \
             really carries",
            model.id, model.upstream
        );
    }
}

/// **The task written from this outcome is in the plan**, so the next worker
/// is sent at what the finding actually leaves open rather than at the run
/// again.
#[test]
fn the_task_written_from_this_outcome_is_in_the_plan() {
    let plan = reading(THE_PLAN);
    assert!(
        plan.contains("### 9. ") && plan.contains("**Done, 2026-09-11.**"),
        "{THE_PLAN} does not mark the grade task done; a task finished and not marked is a task \
         the loop selects again"
    );
    let (_, after) = plan
        .split_once("### 12. ")
        .expect("the plan carries a task 12, written from what this run found");
    assert!(
        after.contains("catalogue") && after.contains("alo-driving"),
        "task 12 is not written from this outcome: nothing cleared the bar and nothing could be \
         measured here, so what comes next is candidates this box can hold, measured the same way"
    );
}

/// **And the check catches each way the finding can stop being true**, which
/// is the half a green run cannot show anybody.
#[test]
fn the_check_catches_a_finding_that_has_stopped_being_true() {
    let said = |grade: Driving| -> Vec<(String, Driving)> {
        THE_THREE
            .iter()
            .map(|id| ((*id).to_owned(), grade))
            .collect()
    };
    let unmeasured = said(Driving::NotMeasured);
    let sound = format!(
        "**Version:** {}, a guest of 5,926 MB. {}\n\
         **Behaviour:** the load outlasts {}; 715 tokens of prompt and \
         0.25 tokens per second of answer. {}\n\
         **Our response:** all three stay `{UNMEASURED}`. {}",
        THE_NUMBERS[0],
        THE_THREE.join(", "),
        THE_NUMBERS[4],
        "a".repeat(AN_ANSWER / 2),
        "b".repeat(AN_ANSWER / 2),
    );
    assert_eq!(
        whether_the_finding_still_holds(&unmeasured, &sound),
        Ok(()),
        "a sound finding was refused, so the refusals below say nothing"
    );

    // The one this test exists for: a grade arrives and the finding still
    // says nobody has one.
    let graded: Vec<(String, Driving)> = unmeasured
        .iter()
        .map(|(id, grade)| {
            let earned = if id == THE_THREE[1] {
                Driving::Rarely
            } else {
                *grade
            };
            (id.clone(), earned)
        })
        .collect();
    assert!(
        whether_the_finding_still_holds(&graded, &sound)
            .is_err_and(|why| why.contains("now graded")),
        "a catalogue with a measured entry was accepted against a finding that says none is"
    );

    // A finding about two of the three.
    let partial = sound.replace(THE_THREE[2], "something else");
    assert!(
        whether_the_finding_still_holds(&unmeasured, &partial)
            .is_err_and(|why| why.contains("does not name")),
        "a finding that dropped one of the three was accepted"
    );

    // A finding that never says what the three are left at.
    let silent = sound.replace(UNMEASURED, "fine for now");
    assert!(
        whether_the_finding_still_holds(&unmeasured, &silent)
            .is_err_and(|why| why.contains("does not guess")),
        "a finding with no conclusion in it was accepted"
    );

    // A finding with the sentence and none of the numbers.
    let shrugged = sound.replace("715", "some");
    assert!(
        whether_the_finding_still_holds(&unmeasured, &shrugged)
            .is_err_and(|why| why.contains("opinion about a machine")),
        "a finding that dropped a number was accepted"
    );

    // A finding too short to be one.
    let brief = format!(
        "{} did not run. All three stay `{UNMEASURED}`.",
        THE_THREE.join(", ")
    );
    assert!(
        whether_the_finding_still_holds(&unmeasured, &brief)
            .is_err_and(|why| why.contains("floor under an answer")),
        "a finding of one line was accepted"
    );

    // And the one that means the check is looking at nothing.
    assert!(
        whether_the_finding_still_holds(&[], &sound)
            .is_err_and(|why| why.contains("nothing was compared")),
        "an empty comparison was accepted as one"
    );
}

/// **The entry is the one under that heading and no other**, so a heading that
/// moved fails loudly rather than checking a neighbour's text.
#[test]
fn the_entry_is_the_one_under_that_heading_and_no_other() {
    let document = format!(
        "## Models\n\n{THE_FINDING} every grade here was made on\nthe finding\n\n\
         ### A different entry entirely\nnot the finding\n"
    );
    assert_eq!(the_entry_in(&document, THE_FINDING), "the finding\n");
    assert!(
        the_entry_in("### A heading nothing is under\ntext", THE_FINDING).is_empty(),
        "the parser found an entry under a heading that is not in the document"
    );
}
