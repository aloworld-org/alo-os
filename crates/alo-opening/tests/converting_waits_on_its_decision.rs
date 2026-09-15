//! Converting a `.docx`, `.xlsx` or `.pptx` is a decision, and the converter
//! waits on it.
//!
//! Task 2 of `docs/autonomy/v0-5-documents-and-paper-plan.md` opens the three
//! formats people are sent "through the rented converter running on this
//! machine". Whoever writes the first line of that chooses what runs the engine,
//! how a verb reaches it, and whether a document's linked pictures can be
//! fetched on the way — which is law 2's shape, the kernel-enforced grant and
//! law 1, none of them a worker's to settle. So the task was the decision:
//! [ADR 0039](../../../docs/decisions/0039-a-document-is-converted-by-an-engine-that-can-reach-nothing.md).
//!
//! # What this file holds
//!
//! The shape `alo-bounding`'s tests give ADR 0029 and ADR 0030, for the same
//! reason: a decision nobody points at is a task the next worker takes up by
//! choosing an option themselves, and a converter built beside a decision still
//! marked *proposed* is the code getting ahead of its owner.
//!
//! - **The ADR exists, once**, under its number, and still stands.
//! - **The plan points at it from task 2, and steps over task 2** while it is
//!   proposed — a task left *ready* would send the next worker straight back at
//!   the same wall.
//! - **It sets out what an owner can answer by choosing**: three options, what
//!   each costs, one recommendation, and the one thing no option may do —
//!   upload a document.
//! - **While it says *proposed*, nothing converts**: no crate declares the verb,
//!   the crate it names does not exist, and the image pins no converter.
//!
//! It reads files and runs nothing, so it holds on any machine.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

/// The decision's number, as `docs/decisions/README.md` says one is cited.
const THE_NUMBER: &str = "0039";

/// The decision's file, relative to the repository.
const THE_DECISION: &str =
    "docs/decisions/0039-a-document-is-converted-by-an-engine-that-can-reach-nothing.md";

/// Where every decision lives.
const THE_DECISIONS: &str = "docs/decisions";

/// The plan that asked for the decision.
const THE_PLAN: &str = "docs/autonomy/v0-5-documents-and-paper-plan.md";

/// The heading of the task that produced it.
const THE_TASK: &str = "### 2. `.docx`, `.xlsx`, `.pptx` — opened, and what the conversion cost";

/// The recipe the machine's image is built from.
const THE_IMAGE: &str = "image/Containerfile";

/// Where every crate of the product lives.
const THE_CRATES: &str = "crates";

/// The crate the decision names for the converting code.
const THE_CRATE_IT_NAMES: &str = "alo-converting";

/// The line a decision records its status on.
const STATUS: &str = "**Status:**";

/// The statuses under which the decision still stands.
const STANDING: [&str; 2] = ["proposed", "accepted"];

/// The one status under which the converter is held to being unbuilt.
const WAITING: &str = "proposed";

/// The headings an owner answers by choosing among.
const WHAT_IT_SETS_OUT: [&str; 4] = ["### A — ", "### B — ", "### C — ", "## The recommendation"];

/// What each option must say, as a phrase the decision carries.
const WHAT_EACH_COSTS: &str = "**What it costs:**";

/// What no option may do, whichever is chosen.
const WHAT_NO_OPTION_MAY_DO: &str = "Nothing is uploaded";

/// The three things the decision says must happen before the task is ready,
/// each as a phrase it carries.
const WHAT_MUST_HAPPEN_FIRST: [&str; 3] = [
    "accepts, amends or rejects",
    "idle handoff",
    "real documents",
];

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

/// Every entry's name in a folder of this repository.
fn names_in(folder: &str) -> Vec<String> {
    let at = the_repository().join(folder);
    fs::read_dir(&at)
        .unwrap_or_else(|why| panic!("{} could not be listed: {why}", at.display()))
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect()
}

/// The decision's status, read off its status line the way `alo-citing` reads
/// one: the first line beginning with the marker, and the first known status in
/// it.
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

/// Task 2's own section of the plan: from its heading to the next task's.
fn the_tasks_section(plan: &str) -> &str {
    let after = plan
        .split_once(THE_TASK)
        .map(|(_, rest)| rest)
        .unwrap_or_else(|| panic!("the plan has no task headed `{THE_TASK}`"));
    after.split("\n### ").next().unwrap_or(after)
}

/// The decision's file name, without its folder.
fn the_decisions_name() -> &'static str {
    Path::new(THE_DECISION)
        .file_name()
        .and_then(|name| name.to_str())
        .expect("the decision's path has a file name")
}

/// Every `.rs` file under a folder.
fn sources_under(folder: &Path, found: &mut Vec<PathBuf>) {
    let Ok(listed) = fs::read_dir(folder) else {
        return;
    };
    for entry in listed.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            sources_under(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
}

/// **The decision exists, once, under its number, and still stands.**
///
/// Two files carrying one number would make every citation of it resolve to
/// whichever a reader opened first.
#[test]
fn the_decision_exists_once_under_its_number() {
    let names = names_in(THE_DECISIONS);
    let carrying: Vec<&String> = names
        .iter()
        .filter(|name| name.starts_with(&format!("{THE_NUMBER}-")))
        .collect();
    match carrying.as_slice() {
        [one] => assert_eq!(
            one.as_str(),
            the_decisions_name(),
            "the decision numbered {THE_NUMBER} is `{one}`, not the file the plan names"
        ),
        [] => panic!("no decision under {THE_DECISIONS} carries the number {THE_NUMBER}"),
        many => panic!(
            "{} decisions carry the number {THE_NUMBER}: {many:?}",
            many.len()
        ),
    }

    let status = status_of(&reading(THE_DECISION)).expect("the decision has a readable status");
    assert!(
        STANDING.contains(&status.as_str()),
        "the decision's status is `{status}`, so task 2 waits on an answer that no longer stands"
    );
}

/// **The plan points at the decision from task 2, and while the decision is
/// proposed the plan steps over the task.**
///
/// The refusal this guards is the loop's: a task left *ready* beside an
/// unanswered decision is selected again, and the next worker is sent at the
/// same wall with no more authority to climb it than the first.
#[test]
fn the_plan_points_at_the_decision_and_steps_over_the_task_while_it_waits() {
    let plan = reading(THE_PLAN);
    let section = the_tasks_section(&plan);
    assert!(
        section.contains(the_decisions_name()),
        "task 2 does not name the decision it waits on, so a worker lands on a task rather than \
         on the argument"
    );

    let status = status_of(&reading(THE_DECISION)).expect("the decision has a readable status");
    if status != WAITING {
        return;
    }
    let task_status = section
        .lines()
        .find(|line| line.starts_with(STATUS))
        .expect("task 2 has a status line");
    assert!(
        task_status.contains("blocked"),
        "task 2 says `{task_status}` while {THE_DECISION} is still proposed, so the loop would \
         offer it as work"
    );
    assert!(
        !section.lines().any(|line| line
            .strip_prefix(STATUS)
            .map_or(line, str::trim_start)
            .starts_with("**Done,")),
        "task 2 is marked done while the decision it waits on is still proposed"
    );
}

/// **It sets out what an owner answers by choosing**: three options, what each
/// costs, one recommendation, what no option may do, and what must happen before
/// the task is ready again.
#[test]
fn the_decision_sets_out_the_options_their_costs_and_a_recommendation() {
    let decision = reading(THE_DECISION);
    for heading in WHAT_IT_SETS_OUT {
        assert!(
            decision.contains(heading),
            "the decision is missing `{heading}`, so it is not yet something an owner can answer \
             by choosing"
        );
    }
    assert_eq!(
        decision.matches(WHAT_EACH_COSTS).count(),
        3,
        "each of the three options says what it costs, once — an option without its cost is a \
         preference"
    );
    assert!(
        decision.contains(WHAT_NO_OPTION_MAY_DO),
        "the decision does not rule out uploading a document, which the plan forbids whichever \
         option is chosen"
    );
    for first in WHAT_MUST_HAPPEN_FIRST {
        assert!(
            decision.contains(first),
            "the decision does not say `{first}`, so the next worker cannot tell what the task is \
             still waiting on"
        );
    }
}

/// **While the decision says *proposed*, nothing converts.**
///
/// The refusal path of an ADR. The day the verb is declared, the crate the
/// decision names appears, or the image pins a converter with this status line
/// unchanged, the code has run ahead of its owner, and this is the test that
/// says so.
#[test]
fn nothing_converts_while_the_decision_is_proposed() {
    let status = status_of(&reading(THE_DECISION)).expect("the decision has a readable status");
    if status != WAITING {
        return;
    }

    assert!(
        !names_in(THE_CRATES)
            .iter()
            .any(|name| name == THE_CRATE_IT_NAMES),
        "crates/{THE_CRATE_IT_NAMES} exists while {THE_DECISION} is still proposed"
    );

    // Assembled rather than written out, so that this file is never the thing it
    // looks for.
    let the_verb = format!("\"{}_{}\"", "convert", "document");
    let mut sources = Vec::new();
    sources_under(&the_repository().join(THE_CRATES), &mut sources);
    let shipped: Vec<&PathBuf> = sources
        .iter()
        .filter(|source| source.components().any(|part| part.as_os_str() == "src"))
        .collect();
    assert!(
        shipped.len() > 50,
        "the product's shipped source was not found"
    );
    for source in shipped {
        let written = fs::read_to_string(source).expect("a source file");
        assert!(
            !written.contains(&the_verb),
            "{} names the verb {the_verb} while {THE_DECISION} is still proposed",
            source.display()
        );
    }

    let image = reading(THE_IMAGE).to_lowercase();
    for engine in ["libreoffice", "soffice", "the_converter"] {
        assert!(
            !image.contains(engine),
            "the image names `{engine}` while {THE_DECISION} is still proposed, so a converter was \
             pinned before anybody chose how it runs"
        );
    }
}

/// And the check that would notice this file going quiet: everything it reads
/// by is still where it looks, so a rename that made the assertions above vacuous
/// fails here instead of passing.
#[test]
fn the_checks_would_notice_what_they_read_going_missing() {
    assert!(
        reading(THE_PLAN).contains(THE_TASK),
        "the task heading this file reads by is no longer in the plan"
    );
    assert!(
        reading(THE_IMAGE).contains("ARG THE_RUNTIME="),
        "the image recipe this file reads no longer pins the one engine it is known to pin, so it \
         is not the recipe the machine is built from"
    );
    assert!(
        names_in(THE_CRATES)
            .iter()
            .any(|name| name == "alo-opening"),
        "the crates folder this file lists does not hold this crate"
    );
    let decision = reading(THE_DECISION);
    assert!(
        status_of(&decision).is_ok(),
        "the decision's status cannot be read, and every check that depends on it would stop"
    );
    assert_eq!(
        status_of("# A decision\n\n**Status:** withdrawn, by nobody\n"),
        Ok("withdrawn".to_owned())
    );
    assert!(status_of("# A decision with no status\n").is_err());
}
