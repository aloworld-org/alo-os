//! `/var/lib/alo/pairings.toml` has one writer, and it is the daemon.
//!
//! A pairing is made by two people on two machines, and the daemon is the
//! thing that hears the second of them — so it writes the file, at the moment a
//! pairing is kept or revoked (`alo_remembering::pairings_kept`, called from
//! `alo-agentd/src/keeping_pairings.rs`). This crate revokes pairings by
//! **asking** the daemon, and ADR 0038 says that is the only road a surface
//! gets. A second writer on the person's side would be a second opinion about
//! which machines may ask this one, and no behaviour test notices one being
//! added somewhere else — so this reads the shipped source of every crate and
//! tool in the repository for one.
//!
//! Shipped means `src/`: a test that stands a pairings file up is not a writer
//! on anybody's machine. Comment lines are skipped, because this repository
//! argues in prose about the file it is careful with.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

/// The crates allowed to name a write of the pairings file: the daemon that
/// writes it, the crate that owns where it lives and how it is replaced, and
/// the crate that owns what its text is.
const ITS_OWNERS: [&str; 3] = ["alo-agentd", "alo-remembering", "alo-nearby"];

/// Words that are a write of the pairings file wherever they appear.
const WRITING_PAIRINGS: [&str; 3] = ["pairings_kept", "keeping::written", "pairings.toml"];

/// Words that are a write of some file — which, beside the pairings file's
/// name, would be a writer spelt another way.
const WRITING_A_FILE: [&str; 5] = [
    "fs::write",
    "File::create",
    "OpenOptions",
    "fs::rename",
    "alo_remembering::kept",
];

/// This repository, from the crate this test is in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("this crate is two directories below the repository")
}

/// Every `.rs` file under this folder.
fn rust_files_under(folder: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files_under(&path, into);
        } else if path.extension().is_some_and(|it| it == "rs") {
            into.push(path);
        }
    }
}

/// Every shipped source file, with the crate or tool it belongs to.
fn shipped_source() -> Vec<(String, PathBuf)> {
    let mut shipped = Vec::new();
    for parent in ["crates", "tools"] {
        let Ok(members) = fs::read_dir(the_repository().join(parent)) else {
            continue;
        };
        for member in members.flatten() {
            let name = member.file_name().to_string_lossy().into_owned();
            let mut files = Vec::new();
            rust_files_under(&member.path().join("src"), &mut files);
            shipped.extend(files.into_iter().map(|file| (name.clone(), file)));
        }
    }
    shipped
}

/// The file's code, without its comment lines.
fn code_of(file: &Path) -> String {
    fs::read_to_string(file)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", file.display()))
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Whether this code writes the pairings file, by name or by constant beside
/// a write.
fn writes_the_pairings(code: &str) -> bool {
    WRITING_PAIRINGS.iter().any(|word| code.contains(word))
        || (code.contains("THE_PAIRINGS") && WRITING_A_FILE.iter().any(|word| code.contains(word)))
}

/// **Nothing outside the daemon and the crates that own the file writes
/// `/var/lib/alo/pairings.toml`** — this crate included, whose revocation of a
/// pairing is a question to the daemon and nothing else.
#[test]
fn no_shipped_source_is_a_second_writer_of_the_pairings_file() {
    let second_writers: Vec<String> = shipped_source()
        .into_iter()
        .filter(|(member, _)| !ITS_OWNERS.contains(&member.as_str()))
        .filter(|(_, file)| writes_the_pairings(&code_of(file)))
        .map(|(_, file)| file.display().to_string())
        .collect();
    assert!(
        second_writers.is_empty(),
        "a second writer of the pairings file, which is the daemon's alone: {second_writers:?}"
    );
}

/// **The search finds the writer that exists**, so the test above failing to
/// find another is a measurement rather than a search that looked nowhere.
#[test]
fn the_search_finds_the_daemons_own_write() {
    let found: Vec<PathBuf> = shipped_source()
        .into_iter()
        .filter(|(member, _)| member == "alo-agentd")
        .map(|(_, file)| file)
        .filter(|file| writes_the_pairings(&code_of(file)))
        .collect();
    assert!(
        found
            .iter()
            .any(|file| file.ends_with("src/keeping_pairings.rs")),
        "the daemon's write of the pairings file was not found, so the search is blind: {found:?}"
    );
}

/// **The search would find a writer spelt with the constant**, not only one
/// spelt with the function — held against text, so the rule is tested
/// without planting a writer in the repository.
#[test]
fn a_writer_spelt_with_the_constant_is_found() {
    assert!(writes_the_pairings(
        "std::fs::write(alo_remembering::THE_PAIRINGS, text)"
    ));
    assert!(!writes_the_pairings(
        "alo_remembering::pairings_remembered(Path::new(THE_PAIRINGS), now)"
    ));
}
