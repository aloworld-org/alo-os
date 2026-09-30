//! Every source file and Rust name this repository's prose points at, held to
//! existing.
//!
//! The third family of pointer a reader believes without opening. An ADR number
//! looks like a fact; a report filename looks copied rather than typed; and
//! `crates/alo-appearance/src/accent.rs` and `Accent::ALL` look like somebody
//! opened the file. Often somebody did — and then the file moved, or the name
//! changed, and the sentence went on reading exactly as it had.
//!
//! # What this was built after
//!
//! On 2026-09-30 the roadmap's *making it yours* box had been unticked for four
//! days on three stated reasons, two of which were untrue: that `Accent::ALL`
//! sat *under a doc comment that still says all five*, when the comment says
//! *All four* and had for weeks, and that the decision required five, when its
//! own amendment says the opposite. The box was read as a fact about the code by
//! three lanes in one night.
//!
//! Six more documents were caught disagreeing with the code the same night, and
//! **in five of the six the code was right and the document was wrong.** An
//! unticked box is a claim somebody made once that nothing re-reads. That is the
//! same shape as every other finding in this repository's records: a signal that
//! nothing can contradict.
//!
//! # What it checks, and what it refuses to pretend to check
//!
//! **Paths, never conclusions.** A path under a source directory, named in
//! prose, with no file behind it: [`Amiss::ASourceNobodyWrote`].
//!
//! **A pass here means the evidence exists, not that the conclusion follows.**
//! That distinction is the whole design and belongs in the docstring rather
//! than in a reviewer's head: of the accent box's three false claims, this
//! would have caught neither the quotation nor *the ADR requires five*, because
//! both are claims about what a document means and no check should pretend to
//! read prose for meaning. A check that tried would have to guess, and a guess
//! that passes reads as agreement.
//!
//! # The Rust-name half, built and then removed
//!
//! A first version also read `Type::member` spans and refused any that no
//! source file defined. **It was run against the repository once and removed,
//! because most of what it found was correct writing:**
//!
//! - `ErrorKind::FilesystemLoop` in the quirk log — that is `std`'s, behind an
//!   unstable feature, and the entry exists to say so;
//! - `DumbMapping::drop` — another project's type, in the same log;
//! - `Measured::NothingOnThisMachine` in a plan, and
//!   `Server::set_window_tiled` in a contract — **names of work not yet done**,
//!   which is what a plan and a contract are for.
//!
//! A name absent from this repository is not evidence of anything: it may be
//! the language's, another project's, or next week's. Only a path under
//! `crates/`, `tools/` or `image/` is unambiguously a claim about this
//! repository, so only that is checked. **Refusing correct writing in a gate
//! stops three lanes**, and a check with a false-positive rate is a check
//! people learn to route around — which is the family this instrument belongs
//! to, turned on itself.
//!
//! # Where it does not look
//!
//! `docs/quirks.md` and `docs/quirks/` are excluded. That log's first line says
//! what it is — *where reality and the specification disagree* — and it exists
//! to describe hardware, other people's applications and upstream source. A
//! path in it is usually deliberately not ours.
//!
//! # The half that is designed and deliberately not built
//!
//! The accent box's first false claim was a **quotation**: *a doc comment that
//! still says all five*. A quoted string either appears in the file or it does
//! not, with no heuristic and no tuning, so it is checkable in principle and it
//! is the sharpest of the three.
//!
//! It is not built here because **attributing a quotation to a file needs a rule
//! this repository does not have.** Italics in these documents mean at least
//! four things — a quotation, a promise, a correction, plain emphasis — and a
//! check that guessed which would refuse correct writing. Refusing correct
//! writing in a gate stops three lanes. So it waits for a written convention for
//! quoting a file, and until then this module says what it does not do rather
//! than doing it badly.
//!
//! # It reads no disk, like the rest of this crate
//!
//! [`held`] is handed the pointers, the filenames and the sources. The test in
//! `tests/` is what puts the real repository behind it, so that every refusal can
//! be shown happening against a fixture — *a check that has never been seen to
//! refuse anything passes on the day it stops looking.*

use thiserror::Error;

/// Where this repository keeps code, which is where a named path must land.
///
/// A path outside these is not this check's business: prose names `/proc`,
/// `/var/home` and a person's `Documents` for good reasons, and none of them is
/// a claim about this repository.
pub const WHERE_THE_CODE_IS: [&str; 3] = ["crates/", "tools/", "image/"];

/// What a source file is called at the end.
const A_SOURCE: [&str; 4] = [".rs", ".toml", ".sh", ".bpf.c"];

/// One pointer into the code, found in prose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Named {
    /// What was named, as written.
    said: String,
    /// The document it is in.
    file: String,
    /// Which line of that document.
    line: usize,
}

impl Named {
    /// What was named, as written.
    #[must_use]
    pub fn said(&self) -> &str {
        &self.said
    }

    /// The document it is in.
    #[must_use]
    pub fn file(&self) -> &str {
        &self.file
    }

    /// Which line of that document.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }
}

/// A pointer into the code that does not land.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum Amiss {
    /// A path to a source file that is not there.
    ///
    /// A file moved or renamed leaves every sentence naming it reading exactly
    /// as it did before, and the sentence is usually offered as the evidence
    /// for a claim about what the code does.
    #[error(
        "{file} line {line} names `{said}`, and there is no such file. A file renamed leaves \
         every sentence naming it reading exactly as it did before, so this is either a rename \
         that was not followed through or a path nobody checked. If the claim around it is \
         evidence for a box, the box is now resting on a pointer that does not land"
    )]
    ASourceNobodyWrote {
        /// The path named.
        said: String,
        /// The document it is in.
        file: String,
        /// Which line of that document.
        line: usize,
    },
}

/// Every pointer into the code that this document's text names.
///
/// Only inline-code spans are read. Prose naming a file without backticks is
/// not a pointer a reader follows, and treating it as one would make ordinary
/// English into findings.
#[must_use]
pub fn cited_in(file: &str, text: &str) -> Vec<Named> {
    let mut found = Vec::new();
    for (which, line) in text.lines().enumerate() {
        for piece in spans_in(line) {
            if !is_a_path(piece) {
                continue;
            }
            found.push(Named {
                said: piece.to_owned(),
                file: file.to_owned(),
                line: which.saturating_add(1),
            });
        }
    }
    found
}

/// Every `` `…` `` span in a line, without the backticks.
fn spans_in(line: &str) -> Vec<&str> {
    let mut spans = Vec::new();
    let mut rest = line;
    while let Some(open) = rest.find('`') {
        let after = rest.get(open.saturating_add(1)..).unwrap_or_default();
        let Some(close) = after.find('`') else {
            break;
        };
        if let Some(inside) = after.get(..close)
            && !inside.is_empty()
        {
            spans.push(inside);
        }
        rest = after.get(close.saturating_add(1)..).unwrap_or_default();
    }
    spans
}

/// Whether this span is a path into this repository's code.
///
/// **A span naming a family of files is not one path**, and both forms this
/// repository uses were found by this check's own first two runs against the
/// documents, each reporting correct writing as a fault:
///
/// - `src/{carrying_out,refusing,words}.rs` — three files, a form a reader
///   expands and this check cannot;
/// - `src/sign_in_*.rs` — every file with a prefix, which a plan uses to name
///   the shape of a task's work rather than its files.
///
/// Reading either as a filename would refuse writing that is correct and
/// clearer than the alternative.
fn is_a_path(span: &str) -> bool {
    WHERE_THE_CODE_IS.iter().any(|at| span.starts_with(at))
        && A_SOURCE.iter().any(|end| span.ends_with(end))
        && !span.contains(' ')
        && !span.contains('{')
        && !span.contains('*')
}

/// Every pointer that does not land.
///
/// `files` is every path in the repository, handed in, so this reads no disk
/// and every refusal can be shown against a fixture.
#[must_use]
pub fn held(cited: &[Named], files: &[&str]) -> Vec<Amiss> {
    let mut amiss = Vec::new();
    for named in cited {
        if !files.contains(&named.said.as_str()) {
            amiss.push(Amiss::ASourceNobodyWrote {
                said: named.said.clone(),
                file: named.file.clone(),
                line: named.line,
            });
        }
    }
    amiss
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A path under a source directory is a pointer; a path anywhere else is
    /// not.** Prose names `/proc/meminfo` and a person's own folders for good
    /// reasons, and none of them is a claim about this repository.
    #[test]
    fn only_paths_into_this_repositorys_code_are_read_as_pointers() {
        let found = cited_in(
            "ROADMAP.md",
            "`crates/alo-dock/src/hiding.rs` holds it, read from `/proc/meminfo`, \
             beside `Documents/march.pdf` and `Accent::ALL`",
        );
        let said: Vec<&str> = found.iter().map(Named::said).collect();
        assert_eq!(said, vec!["crates/alo-dock/src/hiding.rs"]);
    }

    /// **A brace span names several files and is not one path.**
    ///
    /// Found by this check's own first run against the repository, which read
    /// `src/{carrying_out,refusing,words}.rs` as a filename and reported a
    /// correct sentence as a fault. An instrument containing the family it was
    /// written to catch is the ordinary case here rather than the exception.
    #[test]
    fn a_brace_span_is_not_read_as_one_file() {
        let found = cited_in(
            "docs/decisions/0072.md",
            "read from `crates/alo-changing-network/src/{carrying_out,refusing,words}.rs`",
        );
        assert!(found.is_empty(), "{found:?}");
    }

    /// **Nor is a glob**, which a plan uses to name the shape of a task's work
    /// rather than its files. The second false positive this check reported
    /// against the real documents, and the second correct sentence it would
    /// have refused.
    #[test]
    fn a_glob_is_not_read_as_one_file() {
        let found = cited_in(
            "docs/autonomy/v0-5-the-shell-plan.md",
            "the work is in `crates/alo-shell/src/sign_in_*.rs`",
        );
        assert!(found.is_empty(), "{found:?}");
    }

    /// **The fault this exists for**: a claim resting on a file that is not
    /// there.
    #[test]
    fn a_path_with_no_file_behind_it_is_found() {
        let cited = cited_in("ROADMAP.md", "held by `crates/alo-dock/src/moved-away.rs`");
        let amiss = held(&cited, &["crates/alo-dock/src/hiding.rs"]);
        assert_eq!(amiss.len(), 1, "{amiss:?}");
        let said = amiss.first().map(ToString::to_string).unwrap_or_default();
        assert!(said.contains("moved-away.rs"), "{said}");
        assert!(said.contains("rename"), "{said}");
        assert!(
            said.contains("ROADMAP.md line 1"),
            "a finding is read by whoever wrote the line, so it names the line: {said}"
        );
    }

    /// **A pointer that lands is not a finding**, or the check would refuse
    /// every true sentence in the repository — the failure that would stop
    /// three lanes rather than inform them.
    #[test]
    fn what_lands_is_not_reported() {
        let cited = cited_in("ROADMAP.md", "`crates/alo-dock/src/hiding.rs` holds it");
        assert_eq!(cited.len(), 1, "{cited:?}");
        assert!(held(&cited, &["crates/alo-dock/src/hiding.rs"]).is_empty());
    }
}
