//! The agent never reads what was open.
//!
//! *What were you doing yesterday* is not a question an agent on this machine
//! can answer, and the reason is not that nobody wrote the verb. `alo-leaving`
//! keeps a list of the applications a person had open and where; ADR 0001 says
//! context reaches an agent at the moment of invocation, for that turn, and is
//! never harvested. A reader of this list anywhere in the daemon an agent talks
//! to would be exactly the background reader `CLAUDE.md` calls a bug in this
//! product.
//!
//! So this reads **the manifest of every crate in the workspace** and fails on
//! any crate an agent's request is carried out in that depends on this one — and
//! then reads `alo-agentd`'s own shipped source for the file's name, in case the
//! reader arrives one day without a dependency, as a path typed out by hand.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

/// The crates that may depend on this one: the collected vocabulary, so that
/// what this crate says reaches a person in their own language, and the shell,
/// which is the person's own session and is where a log-out is drawn.
const MAY_DEPEND_ON_LEAVING: [&str; 2] = ["alo-saying", "alo-shell"];

/// The crates an agent's request is carried out in, which must not — named so
/// that a rename of any of them fails here rather than passes quietly.
const WHERE_AN_AGENT_IS_ANSWERED: [&str; 5] = [
    "alo-agentd",
    "alo-turn",
    "alo-capability",
    "alo-protocol",
    "alo-broker",
];

/// The workspace's `crates` directory.
fn the_crates() -> PathBuf {
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let Some(crates) = here.parent() else {
        panic!("this crate is not inside a crates directory");
    };
    crates.to_path_buf()
}

/// Whether a manifest's code — not its comments — names this crate.
fn depends_on_leaving(manifest: &str) -> bool {
    manifest
        .lines()
        .map(str::trim_start)
        .filter(|line| !line.starts_with('#'))
        .any(|line| line.starts_with("alo-leaving"))
}

/// Every crate's name and manifest text.
fn every_manifest() -> Vec<(String, String)> {
    let Ok(entries) = fs::read_dir(the_crates()) else {
        panic!("the workspace's crates could not be read");
    };
    let mut found = Vec::new();
    for entry in entries.flatten() {
        let manifest = entry.path().join("Cargo.toml");
        if let Ok(text) = fs::read_to_string(&manifest) {
            found.push((entry.file_name().to_string_lossy().into_owned(), text));
        }
    }
    assert!(found.len() > 50, "the workspace's crates were not read");
    found
}

/// Every file of a crate's shipped source, by path.
fn every_source_file(crate_name: &str) -> Vec<(PathBuf, String)> {
    let src = the_crates().join(crate_name).join("src");
    let mut found = Vec::new();
    let mut looking = vec![src];
    while let Some(folder) = looking.pop() {
        let Ok(entries) = fs::read_dir(&folder) else {
            continue;
        };
        for entry in entries.flatten() {
            let at = entry.path();
            if at.is_dir() {
                looking.push(at);
            } else if at.extension().is_some_and(|it| it == "rs")
                && let Ok(text) = fs::read_to_string(&at)
            {
                found.push((at, text));
            }
        }
    }
    assert!(
        !found.is_empty(),
        "{crate_name} has no source this test could read"
    );
    found
}

/// **No crate an agent's request is answered in depends on this one**, and only
/// the vocabulary and the person's own session may at all.
#[test]
fn no_crate_that_answers_an_agent_reaches_what_was_open() {
    let manifests = every_manifest();
    for answering in WHERE_AN_AGENT_IS_ANSWERED {
        assert!(
            manifests.iter().any(|(name, _)| name == answering),
            "{answering} is not where this test looks for it"
        );
    }
    for (name, manifest) in &manifests {
        if name == "alo-leaving" {
            continue;
        }
        if depends_on_leaving(manifest) {
            assert!(
                MAY_DEPEND_ON_LEAVING.contains(&name.as_str()),
                "{name} depends on alo-leaving: a list of what somebody had open yesterday is not \
                 context an agent is offered"
            );
        }
    }
}

/// **The reader above is looking at real manifests.** A reader that never
/// matched would pass on anything, so it is shown one that does and one that
/// does not.
#[test]
fn the_reader_finds_a_dependency_when_there_is_one() {
    assert!(depends_on_leaving(
        "[dependencies]\nalo-leaving = { path = \"../alo-leaving\" }\n"
    ));
    assert!(!depends_on_leaving(
        "[dependencies]\n# alo-leaving is deliberately not here\nalo-turn = { path = \"../alo-turn\" }\n"
    ));
}

/// **Nothing in `alo-agentd` names this crate's file.** Not as a dependency, not
/// as a module of its own, and not as a path somebody typed out: the daemon that
/// answers an agent has no reader of `leaving.toml`, however it might have
/// arrived at one.
#[test]
fn nothing_in_the_agents_daemon_names_the_file() {
    for crate_name in WHERE_AN_AGENT_IS_ANSWERED {
        for (at, text) in every_source_file(crate_name) {
            for named in ["leaving.toml", "was-open", "alo_leaving"] {
                assert!(!text.contains(named), "{} names `{named}`", at.display());
            }
        }
    }
    assert!(
        !Path::new(&the_crates())
            .join("alo-agentd")
            .join("src")
            .join("leaving.rs")
            .exists(),
        "the agent's service has no leaving of its own either"
    );
}
