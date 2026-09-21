//! No notification is ever read by an agent as context.
//!
//! The plan's constraint for this task, in five words, and it is the one a
//! notification system makes tempting to break. Everything arriving for a
//! person passes through one place: who is trying to reach them, what about,
//! and when. An agent that could read that list would know more about somebody's
//! day than any verb in `docs/contracts/agent-verbs.md` could tell it, and it
//! would have learned it without a grant, without an invocation and without a
//! sentence anybody approved.
//!
//! ADR 0001 is categorical: context reaches an agent at the moment of
//! invocation, for that turn, and is never harvested. So this reads **the
//! manifest of every crate in the workspace** and fails on any crate an agent's
//! request is carried out in that depends on this one — and then reads those
//! crates' own shipped source for this crate's name, in case a reader arrives
//! one day without a dependency.
//!
//! # The agent may *send* one, and that is not the same thing
//!
//! `alo_notifying::arriving::from_the_agent` exists and is how *the invoices
//! are filed* reaches a person. Sending is the agent speaking; reading would be
//! the agent listening. The dependency goes one way — whoever holds the agent's
//! surface calls into this crate, and this crate calls nothing back — and the
//! test below is what keeps it that way.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::PathBuf;

/// The crates that may depend on this one: the collected vocabulary, so that
/// what this crate says reaches a person in their own language, and the shell,
/// which is the person's own session and is where a notification is drawn.
const MAY_DEPEND_ON_NOTIFYING: [&str; 2] = ["alo-saying", "alo-shell"];

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
fn depends_on_notifying(manifest: &str) -> bool {
    manifest
        .lines()
        .map(str::trim_start)
        .filter(|line| !line.starts_with('#'))
        .any(|line| line.starts_with("alo-notifying"))
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
fn no_crate_that_answers_an_agent_reaches_a_notification() {
    let manifests = every_manifest();
    for answering in WHERE_AN_AGENT_IS_ANSWERED {
        assert!(
            manifests.iter().any(|(name, _)| name == answering),
            "{answering} is not where this test looks for it"
        );
    }
    for (name, manifest) in &manifests {
        if name == "alo-notifying" {
            continue;
        }
        if depends_on_notifying(manifest) {
            assert!(
                MAY_DEPEND_ON_NOTIFYING.contains(&name.as_str()),
                "{name} depends on alo-notifying: what has been arriving for somebody today is \
                 not context an agent is offered"
            );
        }
    }
}

/// **The reader above is looking at real manifests.** A reader that never
/// matched would pass on anything, so it is shown one that does and one that
/// does not.
#[test]
fn the_reader_finds_a_dependency_when_there_is_one() {
    assert!(depends_on_notifying(
        "[dependencies]\nalo-notifying = { path = \"../alo-notifying\" }\n"
    ));
    assert!(!depends_on_notifying(
        "[dependencies]\n# alo-notifying is deliberately not here\nalo-turn = { path = \
         \"../alo-turn\" }\n"
    ));
}

/// **Nothing where an agent is answered names this crate or its file.** Not as
/// a dependency, not as a module of its own, and not as a path somebody typed
/// out.
#[test]
fn nothing_where_an_agent_is_answered_names_this_crate() {
    for crate_name in WHERE_AN_AGENT_IS_ANSWERED {
        for (at, text) in every_source_file(crate_name) {
            for named in ["alo_notifying", "alo-notifying", "notifying.toml"] {
                assert!(!text.contains(named), "{} names `{named}`", at.display());
            }
        }
    }
}
