//! **What a walk found**: each check passed, failed, was skipped or was never
//! reached, with its evidence — and what the walk did between them, so a
//! failure can be read against the keys that led to it.
//!
//! *Not reached* is its own outcome rather than a failure or a silence: a walk
//! stops at its first failed check, because a check after it is a check of a
//! screen nobody knows the walk is on, and a report that left those checks out
//! would read as a shorter walk that passed.

use std::fmt::Write as _;

/// One check's outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// It held.
    Passed,
    /// It did not.
    Failed,
    /// It was not tried, for a reason the evidence gives.
    Skipped,
    /// An earlier check failed, so the walk never got here.
    NotReached,
}

impl Verdict {
    /// How a report writes it.
    #[must_use]
    pub fn said(self) -> &'static str {
        match self {
            Self::Passed => "PASS",
            Self::Failed => "FAIL",
            Self::Skipped => "SKIP",
            Self::NotReached => "NOT REACHED",
        }
    }
}

/// One check: its name, its outcome, and what shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    /// Its name.
    pub name: String,
    /// Its outcome.
    pub verdict: Verdict,
    /// What shows it: the line found, the picture kept, the time it took.
    pub evidence: String,
}

/// One entry in a walk's account, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    /// Something the walk did, said with the time it did it.
    Did(String),
    /// A check.
    Checked(Check),
}

/// The whole account of one walk.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Report {
    /// The walk's name.
    pub walk: String,
    /// What happened, in order.
    pub entries: Vec<Entry>,
}

impl Report {
    /// The checks alone.
    #[must_use]
    pub fn checks(&self) -> Vec<&Check> {
        self.entries
            .iter()
            .filter_map(|entry| match entry {
                Entry::Checked(check) => Some(check),
                Entry::Did(_) => None,
            })
            .collect()
    }

    /// Whether the walk passed: at least one check held, and none failed or
    /// went unreached. A skipped check does not fail a walk; it is said.
    #[must_use]
    pub fn passed(&self) -> bool {
        let checks = self.checks();
        checks.iter().any(|it| it.verdict == Verdict::Passed)
            && checks
                .iter()
                .all(|it| matches!(it.verdict, Verdict::Passed | Verdict::Skipped))
    }

    /// How many checks had each outcome, as a sentence.
    #[must_use]
    pub fn tally(&self) -> String {
        let checks = self.checks();
        let count = |verdict: Verdict| checks.iter().filter(|it| it.verdict == verdict).count();
        format!(
            "{} passed, {} failed, {} skipped, {} not reached",
            count(Verdict::Passed),
            count(Verdict::Failed),
            count(Verdict::Skipped),
            count(Verdict::NotReached)
        )
    }

    /// The account as plain text, one entry a line.
    #[must_use]
    pub fn as_text(&self) -> String {
        let mut text = format!("walk: {}\n", self.walk);
        for entry in &self.entries {
            let _ = match entry {
                Entry::Did(what) => writeln!(text, "        {what}"),
                Entry::Checked(check) => writeln!(
                    text,
                    "{:<11} {} — {}",
                    check.verdict.said(),
                    check.name,
                    check.evidence
                ),
            };
        }
        let verdict = if self.passed() { "PASSED" } else { "FAILED" };
        let _ = writeln!(text, "the walk {verdict}: {}", self.tally());
        text
    }

    /// The checks as a Markdown table, for a workflow run's summary page.
    #[must_use]
    pub fn as_markdown(&self) -> String {
        let verdict = if self.passed() { "passed" } else { "failed" };
        let mut text = format!(
            "### The walk `{}` {verdict}\n\n{}\n\n| | check | evidence |\n|---|---|---|\n",
            self.walk,
            self.tally()
        );
        for check in self.checks() {
            let _ = writeln!(
                text,
                "| {} | {} | {} |",
                check.verdict.said(),
                check.name.replace('|', "\\|"),
                check.evidence.replace('|', "\\|")
            );
        }
        text
    }
}

#[cfg(test)]
#[path = "report_tests.rs"]
mod tests;
