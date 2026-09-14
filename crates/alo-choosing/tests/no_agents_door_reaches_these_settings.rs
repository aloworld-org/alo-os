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
//! **the daemon writes these settings in one file, and only the person's door
//! reaches it.** [`alo_choosing::Choosing`] is the only door in this crate that
//! touches a disk in that direction. Until task 15 of the local-network plan
//! nothing under `crates/alo-agentd/src` named it at all; that task gave the
//! person's shell a request that chooses a paired machine to answer their
//! questions, and the plan decided it is answered by the daemon *through
//! `alo-choosing`'s one way out*, because the pairings the choice is held to
//! live behind the daemon's lock and nowhere a shell can keep true. So one file
//! names it — `choosing_to_answer.rs` — and the only file that reaches that one
//! is `answering.rs`, the person's door. The agent's door (`doing.rs`), the
//! network's (`hearing.rs`, `questioned.rs`) and everything else reach neither.
//! That is read off this repository rather than asserted about it, which is the
//! shape `alo-collected` and `alo-citing` settled: a check that only ever reads
//! its own fixtures can pass while the disk says something else. The refusal on
//! the agent's door is the daemon's own test, beside the file.
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

/// The one file in the daemon that may name the writer.
const THE_PERSONS_DOOR_WRITES_HERE: &str = "choosing_to_answer.rs";

/// The one file in the daemon that may reach it: the person's door.
const THE_PERSONS_DOOR: &str = "answering.rs";

/// **The daemon writes a person's settings in one file, and only the person's
/// door reaches it.** It names this crate on purpose — that is how it learns
/// which model answers — and the half that must stay true is the direction:
/// `Choosing` is the only door here that writes, one file of the daemon names
/// it, and nothing but the person's door names that file.
///
/// This is the check that fails the day somebody inside the daemon reaches for
/// the convenient thing, which is to have the machine write a person's choice
/// on their behalf from anywhere but the person's own request.
#[test]
fn the_daemon_writes_these_settings_only_where_the_persons_door_asks() {
    let mut the_writer_found = false;
    for (at, said) in every_source_of("alo-agentd") {
        let name = at
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_owned();
        if name == THE_PERSONS_DOOR_WRITES_HERE {
            the_writer_found = true;
            continue;
        }
        assert!(
            !said.contains("Choosing"),
            "{} names alo_choosing::Choosing: the daemon now has a second road to writing \
             a person's own settings, which is the person's to do and nobody else's",
            at.display()
        );
        let reaches_it =
            said.contains("choosing_to_answer::") || said.contains("use crate::choosing_to_answer");
        assert!(
            !reaches_it || name == THE_PERSONS_DOOR,
            "{} reaches the file that writes a person's settings, and only the person's door may",
            at.display()
        );
    }
    assert!(
        the_writer_found,
        "the daemon's one writer of a person's settings moved: this check would be passing about \
         a file that is not there"
    );
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
