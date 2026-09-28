//! **Every `--bin` the recipe builds is a binary a named package holds.**
//!
//! # The release this is written for
//!
//! `main` could not build an image for about an hour, and twelve merges' worth
//! of nine green gates said nothing about it:
//!
//! ```text
//! error: no bin target named `alo-shipping` in `alo-agentd`, ... packages
//! help: available bin in `alo-software` package: alo-shipping
//! ```
//!
//! `cargo build --bin X` searches only the packages already named by
//! `--package`, and the package holding that binary was not among them. The
//! mistake is invisible to every gate for one reason: **nothing in the nine
//! gates builds the image**, and the image build is the thing nobody runs until
//! a release.
//!
//! That is the same hole `the_image_starts_what_it_ships` was written about, one
//! step further along. That file checks a unit's `ExecStart` names a path the
//! recipe *claims* to create; this one checks the recipe can actually produce
//! it.
//!
//! # It needs no image build, and that is the point
//!
//! `cargo metadata` resolves from `Cargo.lock` without compiling anything, so
//! this runs in the ordinary gates on any machine — which is exactly what the
//! image build is not.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The recipe, read rather than retyped.
const THE_RECIPE: &str = "image/Containerfile";

/// The workspace root, from this crate's own manifest.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the workspace root is two directories above this crate")
}

/// The recipe's text.
fn the_recipe() -> String {
    std::fs::read_to_string(the_repository().join(THE_RECIPE))
        .unwrap_or_else(|why| panic!("{THE_RECIPE} could not be read: {why}"))
}

/// Which binaries each package in this workspace holds.
fn the_binaries_by_package() -> BTreeMap<String, BTreeSet<String>> {
    let out = Command::new(env!("CARGO"))
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(the_repository())
        .output()
        .expect("cargo metadata runs where this workspace builds");
    assert!(out.status.success(), "cargo metadata failed");
    let read: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("cargo metadata answers in JSON");
    let mut held: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let packages = read
        .get("packages")
        .and_then(serde_json::Value::as_array)
        .expect("cargo metadata answers with a list of packages");
    for package in packages {
        let Some(name) = package.get("name").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let Some(targets) = package.get("targets").and_then(serde_json::Value::as_array) else {
            continue;
        };
        for target in targets {
            let is_binary = target
                .get("kind")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|kinds| kinds.iter().any(|k| k.as_str() == Some("bin")));
            if !is_binary {
                continue;
            }
            if let Some(binary) = target.get("name").and_then(serde_json::Value::as_str) {
                held.entry(name.to_owned())
                    .or_default()
                    .insert(binary.to_owned());
            }
        }
    }
    held
}

/// Each `cargo build` invocation in the recipe, as the packages and binaries it
/// names.
fn what_the_recipe_builds(recipe: &str) -> Vec<(BTreeSet<String>, BTreeSet<String>)> {
    let mut built = Vec::new();
    let joined: Vec<String> = {
        // A recipe line ending in a backslash continues; join before reading.
        let mut lines = Vec::new();
        let mut held = String::new();
        for line in recipe.lines() {
            let line = line.trim_end();
            if let Some(rest) = line.strip_suffix('\\') {
                held.push_str(rest);
                held.push(' ');
            } else {
                held.push_str(line);
                lines.push(std::mem::take(&mut held));
            }
        }
        lines
    };
    for line in joined {
        if !line.contains("cargo build") {
            continue;
        }
        let words: Vec<&str> = line.split_whitespace().collect();
        let mut packages = BTreeSet::new();
        let mut binaries = BTreeSet::new();
        for pair in words.windows(2) {
            let [flag, value] = pair else { continue };
            match *flag {
                "--package" | "-p" => {
                    packages.insert((*value).to_owned());
                }
                "--bin" => {
                    binaries.insert((*value).to_owned());
                }
                _ => {}
            }
        }
        if !packages.is_empty() || !binaries.is_empty() {
            built.push((packages, binaries));
        }
    }
    built
}

/// **A `--bin` the recipe asks for is held by a package it also names.**
///
/// `cargo build --bin X` searches only the packages `--package` named. Asking
/// for a binary whose package is absent fails the image build and no gate,
/// because no gate builds the image.
#[test]
fn every_binary_the_recipe_builds_is_in_a_package_it_names() {
    let recipe = the_recipe();
    let held = the_binaries_by_package();
    let asked = what_the_recipe_builds(&recipe);
    assert!(
        !asked.is_empty(),
        "{THE_RECIPE} runs no cargo build, which cannot be right"
    );

    let mut checked = 0_usize;
    for (packages, binaries) in &asked {
        for binary in binaries {
            checked += 1;
            let holder = packages
                .iter()
                .find(|package| held.get(*package).is_some_and(|b| b.contains(binary)));
            assert!(
                holder.is_some(),
                "{THE_RECIPE} builds --bin {binary} while naming only {packages:?}, \
                 and none of those holds it. cargo searches --bin among the packages \
                 --package named, so the image build fails with `no bin target named \
                 {binary}`. The package that holds it is {:?}.",
                held.iter()
                    .filter(|(_, b)| b.contains(binary))
                    .map(|(p, _)| p.as_str())
                    .collect::<Vec<_>>()
            );
        }
    }
    assert!(
        checked >= 1,
        "no --bin was checked, so this test proved nothing"
    );
}

/// **Every package the recipe names is a package this workspace has.**
///
/// A renamed crate leaves the recipe naming one that is gone, which fails the
/// image build for a reason no gate reaches.
#[test]
fn every_package_the_recipe_builds_exists() {
    let recipe = the_recipe();
    let held = the_binaries_by_package();
    let out = Command::new(env!("CARGO"))
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(the_repository())
        .output()
        .expect("cargo metadata runs");
    let read: serde_json::Value = serde_json::from_slice(&out.stdout).expect("JSON");
    let all: BTreeSet<String> = read
        .get("packages")
        .and_then(serde_json::Value::as_array)
        .expect("cargo metadata answers with a list of packages")
        .iter()
        .filter_map(|p| p.get("name").and_then(serde_json::Value::as_str))
        .map(str::to_owned)
        .collect();
    drop(held);

    let mut checked = 0_usize;
    for (packages, _) in what_the_recipe_builds(&recipe) {
        for package in packages {
            checked += 1;
            assert!(
                all.contains(&package),
                "{THE_RECIPE} builds --package {package}, which this workspace does not have"
            );
        }
    }
    assert!(checked >= 4, "only {checked} packages were checked");
}
