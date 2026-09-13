//! How a mapping is reproduced is a decision, and the hook waits on it.
//!
//! Nine rows of `docs/autonomy/kernel-enforcement-plan.md`'s filesystem table
//! are closed. One is not: a file mapped into memory is read by the processor
//! and not by a syscall, so one `mmap` of a descriptor opened before the turn
//! began reaches a file's contents past every hook this workstream has
//! attached. `mmap_file` is the hook that would decide it.
//!
//! It is not written, and the reason is not that nobody got to it. This suite
//! **reproduces a gap before it closes one** — that is what has made the nine
//! closed rows believable rather than asserted — and there is no safe spelling
//! of `mmap` in Rust: the standard library has none, and `rustix::mm::mmap`
//! and `memmap2::Mmap::map` are both `unsafe fn` at the call site, which
//! `unsafe_code = "forbid"` refuses in a test as firmly as anywhere. A worker
//! may not lift that rule, so
//! [ADR 0030](../../../docs/decisions/0030-how-a-mapping-is-reproduced.md) is
//! where the choice is made decidable: one audited `unsafe` in a named
//! fixture, the hook measured once by hand, or a reproduction driven through a
//! pinned component — with what each costs, and one recommended.
//!
//! # What this file holds
//!
//! The same four things `the_records_source_is_decided_before_it_is_built.rs`
//! holds for ADR 0029, because the failure they guard against is the same one:
//! a decision nobody wrote is a task a worker takes up by choosing an option
//! themselves; a decision nobody points at is that failure a week later; and a
//! decision still marked *proposed* beside a hook that has already been
//! written is the plan getting ahead of its owner.
//!
//! - **The ADR exists, once**, under its number, and its status line says
//!   *proposed* or *accepted*.
//! - **The plan points at it** — both the task that produced it and the
//!   section-3 row it is about — by filename, so a reader lands on the
//!   argument rather than on a row saying *not reproduced*.
//! - **It sets out what the task asked for**: three options, what each costs,
//!   and one recommendation.
//! - **While it says *proposed*, `mmap_file` is not on the programme.** The
//!   day that hook appears with this line unchanged, the code has run ahead of
//!   the decision, and this is the test that says so.
//!
//! # It needs no kernel
//!
//! Nothing here loads a programme or makes a control group: it reads files, so
//! whether the decision has been taken is checkable on a machine that cannot
//! run the boundary at all.

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
const THE_NUMBER: &str = "0030";

/// The decision's file, relative to the repository.
const THE_DECISION: &str = "docs/decisions/0030-how-a-mapping-is-reproduced.md";

/// Where every decision lives.
const THE_DECISIONS: &str = "docs/decisions";

/// The plan that asked for the decision, relative to the repository.
const THE_PLAN: &str = "docs/autonomy/kernel-enforcement-plan.md";

/// The heading of the task in that plan that produced it.
const THE_TASK: &str = "### 21. How a mapping is reproduced, decided";

/// Where the hooks are declared, relative to the repository.
const THE_PROGRAMME: &str = "crates/alo-bounding-kernel/src/kernel.rs";

/// How the hook would be declared on the programme, the day it is written.
const THE_HOOK: &str = r#"#[lsm(hook = "mmap_file")]"#;

/// The line a decision records its status on.
const STATUS: &str = "**Status:**";

/// The two statuses under which the decision still stands.
const STANDING: [&str; 2] = ["proposed", "accepted"];

/// The one status under which the hook is held to being unwritten.
const WAITING: &str = "proposed";

/// The headings the task's acceptance asks the decision to carry: the three
/// shapes it named, and one recommendation.
const WHAT_IT_SETS_OUT: [&str; 4] = ["### A — ", "### B — ", "### C — ", "## The recommendation"];

/// What the task asks the decision to say each option costs, as a phrase its
/// text must carry: the laws it touches, and the gate.
const WHAT_EACH_COSTS: [&str; 2] = ["What it costs", "gate"];

/// The sentence the task insists the decision carries about what it must
/// **not** decide, because a hook that walked an anonymous mapping would stop
/// every process in a turn's control group.
const WHAT_IT_MUST_NOT_DECIDE: &str = "no file behind it";

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

/// Every decision's file name.
fn every_decision() -> Vec<String> {
    let at = the_repository().join(THE_DECISIONS);
    fs::read_dir(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect()
}

/// The decision's status, read off its status line.
///
/// The first line beginning with the marker, lowercased, and the first known
/// status it contains — the way `alo-citing` reads one.
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

/// **The decision exists, once, under its number, and still stands.**
///
/// Two files carrying one number is not a tidiness complaint: every citation
/// of it resolves to whichever a reader opens first, and both look like the
/// answer. It happened twice on 2026-09-11, once to a plan's tasks and once to
/// `docs/decisions/` itself.
#[test]
fn the_decision_exists_once_under_its_number() {
    let names = every_decision();
    let carrying: Vec<&String> = names
        .iter()
        .filter(|name| name.starts_with(&format!("{THE_NUMBER}-")))
        .collect();
    let expected = Path::new(THE_DECISION)
        .file_name()
        .and_then(|name| name.to_str())
        .expect("the decision's path has a file name");

    match carrying.as_slice() {
        [one] => assert_eq!(
            one.as_str(),
            expected,
            "the decision numbered {THE_NUMBER} is `{one}`, not the file this test and the plan \
             name; a rename leaves every link to it reading exactly as it did"
        ),
        [] => panic!("no decision under {THE_DECISIONS} carries the number {THE_NUMBER}"),
        many => panic!(
            "{} decisions carry the number {THE_NUMBER}, and a citation of it would resolve to \
             whichever a reader opened first: {many:?}",
            many.len()
        ),
    }

    let status = status_of(&reading(THE_DECISION)).expect("the decision has a readable status");
    assert!(
        STANDING.contains(&status.as_str()),
        "the decision's status is `{status}`, so it no longer stands and the plan's mapping row \
         is waiting on an answer that has been withdrawn"
    );
}

/// **The plan points at it, from both places a reader arrives from** — the
/// task that produced it, and the section-3 row it is about.
#[test]
fn the_plan_points_at_the_decision_from_the_task_and_from_the_row() {
    let plan = reading(THE_PLAN);
    let named = Path::new(THE_DECISION)
        .file_name()
        .and_then(|name| name.to_str())
        .expect("the decision's path has a file name");

    let after_the_task = plan
        .split_once(THE_TASK)
        .map(|(_, rest)| rest)
        .unwrap_or_else(|| panic!("the plan has no task headed `{THE_TASK}`"));
    let task_section = after_the_task
        .split("\n### ")
        .next()
        .unwrap_or(after_the_task);
    assert!(
        task_section.contains(named),
        "the task that produced the decision does not name it, so a worker reading the plan \
         lands on a task rather than on the argument"
    );

    let row = plan
        .lines()
        .find(|line| line.starts_with("| **A mapping of a file opened before the turn began**"))
        .expect("section 3 still has a row for the mapping");
    assert!(
        row.contains(named),
        "the mapping's row does not name the decision it waits on, so it reads as a gap \
         nobody has thought about: {row}"
    );
}

/// **It sets out what the task asked for**: three options, what each costs,
/// one recommendation, and what the hook must never be made to decide.
#[test]
fn the_decision_sets_out_the_options_their_costs_and_a_recommendation() {
    let decision = reading(THE_DECISION);

    for heading in WHAT_IT_SETS_OUT {
        assert!(
            decision.contains(heading),
            "the decision is missing `{heading}`, so it is not yet something an owner can \
             answer by choosing"
        );
    }
    for cost in WHAT_EACH_COSTS {
        assert!(
            decision.contains(cost),
            "the decision never says `{cost}`, and an option without its cost is a preference"
        );
    }
    assert!(
        decision.contains(WHAT_IT_MUST_NOT_DECIDE),
        "the decision does not say that a mapping with no file behind it is out of scope — a \
         hook that walked one would ask its question of every allocation on the machine, which \
         is the difference between a boundary and a machine that stops working"
    );
}

/// **While the decision says *proposed*, the hook is not on the programme.**
///
/// This is the refusal path of an ADR: the day `mmap_file` is attached with
/// this status line unchanged, the code has run ahead of its owner, and this
/// test is what says so.
#[test]
fn the_hook_waits_on_the_decision() {
    let status = status_of(&reading(THE_DECISION)).expect("the decision has a readable status");
    if status != WAITING {
        return;
    }
    let programme = reading(THE_PROGRAMME);
    assert!(
        !programme.contains(THE_HOOK),
        "`{THE_HOOK}` is on the programme while {THE_DECISION} still says *proposed*: the hook \
         was written before the decision that says how its gap is reproduced was answered"
    );
}

/// And the check that would notice this file going quiet: every constant it
/// reads by is a string that must still be found in the repository, so a
/// rename that made the assertions vacuous fails here instead of passing.
#[test]
fn the_checks_would_notice_what_they_read_going_missing() {
    assert!(
        reading(THE_PLAN).contains(THE_TASK),
        "the task heading this file reads by is no longer in the plan"
    );
    assert!(
        reading(THE_PROGRAMME).contains("#[lsm(hook ="),
        "the programme no longer declares hooks the way this file looks for one, so the \
         waiting check above could never fail"
    );
    assert!(
        !reading(THE_DECISION).is_empty(),
        "the decision is empty, and every assertion about its contents would pass on nothing"
    );
}
