//! What undoing rewinds to is a decision, and the code waits on it.
//!
//! Task 4 of `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md` asks for
//! ★ *undo what the agent did*, undone *through the mechanism that made it
//! undoable — the base's own snapshot* — and never by an invented inverse. The
//! machine this repository installs is formatted `ext4`, which has no snapshot
//! at all, and every road to one runs through another lane's crate or an
//! accepted decision.
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md) sets
//! the roads out with what each costs and recommends one, and nothing is built
//! until it is answered.
//!
//! # What this file holds
//!
//! The pattern `alo-bounding`'s `the_records_source_is_decided_before_it_is_built.rs`
//! set for ADR 0029, because the failure is the same one: a decision nobody
//! points at is a task a worker takes up by choosing an option themselves.
//!
//! - **The ADR exists, once**, and its status says *proposed* or *accepted*.
//! - **The plan's task 4 names it by filename**, so the next worker lands on
//!   the argument rather than on a task that looks ready.
//! - **It sets out four roads and one recommendation**, and each road says
//!   what it costs a person, the disk and another lane.
//! - **While it says *proposed*, nothing of undoing exists**: the record has no
//!   `Undone` kind and this crate has no file about undoing. The day one
//!   appears with the line unchanged, the code has run ahead of its owner.
//!
//! It reads files and needs no machine, so whether the decision was taken is
//! answerable anywhere this crate builds.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{
    fs,
    path::{Path, PathBuf},
};

/// The decision's number, as `docs/decisions/README.md` says one is cited.
const THE_NUMBER: &str = "0045";

/// The decision's file, relative to the repository.
const THE_DECISION: &str = "docs/decisions/0045-what-undoing-rewinds-to.md";

/// Where every decision lives.
const THE_DECISIONS: &str = "docs/decisions";

/// The plan that asked for the decision, relative to the repository.
const THE_PLAN: &str = "docs/autonomy/v0-5-the-machine-keeps-itself-plan.md";

/// The heading of the task in that plan that produced it.
const THE_TASK: &str = "### 4. Undo what the agent did";

/// Where the kinds of record entry are declared, relative to the repository.
const THE_KINDS: &str = "crates/alo-record/src/happened.rs";

/// The kind an undo would be recorded as, which may not exist yet.
const AN_UNDO_ENTRY: &str = "Undone";

/// Where this crate's decisions live, relative to the repository.
const THIS_CRATE: &str = "crates/alo-keeping-up/src";

/// What a file about undoing would be called.
const ABOUT_UNDOING: &str = "undo";

/// The line a decision records its status on.
const STATUS: &str = "**Status:**";

/// The two statuses under which the decision still stands.
const STANDING: [&str; 2] = ["proposed", "accepted"];

/// The one status under which the code waits.
const WAITING: &str = "proposed";

/// The roads, the name for doing nothing yet, and one recommendation.
const WHAT_IT_SETS_OUT: [&str; 5] = [
    "### Option A",
    "### Option B",
    "### Option C",
    "### Option D",
    "## The recommendation",
];

/// What each road has to say it costs.
const WHAT_EACH_COSTS: [&str; 3] = ["a person", "the disk", "another lane"];

/// Where the repository is, from this crate.
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

/// The names of every file in a directory of this repository, sorted.
fn names_in(directory: &str) -> Vec<String> {
    let at = the_repository().join(directory);
    let mut names: Vec<String> = fs::read_dir(&at)
        .unwrap_or_else(|why| panic!("{} could not be listed: {why}", at.display()))
        .map(|entry| {
            entry
                .expect("an entry of the directory can be read")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort_unstable();
    names
}

/// The decision's status, read off its status line, or why it could not be.
fn status_of(decision: &str) -> Result<String, String> {
    let line = decision
        .lines()
        .find(|line| line.starts_with(STATUS))
        .ok_or_else(|| format!("the decision has no line beginning `{STATUS}`"))?
        .to_lowercase();
    [
        "accepted",
        "proposed",
        "superseded",
        "rejected",
        "withdrawn",
    ]
    .into_iter()
    .find(|status| line.contains(status))
    .map(str::to_owned)
    .ok_or_else(|| format!("the status line says neither what it is nor what it was: `{line}`"))
}

/// Whether exactly one decision carries the number, and it is the one named.
fn the_decision_exists_once(names: &[String]) -> Result<(), String> {
    let carrying: Vec<&String> = names
        .iter()
        .filter(|name| name.starts_with(&format!("{THE_NUMBER}-")))
        .collect();
    let expected = Path::new(THE_DECISION)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("`{THE_DECISION}` has no file name"))?;
    match carrying.as_slice() {
        [one] if *one == expected => Ok(()),
        [one] => Err(format!(
            "the decision numbered {THE_NUMBER} is `{one}`, not `{expected}`, and every link to \
             it still reads as it did"
        )),
        [] => Err(format!(
            "no decision under {THE_DECISIONS} carries the number {THE_NUMBER}"
        )),
        many => Err(format!(
            "{} decisions carry the number {THE_NUMBER}: {many:?}",
            many.len()
        )),
    }
}

/// Whether the decision still stands — proposed, or accepted.
fn the_decision_stands(decision: &str) -> Result<String, String> {
    let status = status_of(decision)?;
    if STANDING.contains(&status.as_str()) {
        Ok(status)
    } else {
        Err(format!(
            "the decision is `{status}`, so it no longer stands and the plan's task 4 has to move"
        ))
    }
}

/// Whether the plan names the decision under the task that produced it.
fn the_plan_points_at_it(plan: &str) -> Result<(), String> {
    let task = plan
        .split(THE_TASK)
        .nth(1)
        .ok_or_else(|| format!("the plan no longer has the heading `{THE_TASK}`"))?;
    let section = task.split("\n### ").next().unwrap_or_default();
    if section.contains(THE_DECISION) {
        Ok(())
    } else {
        Err(format!(
            "task 4 of the plan does not name `{THE_DECISION}`, so a worker reading the plan \
             finds a task and not the decision it waits on"
        ))
    }
}

/// Whether the decision sets out the roads, the recommendation, and under each
/// road what it costs.
fn the_decision_sets_out_what_was_asked(decision: &str) -> Result<(), String> {
    for heading in WHAT_IT_SETS_OUT {
        if !decision.lines().any(|line| line.starts_with(heading)) {
            return Err(format!("the decision has no heading beginning `{heading}`"));
        }
    }
    let options = decision
        .split("\n### Option ")
        .skip(1)
        .map(|option| option.split("\n## ").next().unwrap_or_default());
    for option in options {
        let letter = option.chars().next().unwrap_or_default();
        for cost in WHAT_EACH_COSTS {
            if !option.contains(cost) {
                return Err(format!(
                    "option {letter} does not say what it costs `{cost}`, and a road whose price \
                     is not named is not decidable"
                ));
            }
        }
    }
    Ok(())
}

/// Whether the code is where the decision left it: no entry for an undo and no
/// file about undoing, for as long as the decision is only proposed.
fn the_code_waits_on_it(status: &str, kinds: &str, this_crate: &[String]) -> Result<(), String> {
    if status != WAITING {
        return Ok(());
    }
    if kinds.contains(AN_UNDO_ENTRY) {
        return Err(format!(
            "the record declares `{AN_UNDO_ENTRY}` while ADR {THE_NUMBER} still says `{WAITING}`: \
             the code has run ahead of the decision it was told to wait on"
        ));
    }
    if let Some(file) = this_crate
        .iter()
        .find(|name| name.to_lowercase().contains(ABOUT_UNDOING))
    {
        return Err(format!(
            "`{THIS_CRATE}/{file}` exists while ADR {THE_NUMBER} still says `{WAITING}`: the code \
             has run ahead of the decision it was told to wait on"
        ));
    }
    Ok(())
}

/// The decision exists, once, under the number the plan cites it by.
#[test]
fn the_decision_exists_once_under_its_number() {
    the_decision_exists_once(&names_in(THE_DECISIONS)).unwrap_or_else(|why| panic!("{why}"));
}

/// Its status line says it stands — proposed until the owner answers, accepted
/// after — and nothing else.
#[test]
fn the_decision_stands_as_proposed_or_accepted() {
    let status = the_decision_stands(&reading(THE_DECISION)).unwrap_or_else(|why| panic!("{why}"));
    assert!(STANDING.contains(&status.as_str()), "`{status}`");
}

/// The plan's task 4 names the decision by its filename.
#[test]
fn the_plan_points_at_the_decision() {
    the_plan_points_at_it(&reading(THE_PLAN)).unwrap_or_else(|why| panic!("{why}"));
}

/// The decision carries four roads and a recommendation, and every road names
/// what it costs a person, the disk and another lane.
#[test]
fn the_decision_sets_out_the_roads_their_costs_and_a_recommendation() {
    the_decision_sets_out_what_was_asked(&reading(THE_DECISION))
        .unwrap_or_else(|why| panic!("{why}"));
}

/// While the decision is proposed, the record has no entry for an undo and this
/// crate has no file about undoing.
#[test]
fn the_code_waits_on_the_decision() {
    let status = status_of(&reading(THE_DECISION)).unwrap_or_else(|why| panic!("{why}"));
    the_code_waits_on_it(&status, &reading(THE_KINDS), &names_in(THIS_CRATE))
        .unwrap_or_else(|why| panic!("{why}"));
}

/// **The checks above would notice**, which a green run alone cannot show:
/// each is handed the thing it exists to catch and must refuse it.
#[test]
fn the_checks_would_notice_the_decision_going_missing_or_the_code_running_ahead() {
    let decision = reading(THE_DECISION);
    let plan = reading(THE_PLAN);
    let kinds = reading(THE_KINDS);
    let this_crate = names_in(THIS_CRATE);

    // Two files claiming the number, none, and a renamed one.
    let mut twice = names_in(THE_DECISIONS);
    twice.push(format!("{THE_NUMBER}-a-second-file-claiming-the-number.md"));
    assert!(
        the_decision_exists_once(&twice).is_err(),
        "two decisions carrying one number were not refused"
    );
    assert!(
        the_decision_exists_once(&[]).is_err(),
        "an empty decisions directory was not refused"
    );
    assert!(
        the_decision_exists_once(&[format!("{THE_NUMBER}-a-different-name.md")]).is_err(),
        "a decision renamed away from what the plan cites was not refused"
    );

    // A status that no longer stands, one that names nothing, and no status line.
    // Rewritten on its status line, rather than at the first `proposed`
    // anywhere in the file: once the decision was accepted the first one was a
    // word in the body, and this check was handed a decision that still stood
    // and would have passed for the wrong reason.
    let withdrawn = decision
        .lines()
        .map(|line| {
            if line.starts_with(STATUS) {
                format!("{STATUS} withdrawn")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        the_decision_stands(&withdrawn).is_err(),
        "a withdrawn decision was read as standing"
    );
    let pending = decision
        .lines()
        .map(|line| {
            if line.starts_with(STATUS) {
                format!("{STATUS} pending")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        status_of(&pending).is_err(),
        "a status line naming none of the five statuses was read as one"
    );
    assert!(
        status_of(&decision.replacen(STATUS, "**State:**", 1)).is_err(),
        "a decision with no status line was read as having one"
    );

    // A plan whose task 4 no longer names the decision, and one without the task.
    assert!(
        the_plan_points_at_it(&plan.replace(THE_DECISION, "docs/decisions/")).is_err(),
        "a plan whose task does not name the decision was not refused"
    );
    assert!(
        the_plan_points_at_it(&plan.replace(THE_TASK, "### 4. Something else")).is_err(),
        "a plan without the task was not refused"
    );

    // A decision missing a road, and a road missing a cost.
    assert!(
        the_decision_sets_out_what_was_asked(&decision.replace("### Option D", "### Road D"))
            .is_err(),
        "a decision without the fourth road was not refused"
    );
    assert!(
        the_decision_sets_out_what_was_asked(&decision.replace("another lane", "a neighbour"))
            .is_err(),
        "a road that does not say what it costs another lane was not refused"
    );

    // An undo entry, or a file about undoing, while the decision is proposed —
    // and the same once it is accepted, which is then the building's to prove.
    let with_an_entry = format!("{kinds}\n    {AN_UNDO_ENTRY} {{ }},\n");
    assert!(
        the_code_waits_on_it(WAITING, &with_an_entry, &this_crate).is_err(),
        "an undo entry beside a proposed decision was not refused"
    );
    let mut with_a_file = this_crate.clone();
    with_a_file.push("undoing.rs".to_owned());
    assert!(
        the_code_waits_on_it(WAITING, &kinds, &with_a_file).is_err(),
        "a file about undoing beside a proposed decision was not refused"
    );
    assert!(
        the_code_waits_on_it("accepted", &with_an_entry, &with_a_file).is_ok(),
        "building beside an accepted decision was refused, which would forbid building it"
    );
}
