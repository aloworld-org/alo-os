//! **Nothing stands behind these words but the verbs and a digest.**
//!
//! The reason this crate exists is that the daemon must be able to take the
//! words a model is shown without taking a measurement harness with them
//! ([ADR 0037](../../../docs/decisions/0037-the-words-a-turn-shows-a-model-are-the-products-own.md)).
//! That is a claim about a dependency list, so it is read here rather than
//! promised in a crate header: the shipped crate carries `alo-capability` and
//! `ring`, and this crate's own source opens no socket, reads no file and
//! builds no request.
//!
//! A dependency added above would be asked, here, why the words a model is
//! shown need it.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

/// What the shipped crate carries, and the whole of it.
const WHAT_IT_CARRIES: [&str; 2] = ["alo-capability", "ring"];

/// What would let this crate reach anything but its caller.
const A_WAY_OFF_THE_WORDS: [&str; 12] = [
    "std::net",
    "std::fs",
    "std::process",
    "TcpStream",
    "UdpSocket",
    "ureq",
    "reqwest",
    "hyper",
    "Asking",
    "Catalogue",
    "read_to_string",
    "include_str!(\"../..",
];

/// **The dependency list is the two this crate needs.**
#[test]
fn the_shipped_crate_carries_the_verbs_and_a_digest_and_nothing_else() {
    let manifest =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();
    let dependencies = manifest
        .split("[dependencies]")
        .nth(1)
        .unwrap()
        .split("\n[")
        .next()
        .unwrap();
    let carried: Vec<&str> = dependencies
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.split('=').next().unwrap().trim())
        .collect();
    assert_eq!(carried, WHAT_IT_CARRIES, "{dependencies}");
}

/// **What a model is shown is built, not fetched.** No socket, no file, no
/// process: the words are the instructions in this crate, the registry the
/// caller hands over, and the request the caller was given.
#[test]
fn nothing_in_this_crate_reaches_anything() {
    for (path, text) in the_shipped_source() {
        for way in A_WAY_OFF_THE_WORDS {
            assert!(
                !text.contains(way),
                "{} names {way}",
                path.file_name().unwrap().to_string_lossy()
            );
        }
    }
}

/// This crate's shipped source, file by file. `testing.rs` is `cfg(test)` and
/// is read like any other: a fixture that opened a socket would still be a
/// socket in this crate's source tree.
fn the_shipped_source() -> Vec<(PathBuf, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    let mut folders = vec![root];
    while let Some(folder) = folders.pop() {
        for entry in fs::read_dir(&folder).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                folders.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let text = fs::read_to_string(&path).unwrap();
                files.push((path, text));
            }
        }
    }
    files
}
