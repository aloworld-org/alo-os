//! The check: every decision this repository points at, against the decisions it
//! has.
//!
//! Three directions, and only the first is what anybody would write by hand.
//! **Every citation names a decision that exists** — that is the rule. **Every
//! number belongs to one decision** — that is what stops a citation resolving to
//! whichever file a reader opened first. **Every decision says what its status
//! is** — that is what stops a citation carrying the weight of a settled rule
//! that nobody settled.
//!
//! Findings are collected rather than returned one at a time. A check that
//! stopped at the first problem would be read as *one problem*, and a decision
//! renamed moves every pointer at it at once.

use crate::{
    citation::Citation,
    citing::cited_in,
    decisions::{Decision, THE_DECISIONS, among},
    finding::Finding,
    naming::named_in,
};

/// What this repository's pointers add up to, once every one of them has been
/// followed.
///
/// The counts are the shape of the claim rather than a score: **cited** is every
/// reference by number that names a decision here, **elsewhere** is every one
/// that names a neighbouring repository's, and **named** is every reference by
/// filename.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Held {
    /// How many decisions this repository has.
    decisions: usize,

    /// How many references by number point into them.
    cited: usize,

    /// How many point into a neighbouring repository's decisions.
    elsewhere: usize,

    /// How many references name a decision's file.
    named: usize,
}

impl Held {
    /// How many decisions this repository has.
    #[must_use]
    pub fn decisions(&self) -> usize {
        self.decisions
    }

    /// How many references by number point into them.
    #[must_use]
    pub fn cited(&self) -> usize {
        self.cited
    }

    /// How many point into a neighbouring repository's decisions.
    #[must_use]
    pub fn elsewhere(&self) -> usize {
        self.elsewhere
    }

    /// How many references name a decision's file.
    #[must_use]
    pub fn named(&self) -> usize {
        self.named
    }
}

/// Every reference to a decision in `files`, against the decisions in
/// `decisions`.
///
/// `decisions` is the contents of `docs/decisions/`, each as its filename without
/// the directory in front of it and its text. `files` is the repository's Rust
/// and Markdown, each as its repository-relative path and its text. `neighbours`
/// are the other repositories whose decisions these files may point into.
///
/// Nothing here opens anything — which is what lets every finding this crate
/// exists for be shown happening against a fixture, rather than only after
/// somebody has written the citation that would cause it.
///
/// # Errors
///
/// Every [`Finding`], in the order the decisions, then the files, then the
/// neighbours raise them.
pub fn held(
    decisions: &[(&str, &str)],
    files: &[(&str, &str)],
    neighbours: &[&str],
) -> Result<Held, Vec<Finding>> {
    let written = among(decisions);
    if written.is_empty() {
        return Err(vec![Finding::NoDecisionsToCheck {
            directory: THE_DECISIONS,
        }]);
    }

    let mut findings = Vec::new();
    whether_each_number_belongs_to_one_decision(&written, &mut findings);
    for decision in &written {
        if !decision.says_what_it_is() {
            findings.push(Finding::ADecisionWithNoStatus {
                file: decision.file().to_owned(),
            });
        }
    }

    let mut cited: usize = 0;
    let mut elsewhere: usize = 0;
    let mut named: usize = 0;
    let mut qualified_by: Vec<String> = Vec::new();
    for (file, text) in files {
        for citation in cited_in(file, text, neighbours) {
            match citation.elsewhere() {
                Some(repository) => {
                    elsewhere = elsewhere.saturating_add(1);
                    let repository = repository.to_owned();
                    if !qualified_by.contains(&repository) {
                        qualified_by.push(repository);
                    }
                }
                None => {
                    cited = cited.saturating_add(1);
                    whether_the_number_is_a_decision(&citation, &written, &mut findings);
                }
            }
        }
        for names in named_in(file, text) {
            named = named.saturating_add(1);
            if !written
                .iter()
                .any(|decision| decision.file() == names.named())
            {
                findings.push(Finding::AFileNobodyWrote {
                    named: names.named().to_owned(),
                    file: names.file().to_owned(),
                    line: names.line(),
                    directory: THE_DECISIONS,
                });
            }
        }
    }

    if cited == 0 && elsewhere == 0 {
        findings.push(Finding::NothingCitesAnything { files: files.len() });
    }
    for repository in neighbours {
        if !qualified_by.iter().any(|named| named == repository) {
            findings.push(Finding::ANeighbourNobodyCites {
                repository: (*repository).to_owned(),
            });
        }
    }

    if findings.is_empty() {
        Ok(Held {
            decisions: written.len(),
            cited,
            elsewhere,
            named,
        })
    } else {
        Err(findings)
    }
}

/// Whether one citation's number is a decision this repository has.
fn whether_the_number_is_a_decision(
    citation: &Citation,
    written: &[Decision],
    findings: &mut Vec<Finding>,
) {
    if written
        .iter()
        .any(|decision| decision.number() == citation.number())
    {
        return;
    }
    findings.push(Finding::ADecisionNobodyWrote {
        number: citation.number().to_owned(),
        said: citation.said().to_owned(),
        file: citation.file().to_owned(),
        line: citation.line(),
        directory: THE_DECISIONS,
    });
}

/// Whether two decisions claim one number.
///
/// A citation of a number two files claim resolves to whichever one a reader
/// opens, and both of them look like the answer — so this is not a tidiness
/// finding, it is the one case where a pointer lands somewhere and is still
/// wrong.
fn whether_each_number_belongs_to_one_decision(written: &[Decision], findings: &mut Vec<Finding>) {
    for (at, decision) in written.iter().enumerate() {
        for other in written.iter().skip(at.saturating_add(1)) {
            if other.number() == decision.number() {
                findings.push(Finding::TwoDecisionsOneNumber {
                    number: decision.number().to_owned(),
                    one: decision.file().to_owned(),
                    other: other.file().to_owned(),
                });
            }
        }
    }
}
