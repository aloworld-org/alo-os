//! **`alo-drawing` never depends on Wayland, Smithay, a compositor or D-Bus** —
//! not by name in its manifest, and not through anything it pulls in.
//!
//! The crate exists so that the shell and alo's own applications can draw the
//! same pixels, and so that a test can draw them on any machine
//! (`docs/autonomy/the-alo-ui-kit-plan.md`, task 2). A dependency on a
//! compositor's crates would undo both: an application would link a server's
//! code, and a picture could not be drawn without a display.
//!
//! **The whole graph, from `Cargo.lock`, rather than the manifest alone**,
//! because the manifest names only what this crate asks for directly. A
//! harmless-looking crate that itself pulls in `wayland-client` passes a
//! manifest check and fails this one.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic naming what was measured is the failure being reported"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The start of a crate name this crate may never reach.
const REFUSED: [&str; 7] = [
    "smithay",
    "wayland",
    "calloop",
    "xkbcommon",
    "zbus",
    "dbus",
    "libseat",
];

/// Whether `name` is a crate this one may never reach.
fn refused(name: &str) -> bool {
    REFUSED.iter().any(|it| name.starts_with(it))
}

/// The workspace's `Cargo.lock`, read.
fn the_lock() -> toml::Table {
    let at = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.lock");
    let text = std::fs::read_to_string(&at).expect("the workspace's Cargo.lock");
    toml::from_str(&text).expect("Cargo.lock is TOML")
}

/// Every package in the lock, by name, with the names it depends on. Two
/// versions of one name are merged, which can only widen what is reached.
fn the_graph(lock: &toml::Table) -> BTreeMap<String, BTreeSet<String>> {
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .expect("Cargo.lock lists its packages");
    for package in packages {
        let name = package
            .get("name")
            .and_then(toml::Value::as_str)
            .expect("a package has a name");
        let depends = graph.entry(name.to_owned()).or_default();
        for dependency in package
            .get("dependencies")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(toml::Value::as_str)
        {
            // `name 1.2.3` where two versions are locked; the name is enough.
            let dependency = dependency.split(' ').next().unwrap_or(dependency);
            depends.insert(dependency.to_owned());
        }
    }
    graph
}

/// Every crate `from` reaches, itself included.
fn reached(graph: &BTreeMap<String, BTreeSet<String>>, from: &str) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut waiting = vec![from.to_owned()];
    while let Some(name) = waiting.pop() {
        if seen.insert(name.clone()) {
            waiting.extend(graph.get(&name).into_iter().flatten().cloned());
        }
    }
    seen
}

/// **Nothing `alo-drawing` pulls in is a compositor's, Wayland's or D-Bus's.**
#[test]
fn nothing_it_pulls_in_is_a_compositors() {
    let graph = the_graph(&the_lock());
    let reached = reached(&graph, "alo-drawing");
    let wrong: Vec<&String> = reached.iter().filter(|it| refused(it)).collect();
    assert!(
        wrong.is_empty(),
        "alo-drawing reaches {wrong:?}; it may not depend on a compositor"
    );
}

/// **The check can fail**: `alo-shell` is in the same lock and reaches
/// Smithay and Wayland, so the same walk from it finds them. Without this, a
/// lock the walk could not read would pass.
#[test]
fn the_same_walk_from_the_shell_finds_the_compositor() {
    let graph = the_graph(&the_lock());
    assert!(
        graph.contains_key("alo-drawing"),
        "the lock names this crate"
    );
    let seen = reached(&graph, "alo-shell");
    for expected in ["smithay", "wayland-server", "zbus"] {
        assert!(
            seen.contains(expected),
            "the walk from alo-shell did not reach {expected}"
        );
    }
}

/// **And the manifest names none of them either**, so the refusal is said where
/// somebody adding one would look.
#[test]
fn its_manifest_names_none_of_them() {
    let at = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let manifest: toml::Table =
        toml::from_str(&std::fs::read_to_string(at).expect("the manifest")).expect("TOML");
    for section in ["dependencies", "build-dependencies"] {
        let Some(named) = manifest.get(section).and_then(toml::Value::as_table) else {
            continue;
        };
        for name in named.keys() {
            if refused(name) {
                panic!("alo-drawing's [{section}] names {name}");
            }
        }
    }
}
