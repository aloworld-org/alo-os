//! **No daemon on this machine serves a person out of the file that had no
//! person in it.**
//!
//! `docs/decisions/0088-a-machines-grants-belong-to-a-person.md` moved a
//! machine's grants from `/var/lib/alo/grants.toml` to
//! `/var/lib/alo/grants-<uid>.toml`. The file side of that was built first and
//! **called by nothing**, and this test exists because of what the task that
//! scoped the caller found:
//!
//! > Two daemons read `THE_GRANTS`, not one … A migration wired into the agent
//! > alone leaves it reading a path that no longer holds anybody's grants, **and
//! > every test in `alo-remembering` would still pass**, because that crate
//! > knows neither daemon.
//!
//! That is the whole reason this is a test about *source* rather than about
//! behaviour. Every behavioural test either crate has passes in both worlds.
//! The thing that is wrong in the half-migrated world is **which path a daemon
//! names**, and nothing but reading the daemons can see it.
//!
//! # It strips comments before it matches, because that is where my checks fail
//!
//! `rereading.rs`, `starting.rs` and `changing.rs` all name `THE_GRANTS` in
//! prose, correctly — they are explaining the file's history. A per-line
//! `grep THE_GRANTS` over these crates reports all of them and is useless. So
//! the source is stripped of comments first and the match is made against what
//! is left, which is the rule this repository keeps relearning: **match the
//! stripped form, and strip both sides the same way.**
//!
//! # And it asserts the positive as well as the negative
//!
//! A test that only says *the old name is absent* passes on a file that reads
//! no grants at all — including an empty file, a deleted one, or a daemon that
//! was refactored into naming the path through a variable this test cannot see.
//! So each daemon must also be shown **naming the per-person reader**. Absence
//! and presence are different claims and a guard that makes only one of them is
//! half a guard.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// The daemons that serve one person, and must therefore read that person's
/// own grants.
///
/// `alo-agentd` is the one that answers an agent, and `alo-portald` is the one
/// that answers an application through a portal. ADR 0088 names the second
/// **zero times** — it was found by reading rather than by the record, which is
/// why it is named here where a reader of this test will meet it.
const EVERY_DAEMON_THAT_SERVES_A_PERSON: [&str; 2] = ["alo-agentd", "alo-portald"];

/// The name of the file a machine kept before anybody's grants were their own.
///
/// Spelled in pieces so this test's own source does not match the thing it
/// forbids — otherwise the guard reports itself, which is a failure mode this
/// repository has met before.
fn the_retired_name() -> String {
    format!("THE{}GRANTS", "_")
}

/// This repository, from the crate this test lives in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the repository is where this crate says it is")
}

/// The same Rust with every comment taken out.
///
/// Line comments (`//`, `///`, `//!`) and block comments, and **string
/// literals are left alone** — a path inside a string is code for this
/// purpose, because a daemon that opened a hardcoded `"/var/lib/alo/
/// grants.toml"` would be exactly the fault being looked for.
fn without_comments(rust: &str) -> String {
    let mut out = String::with_capacity(rust.len());
    let mut rest = rust;
    let mut in_a_block = false;

    while !rest.is_empty() {
        if in_a_block {
            match rest.find("*/") {
                Some(at) => {
                    rest = rest.get(at + 2..).unwrap_or_default();
                    in_a_block = false;
                }
                None => break,
            }
            continue;
        }
        let line_at = rest.find("//");
        let block_at = rest.find("/*");
        match (line_at, block_at) {
            (Some(line), Some(block)) if line < block => {
                out.push_str(rest.get(..line).unwrap_or_default());
                rest = rest.find('\n').map_or("", |at| {
                    out.push('\n');
                    rest.get(at + 1..).unwrap_or_default()
                });
            }
            (Some(_), Some(block)) => {
                out.push_str(rest.get(..block).unwrap_or_default());
                rest = rest.get(block + 2..).unwrap_or_default();
                in_a_block = true;
            }
            (Some(line), None) => {
                out.push_str(rest.get(..line).unwrap_or_default());
                rest = rest.find('\n').map_or("", |at| {
                    out.push('\n');
                    rest.get(at + 1..).unwrap_or_default()
                });
            }
            (None, Some(block)) => {
                out.push_str(rest.get(..block).unwrap_or_default());
                rest = rest.get(block + 2..).unwrap_or_default();
                in_a_block = true;
            }
            (None, None) => {
                out.push_str(rest);
                break;
            }
        }
    }
    out
}

/// Every `.rs` file under a crate's `src`, as (path relative to the repository,
/// source).
fn every_source(at: &Path, root: &Path, into: &mut Vec<(String, String)>) {
    let Ok(entries) = std::fs::read_dir(at) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            every_source(&path, root, into);
        } else if path.extension().is_some_and(|it| it == "rs")
            && let Ok(text) = std::fs::read_to_string(&path)
        {
            let named = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            into.push((named, text));
        }
    }
}

/// Every source file of the daemons that serve a person.
fn the_daemons_source() -> Vec<(String, String)> {
    let root = the_repository();
    let mut found = Vec::new();
    for daemon in EVERY_DAEMON_THAT_SERVES_A_PERSON {
        let src = root.join("crates").join(daemon).join("src");
        assert!(
            src.is_dir(),
            "{} has no src directory, so this guard is looking at nothing",
            src.display()
        );
        every_source(&src, &root, &mut found);
    }
    assert!(
        found.len() >= EVERY_DAEMON_THAT_SERVES_A_PERSON.len(),
        "read {} source files for {} daemons, which is a walk that stopped \
         rather than two daemons with no source",
        found.len(),
        EVERY_DAEMON_THAT_SERVES_A_PERSON.len()
    );
    found
}

/// **Neither daemon names the machine-wide grants file in code.**
#[test]
fn no_daemon_names_the_file_that_had_no_person_in_it() {
    let retired = the_retired_name();
    let mut naming = Vec::new();

    for (named, source) in the_daemons_source() {
        let code = without_comments(&source);
        if code.contains(&retired) || code.contains("/var/lib/alo/grants.toml") {
            naming.push(named);
        }
    }

    assert!(
        naming.is_empty(),
        "{} file(s) still name the grants file that had no person in it, in code \
         rather than in a comment about its history: {}.\n\nADR 0088 made a \
         person's grants `grants-<uid>.toml`; a daemon left on the old name \
         reads a file nobody's grants are in, and every behavioural test still \
         passes. Read it through `alo_remembering::the_persons_grants` instead.",
        naming.len(),
        naming.join(", ")
    );
}

/// **And each of them reads the per-person file**, so the absence above is a
/// migration rather than a daemon that stopped reading grants at all.
#[test]
fn every_daemon_that_serves_a_person_reads_that_persons_own_grants() {
    let root = the_repository();
    for daemon in EVERY_DAEMON_THAT_SERVES_A_PERSON {
        let src = root.join("crates").join(daemon).join("src");
        let mut found = Vec::new();
        every_source(&src, &root, &mut found);

        let reads = found
            .iter()
            .any(|(_, source)| without_comments(source).contains("the_persons_grants"));
        assert!(
            reads,
            "{daemon} names no per-person grants file anywhere in its {} source \
             files. Either it no longer reads grants at all — which would make \
             the other test in this file pass for the wrong reason — or it \
             reaches the path some way this guard cannot see, which is the same \
             problem one step further away.",
            found.len()
        );
    }
}
