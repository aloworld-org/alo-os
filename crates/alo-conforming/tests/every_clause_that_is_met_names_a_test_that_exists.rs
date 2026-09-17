//! The acceptance of task 4: **no clause may be met by a sentence.**
//!
//! `crates/alo-conforming`'s list says, for every clause of EN 301 549 that
//! applies to this shell, one of three things — met by a named test, not yet
//! because of a named task in a named plan, or not applicable for a stated
//! reason. This test reads **this repository** and holds every one of those
//! claims to something in it:
//!
//! - a clause that says a test meets it names a file in the workspace that
//!   contains a test of that name;
//! - a clause that says *not yet* names a plan that exists and a task that plan
//!   has;
//! - a clause that says *not applicable* says why.
//!
//! # What *met* means, exactly
//!
//! **The test exists and is in the workspace, so the gate runs it.** This test
//! does not run it again: `CLAUDE.md`'s gate runs the whole suite, and a green
//! gate is what makes every *met* row true. What would make a row false is a
//! name nobody wrote, or a test deleted by somebody who did not know a clause
//! was standing on it, and that is what this catches — the day it happens,
//! rather than the day somebody reads the report.
//!
//! # It reads the repository rather than a list
//!
//! The files are walked, as `alo-citing` walks decisions and `alo-collected`
//! walks words. Nothing here keeps a second copy of anything: a test renamed
//! anywhere in the workspace is a finding here with nothing to update.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_conforming::{Finding, THE_CLAUSES, THE_STANDARD, THE_VERSION, held};

/// This repository.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("this crate is two directories below the repository")
}

/// Every Rust file under `crates/`, as its path from the root and its text.
fn every_rust_file() -> Vec<(String, String)> {
    let mut found = Vec::new();
    walk(&the_repository().join("crates"), &mut found, "rs");
    found
}

/// Every plan, the same way.
fn every_plan() -> Vec<(String, String)> {
    let mut found = Vec::new();
    walk(&the_repository().join("docs/autonomy"), &mut found, "md");
    found
}

/// Everything under this directory with this extension, read.
fn walk(at: &Path, into: &mut Vec<(String, String)>, ending: &str) {
    let Ok(entries) = std::fs::read_dir(at) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // `target` directories hold other people's code and are enormous.
            if path.file_name().is_some_and(|named| named == "target") {
                continue;
            }
            walk(&path, into, ending);
        } else if path.extension().is_some_and(|had| had == ending)
            && let Ok(text) = std::fs::read_to_string(&path)
            && let Ok(from_the_root) = path.strip_prefix(the_repository())
        {
            into.push((from_the_root.display().to_string(), text));
        }
    }
}

/// **Every clause claims something this repository can show**, and the list
/// says what it adds up to.
#[test]
fn every_clause_that_is_met_names_a_test_that_exists_and_every_not_yet_names_a_task() {
    let files = every_rust_file();
    let plans = every_plan();
    assert!(
        files.len() > 100,
        "the walk found {} files, which is not this repository",
        files.len()
    );

    let outcome = held(&THE_CLAUSES, &files, &plans);
    let held = match outcome {
        Ok(held) => held,
        Err(findings) => panic!(
            "{} clause(s) of {THE_STANDARD} {THE_VERSION} claim something this repository cannot \
             show:\n{}",
            findings.len(),
            findings
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        ),
    };

    assert_eq!(held.clauses, THE_CLAUSES.len());
    assert_eq!(
        held.met + held.waiting + held.not_applicable,
        held.clauses,
        "a clause is met, waiting or not applicable, and there is no fourth thing"
    );
    assert!(
        held.met > 0,
        "nothing at all is met, which is not this shell"
    );

    // The admission, in the open: nobody has read these against the published
    // text yet, and the day somebody does this number moves.
    eprintln!(
        "{THE_STANDARD} {THE_VERSION}: {} clauses — {} met by a test, {} waiting on a task, {} not \
         applicable; {} read against the standard's own text",
        held.clauses, held.met, held.waiting, held.not_applicable, held.checked_against_the_text
    );
}

/// **A clause met by a test nobody wrote is caught**, which is the whole point
/// of the check above — held here against a name that is deliberately not in
/// this repository, so the check cannot pass by finding nothing to do.
#[test]
fn a_clause_met_by_a_name_nobody_wrote_is_a_finding() {
    use alo_conforming::{Checked, Clause, Evidence, Standing};

    let invented = Clause {
        number: "11.4.1.2",
        requirement: "Everything on the screen must say what it is.",
        standing: Standing::Met {
            by: Evidence {
                crate_named: "alo-access",
                file: "src/tree.rs",
                test: "a_test_that_was_never_written_anywhere",
            },
        },
        checked: Checked::NotAgainstTheText,
    };
    let findings = held(&[invented], &every_rust_file(), &every_plan())
        .expect_err("that test is not in this repository");
    assert!(matches!(
        findings.first(),
        Some(Finding::ATestNobodyWrote { .. })
    ));
}
