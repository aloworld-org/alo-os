//! The list of public functions nothing reaches, as a committed snapshot.
//!
//! **A file rather than an array in Rust**, for the reason
//! `crates/alo-saying/the-vocabulary.txt` is a file: a list somebody edits by
//! hand drifts from what the code says, and a list the test regenerates cannot.
//! The vocabulary snapshot is the pattern this repository already trusts, and
//! the same command shape regenerates this one.
//!
//! # The asymmetry, which is the whole design
//!
//! **It may shrink silently and may not grow silently.** A name leaving needs no
//! explanation — something gained a caller, which is the good news. A name
//! arriving means a public function was added that nothing reaches, and that is
//! the fault this gate exists for, so it costs a sentence in the change that
//! adds it.
//!
//! `docs/autonomy/what-nothing-calls.md` declined to write this gate for exactly
//! this reason: *an accepted list is only honest if the test refuses growth of
//! the list rather than merely tolerating its contents.*

/// Where the list lives, relative to the repository.
pub const THE_LIST: &str = "crates/alo-reached/what-nothing-calls.txt";

/// The list as it is committed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Accepted {
    /// One line per function: its file and its name, tab separated.
    lines: Vec<String>,
}

impl Accepted {
    /// Read the list, or an empty one where it is not there yet.
    ///
    /// **Absent and empty are the same answer here and that is deliberate**: a
    /// missing list means every unreached function is reported, which is the
    /// safe direction. The unsafe direction would be a missing list read as
    /// *everything is accepted*.
    #[must_use]
    pub fn read(text: &str) -> Self {
        Self {
            lines: text
                .lines()
                .map(str::trim_end)
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .map(str::to_owned)
                .collect(),
        }
    }

    /// The lines, in the order the file holds them.
    #[must_use]
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// How the file should read for a given sweep.
    ///
    /// Sorted, so two runs of one tree write the same bytes, with a header
    /// saying what the file is and how to regenerate it — a snapshot whose own
    /// first line does not say that is a file somebody edits by hand.
    #[must_use]
    pub fn as_written(lines: &[String]) -> String {
        let mut sorted: Vec<&String> = lines.iter().collect();
        sorted.sort();
        sorted.dedup();
        let mut out = String::from(HEADER);
        for line in sorted {
            out.push_str(line);
            out.push('\n');
        }
        out
    }
}

/// What the file says about itself, above the list.
const HEADER: &str = "\
# Every public function of this workspace that nothing reaches: no call outside
# its own declaration, and no mention in any test of any crate.
#
# Generated. Do not edit by hand — regenerate with:
#
#     UPDATE_REACHED_SNAPSHOT=1 cargo test -p alo-reached
#
# This list MAY SHRINK SILENTLY and MAY NOT GROW SILENTLY. A name leaving means
# something gained a caller, which needs no explanation. A name arriving means a
# public function was added that nothing reaches, which is the fault
# docs/autonomy/what-nothing-calls.md exists for — found seven times in this
# repository — so it costs a sentence in the change that adds it.
#
# What this list cannot tell you: a caller count of one and a caller count of
# nine are the same answer to \"is this reached\". The blank install screen of
# 2026-10-21 was a mechanism with one correct caller and a second surface that
# should have used it, and no reference count finds that.
#
# One line per function: its file, a tab, its name. No line numbers, so adding a
# comment above an unrelated function does not churn the file.
";

#[cfg(test)]
#[path = "accepted_tests.rs"]
mod tests;
