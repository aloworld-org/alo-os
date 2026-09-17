//! An agent cannot keep this machine awake by asking.
//!
//! `alo_sleeping::Holding` has no door that takes an agent, a verb or a
//! duration, and a compile-fail example on it holds that a turn's hold is made
//! from a turn. What that cannot hold is the other road: an agent's service
//! reaching this crate at all — a verb, somewhere in the daemon an agent talks
//! to, that calls `Holding::a_turn` for as long as the model likes, or a turn
//! whose length an agent could choose.
//!
//! So this reads the manifest of every crate in the workspace, and fails on any
//! crate an agent's request is carried out in that depends on `alo-sleeping`.
//! The crates allowed to are the ones a person's session is: the shell that
//! holds the seat, and the vocabulary that collects every sentence.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

/// The crates that may depend on this one: the person's session, and the
/// collected vocabulary.
const MAY_DEPEND_ON_SLEEPING: [&str; 2] = ["alo-saying", "alo-shell"];

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
fn depends_on_sleeping(manifest: &str) -> bool {
    manifest
        .lines()
        .map(str::trim_start)
        .filter(|line| !line.starts_with('#'))
        .any(|line| line.starts_with("alo-sleeping"))
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
    found
}

/// **No crate an agent's request is answered in depends on this one**, and
/// only the person's session and the vocabulary do at all.
#[test]
fn no_crate_that_answers_an_agent_reaches_sleep() {
    let manifests = every_manifest();
    for answering in WHERE_AN_AGENT_IS_ANSWERED {
        assert!(
            manifests.iter().any(|(name, _)| name == answering),
            "{answering} is not where this test looks for it"
        );
    }
    for (name, manifest) in &manifests {
        if name == "alo-sleeping" {
            continue;
        }
        if depends_on_sleeping(manifest) {
            assert!(
                MAY_DEPEND_ON_SLEEPING.contains(&name.as_str()),
                "{name} depends on alo-sleeping: an agent's service must not be able to hold \
                 this machine awake"
            );
        }
    }
}

/// **The reader above is looking at real manifests.** A reader that never
/// matched would pass on anything, so it is shown one that does.
#[test]
fn the_reader_finds_a_dependency_when_there_is_one() {
    assert!(depends_on_sleeping(
        "[dependencies]\nalo-sleeping = { path = \"../alo-sleeping\" }\n"
    ));
    assert!(!depends_on_sleeping(
        "[dependencies]\n# alo-sleeping is deliberately not here\nalo-turn = { path = \"../alo-turn\" }\n"
    ));
    assert!(
        !Path::new(&the_crates())
            .join("alo-agentd")
            .join("src")
            .join("sleeping.rs")
            .exists(),
        "the agent's service has no sleeping of its own either"
    );
}
