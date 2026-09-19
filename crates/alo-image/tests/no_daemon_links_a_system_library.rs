//! **No daemon this image installs may reach a crate that links a system
//! library**, because the image links them statically against musl, where those
//! libraries do not exist.
//!
//! # The release this is written for
//!
//! Release 0.0.3 would not build. `alo-agentd` failed to link with `cannot find
//! -linput`. That daemon draws nothing and reads no touchpad: libinput reached it
//! through `alo-saying`, which collects the machine's one vocabulary **by
//! depending on every crate that declares any**, and `alo-agentd` depends on
//! `alo-saying`. So `alo-desktops` taking a touchpad library made an agent daemon
//! unbuildable.
//!
//! **The nine gates passed the whole time**, because none of them builds the
//! image. The fix was one word in a manifest — an optional dependency behind a
//! feature — and it left nothing to stop the next crate doing the same thing.
//! This is that something.
//!
//! # Why an allowlist rather than a ban
//!
//! *Declares a `links` key* is the obvious rule and it is wrong. Three crates in
//! this workspace declare one and they are not alike:
//!
//! - **`ring`** compiles its own C and links what it compiled. Nothing has to
//!   exist on the machine, and it builds against musl. It is reached by every
//!   daemon that speaks TLS, and banning it would ban the product.
//! - **`libudev-sys`** links the system's `libudev`, which musl images do not
//!   have. This is the one that broke the release, through `input`.
//!
//! So the rule is a **closed set with a reason each**. Anything reaching a
//! `links` crate that is not on the list fails here, and adding to the list means
//! writing down why that library is safe to need.
//!
//! # It needs no image build
//!
//! `cargo metadata` resolves from `Cargo.lock` and evaluates `cfg` for a target
//! without that target being installed and without compiling anything, so this
//! runs in the ordinary gates on any machine — which is the whole point, the
//! image build being the thing nobody runs until a release.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The recipe, which says which target and which daemons — read rather than
/// retyped, so a daemon added there is covered here without anybody remembering.
const THE_RECIPE: &str = "image/Containerfile";

/// The build argument naming the target the daemons are linked for.
const THE_TARGET_ARG: &str = "ARG THE_TARGET=";

/// Every crate that may declare a `links` key in a daemon's tree, and why it is
/// not the fault this file exists to catch.
///
/// **To add one, write the reason.** A library that must already be on the
/// machine does not belong here at all; the image links statically, and there is
/// no machine underneath to have it.
const MAY_LINK: [(&str, &str); 1] = [(
    "ring",
    "compiles its own C and links what it compiled — nothing has to exist on the \
     machine, and it builds against musl",
)];

/// This repository.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("this crate is two directories below the repository")
}

/// The recipe, as published.
fn the_recipe() -> String {
    std::fs::read_to_string(the_repository().join(THE_RECIPE))
        .unwrap_or_else(|why| panic!("{THE_RECIPE} could not be read: {why}"))
}

/// The target the image links its daemons for, from the recipe itself.
fn the_target(recipe: &str) -> String {
    recipe
        .lines()
        .find_map(|line| line.trim().strip_prefix(THE_TARGET_ARG))
        .map(str::trim)
        .map(str::to_owned)
        .expect("the recipe names the target its daemons are built for")
}

/// Every binary the recipe installs onto the machine, by the name it is built
/// under.
///
/// Read from the `COPY` lines that land something out of the build stage in
/// `/usr/bin` or `/usr/libexec`: those are the programs that end up on a
/// person's machine, and they are exactly the ones whose link must succeed.
fn the_daemons(recipe: &str) -> BTreeSet<String> {
    recipe
        .lines()
        .filter(|line| line.starts_with("COPY") && line.contains("/release/"))
        .filter(|line| line.contains("/usr/bin/") || line.contains("/usr/libexec/"))
        .filter_map(|line| {
            line.split_whitespace()
                .find(|word| word.contains("/release/"))
                .and_then(|word| word.rsplit('/').next())
                .map(str::to_owned)
        })
        .collect()
}

/// What `cargo metadata` said, parsed.
fn metadata(arguments: &[&str]) -> serde_json::Value {
    let output = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned()))
        .args(["metadata", "--format-version", "1"])
        .args(arguments)
        .current_dir(the_repository())
        .output()
        .expect("cargo metadata runs where this workspace builds");
    assert!(
        output.status.success(),
        "cargo metadata refused: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("cargo metadata answers in JSON")
}

/// Every package `cargo metadata` listed.
fn packages(read: &serde_json::Value) -> impl Iterator<Item = &serde_json::Value> {
    read.get("packages")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
}

/// Which package builds each binary in the workspace.
fn packages_by_binary() -> BTreeMap<String, String> {
    let read = metadata(&["--no-deps"]);
    let mut found = BTreeMap::new();
    for package in packages(&read) {
        let Some(name) = package.get("name").and_then(serde_json::Value::as_str) else {
            continue;
        };
        for target in package
            .get("targets")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
        {
            let is_a_binary = target
                .get("kind")
                .and_then(serde_json::Value::as_array)
                .into_iter()
                .flatten()
                .any(|kind| kind.as_str() == Some("bin"));
            if is_a_binary
                && let Some(binary) = target.get("name").and_then(serde_json::Value::as_str)
            {
                found.insert(binary.to_owned(), name.to_owned());
            }
        }
    }
    found
}

/// Every crate reached from this one, following the resolved graph.
fn everything_reached_from(read: &serde_json::Value, root: &str) -> BTreeSet<String> {
    let mut deps_of: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for node in read
        .get("resolve")
        .and_then(|resolve| resolve.get("nodes"))
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(id) = node.get("id").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let reaches = node
            .get("deps")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|dep| dep.get("pkg").and_then(serde_json::Value::as_str))
            .collect();
        deps_of.insert(id, reaches);
    }

    let start = packages(read)
        .find(|package| {
            package.get("name").and_then(serde_json::Value::as_str) == Some(root)
        })
        .and_then(|package| package.get("id").and_then(serde_json::Value::as_str))
        .unwrap_or_else(|| panic!("{root} is a package in this workspace"));

    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut waiting = vec![start];
    while let Some(id) = waiting.pop() {
        if !seen.insert(id) {
            continue;
        }
        for next in deps_of.get(id).into_iter().flatten() {
            waiting.push(next);
        }
    }
    seen.into_iter().map(str::to_owned).collect()
}

/// The `links` a set of package ids declares, by crate name.
fn links_among(read: &serde_json::Value, ids: &BTreeSet<String>) -> BTreeMap<String, String> {
    let mut found = BTreeMap::new();
    for package in packages(read) {
        let (Some(id), Some(name)) = (
            package.get("id").and_then(serde_json::Value::as_str),
            package.get("name").and_then(serde_json::Value::as_str),
        ) else {
            continue;
        };
        if !ids.contains(id) {
            continue;
        }
        if let Some(links) = package.get("links").and_then(serde_json::Value::as_str) {
            found.insert(name.to_owned(), links.to_owned());
        }
    }
    found
}

/// **No daemon the image installs reaches a crate that links a system library.**
#[test]
fn no_daemon_the_image_installs_links_a_system_library() {
    let recipe = the_recipe();
    let target = the_target(&recipe);
    let daemons = the_daemons(&recipe);
    assert!(
        !daemons.is_empty(),
        "{THE_RECIPE} installs no daemon, which cannot be right"
    );

    let by_binary = packages_by_binary();
    let read = metadata(&["--filter-platform", &target]);
    let allowed: BTreeMap<&str, &str> = MAY_LINK.into_iter().collect();

    for daemon in &daemons {
        let package = by_binary
            .get(daemon)
            .unwrap_or_else(|| panic!("{daemon} is installed by {THE_RECIPE} and built by nothing"));
        let reached = everything_reached_from(&read, package);
        for (crate_named, library) in links_among(&read, &reached) {
            assert!(
                allowed.contains_key(crate_named.as_str()),
                "`{daemon}` (package `{package}`) reaches `{crate_named}`, which links the system \
                 library `{library}`. The image links its daemons statically against `{target}`, \
                 where that library does not exist, so this is a release that will not build — \
                 and no gate builds the image, so nothing else would have said so.\n\n\
                 This is how `cannot find -linput` reached release 0.0.3: a crate took a system \
                 library, and `alo-saying` carried it into every daemon by collecting the \
                 vocabulary through a dependency edge.\n\n\
                 Put the dependency behind a feature the daemons do not enable, as \
                 `crates/alo-desktops/Cargo.toml` does — or, if the library really is safe to \
                 need, add `{crate_named}` to MAY_LINK in this file with the reason why."
            );
        }
    }
}

/// **The allowlist is a list of reasons, not a list of names.**
///
/// An entry with an empty reason is somebody silencing this test, which is the
/// one way it can be made useless without being deleted.
#[test]
fn every_crate_allowed_to_link_says_why() {
    for (crate_named, why) in MAY_LINK {
        assert!(
            why.len() > 30,
            "{crate_named} is allowed to link with no reason worth reading"
        );
    }
}

/// **The recipe still names daemons and a target this test can read.**
///
/// If the recipe changes shape, this test would quietly check nothing — the
/// failure mode of every test that parses another file. So the parsing is
/// asserted rather than assumed.
#[test]
fn the_recipe_still_says_what_this_test_reads() {
    let recipe = the_recipe();
    let target = the_target(&recipe);
    assert!(
        target.contains("musl"),
        "the image no longer links against musl ({target}), so re-read whether this rule still \
         applies: it exists because a static musl binary has no system libraries under it"
    );
    let daemons = the_daemons(&recipe);
    assert!(
        daemons.len() >= 4,
        "only {} daemons were read out of {THE_RECIPE}: {daemons:?}",
        daemons.len()
    );
}
