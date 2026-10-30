//! **The desktop and the shell are built without the agent service, and
//! without the kernel program it carries.**
//!
//! Measured on 2026-10-30: the image build of `main` failed at the compositor
//! step, `cargo build --release --package alo-shell --package alo-desktop`,
//! with `linker bpf-linker not found` while compiling `alo-bounding-kernel`.
//! #1027 had made `alo-desktop` depend on `alo-agentd` for one module, and that
//! pulled the agent service's kernel half into a build that has no BPF linker
//! and should never need one. Main could not produce a bootable image.
//!
//! The module now lives in `alo-interfaces`. This file is what stops the road
//! coming back: it walks every manifest the two binaries' builds reach —
//! ordinary, per-target and build dependencies, never dev-dependencies, which
//! the image does not build — and fails if any of them is `alo-agentd` or an
//! `alo-bounding*` crate.
//!
//! # The walk proves it walked
//!
//! A walk that read nothing would find nothing and pass. So the same walk is
//! run from `alo-agentd`, where it **must** find `alo-bounding` — whose build
//! script compiles `alo-bounding-kernel` — and the
//! desktop's walk must reach a known crate (`alo-interfaces`) through more than
//! a handful of manifests. Read off the repository's manifests rather than
//! `cargo metadata`, which is the shape every other guard here takes
//! (`alo-choosing`'s `no_agents_door_reaches_these_settings.rs`).

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// This repository's `crates` folder, from the crate this test is in.
fn the_crates() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .expect("this crate is one directory below `crates`")
}

/// The repository crates this one's manifest depends on in a build — every
/// `[...dependencies]` table except a `[...dev-dependencies]` one, read by the
/// `path = "../<crate>"` each entry names.
fn built_with(crate_named: &str) -> Vec<String> {
    let at = the_crates().join(crate_named).join("Cargo.toml");
    let manifest = fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()));
    let mut in_a_build_table = false;
    let mut found = Vec::new();
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            in_a_build_table =
                line.ends_with("dependencies]") && !line.ends_with("dev-dependencies]");
            continue;
        }
        if !in_a_build_table || line.starts_with('#') {
            continue;
        }
        if let Some((_, after)) = line.split_once("path = \"../")
            && let Some((named, _)) = after.split_once('"')
        {
            found.push(named.to_owned());
        }
    }
    found
}

/// Every repository crate a build of `roots` compiles, the roots included.
fn everything_built_with(roots: &[&str]) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut waiting: Vec<String> = roots.iter().map(|root| (*root).to_owned()).collect();
    while let Some(next) = waiting.pop() {
        if seen.insert(next.clone()) {
            waiting.extend(built_with(&next));
        }
    }
    seen
}

/// Whether this crate is the agent service or a part of its kernel boundary.
fn is_the_agent_service_or_its_kernel_half(named: &str) -> bool {
    named == "alo-agentd" || named.starts_with("alo-bounding")
}

/// **Neither binary the image's compositor step builds reaches the agent
/// service or a kernel-boundary crate**, directly or through anything else.
#[test]
fn the_desktop_and_the_shell_reach_neither_the_agent_service_nor_its_kernel_half() {
    let built = everything_built_with(&["alo-desktop", "alo-shell"]);
    let reached: Vec<&String> = built
        .iter()
        .filter(|named| is_the_agent_service_or_its_kernel_half(named))
        .collect();
    assert!(
        reached.is_empty(),
        "the image's compositor step now builds {reached:?}, which needs a BPF linker that step \
         does not have — main would not produce a bootable image (2026-10-30)"
    );

    // And the walk walked: it reached the crate the desktop really reads this
    // machine's addresses through, among many others.
    assert!(built.contains("alo-interfaces"), "{built:?}");
    assert!(built.len() > 20, "the walk read only {built:?}");
}

/// **The control: the same walk from the agent service finds its kernel
/// half**, so the test above is a walk that would have found it.
///
/// What it finds is `alo-bounding`, and that is the crate that matters:
/// `alo-bounding-kernel` is not a manifest dependency of anything, it is
/// compiled by `alo-bounding`'s `build.rs` — which is where the image's
/// `bpf-linker not found` came from. So the build script is read too, so this
/// cannot pass the day the kernel half is compiled somewhere else.
#[test]
fn the_same_walk_from_the_agent_service_finds_its_kernel_half() {
    let built = everything_built_with(&["alo-agentd"]);
    assert!(built.contains("alo-bounding"), "{built:?}");
    assert!(
        built
            .iter()
            .any(|named| is_the_agent_service_or_its_kernel_half(named) && named != "alo-agentd"),
        "{built:?}"
    );

    let build_script = the_crates().join("alo-bounding").join("build.rs");
    let said = fs::read_to_string(&build_script)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", build_script.display()));
    assert!(
        said.contains("alo-bounding-kernel"),
        "alo-bounding's build no longer compiles the kernel half; the crate that does is the one \
         the desktop must not reach, and this guard names the wrong one"
    );
}
