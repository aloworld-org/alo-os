//! **The keyring fixture is never in anything a machine runs.**
//!
//! `alo-keyring-fixture` starts a `gnome-keyring-daemon` with a password written
//! in its own source. That is exactly right for a test and would be a hole in a
//! shipped image, so *it is only ever a dev-dependency* is a claim worth
//! checking rather than remembering.
//!
//! Two things keep it out, and this file holds up both:
//!
//! 1. `image/Containerfile` builds `--package alo-agentd --package
//!    alo-boundaryd`, and a `--package` release build compiles **no**
//!    dev-dependency of those packages;
//! 2. no crate names it outside a `dev-dependencies` table, so it cannot become
//!    a real dependency by somebody moving one line.
//!
//! The second is what this file actually reads, because the first follows from
//! it: a crate nothing depends on normally cannot be built into anything.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// What must never appear outside a `dev-dependencies` table.
const THE_FIXTURE: &str = "alo-keyring-fixture";

/// The other thing that must not: the feature letting a test name a certificate
/// authority of its own.
///
/// Off, `alo-asking` trusts the compiled-in Mozilla roots and the seam is not
/// compiled at all. On in anything a machine runs, a credential-bearing
/// connection could be verified against an authority somebody else chose.
const THE_TRUST_SEAM: &str = "trust-a-test-authority";

/// Where the crates are, from this test's own location rather than a guess
/// about the working directory.
fn every_manifest() -> Vec<PathBuf> {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/alo-secrets has a parent")
        .to_owned();
    let mut found: Vec<PathBuf> = std::fs::read_dir(&crates)
        .expect("the crates directory can be read")
        .filter_map(|entry| {
            let at = entry.ok()?.path().join("Cargo.toml");
            at.is_file().then_some(at)
        })
        .collect();

    // The workspace root and the tools beside it, so that a binary somewhere
    // else in the repository is covered by the same rule.
    let root = crates.parent().expect("crates has a parent");
    found.push(root.join("Cargo.toml"));
    if let Ok(tools) = std::fs::read_dir(root.join("tools")) {
        found.extend(tools.filter_map(|entry| {
            let at = entry.ok()?.path().join("Cargo.toml");
            at.is_file().then_some(at)
        }));
    }
    found
}

/// **Nothing depends on the fixture except for tests.**
///
/// Read as sections rather than as lines: a manifest names the fixture legally
/// only under a table whose name ends in `dev-dependencies`, which covers both
/// the plain one and the `[target.'cfg(..)'.dev-dependencies]` this workspace
/// actually uses.
#[test]
fn the_fixture_and_the_trust_seam_are_named_only_under_dev_dependencies() {
    let mut wrong = Vec::new();

    for manifest in every_manifest() {
        // Its own manifest names it under `[package]`, which is what a package
        // is, and the workspace root lists it as a member so it gets built and
        // linted at all. Neither makes it reachable from a binary.
        if manifest.ends_with(format!("{THE_FIXTURE}/Cargo.toml")) {
            continue;
        }
        let written = std::fs::read_to_string(&manifest).expect("a manifest can be read");
        let mut table = String::new();
        for line in written.lines() {
            let bare = line.trim();
            if bare.starts_with('[') && bare.ends_with(']') {
                table = bare.trim_matches(['[', ']']).to_owned();
                continue;
            }
            // A comment mentioning the fixture is prose, not a dependency —
            // and this file's own reason for existing is written in several.
            let names_one = bare.contains(THE_FIXTURE) || bare.contains(THE_TRUST_SEAM);
            if bare.starts_with('#') || !names_one {
                continue;
            }
            if table == "workspace" || table.starts_with("workspace.") {
                continue;
            }
            // **Declaring a feature is not enabling one.** `[features]` is where
            // `alo-asking` says the seam exists at all, and it must. What is not
            // allowed here is a line turning on somebody else's feature, which
            // in this table is spelled `their-crate/their-feature` — so the
            // slash is the whole difference and the check is on the slash.
            if table == "features" && !bare.contains('/') {
                continue;
            }
            if !table.ends_with("dev-dependencies") {
                wrong.push(format!(
                    "{} names it under [{table}]: {bare}",
                    manifest.display()
                ));
            }
        }
    }

    assert!(
        wrong.is_empty(),
        "{THE_FIXTURE} starts a keyring daemon with a password in its own source, so it must \
         never be reachable from anything a machine runs — and it now is:\n{}",
        wrong.join("\n")
    );
}

/// **And the image builds only the two processes**, which is the other half.
///
/// If this ever became `--workspace`, every dev-dependency in the repository
/// would be compiled into the build that produces the image — so the assertion
/// is on the flags rather than on the outcome.
#[test]
fn the_image_builds_only_the_two_processes() {
    let containerfile = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("the workspace root")
        .join("image/Containerfile");
    let written = std::fs::read_to_string(&containerfile).expect("the Containerfile can be read");
    // Only what it *runs*. The comment above that line explains why a
    // `--workspace` build would be wrong, and a check that read comments would
    // fail on the sentence saying so.
    let runs: String = written
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        runs.contains("--package alo-agentd") && runs.contains("--package alo-boundaryd"),
        "the image no longer names the two processes it builds, so what it compiles is unknown"
    );
    assert!(
        !runs.contains("--workspace"),
        "the image builds the whole workspace, which pulls every dev-dependency — including a \
         fixture that starts a keyring with a password in its source"
    );

    // **And that command turns nothing on.** A `--features` on it could enable
    // the trust seam without any manifest naming it, which is the one road past
    // the check in the other test. Measured beside this: `cargo tree -e features
    // --edges no-dev` for `alo-agentd` mentions the seam **zero** times, and
    // once when dev-dependencies are resolved.
    //
    // Read from `cargo build` to the end of its continuations and no further.
    // The whole file will not do: `cargo install bpf-linker` a few lines above
    // carries `--features llvm-…`, which is a tool this image needs and has
    // nothing to do with what our workspace compiles. A check that read the file
    // whole failed on it, which is a check reporting the wrong thing rather than
    // a finding.
    let building: String = runs
        .lines()
        .skip_while(|line| !line.contains("cargo build"))
        .take_while(|line| line.trim_end().ends_with('\\') || line.contains("cargo build"))
        .chain(
            runs.lines()
                .skip_while(|line| !line.contains("cargo build"))
                .skip(1)
                .take(1),
        )
        .collect::<Vec<_>>()
        .join(" ");

    assert!(
        building.contains("cargo build"),
        "the image has no `cargo build` line, so what it compiles could not be read"
    );
    assert!(
        !building.contains("--features") && !building.contains("--all-features"),
        "the image's build command enables cargo features, so what it compiles is no longer \
         decided by the manifests the other test reads: {building}"
    );
}
