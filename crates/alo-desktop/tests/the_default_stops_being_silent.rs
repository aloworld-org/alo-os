//! **When something starts keeping a person's display arrangement, this
//! desktop must stop claiming they have arranged nothing** —
//! `docs/autonomy/more-than-one-display-plan.md` task 12.
//!
//! # The sentence this protects, and why it needs protecting
//!
//! `their_displays` hands the shell `Changes::untouched()`, which says *this
//! person has changed nothing*. On 2026-10-08 that was **literally true**:
//! nothing in production wrote a display arrangement, so nobody could have made
//! one. The claim was not a placeholder; it was the only honest answer.
//!
//! **It stops being true the moment the settings road lands, and it stops
//! silently.** A person would arrange their displays, something would keep that
//! arrangement in their folder, and this desktop would go on passing
//! `untouched` — so the arrangement would be read from nowhere and their work
//! thrown away on every frame. Nothing would fail. The code would still read as
//! though it honoured them.
//!
//! # What this refuses, and the shape it had to take
//!
//! **A test asserting *nothing writes an arrangement* would pass forever and
//! say nothing.** It would be true today, true tomorrow, and still true on the
//! day the fault arrived — because the fault is not that something writes one,
//! it is the *pair*: something writes one **and** this desktop still says
//! nobody has.
//!
//! So the assertion is the pair, and either half alone is fine:
//!
//! - nothing writes one, and the desktop says `untouched` — **today, correct**;
//! - something writes one, and the desktop reads it — **the goal, correct**;
//! - something writes one, and the desktop still says `untouched` — **refused**;
//! - nothing writes one, and the desktop claims to read one — not this test's
//!   business, and `alo-citing`'s surface check already refuses it.
//!
//! Where that lands: `where-a-persons-settings-are-kept-plan`'s task 8 is what
//! changes this, and whoever lands it will meet this test rather than a comment.

use std::path::{Path, PathBuf};

/// The repository root, from this crate's own manifest directory.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("this crate sits two levels under the repository root")
        .to_path_buf()
}

/// Every production source file: `crates/*/src/**.rs`, and nothing under
/// `tests/`.
///
/// **Test files are excluded on purpose and that is the whole scope of this
/// check.** `alo-displays`' own suite writes arrangements constantly, correctly,
/// and proves the keeping works — a test that counted those would be red from
/// the day it was written and would have to be deleted rather than read.
fn every_production_source(at: &Path, into: &mut Vec<(String, String)>) {
    let Ok(entries) = std::fs::read_dir(at) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            if name == "target" || name == "tests" || name.starts_with('.') {
                continue;
            }
            every_production_source(&path, into);
        } else if name.ends_with(".rs") && !name.ends_with("_tests.rs") {
            if let Ok(text) = std::fs::read_to_string(&path) {
                into.push((
                    path.to_string_lossy().into_owned(),
                    production_part_of(&text),
                ));
            }
        }
    }
}

/// A source file with its test module cut off.
///
/// **Excluding `tests/` and `*_tests.rs` was not enough, measured.** The first
/// version of this test reported **28 writers in production** and every one was
/// inside a `#[cfg(test)]` module in `attached.rs` or `changes.rs` — a crate's
/// own suite, writing arrangements correctly, which is exactly what it should
/// do. A test that counted those would have been red the day it was written and
/// would have been deleted rather than read.
///
/// Cut at the **first** `#[cfg(test)]`, which is this repository's shape: a test
/// module is the last thing in a file, usually a `#[path = "…_tests.rs"] mod
/// tests;`. **If somebody puts production code after a test module this reader
/// stops seeing it**, which is the honest limit of a line-based reader and is
/// written here rather than discovered.
fn production_part_of(text: &str) -> String {
    text.split_once("#[cfg(test)]")
        .map_or_else(|| text.to_owned(), |(before, _)| before.to_owned())
}

/// The calls that keep a person's **display** arrangement.
///
/// **Each spelling was narrowed by a measurement, not by caution.** The first
/// version matched `keeping::keep(`, `.remember(` and `.forget_everything(`
/// anywhere, and reported five writers. Every one was something else:
///
/// - `alo_arranging::keeping::keep` in this very binary — the **canvas**
///   layout, a different subject with the same function name;
/// - `self.changes.forget_everything()` in `alo-dock`, `alo-shortcuts` and
///   `alo-appearance` — their own `changes` fields, of their own types. All
///   three contain **zero** mentions of `alo_displays`.
///
/// So `keeping::keep` is only matched when it is spelled with the crate that
/// owns displays, and the mutators only in a file that actually reaches that
/// crate.
const KEEPS_AN_ARRANGEMENT: [&str; 3] = [
    "alo_displays::keeping::keep(",
    ".remember(",
    ".forget_everything(",
];

/// Whether a file can reach `alo_displays::Changes` at all.
///
/// `.remember(` and `.forget_everything(` are ordinary English and belong to
/// other types, so a hit means nothing unless the file is holding the type this
/// test is about. `alo-shell` re-exports `Changes`, so either spelling counts.
fn can_reach_the_display_changes(text: &str) -> bool {
    text.contains("alo_displays::") || text.contains("alo_shell::Changes")
}

/// Whether this file is `alo-displays` itself.
///
/// **The mechanism existing is not the same as it being used**, and that
/// distinction is the whole of this test. `alo-displays` must be able to keep an
/// arrangement — that is its job, tested in its own suite — and
/// `impl From<Written> for Changes` calls `remember` while *reading* a file
/// rather than keeping one. What task 12 watches for is a **caller** outside
/// that crate.
fn is_the_mechanism_itself(file: &str) -> bool {
    file.contains("/alo-displays/")
}

/// Whether a line is a comment or a doc comment, which claims nothing.
fn is_prose(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("//") || line.starts_with("*") || line.starts_with("#[")
}

/// **The pair, refused.**
#[test]
fn nothing_keeps_an_arrangement_while_this_desktop_says_nobody_has() {
    let root = the_repository();
    let mut sources = Vec::new();
    every_production_source(&root.join("crates"), &mut sources);

    // The walk is measured first. A walk that found nothing would make every
    // assertion below vacuous, which is the shape this repository refuses.
    assert!(
        sources.len() > 200,
        "this walk read {} production source files, and fewer than 200 means it \
         stopped looking rather than that the repository stopped having them",
        sources.len()
    );

    let mut keeps: Vec<String> = Vec::new();
    let mut says_untouched: Vec<String> = Vec::new();
    for (file, text) in &sources {
        for (number, line) in text.lines().enumerate() {
            if is_prose(line) {
                continue;
            }
            let at = format!("{file}:{}", number + 1);
            if line.contains("Changes::untouched()") {
                says_untouched.push(at.clone());
            }
            if is_the_mechanism_itself(file) || !can_reach_the_display_changes(text) {
                continue;
            }
            for call in KEEPS_AN_ARRANGEMENT
                .iter()
                .filter(|call| line.contains(**call))
            {
                keeps.push(format!("{at} — `{}`", call.trim_end_matches('(')));
            }
        }
    }

    // Today: one `untouched`, and nothing keeping an arrangement. Asserted so
    // that the pair below cannot pass because the reader stopped finding the
    // desktop's own line.
    assert!(
        !says_untouched.is_empty(),
        "no production code passes `Changes::untouched()` any more. If the \
         desktop now reads a person's kept arrangement, this test has done its \
         job and should be replaced by one asserting that; if the call was \
         merely moved, point this reader at it"
    );

    assert!(
        keeps.is_empty(),
        "something in production now keeps a person's display arrangement:\n  {}\n\
         and {} still passes `Changes::untouched()`, which says they have \
         arranged nothing:\n  {}\n\n\
         So a person could arrange their displays, have it kept, and have this \
         desktop ignore it on every frame — with nothing failing and the code \
         still reading as though it honoured them. **Make `their_displays` read \
         what was kept.** This is `more-than-one-display-plan.md` task 12, and \
         the road that changes it is `where-a-persons-settings-are-kept-plan`'s \
         task 8.",
        keeps.join("\n  "),
        "alo-desktop",
        says_untouched.join("\n  ")
    );
}
