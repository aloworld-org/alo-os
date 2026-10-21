//! What a finding says to whoever is holding the diff.
//!
//! **Written to be acted on rather than to be right.** A gate that reports a
//! name and a number gets an allowance added to it; a gate that says what to do
//! gets the thing fixed. So each of these names the function, where it is, and
//! the three honest ways out — wire it, delete it, or say why it stays.
//!
//! This crate says nothing to a person in their own language. It speaks to
//! whoever is adding the function, in the author's English, the way
//! `alo-collected` and `alo-reconciling` do.

use crate::reaching::APubFn;

/// One thing wrong between the workspace's public functions and the list of the
/// ones nothing reaches.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Finding {
    /// A `pub fn` nothing reaches that the list does not name.
    ///
    /// **The case this gate exists for.** Seven times in this repository a
    /// mechanism has been built, tested, gated green and reachable by nobody,
    /// and every one of them would have arrived as this finding.
    #[error(
        "{at}:{line} `{named}` is public and nothing reaches it — no call outside its own \
         declaration, and no mention in any test of any crate.\n  \
         This is the fault `docs/autonomy/what-nothing-calls.md` exists for, and it has been \
         found seven times here: a mechanism built, tested, green, and reachable by nobody.\n  \
         Three honest ways out, and the first is almost always right:\n    \
         1. give it a caller, in the same change — a surface a person can reach, or the \
         function it was written for;\n    \
         2. delete it, if the caller turned out not to be wanted;\n    \
         3. if it must stay uncalled for now, add it to \
         `crates/alo-reached/what-nothing-calls.txt` with \
         `UPDATE_REACHED_SNAPSHOT=1 cargo test -p alo-reached` and say in the change why — \
         the list may shrink without a word and may not grow without one."
    )]
    NothingReachesIt {
        /// The file, relative to the repository.
        at: String,
        /// Its line.
        line: usize,
        /// Its name.
        named: String,
    },
    /// The list names something that is reached now.
    ///
    /// **Not a nuisance — the half that makes the list honest.** A list that
    /// only tolerated its contents would keep every name somebody ever added,
    /// and a gate whose allowance never shrinks stops being a gate. This fires
    /// the moment a listed function gains a caller, which is the good news.
    #[error(
        "`{named}` is in `crates/alo-reached/what-nothing-calls.txt` and something reaches it \
         now — so the list is out of date in the good direction.\n  \
         Regenerate it with `UPDATE_REACHED_SNAPSHOT=1 cargo test -p alo-reached`. Nothing \
         else is needed: a name leaving this list needs no explanation, which is the whole \
         asymmetry — it may shrink silently and may not grow silently."
    )]
    TheListIsStale {
        /// The name the list still carries.
        named: String,
    },
    /// The sweep matched nothing at all, so it is not evidence.
    ///
    /// **A run that reports nothing and a run that could not run look the
    /// same**, and the hand sweep's own page required this guard for that
    /// reason. If the most-mentioned public function in the whole workspace is
    /// mentioned nowhere, the walk found no code and must refuse rather than
    /// report a clean sweep.
    #[error(
        "the sweep matched nothing: not one public function of the workspace was mentioned \
         anywhere, which cannot be true of this repository.\n  \
         It means the walk read no code — a wrong root, an unreadable tree — and it must refuse \
         rather than report that everything is reached. Nothing about the workspace is claimed \
         by this failure."
    )]
    ItMatchedNothing,
}

impl Finding {
    /// Each finding for one sweep against one list, in the order a reader
    /// should meet them: what is newly uncalled first, because that is what
    /// somebody has to act on, then what the list has outlived.
    #[must_use]
    pub fn every(found: &[APubFn], listed: &[String], read_any_code: bool) -> Vec<Self> {
        if !read_any_code {
            return vec![Self::ItMatchedNothing];
        }
        let mut findings: Vec<Self> = found
            .iter()
            .filter(|it| !listed.iter().any(|named| named == &the_line_for(it)))
            .map(|it| Self::NothingReachesIt {
                at: it.at.display().to_string(),
                line: it.line,
                named: it.named.clone(),
            })
            .collect();
        findings.extend(
            listed
                .iter()
                .filter(|line| !found.iter().any(|it| &the_line_for(it) == *line))
                .map(|line| Self::TheListIsStale {
                    named: line.clone(),
                }),
        );
        findings
    }
}

/// How one function is written in the list: its file and its name, and **not
/// its line**.
///
/// A line number in the list would make the snapshot churn every time somebody
/// added a comment above an unrelated function, and a diff nobody reads is a
/// diff that hides the one change that mattered.
#[must_use]
pub fn the_line_for(it: &APubFn) -> String {
    format!("{}\t{}", it.at.display(), it.named)
}

#[cfg(test)]
#[path = "finding_tests.rs"]
mod tests;
