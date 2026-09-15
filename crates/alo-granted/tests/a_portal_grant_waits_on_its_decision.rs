//! What an application's grant is over is a decision, and the portals wait on
//! it.
//!
//! Task 1 of `docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`
//! asks for a portal request judged against a grant naming the application and
//! "the same `Reach` an agent's grant names", reusing `alo-capability` and never
//! editing it. Eleven of the fifteen v0.5 portals ask about something no
//! `Reach` can name, an application's grants kept in the agent's list would end
//! when a person declines the agent, and every refusal speaks of an agent. So
//! the task was the decision:
//! [ADR 0040](../../../docs/decisions/0040-what-an-applications-grant-is-over.md).
//!
//! # What this file holds
//!
//! The shape `alo-opening`'s `converting_waits_on_its_decision.rs` gives ADR
//! 0039, for the same reason: a decision nobody points at is a task the next
//! worker takes up by choosing an option themselves, and a portal model built
//! beside a decision still marked *proposed* is the code choosing option A or B
//! without saying so.
//!
//! - **The ADR exists, once**, under its number, and still stands.
//! - **The plan points at it from task 1, and steps over task 1** while it is
//!   proposed.
//! - **It sets out what an owner can answer by choosing**: three options, what
//!   each costs, one recommendation, and the one thing no option may do —
//!   answer a portal *yes* by default.
//! - **It is about the portals the promise lists**: the line in
//!   `docs/features.md` it was written against, and a row for each of them.
//! - **While it says *proposed*, no portal is modelled**: the crate the plan
//!   names does not exist, the workspace does not list it, and no shipped
//!   source names a portal interface.
//!
//! It lives in `alo-granted` because the one list is where the answer shows:
//! whatever an application's grant turns out to be over is a row this crate
//! will read. It reads files and runs nothing, so it holds on any machine.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

/// The decision's number, as `docs/decisions/README.md` says one is cited.
const THE_NUMBER: &str = "0040";

/// The decision's file, relative to the repository.
const THE_DECISION: &str = "docs/decisions/0040-what-an-applications-grant-is-over.md";

/// Where every decision lives.
const THE_DECISIONS: &str = "docs/decisions";

/// The plan that asked for the decision.
const THE_PLAN: &str = "docs/autonomy/v0-5-applications-and-what-they-expect-plan.md";

/// The heading of the task that produced it.
const THE_TASK: &str = "### 1. A portal request is a grant, and is refused like one";

/// The promises the decision was written against.
const THE_FEATURES: &str = "docs/features.md";

/// The v0.5 portal line of `docs/features.md`, as the decision read it.
///
/// Held whole, so that a portal added to or taken from the promise fails here
/// and sends whoever changed it to the decision's table, which would otherwise
/// go on describing a list that is no longer the one promised.
const THE_PORTALS_PROMISED: &str = "- [v0.5] Portals: file chooser and documents, open-with and \
                                    default applications, notifications, print, screenshot, \
                                    screen capture, camera, microphone, clipboard, trash, \
                                    wallpaper, settings, inhibit (no sleep mid-presentation), \
                                    network and power-profile monitors";

/// The fifteen portals, as the decision's table names them — one row each.
const THE_PORTALS: [&str; 15] = [
    "file chooser and documents",
    "open-with and default applications",
    "print",
    "trash",
    "wallpaper",
    "notifications",
    "screenshot",
    "screen capture",
    "camera",
    "microphone",
    "clipboard",
    "settings",
    "inhibit",
    "network monitor",
    "power-profile monitor",
];

/// How many of them ask about something no `Reach` can name today.
const WITH_NO_REACH: usize = 11;

/// The workspace manifest.
const THE_WORKSPACE: &str = "Cargo.toml";

/// Where every crate of the product lives.
const THE_CRATES: &str = "crates";

/// The crate the plan names for the portal model.
const THE_CRATE_IT_NAMES: &str = "alo-portals";

/// The line a decision records its status on.
const STATUS: &str = "**Status:**";

/// The statuses under which the decision still stands.
const STANDING: [&str; 2] = ["proposed", "accepted"];

/// The one status under which the portal model is held to being unbuilt.
const WAITING: &str = "proposed";

/// The headings an owner answers by choosing among.
const WHAT_IT_SETS_OUT: [&str; 4] = ["### A — ", "### B — ", "### C — ", "## The recommendation"];

/// What each option must say, as a phrase the decision carries.
const WHAT_EACH_COSTS: &str = "**What it costs:**";

/// What no option may do, whichever is chosen.
const WHAT_NO_OPTION_MAY_DO: &str = "No portal is answered yes by default";

/// The three things the decision says must happen before the task is ready,
/// each as a phrase it carries.
const WHAT_MUST_HAPPEN_FIRST: [&str; 3] = [
    "accepts, amends or rejects",
    "the crate that owns",
    "grants file",
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

/// Task 1's own section of the plan: from its heading to the next task's.
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

/// The row of the decision's table that begins with this portal, if there is one.
fn the_row_for<'a>(decision: &'a str, portal: &str) -> Option<&'a str> {
    let begins = format!("| {portal} |");
    decision
        .lines()
        .map(str::trim_start)
        .find(|line| line.starts_with(&begins))
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
        "the decision's status is `{status}`, so task 1 waits on an answer that no longer stands"
    );
}

/// **The plan points at the decision from task 1, and while the decision is
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
        "task 1 does not name the decision it waits on, so a worker lands on a task rather than \
         on the argument"
    );

    let status = status_of(&reading(THE_DECISION)).expect("the decision has a readable status");
    if status != WAITING {
        return;
    }
    let task_status = section
        .lines()
        .find(|line| line.starts_with(STATUS))
        .expect("task 1 has a status line");
    assert!(
        task_status.contains("blocked"),
        "task 1 says `{task_status}` while {THE_DECISION} is still proposed, so the loop would \
         offer it as work"
    );
    assert!(
        !section.lines().any(|line| line
            .strip_prefix(STATUS)
            .map_or(line, str::trim_start)
            .starts_with("**Done,")),
        "task 1 is marked done while the decision it waits on is still proposed"
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
        "the decision does not rule out answering a portal yes by default, which no option may do"
    );
    for first in WHAT_MUST_HAPPEN_FIRST {
        assert!(
            decision.contains(first),
            "the decision does not say `{first}`, so the next worker cannot tell what the task is \
             still waiting on"
        );
    }
}

/// **The decision is about the portals the promise lists, each of them once.**
///
/// Its argument is a table: which portals ask about something a `Reach` can
/// name, and which do not. A table that silently lost a row, or a promise that
/// gained a portal the table never considered, would leave the argument
/// resting on a list nobody promised.
#[test]
fn the_decision_reads_every_portal_the_promise_lists() {
    let features = reading(THE_FEATURES);
    assert!(
        features.lines().any(|line| line == THE_PORTALS_PROMISED),
        "the v0.5 portal line of {THE_FEATURES} is no longer the one {THE_DECISION} was written \
         against; read the decision's table again before changing this constant"
    );

    let decision = reading(THE_DECISION);
    let mut unnamed = 0;
    for portal in THE_PORTALS {
        let row = the_row_for(&decision, portal)
            .unwrap_or_else(|| panic!("the decision's table has no row for `{portal}`"));
        assert_eq!(
            decision
                .lines()
                .map(str::trim_start)
                .filter(|line| line.starts_with(&format!("| {portal} |")))
                .count(),
            1,
            "`{portal}` has more than one row, so the table says two things about it"
        );
        if row.contains("**none**") {
            unnamed += 1;
        }
    }
    assert_eq!(
        unnamed, WITH_NO_REACH,
        "the number of portals no Reach can name is what the plan's blocked status states"
    );

    // And the v1 portals are absent from the table rather than present and
    // refused — a portal this machine does not offer is not one it lists.
    for later in [
        "usb",
        "global shortcuts",
        "dynamic launchers",
        "remote desktop",
    ] {
        assert!(
            the_row_for(&decision, later).is_none(),
            "the table has a row for the v1 portal `{later}`"
        );
    }
}

/// **While the decision says *proposed*, no portal is modelled.**
///
/// The refusal path of an ADR. The day `crates/alo-portals` appears, the
/// workspace lists it, or shipped source names a portal interface with this
/// status line unchanged, the code has run ahead of its owner and chosen what
/// an application's grant is over by itself, and this is the test that says so.
#[test]
fn no_portal_is_modelled_while_the_decision_is_proposed() {
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
    assert!(
        !reading(THE_WORKSPACE).contains(&format!("\"crates/{THE_CRATE_IT_NAMES}\"")),
        "the workspace lists crates/{THE_CRATE_IT_NAMES} while {THE_DECISION} is still proposed"
    );

    // Assembled rather than written out, so that this file is never the thing it
    // looks for.
    let an_interface = format!("{}.{}.{}.", "org", "freedesktop", "portal");
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
            !written.contains(&an_interface),
            "{} names a portal interface while {THE_DECISION} is still proposed",
            source.display()
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
        reading(THE_WORKSPACE).contains("\"crates/alo-granted\""),
        "the workspace manifest this file reads does not list this crate, so it is not the \
         manifest the product is built from"
    );
    assert!(
        names_in(THE_CRATES)
            .iter()
            .any(|name| name == "alo-granted"),
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
    assert!(
        the_row_for("| camera | the camera | **none** |\n", "camera").is_some(),
        "a row is not found by the portal it begins with"
    );
    // A table inside a list item is indented, which is where the decision's is.
    assert!(
        the_row_for(
            "1. Facts:\n\n   | camera | the camera | **none** |\n",
            "camera"
        )
        .is_some(),
        "a row indented under a list item is not found"
    );
    assert!(the_row_for("| screen capture | x |\n", "screen").is_none());
}
