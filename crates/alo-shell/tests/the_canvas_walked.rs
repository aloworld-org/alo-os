//! **The canvas, walked** — the order of the walk, held against the table in the
//! published report.
//!
//! Task 10 of `docs/autonomy/the-smallest-canvas-worth-showing.md` asks two things
//! of one walk: *a raster at each step*, and *the exact sequence recorded as a
//! table the test reads out of the published report*. This is the second.
//!
//! The first is `crates/alo-shell/examples/support/the_canvas_walk_check.rs`, run
//! as the `canvas-walk` sub-mode of `the_nested_fixtures`, which needs a Wayland
//! parent and a GLES context to submit and read back ten frames. This needs
//! neither: **an order is the same order on every machine, and a walk nobody can
//! run on their own laptop is a walk nobody checks.**
//!
//! # The table is the report's, and this reads it
//!
//! Not a copy of it. Both halves label their steps from
//! `EVERY_MOMENT` in `the_canvas_walk_moments.rs`, and this test requires the
//! report's table to carry those ten in that order. **A step renamed, reordered,
//! added or dropped in the code and not in the report fails here**, which is what
//! the acceptance's *read from the report rather than a copy* is for: a table
//! transcribed into a test would agree with the test forever and with the
//! repository never.
//!
//! A later change that moves a step publishes the table again in a follow-up
//! report and points [`THE_REPORT`] at it, because a published report is not
//! rewritten.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

#[path = "../examples/support/the_canvas_walk_moments.rs"]
mod the_canvas_walk_moments;

use the_canvas_walk_moments::EVERY_MOMENT;

/// Where the walk is recorded, relative to the repository root.
const THE_REPORT: &str = "docs/autonomy/updates/the-canvas-walked.md";

/// The heading its table sits under.
const THE_WALK: &str = "## The walk, step by step";

/// The repository root, found from this file rather than from the process's
/// working directory, which a test runner is free to choose.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("this crate sits two levels below the repository root")
        .to_path_buf()
}

/// Every row of the report's walk table, as the step name in its second column.
///
/// Read out of the file at the heading [`THE_WALK`] names, stopping at the next
/// heading. Rows whose first column is not a number are skipped — the table
/// carries one deliberately, the `—` row that says dragging and resizing have no
/// keyboard form, and that row is a statement about scope rather than a step of
/// the walk.
fn the_table() -> Vec<String> {
    let path = the_repository().join(THE_REPORT);
    let report = fs::read_to_string(&path)
        .unwrap_or_else(|why| panic!("the report at {} could not be read: {why}", path.display()));
    let mut rows = Vec::new();
    let mut inside = false;
    for line in report.lines() {
        if line.trim() == THE_WALK {
            inside = true;
            continue;
        }
        if inside && line.starts_with("## ") {
            break;
        }
        if !inside || !line.trim_start().starts_with('|') {
            continue;
        }
        let columns: Vec<&str> = line.trim().trim_matches('|').split('|').collect();
        let Some((first, second)) = columns.first().zip(columns.get(1)) else {
            continue;
        };
        if first.trim().parse::<usize>().is_err() {
            continue;
        }
        rows.push(second.trim().to_owned());
    }
    rows
}

/// **Every step of the walk is in the report's table, in the walk's own order.**
///
/// Asserted as the whole sequence at once rather than step by step, because the
/// order *is* what task 10 records — two steps swapped is a different walk, and a
/// per-step containment check would pass it.
#[test]
fn the_report_carries_every_step_of_the_walk_in_order() {
    let table = the_table();
    assert!(
        !table.is_empty(),
        "no numbered rows were found under {THE_WALK:?} in {THE_REPORT} — the table \
         is the thing this test reads, so an empty reading is a failure rather than \
         a pass"
    );
    assert_eq!(
        table.as_slice(),
        EVERY_MOMENT.map(str::to_owned).as_slice(),
        "the report's table and the walk's own order disagree"
    );
}

/// **The report says what the raster is evidence of**, which is this task's
/// constraint rather than a nicety.
///
/// *The report says what the raster is evidence of — what this compositor drew,
/// and not what a display showed — unless it was run on a machine with one.* Held
/// as a test because a constraint nobody checks is a sentence somebody deletes
/// while tidying, and this one is the difference between evidence and a claim.
#[test]
fn the_report_says_what_the_raster_is_evidence_of() {
    let path = the_repository().join(THE_REPORT);
    // **Lowercased and its whitespace collapsed**, because this is a search through
    // prose and the two ways it first failed were both about the shape of the text
    // rather than its content: *what this compositor drew* opens a sentence, so the
    // capital W missed it, and *No panel has shown any of it* is wrapped across a
    // line break, so the space between *No* and *panel* is a newline. Both are a
    // report saying exactly the right thing and a test looking for the wrong form
    // of it — which is the fault that let this plan's task 5 look finished. A report
    // is prose; it will be re-wrapped, and a test that breaks when a paragraph is
    // reflowed is a test somebody will delete.
    let report = fs::read_to_string(&path)
        .expect("the report is readable")
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for owed in [
        "what this compositor drew",
        "not what a display showed",
        "weston --backend=headless",
        "no panel has shown any of it",
    ] {
        assert!(
            report.contains(owed),
            "{THE_REPORT} does not say {owed:?}, which is what task 10's constraint asks of it"
        );
    }
}

/// **The two steps with no keyboard form are named in the table itself.**
///
/// Not only in prose further down. *Then the same by keyboard alone* is six steps
/// in the plan's sentence and four in this walk, and the place a person looks for
/// the missing two is the table — so the row that says they are unpromised has to
/// be there, between the pointer half and the keyboard half.
///
/// This is the guard against the fault this plan kept producing: an acceptance
/// half-met, where the half that was dropped is true but unstated, and an
/// unstated reason is indistinguishable from an oversight.
#[test]
fn the_table_names_what_has_no_keyboard_form_where_a_person_would_look() {
    let path = the_repository().join(THE_REPORT);
    let report = fs::read_to_string(&path).expect("the report is readable");
    let under = report
        .split_once(THE_WALK)
        .map(|(_, rest)| rest.split("\n## ").next().unwrap_or(rest))
        .expect("the table's heading is in the report");
    let row = under
        .lines()
        .find(|line| line.starts_with('|') && line.contains("dragging and resizing"))
        .unwrap_or_else(|| {
            panic!(
                "the table under {THE_WALK:?} has no row naming dragging and resizing, \
                 which have no keyboard form in v0.5"
            )
        });
    assert!(
        row.contains("no keyboard form exists in v0.5") && row.contains("none is promised"),
        "the table's row about dragging and resizing does not say that no keyboard \
         form exists and none is promised: {row}"
    );
}
