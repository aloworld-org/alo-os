//! A locked session is not a signed-out one, and this crate cannot make it one.
//!
//! `alo-locking` holds the person's session in both of a seat's states and has
//! no method that ends it — but *has no method* is a sentence about today's
//! code, and the failure it guards against is tomorrow's: a lock screen that
//! "tidies up" after a timeout by signing somebody out, taking their unsaved
//! work and their running turn with it. So this reads the crate's own source
//! and manifest for every road to ending a session this workspace has, and
//! fails on any of them.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

/// This crate's directory.
fn here() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every Rust file under `src`, with its text.
fn the_source() -> Vec<(PathBuf, String)> {
    let mut found = Vec::new();
    let Ok(entries) = fs::read_dir(here().join("src")) else {
        panic!("this crate's source could not be read");
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|extension| extension == "rs") {
            let Ok(text) = fs::read_to_string(&path) else {
                panic!("{} could not be read", path.display());
            };
            found.push((path, text));
        }
    }
    found
}

/// The lines of a file that are code rather than documentation.
fn code_of(text: &str) -> String {
    text.lines()
        .filter(|line| {
            let line = line.trim_start();
            !line.starts_with("//")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Everything in this workspace that ends, closes or kills a session, and
/// everything that would reach the machinery that does.
const ROADS_TO_ENDING_A_SESSION: [&str; 14] = [
    // `alo-sessiond`'s privileged half: the opener's door, the machine's
    // session manager and the opening itself.
    "Logind",
    "TheMachinesLogind",
    "Opening",
    "Door",
    "TheOpenersDoor",
    "THE_DOOR",
    // The rented session manager's own words for it.
    "TerminateSession",
    "KillSession",
    "TerminateUser",
    "logind",
    // A process of our own, or a signal to somebody else's.
    "std::process",
    "Command::new",
    "kill(",
    "zbus",
];

/// **Nothing in this crate's code names a road to ending a session.**
#[test]
fn nothing_here_can_end_a_session() {
    let source = the_source();
    assert!(
        source.iter().any(|(path, _)| path.ends_with("seat.rs")),
        "the source was not where this test looks for it"
    );
    for (path, text) in source {
        let code = code_of(&text);
        for road in ROADS_TO_ENDING_A_SESSION {
            assert!(
                !code.contains(road),
                "{} names {road}: a lock must never end the session it covers",
                path.display()
            );
        }
    }
}

/// **And nothing it depends on is there to reach one with.** The manifest names
/// no bus, no process machinery and no second opener.
#[test]
fn nothing_this_crate_depends_on_is_a_way_to_end_a_session() {
    let Ok(manifest) = fs::read_to_string(here().join("Cargo.toml")) else {
        panic!("this crate's manifest could not be read");
    };
    let dependencies: Vec<&str> = manifest
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect();
    for forbidden in ["zbus", "rustix", "nix", "libc", "signal-hook"] {
        assert!(
            !dependencies.iter().any(|line| line
                .trim_start()
                .starts_with(&format!("{forbidden} "))
                || line.trim_start().starts_with(&format!("{forbidden}="))),
            "this crate depends on {forbidden}"
        );
    }
}

/// **The test above is looking at real code.** A reader that found nothing
/// would pass on an empty directory, so it is shown refusing a line that
/// would end a session.
#[test]
fn the_reader_refuses_a_line_that_ends_a_session() {
    let written = "    logind.terminate(session);\n    // Logind in a comment is fine\n";
    let code = code_of(written);
    assert!(
        ROADS_TO_ENDING_A_SESSION
            .iter()
            .any(|road| code.contains(road))
    );
    assert!(!code_of("// Logind, only in prose").contains("Logind"));
    assert!(Path::new(env!("CARGO_MANIFEST_DIR")).join("src").is_dir());
}
