//! **Putting a folder back is a road only a person may take, and nothing in this
//! workspace takes it but the command a person types.**
//!
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md) term 2,
//! in its own words: *there is **no agent verb** that undoes, and none that
//! proposes an undo — an agent choosing which of a person's approvals to reverse
//! is the thing the plan's constraint forbids.* `alo-letting-go` has had a guard
//! for the sibling road since it was written, and the other half of ADR 0045's
//! narrowing — *forgetting is a person's act in Settings, and never a broker
//! verb* — is held by
//! `crates/alo-letting-go/tests/a_turn_cannot_arrive_at_this_road.rs`.
//!
//! # Why this crate needs one now, and did not when it was written
//!
//! Because its **words are collected**. `alo_saying::everything_this_machine_can_say`
//! assembles one vocabulary from every crate that declares sentences, and
//! `alo-agentd` and `alo-turn` both reach `alo-saying` — so the moment this
//! crate's words joined that list, **its library joined the turn's dependency
//! graph**. The act is now linked into the daemon's binary.
//!
//! That is the right trade: a crate outside the one vocabulary is a crate whose
//! sentences no translator's file is ever checked against, and `alo-agentd`
//! stands apart for exactly that reason at exactly that cost. But it means the
//! capability is reachable from where it must not be called, and a guard is the
//! difference between *nothing calls it* and *nothing can have called it without
//! somebody noticing*.
//!
//! **Found by the sibling guard failing**, 2026-10-22: it refused a doc comment
//! in this crate for naming `alo_letting_go`'s Rust path, which is how anybody
//! learnt this crate was in the graph at all.
//!
//! # What it reads, and why that is wider than the dependency walk
//!
//! Every `crates/*/src` in the workspace, not only what a turn reaches. The
//! question *who may put a person's folder back* has one answer — the person —
//! and widening the check costs nothing while removing a step that can be wrong:
//! a crate that enters the turn's graph later is already covered.

#![expect(
    clippy::panic,
    clippy::expect_used,
    reason = "in a test, a panic naming the file and the line is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// The two names that are the act itself.
///
/// Reading a list of places is not one of them: a person's own session may show
/// them, a settings window may count them, and nothing is changed by looking.
/// What may not be reached is **the act** — the plan and the swap.
const THE_ACT: [&str; 2] = ["what_would_go_back", "go_back"];

/// The crates that may name it, with why each one may.
const MAY: [(&str, &str); 2] = [
    // The command a person types. ADR 0045 term 2's *the person's*, and the only
    // caller there is.
    ("alo-back", "the command a person types"),
    // Itself: the act is declared here, and its own tests exercise it.
    ("alo-as-it-was", "where the act is declared"),
];

/// This repository's root, from this crate's manifest.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_owned()
}

/// Every `.rs` file under a directory, however deep.
fn every_source_under(at: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(at) else {
        return found;
    };
    let mut each: Vec<PathBuf> = entries
        .map(|entry| entry.expect("an entry of this repository").path())
        .collect();
    // `read_dir` has no order and the order differs by filesystem, so a check
    // that reported the first failure it met would name a different file on
    // another disk.
    each.sort();
    for path in each {
        if path.is_dir() {
            found.extend(every_source_under(&path));
        } else if path.extension().is_some_and(|had| had == "rs") {
            found.push(path);
        }
    }
    found
}

/// **Nothing but the person's own command reaches the act.**
#[test]
fn nothing_but_the_command_a_person_types_puts_a_folder_back() {
    let crates = the_repository().join("crates");
    let mut read = 0_usize;
    let mut found = Vec::new();

    let mut each: Vec<PathBuf> = std::fs::read_dir(&crates)
        .expect("crates/ is readable")
        .map(|entry| entry.expect("an entry of this repository").path())
        .collect();
    each.sort();

    for at in each {
        let named = at
            .file_name()
            .expect("a directory this test just listed")
            .to_string_lossy()
            .into_owned();
        if MAY.iter().any(|(may, _)| *may == named) {
            continue;
        }
        for file in every_source_under(&at.join("src")) {
            let text = std::fs::read_to_string(&file)
                .unwrap_or_else(|why| panic!("{} was not read: {why}", file.display()));
            read += 1;
            for act in THE_ACT {
                if text.contains(&format!("alo_as_it_was::{act}"))
                    || text.contains(&format!("as_it_was::putting_back::{act}"))
                {
                    found.push(format!("{} names {act}", file.display()));
                }
            }
        }
    }

    // The control. A walk that read nothing would pass against any workspace,
    // and this repository's own commonest false green is a check whose subject
    // was empty.
    assert!(
        read > 100,
        "read {read} source file(s), so this test is broken rather than the \
         workspace: it is meant to walk every crate's src"
    );

    assert!(
        found.is_empty(),
        "putting a person's folder back is a road only they may take — ADR 0045 \
         term 2: *there is no agent verb that undoes, and none that proposes an \
         undo*. {} place(s) outside {:?} reach it:\n\n- {}",
        found.len(),
        MAY.map(|(named, _)| named),
        found.join("\n- ")
    );
}

/// **The check can fail**, demonstrated against a text of this test's own rather
/// than by trusting the walk above.
///
/// The walk's one job is finding a spelling of the act in a file, and the
/// assertion it feeds passes when nothing is found — so a spelling it cannot see
/// is a silent pass. This holds the matching itself against both forms.
#[test]
fn the_two_spellings_this_was_written_for_are_both_caught() {
    for text in [
        "let _ = alo_as_it_was::go_back(&goes);",
        "crate::as_it_was::putting_back::what_would_go_back(a, b, c, d, e)",
    ] {
        assert!(
            THE_ACT.iter().any(|act| {
                text.contains(&format!("alo_as_it_was::{act}"))
                    || text.contains(&format!("as_it_was::putting_back::{act}"))
            }),
            "this spelling would reach the act and the check would not see it: {text}"
        );
    }
    // And something that is not the act is not matched, so the check is not a
    // substring match that would call any mention a finding.
    let looking = "let places = alo_as_it_was::Places::under(&whose);";
    assert!(
        !THE_ACT.iter().any(|act| {
            looking.contains(&format!("alo_as_it_was::{act}"))
                || looking.contains(&format!("as_it_was::putting_back::{act}"))
        }),
        "reading the list of places is not the act and must not be refused"
    );
}
