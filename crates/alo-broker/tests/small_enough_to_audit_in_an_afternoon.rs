//! The broker stays small enough to be audited in an afternoon, and growing it
//! is a visible decision.
//!
//! ADR 0001 §2 makes the size of this component *a design constraint on it, not
//! an aspiration*, and a constraint nothing measures is an aspiration. So two
//! things are held here, by number and by name:
//!
//! - **how much there is to read.** [`MOST_CODE`] is the ceiling on lines of
//!   code in `src/` — not blank, not a comment, not a file's own tests — which
//!   is what somebody auditing the broker has to follow. [`MOST_LINES`] is the
//!   ceiling on everything in `src/`, documentation and tests included, which
//!   is what they have to open. When this was written the broker was 744 lines
//!   of code in 1,904 lines; the ceilings leave room for a small fix and none
//!   for a new responsibility.
//! - **what it depends on**, exactly. A privileged component's dependency list
//!   is part of what an audit reads, so a dependency added or removed fails
//!   [`the_broker_depends_on_exactly_what_it_names`] until this file says so.
//!
//! Raising either number is allowed. It is a change to this file, in the same
//! commit as whatever needed it, with the reason in the report — which is the
//! point: the broker cannot grow *quietly*.

use std::collections::BTreeSet;
use std::path::Path;

/// The most lines of code `src/` may hold, tests and comments aside.
const MOST_CODE: usize = 900;

/// The most lines `src/` may hold altogether.
const MOST_LINES: usize = 2_200;

/// Every dependency, of every kind, the broker is allowed.
const DEPENDS_ON: [&str; 5] = [
    // [dependencies]
    "alo-record",
    "getrandom",
    "ring",
    // [target.'cfg(unix)'.dependencies]
    "rustix",
    // [dev-dependencies]
    "serde_json",
];

/// The crate's own directory.
fn here() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Every Rust file in `src/`, read.
fn every_source_file() -> Vec<(String, String)> {
    let listed = std::fs::read_dir(here().join("src"));
    assert!(listed.is_ok(), "src/ could not be listed");
    let mut files: Vec<(String, String)> = listed
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
        .map(|path| {
            let written = std::fs::read_to_string(&path).unwrap_or_default();
            (path.display().to_string(), written)
        })
        .collect();
    files.sort();
    assert!(
        files.iter().all(|(_, written)| !written.is_empty()),
        "a file in src/ could not be read"
    );
    files
}

/// The lines of one file an audit follows: before its tests, not blank, not a
/// comment.
fn code_in(written: &str) -> usize {
    written
        .split("\n#[cfg(test)]\n")
        .next()
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .count()
}

/// **The broker is no longer than an afternoon's audit**, by the two numbers
/// this file names.
#[test]
fn the_broker_is_no_longer_than_an_afternoon() {
    let files = every_source_file();
    assert!(files.len() >= 10, "src/ was not read: {files:?}");

    let code: usize = files.iter().map(|(_, written)| code_in(written)).sum();
    let lines: usize = files
        .iter()
        .map(|(_, written)| written.lines().count())
        .sum();

    assert!(
        code <= MOST_CODE,
        "the broker has grown to {code} lines of code, past the {MOST_CODE} an afternoon's audit \
         allows. Split what does not belong in a privileged component out of it, or raise \
         MOST_CODE here and say why in the task's report."
    );
    assert!(
        lines <= MOST_LINES,
        "src/ has grown to {lines} lines, past {MOST_LINES}. Raise MOST_LINES here and say why, \
         or make it smaller."
    );
}

/// **The broker depends on exactly what this file names**, and a dependency
/// added or dropped is a change somebody has to make here.
#[test]
fn the_broker_depends_on_exactly_what_it_names() {
    let manifest = std::fs::read_to_string(here().join("Cargo.toml")).unwrap_or_default();
    assert!(!manifest.is_empty(), "Cargo.toml could not be read");

    let mut in_dependencies = false;
    let mut named = BTreeSet::new();
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            in_dependencies = line.ends_with("dependencies]");
            continue;
        }
        if !in_dependencies || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((name, _)) = line.split_once('=') {
            named.insert(name.trim().to_owned());
        }
    }

    let allowed: BTreeSet<String> = DEPENDS_ON.iter().map(|name| (*name).to_owned()).collect();
    assert_eq!(
        named, allowed,
        "the broker's dependencies are not the ones this test names — a privileged component's \
         dependency list is part of its audit, so change DEPENDS_ON here in the same commit and \
         say why in the report"
    );
}

/// **The line counter counts what it says**: comments, blank lines and a
/// file's tests are not code; everything else is.
#[test]
fn the_counter_counts_code_and_nothing_else() {
    let written = "//! A module.\n\n/// A thing.\nfn thing() {\n    // inside\n    1\n}\n\
                   \n#[cfg(test)]\nmod tests {\n    fn a() {}\n}\n";
    assert_eq!(code_in(written), 3);
}
