//! **Every source file and Rust name this repository's documents point at,
//! against the disk it is checked out on.**
//!
//! `src/sources.rs` shows both refusals happening against fixtures. This is
//! where the rule meets the real documents — the half that catches a box whose
//! stated evidence names a file that moved or a symbol that was renamed.
//!
//! # Why this exists
//!
//! On 2026-09-30, six of seven documents caught disagreeing with the code had
//! the code in the right and the document in the wrong. An unticked roadmap box
//! is a claim somebody made once that nothing re-reads, and its stated reason is
//! prose — so nothing could even parse it, let alone check it.
//!
//! This checks the half of that prose which is checkable: **the pointers.** A
//! pass here means the evidence exists, not that the conclusion follows.
//!
//! # It counts what it read, and refuses to pass on nothing
//!
//! A walk that quietly finds nothing reports nothing, and a green bar from a
//! check that stopped looking is indistinguishable from one that looked. The
//! count is asserted before the findings are.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

use alo_citing::sources::{Named, cited_in, held};

/// Fewer pointers than this means the walk is broken, not that the repository
/// stopped naming its own code. Measured on 2026-09-30.
const TOO_FEW_TO_BELIEVE: usize = 200;

/// This repository, from the crate this test is in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the repository is where this crate says it is")
}

/// What is not this repository's own text.
fn is_not_ours(name: &str) -> bool {
    matches!(name, ".git" | "target" | "node_modules" | "engines") || name.starts_with("target-")
}

/// Every file under `directory`, as a repository-relative path, and the text of
/// the ones whose names end in `reading`.
fn walk(
    directory: &Path,
    below: &str,
    reading: &[&str],
    paths: &mut Vec<String>,
    texts: &mut Vec<(String, String)>,
) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = if below.is_empty() {
            name.clone()
        } else {
            format!("{below}/{name}")
        };
        if entry.path().is_dir() {
            if !is_not_ours(&name) {
                walk(&entry.path(), &path, reading, paths, texts);
            }
            continue;
        }
        paths.push(path.clone());
        if reading.iter().any(|end| name.ends_with(end))
            && let Ok(text) = fs::read_to_string(entry.path())
        {
            texts.push((path, text));
        }
    }
}

/// The log of other people's reality, which is excluded.
///
/// Its own first line says what it is — *where reality and the specification
/// disagree* — and its entries quote upstream source, another project's types
/// and the standard library's unstable names on purpose. A path in it is
/// usually deliberately not ours.
fn is_the_quirk_log(named: &str) -> bool {
    named == "docs/quirks.md" || named.starts_with("docs/quirks/")
}

/// **Every path into this repository's code that its documents name lands.**
///
/// The finding names the document and the line, because the reader of it is
/// whoever wrote the sentence, with the file open.
#[test]
fn every_source_this_repository_points_at_exists() {
    let root = the_repository();

    let mut paths = Vec::new();
    let mut documents = Vec::new();
    walk(&root, "", &[".md"], &mut paths, &mut documents);

    let files: Vec<&str> = paths.iter().map(String::as_str).collect();
    assert!(
        !files.is_empty(),
        "the walk found no files at all, so nothing below was checked"
    );

    let cited: Vec<Named> = documents
        .iter()
        .filter(|(named, _)| !is_the_quirk_log(named))
        .flat_map(|(named, text)| cited_in(named, text))
        .collect();

    assert!(
        cited.len() >= TOO_FEW_TO_BELIEVE,
        "only {} pointers were read across {} documents, which is a broken walk rather \
         than a repository that stopped naming its own code",
        cited.len(),
        documents.len()
    );

    let amiss = held(&cited, &files);
    assert!(
        amiss.is_empty(),
        "{} pointer(s) into the code do not land:\n{}",
        amiss.len(),
        amiss
            .iter()
            .map(|one| format!("  {one}\n"))
            .collect::<String>()
    );
}
