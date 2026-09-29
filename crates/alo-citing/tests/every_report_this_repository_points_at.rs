//! **Every task report this repository points at, against the disk it is checked
//! out on.**
//!
//! `src/reports.rs` shows both refusals happening against fixtures. This is where
//! the rule meets the real documents — which is the half that would have caught
//! the 46 citations that resolved under no rule at all, and the half a convention
//! could never be.
//!
//! # It counts what it read, and refuses to pass on nothing
//!
//! A walk that quietly finds no files reports no findings, and a green bar from a
//! check that stopped looking is indistinguishable from a green bar from a check
//! that looked. So the count is asserted before the findings are: this repository
//! cites reports in the hundreds, and a run that sees a handful has a broken walk
//! rather than a clean repository.
//!
//! That is not a hypothetical caution. On 2026-09-28 three separate faults in this
//! repository's own tooling had exactly this shape — a gate that batched its
//! output printed which tests failed and never why, a skipped test is the same
//! colour as a passing one, and a diagnostic written to explain a failure died
//! before it could report the failure it was written for. **The reporting path is
//! the one path nobody exercises in the failing case**, so it gets exercised here
//! on purpose.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

use alo_citing::reports::{Amiss, Cited, cited_in, held};

/// Where the reports live, from the repository root.
const WHERE_THE_REPORTS_ARE: &str = "docs/autonomy/updates/";

/// Fewer citations than this means the walk is broken, not that the repository
/// stopped citing its own reports. Measured on 2026-09-28: 284 in 30 files.
const TOO_FEW_TO_BELIEVE: usize = 150;

/// This repository, from the crate this test is in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the repository is where this crate says it is")
}

/// What is not this repository's own text: what the build wrote, and what git
/// keeps.
fn is_not_ours(name: &str) -> bool {
    matches!(name, ".git" | "target" | "node_modules") || name.starts_with("target-")
}

/// Every `.md` file under `directory`, as a repository-relative path with forward
/// slashes and the text in it.
///
/// Markdown only. A report citation in Rust would be a rustdoc link and this
/// crate's other test already reads every `.rs` file for decisions; widening this
/// walk would mean deciding what a citation means inside a doc comment, which is a
/// separate question and not one this change is answering.
fn every_document_under(directory: &Path, below: &str, into: &mut Vec<(String, String)>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = if below.is_empty() {
            name.clone()
        } else {
            format!("{below}/{name}")
        };
        if entry.path().is_dir() {
            if !is_not_ours(&name) {
                every_document_under(&entry.path(), &path, into);
            }
        } else if name.ends_with(".md")
            && let Ok(text) = fs::read_to_string(entry.path())
        {
            into.push((path, text));
        }
    }
}

/// The filename of every report that exists.
fn the_reports() -> Vec<String> {
    let at = the_repository().join(WHERE_THE_REPORTS_ARE);
    let entries =
        fs::read_dir(&at).expect("this repository keeps its reports where it says it does");
    let mut found: Vec<String> = entries
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".md"))
        .collect();
    found.sort();
    assert!(
        found.len() > 50,
        "{} report(s) under {WHERE_THE_REPORTS_ARE}, which is not this repository",
        found.len()
    );
    found
}

/// **Every report this repository points at is one somebody wrote, and every
/// citation of one can be followed from where it is written.**
#[test]
fn every_report_this_repository_points_at_can_be_followed() {
    let mut documents = Vec::new();
    every_document_under(&the_repository(), "", &mut documents);
    assert!(
        documents.len() > 100,
        "{} document(s) were read, which is not this repository",
        documents.len()
    );

    let cited: Vec<Cited> = documents
        .iter()
        .flat_map(|(file, text)| cited_in(file, text))
        .collect();
    assert!(
        cited.len() >= TOO_FEW_TO_BELIEVE,
        "only {} report citation(s) found in {} document(s). This repository cites its reports in \
         the hundreds, so this is a walk that stopped looking rather than a repository that \
         stopped citing — and a check that found nothing passes in exactly the same colour as one \
         that found nothing wrong.",
        cited.len(),
        documents.len()
    );

    let reports = the_reports();
    let names: Vec<&str> = reports.iter().map(String::as_str).collect();
    let amiss = held(&cited, &names);
    assert!(
        amiss.is_empty(),
        "{} report citation(s) in this repository cannot be followed:\n\n{}\n\n{} citation(s) were \
         read from {} document(s), against {} report(s).",
        amiss.len(),
        amiss
            .iter()
            .map(|one| format!("- {one}"))
            .collect::<Vec<_>>()
            .join("\n\n"),
        cited.len(),
        documents.len(),
        reports.len()
    );
}

/// **A real citation, shortened where it does not resolve, is refused** — the
/// check shown working against this repository's own text rather than a fixture.
///
/// It takes a citation that is correct today, rewrites it the way the 46 were
/// written, and asserts the finding appears and names what to write instead. A
/// check nobody has watched refuse anything is a check that passes on the day it
/// stops looking.
#[test]
fn a_real_citation_shortened_the_way_the_forty_six_were_is_refused() {
    let reports = the_reports();
    let names: Vec<&str> = reports.iter().map(String::as_str).collect();
    let first = names
        .first()
        .expect("this repository has reports, which the reader above asserted");

    // ROADMAP.md is at the root, so the short form cannot resolve from it. This is
    // exactly the shape of 33 of the 46.
    let shortened = format!("Evidence: `updates/{first}`.");
    let amiss = held(&cited_in("ROADMAP.md", &shortened), &names);
    assert_eq!(
        amiss,
        vec![Amiss::ShortenedWhereItDoesNotResolve {
            named: (*first).to_owned(),
            file: "ROADMAP.md".to_owned(),
            line: 1,
        }],
        "a real report cited the way the forty-six were was not refused"
    );

    // And written out, the same sentence is no finding at all.
    let written_out = format!("Evidence: `{WHERE_THE_REPORTS_ARE}{first}`.");
    assert_eq!(
        held(&cited_in("ROADMAP.md", &written_out), &names),
        vec![],
        "the same citation written from the root was refused, which would make the rule \
         unsatisfiable"
    );
}
