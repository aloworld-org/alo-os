//! Setup writes into the person's own settings and nowhere else, and nothing an
//! agent can send reaches it.
//!
//! The acceptance for this work says *choosing writes through `alo-choosing`'s
//! own shapes into the person's own file and nowhere else*, and "nowhere else"
//! is not a thing a unit test can measure: a second store added later would
//! have its own tests and they would pass. So it is read off this repository —
//! the manifest and the source — which is the shape `alo-collected`,
//! `alo-citing` and `alo-choosing`'s own manifest test settled: a check that
//! only ever reads its own fixtures can pass while the disk says something else.
//!
//! Three things are held here:
//!
//! - **there is one writer**, and it is `alo_choosing::Choosing`. Nothing in
//!   this crate opens, creates or renames a file;
//! - **there is no wire**, so nothing an agent sends can answer setup on
//!   somebody's behalf — which would be a model deciding whether its owner
//!   wanted an agent;
//! - **no verb was added**, because a person answering a question about their
//!   own machine is not an agent doing something (law 2).

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

/// Every Rust file under this crate's `src`, read.
fn every_source() -> Vec<(PathBuf, String)> {
    let mut found = Vec::new();
    let mut looking = vec![
        the_repository()
            .join("crates")
            .join("alo-setting-up")
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
    assert!(!found.is_empty(), "no source was found for alo-setting-up");
    found
}

/// **Setup opens no file of its own.** Every byte it causes to be written goes
/// out through `alo_choosing::Choosing`, which is the one writer of a person's
/// settings in this workspace — so there is no second store for a release to
/// move half of, and no second shape of that file for a reader to disagree
/// with.
///
/// **Comments are not code, and the check reads only code.** The fixture that
/// writes a real file lives in `src/testing.rs` and is `cfg(test)`, so it is
/// named as the one exception; the example in `src/lib.rs` writes a settings
/// file through the same door a person does, and lives in a doc comment, which
/// is skipped along with every other line of prose in this crate. A check that
/// read prose would fire on this file's own argument.
#[test]
fn nothing_here_writes_a_file_except_through_the_persons_own_settings() {
    for (at, said) in every_source() {
        if at.file_name().is_some_and(|named| named == "testing.rs") {
            continue;
        }
        // What ships, which is everything above the crate's own tests — those
        // put real files on a real disk on purpose, and a machine a person
        // signs in to has none of them compiled into it.
        let ships = said.split("#[cfg(test)]").next().unwrap_or_default();
        let code: String = ships
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        for road in [
            "fs::write",
            "File::create",
            "OpenOptions",
            "create_dir",
            "fs::rename",
            "fs::remove",
        ] {
            assert!(
                !code.contains(road),
                "{} names {road}: setup now has a road to a disk that is not the person's own \
                 settings, and a second writer of what somebody chose",
                at.display()
            );
        }
    }
}

/// **Setup has nothing to speak with.** No wire words, no daemon, no turn, no
/// record — so an agent cannot answer setup, and the absence is a fact about
/// what ships rather than about what anybody remembered not to call.
///
/// A verb that could answer this would be a model deciding whether its owner
/// wanted a model, which is the one question it can have no standing in.
#[test]
fn nothing_an_agent_sends_can_answer_setup() {
    let ours = manifest("alo-setting-up");
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
            "alo-setting-up depends on {elsewhere}, and a person answering a question about \
             their own machine is not a conversation with a daemon"
        );
    }
}

/// **No verb was added, and there is nowhere here for one to be.** The check is
/// the convention `alo-by-hand` walks the workspace for, which is a
/// `src/verbs.rs` holding a `declare_into`.
#[test]
fn nothing_here_declares_a_verb() {
    let verbs = the_repository()
        .join("crates")
        .join("alo-setting-up")
        .join("src")
        .join("verbs.rs");
    assert!(
        !verbs.exists(),
        "alo-setting-up declares verbs: what a person answers at setup is theirs to answer and \
         is not something an agent asks for"
    );
}

/// **The daemon does not reach setup either.** `alo-agentd` reads a person's
/// settings and has no way to write them, which `alo-choosing`'s own manifest
/// test holds one crate down; what is held here is the half this crate adds,
/// which is that nothing inside the daemon names the value that answers setup.
#[test]
fn the_daemon_has_no_road_to_answering_setup() {
    let mut looking = vec![
        the_repository()
            .join("crates")
            .join("alo-agentd")
            .join("src"),
    ];
    let mut read = 0_usize;
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
                read += 1;
                assert!(
                    !said.contains("SettingUp"),
                    "{} names alo_setting_up::SettingUp: the daemon now has a road to answering \
                     setup on somebody's behalf",
                    at.display()
                );
            }
        }
    }
    assert!(read > 0, "no source was found for alo-agentd");
}
