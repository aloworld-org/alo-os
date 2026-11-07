//! The record's source is a decision, and the code waits on it.
//!
//! [ADR 0013](../../../docs/decisions/0013-the-grant-is-enforced-by-the-kernel.md)
//! and [ADR 0015](../../../docs/decisions/0015-the-kernel-learns-what-a-turn-is.md)
//! promise that what a turn touched becomes what the kernel watched it touch.
//! ADR 0015 also says the programme *decides and forgets*, and
//! `the_boundary_decides_and_forgets.rs` counts its maps on a running kernel
//! and calls a third one the finding. Those two cannot both be built as
//! written, and
//! [ADR 0029](../../../docs/decisions/0029-what-the-kernel-writes-down-about-a-turn.md)
//! is where the choice between them is made decidable: options, costs, a
//! recommendation — and no code.
//!
//! # What this file holds, and why a decision needs a test at all
//!
//! A decision nobody wrote is a task a worker will take up by choosing one of
//! the options themselves, which is the failure ADR 0024 and ADR 0025 were
//! written to avoid. A decision somebody wrote and nobody points at is the
//! same failure a week later. And a decision still marked *proposed* beside a
//! programme that has already grown the map it proposes is the plan getting
//! ahead of its owner. So four things are read off this repository's own
//! files:
//!
//! - **The ADR exists, once.** One file under `docs/decisions/` carries the
//!   number, and its status line says *proposed* or *accepted* — anything
//!   else means the decision no longer stands and the plan has to move.
//! - **The plan points at it**, under the task that produced it, by filename,
//!   so the next worker reading the plan lands on the argument rather than on
//!   a row saying *needs a decision*.
//! - **It sets out what the plan asked for**: the three shapes the task named
//!   and the fourth that is the honest name for doing nothing, the costs to a
//!   person reading, to the record file and to the loader, and one
//!   recommendation.
//! - **While it says *proposed*, the programme has two maps** — beside one
//!   more the owner allowed by name in ADR 0148, `COUNTED`, which holds nothing
//!   about a turn and is set aside here exactly once. The day another appears
//!   with this line unchanged, the code has run ahead of the decision, and this
//!   is the test that says so — the refusal path of an ADR.
//!
//! # It needs no kernel
//!
//! Nothing here loads a programme or makes a control group: it reads files.
//! That is deliberate — whether the decision has been taken has to be
//! checkable on a machine that cannot run the boundary at all, which is most
//! of them.

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
const THE_NUMBER: &str = "0029";

/// The decision's file, relative to the repository.
const THE_DECISION: &str = "docs/decisions/0029-what-the-kernel-writes-down-about-a-turn.md";

/// Where every decision lives.
const THE_DECISIONS: &str = "docs/decisions";

/// The plan that asked for the decision, relative to the repository.
const THE_PLAN: &str = "docs/autonomy/kernel-enforcement-plan.md";

/// The heading of the task in that plan that produced it.
const THE_TASK: &str = "### 16. Kernel-sourced enforcement records — the decision";

/// Where the hooks and the maps are declared, relative to the repository.
const THE_PROGRAMME: &str = "crates/alo-bounding-kernel/src/kernel.rs";

/// How a map is declared on the programme.
const A_MAP: &str = "#[map(";

/// **The one map this check does not count, by name**: the counter of what an
/// application sends during a person's command, which the owner allowed in
/// ADR 0148. It is not ADR 0029's map — that decision asks what the kernel
/// writes down *about a turn*, and this map holds nothing about any turn — so
/// it is set aside here, exactly once, and only while ADR 0148 stands accepted.
const THE_COUNTER: &str = "#[map(name = \"COUNTED\")]";

/// The decision that allows [`THE_COUNTER`], relative to the repository.
const THE_COUNTERS_DECISION: &str =
    "docs/decisions/0148-the-kernel-counts-what-an-application-sends-during-a-persons-command.md";

/// How many maps the programme has while the decision is still proposed: the
/// two the daemon and the loader fill, and no third.
const THE_TWO_MAPS: usize = 2;

/// The line a decision records its status on.
const STATUS: &str = "**Status:**";

/// The two statuses under which the decision still stands. *Accepted* is
/// how ADR 0024 and ADR 0025 record it; *proposed* is how this one does until
/// the owner answers.
const STANDING: [&str; 2] = ["proposed", "accepted"];

/// The one status under which the programme is held to two maps.
const WAITING: &str = "proposed";

/// The headings the plan's acceptance asks the decision to carry: the three
/// shapes it named, the fourth that is the name for doing nothing, and one
/// recommendation.
const WHAT_IT_SETS_OUT: [&str; 5] = [
    "### Option A",
    "### Option B",
    "### Option C",
    "### Option D",
    "## The recommendation",
];

/// The three costs the plan asks each option to name, as a phrase each
/// option's text must carry: a person reading, the record file's contract,
/// and the loader.
const WHAT_EACH_COSTS: [&str; 3] = ["a person", "record file", "loader"];

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

/// The decision's status, read off its status line, or why it could not be.
///
/// The first line beginning with the marker, lowercased, and the first of the
/// known statuses it contains — so `**Status:** **ACCEPTED, 2026-09-11 —
/// Option D**` reads as *accepted*, the way `alo-citing` reads it.
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
            "the decision numbered {THE_NUMBER} is `{one}`, not `{expected}`: a rename leaves every \
             link to it reading as it did"
        )),
        [] => Err(format!(
            "no decision under {THE_DECISIONS} carries the number {THE_NUMBER}"
        )),
        many => Err(format!(
            "{} decisions carry the number {THE_NUMBER}, and a citation of it would resolve to \
             whichever a reader opened first: {many:?}",
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
            "the decision is `{status}`, so it no longer stands and the plan's task 16 has to \
             move: a row saying the decision is made, pointing at one that was withdrawn, is the \
             same gap it was written to close"
        ))
    }
}

/// Whether the plan points at the decision under the task that produced it.
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
            "task 16 of the plan does not name `{THE_DECISION}`, so a worker reading the plan \
             finds a task and not the decision it produced"
        ))
    }
}

/// Whether the decision sets out what the plan asked for: the shapes, the
/// recommendation, and under each option the three costs.
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
                    "option {letter} does not say what it costs `{cost}`, and an option whose \
                     price is not named is not decidable"
                ));
            }
        }
    }
    Ok(())
}

/// How many maps the programme declares.
fn maps_on(programme: &str) -> usize {
    programme.matches(A_MAP).count()
}

/// How many of those this decision is waiting on: every map, less
/// [`THE_COUNTER`] — **once, by its exact declaration, and only while ADR 0148
/// is accepted**. A second `COUNTED`, a counter under another name, or a
/// counter beside an ADR 0148 that is no longer accepted is counted like any
/// other map.
fn maps_waiting_on_it(programme: &str, counters_status: &str) -> usize {
    let every = maps_on(programme);
    if counters_status == "accepted" && programme.matches(THE_COUNTER).count() == 1 {
        every.saturating_sub(1)
    } else {
        every
    }
}

/// Whether the programme is still where the decision left it: two maps, and
/// the one counter ADR 0148 names, for as long as the decision is only proposed.
fn the_code_waits_on_it(status: &str, counters_status: &str, programme: &str) -> Result<(), String> {
    let maps = maps_waiting_on_it(programme, counters_status);
    if status == WAITING && maps != THE_TWO_MAPS {
        return Err(format!(
            "the programme declares {maps} maps besides ADR 0148's counter while ADR {THE_NUMBER} \
             still says `{WAITING}`: the code has run ahead of the decision it was told to wait on"
        ));
    }
    Ok(())
}

/// The names of every file under `docs/decisions/`.
fn every_decision() -> Vec<String> {
    let at = the_repository().join(THE_DECISIONS);
    let mut names: Vec<String> = fs::read_dir(&at)
        .unwrap_or_else(|why| panic!("{} could not be listed: {why}", at.display()))
        .map(|entry| {
            entry
                .expect("an entry of the decisions directory can be read")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort_unstable();
    names
}

/// The decision exists, once, under the number the plan cites it by.
#[test]
fn the_decision_exists_once_under_its_number() {
    the_decision_exists_once(&every_decision()).unwrap_or_else(|why| panic!("{why}"));
}

/// Its status line says it stands — proposed until the owner answers, accepted
/// after — and nothing else.
#[test]
fn the_decision_stands_as_proposed_or_accepted() {
    let status = the_decision_stands(&reading(THE_DECISION)).unwrap_or_else(|why| panic!("{why}"));
    assert!(
        STANDING.contains(&status.as_str()),
        "`{status}` is not a status under which the decision stands"
    );
}

/// The plan's task 16 names the decision by its filename.
#[test]
fn the_plan_points_at_the_decision() {
    the_plan_points_at_it(&reading(THE_PLAN)).unwrap_or_else(|why| panic!("{why}"));
}

/// The decision carries the four options and the recommendation, and every
/// option names what it costs a person reading, the record file and the
/// loader.
#[test]
fn the_decision_sets_out_the_options_their_costs_and_a_recommendation() {
    the_decision_sets_out_what_was_asked(&reading(THE_DECISION))
        .unwrap_or_else(|why| panic!("{why}"));
}

/// While the decision is proposed, the programme has the two maps it had when
/// the decision was written, and no third — beside the one counter ADR 0148
/// allows by name.
#[test]
fn the_code_waits_on_the_decision() {
    let status = status_of(&reading(THE_DECISION)).unwrap_or_else(|why| panic!("{why}"));
    let counters = status_of(&reading(THE_COUNTERS_DECISION)).unwrap_or_else(|why| panic!("{why}"));
    the_code_waits_on_it(&status, &counters, &reading(THE_PROGRAMME))
        .unwrap_or_else(|why| panic!("{why}"));
}

/// **The checks above would notice**, which is the half a green run cannot
/// tell you: each is handed the thing it exists to catch and must refuse it.
#[test]
fn the_checks_would_notice_the_decision_going_missing_or_the_code_running_ahead() {
    let decision = reading(THE_DECISION);
    let plan = reading(THE_PLAN);
    let programme = reading(THE_PROGRAMME);

    // Two files claiming the number, and none.
    let mut twice = every_decision();
    twice.push(format!("{THE_NUMBER}-a-second-file-claiming-the-number.md"));
    assert!(
        the_decision_exists_once(&twice).is_err(),
        "two decisions carrying one number were not refused"
    );
    assert!(
        the_decision_exists_once(&[]).is_err(),
        "an empty decisions directory was not refused"
    );
    let renamed = vec![format!("{THE_NUMBER}-a-different-name.md")];
    assert!(
        the_decision_exists_once(&renamed).is_err(),
        "a decision renamed away from what the plan cites was not refused"
    );

    // A status that no longer stands, and a status line that says nothing.
    let withdrawn = decision.replacen("proposed", "withdrawn", 1);
    assert!(
        the_decision_stands(&withdrawn).is_err(),
        "a withdrawn decision was read as standing"
    );
    let unsaid = decision.replacen("proposed", "pending", 1);
    assert!(
        status_of(&unsaid).is_err(),
        "a status line naming none of the five statuses was read as one"
    );
    let unlined = decision.replacen(STATUS, "**State:**", 1);
    assert!(
        status_of(&unlined).is_err(),
        "a decision with no status line was read as having one"
    );

    // A plan whose task 16 no longer names the decision, and one without the task.
    let unpointed = plan.replace(THE_DECISION, "docs/decisions/");
    assert!(
        the_plan_points_at_it(&unpointed).is_err(),
        "a plan whose task does not name the decision was not refused"
    );
    let untasked = plan.replace(THE_TASK, "### 16. Something else");
    assert!(
        the_plan_points_at_it(&untasked).is_err(),
        "a plan without the task was not refused"
    );

    // A decision missing one of the shapes, and an option missing one cost.
    let without_c = decision.replace("### Option C", "### Shape C");
    assert!(
        the_decision_sets_out_what_was_asked(&without_c).is_err(),
        "a decision without the third shape was not refused"
    );
    let without_a_cost = decision.replace("loader", "installer");
    assert!(
        the_decision_sets_out_what_was_asked(&without_a_cost).is_err(),
        "an option that does not name what it costs the loader was not refused"
    );

    // A third map while the decision is proposed — and the same map once it
    // is accepted, which is then the implementation's to prove and not this
    // file's to forbid.
    let with_a_third = format!("{programme}\n{A_MAP}name = \"WATCHED\")]\n");
    assert_eq!(
        maps_on(&with_a_third),
        THE_TWO_MAPS + 2,
        "the count of maps did not move when a map was added, so the check reads nothing"
    );
    assert!(
        the_code_waits_on_it(WAITING, "accepted", &with_a_third).is_err(),
        "a third map beside a proposed decision was not refused"
    );
    assert!(
        the_code_waits_on_it("accepted", "accepted", &with_a_third).is_ok(),
        "a third map beside an accepted decision was refused, which would forbid building it"
    );

    // **ADR 0148's counter is set aside by name, once, and only while that
    // decision is accepted** — each of the three ways round it is refused.
    assert!(
        the_code_waits_on_it(WAITING, "proposed", &programme).is_err(),
        "the counter was set aside beside an ADR 0148 that is not accepted"
    );
    let a_second_counter = format!("{programme}\n{THE_COUNTER}\n");
    assert!(
        the_code_waits_on_it(WAITING, "accepted", &a_second_counter).is_err(),
        "a second map named COUNTED was set aside as though it were the one ADR 0148 allows"
    );
    let renamed = programme.replace(THE_COUNTER, "#[map(name = \"SENT\")]");
    assert!(
        the_code_waits_on_it(WAITING, "accepted", &renamed).is_err(),
        "a counter under a name ADR 0148 does not give was set aside"
    );
    assert_eq!(
        maps_waiting_on_it(&programme, "accepted"),
        THE_TWO_MAPS,
        "the programme as it is does not have the two maps the decision was written against, \
         beside ADR 0148's counter"
    );
    assert_eq!(
        programme.matches(THE_COUNTER).count(),
        1,
        "the programme does not declare ADR 0148's counter exactly once"
    );
    assert_eq!(
        status_of(&reading(THE_COUNTERS_DECISION)).as_deref(),
        Ok("accepted"),
        "the counter is in the programme and the decision that allows it is not accepted"
    );
}
