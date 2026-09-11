//! Nothing an agent can send writes a person's settings, and nothing here
//! knocks on a daemon.
//!
//! `alo-changing` owes this argument one file over and can make half of it with
//! a manifest: `alo-agentd` does not name that crate, so there is no road from
//! the socket to the writer of the grants. **This crate cannot make that half**,
//! because the daemon genuinely does name it — `alo_choosing::Settings` is how
//! `alo-agentd` finds out which model answers, and it is meant to.
//!
//! So the guarantee is a different one and it is checked a different way:
//! **the daemon reads these settings and has no way to write them.**
//! [`alo_choosing::Choosing`] is the only door in this crate that touches a
//! disk in that direction, and nothing under `crates/alo-agentd/src` names it.
//! That is read off this repository rather than asserted about it, which is the
//! shape `alo-collected` and `alo-citing` settled: a check that only ever reads
//! its own fixtures can pass while the disk says something else.
//!
//! The second thing measured here is the **absence of a knock**, and it is a
//! finding rather than an omission. `alo-changing` ends in
//! `alo_protocol::FromAPerson::Granted` because a daemon holds the grants it
//! read at start-up and would otherwise never learn of a change. Settings are
//! not held that way: `alo-agentd` reads this file once a turn, at the first
//! question of that turn, and the daemon measures that itself in
//! `questions.rs`'s `a_new_turn_reads_the_file_the_person_has_just_written`. A
//! knock here would be a message telling a service to do what it already does —
//! and what holds it that way is that this crate names nothing that could speak
//! one.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

/// This repository, from the crate this test is in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("this crate is two directories below the repository")
}

/// A manifest of this repository, read.
fn manifest(crate_named: &str) -> String {
    let at = the_repository()
        .join("crates")
        .join(crate_named)
        .join("Cargo.toml");
    fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// Whether this manifest names this crate as a dependency — a line that opens
/// its table, rather than any mention of the name, because manifests in this
/// repository argue in prose about what they deliberately do not depend on and
/// a check should not be confused by an argument.
fn depends_on(manifest: &str, crate_named: &str) -> bool {
    manifest
        .lines()
        .any(|line| line.trim_start().starts_with(&format!("{crate_named} = ")))
}

/// Every Rust file under a crate's `src`, read.
fn every_source_of(crate_named: &str) -> Vec<(PathBuf, String)> {
    let mut found = Vec::new();
    let mut looking = vec![
        the_repository()
            .join("crates")
            .join(crate_named)
            .join("src"),
    ];
    while let Some(folder) = looking.pop() {
        let listed = fs::read_dir(&folder)
            .unwrap_or_else(|why| panic!("{} could not be read: {why}", folder.display()));
        for entry in listed {
            let at = entry.expect("a directory entry reads").path();
            if at.is_dir() {
                looking.push(at);
            } else if at.extension().is_some_and(|kind| kind == "rs") {
                let said = fs::read_to_string(&at)
                    .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()));
                found.push((at, said));
            }
        }
    }
    assert!(!found.is_empty(), "no source was found for {crate_named}");
    found
}

/// **The daemon reads a person's settings and cannot write them.** It names
/// this crate on purpose — that is how it learns which model answers — and the
/// half that must stay true is the direction: `Choosing` is the only door here
/// that writes, and nothing an agent's door can reach names it.
///
/// This is the check that fails the day somebody inside the daemon reaches for
/// the convenient thing, which is to have the machine write a person's choice
/// on their behalf.
#[test]
fn the_daemon_reads_these_settings_and_never_writes_them() {
    for (at, said) in every_source_of("alo-agentd") {
        assert!(
            !said.contains("Choosing"),
            "{} names alo_choosing::Choosing: the daemon now has a road to writing \
             a person's own settings, which is the person's to do and nobody else's",
            at.display()
        );
    }
}

/// **This crate cannot knock on anything.** No wire words, no daemon, no turn,
/// no record — so the absence of a knock is a fact about what ships rather than
/// about what anybody remembered not to call.
///
/// `alo-agentd` reads the file once a turn, so there is nothing to tell it;
/// what this holds is that nobody can quietly decide otherwise without the
/// dependency arriving here first, where it is visible.
#[test]
fn the_persons_writer_has_nothing_to_knock_with() {
    let ours = manifest("alo-choosing");
    for elsewhere in [
        "alo-protocol",
        "alo-agentd",
        "alo-turn",
        "alo-record",
        "alo-context",
        "alo-asking",
        "alo-capability",
    ] {
        assert!(
            !depends_on(&ours, elsewhere),
            "alo-choosing depends on {elsewhere}, and a person writing down their own \
             choice is not a conversation with a daemon"
        );
    }
}

/// **No verb was added, and there is nowhere here for one to be.** A person
/// picking which model answers their own questions is not an agent doing
/// something, and law 2's enumerated verbs are not where it belongs — the same
/// answer `alo-clipboard` gave about copy and paste.
///
/// The check is the convention `alo-by-hand` walks the workspace for, which is
/// a `src/verbs.rs` holding a `declare_into`: a crate that declares verbs and
/// was not handed in is a finding there, and a crate that declares none is one
/// this test keeps declaring none.
#[test]
fn nothing_here_declares_a_verb() {
    let verbs = the_repository()
        .join("crates")
        .join("alo-choosing")
        .join("src")
        .join("verbs.rs");
    assert!(
        !verbs.exists(),
        "alo-choosing declares verbs: changing what answers a person's questions \
         is theirs to do and is not something an agent asks for"
    );
    assert!(
        !depends_on(&manifest("alo-choosing"), "alo-capability"),
        "alo-choosing names the crate that decides what an agent may reach"
    );
}
