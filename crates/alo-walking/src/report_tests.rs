//! When a walk passed, and how its account reads.

use super::{Check, Entry, Report, Verdict};

/// A report of these checks.
fn report(verdicts: &[Verdict]) -> Report {
    Report {
        walk: "a walk".to_owned(),
        entries: verdicts
            .iter()
            .enumerate()
            .map(|(at, verdict)| {
                Entry::Checked(Check {
                    name: format!("check {at}"),
                    verdict: *verdict,
                    evidence: "seen".to_owned(),
                })
            })
            .collect(),
    }
}

/// Passed and skipped checks pass a walk; a failed or unreached one fails it.
#[test]
fn a_walk_passes_only_when_nothing_failed_or_went_unreached() {
    assert!(report(&[Verdict::Passed, Verdict::Skipped]).passed());
    assert!(!report(&[Verdict::Passed, Verdict::Failed]).passed());
    assert!(!report(&[Verdict::Passed, Verdict::NotReached]).passed());
}

/// **A walk that checked nothing did not pass**, even though nothing failed:
/// all skipped, or empty, is not a walk that held.
#[test]
fn a_walk_that_checked_nothing_did_not_pass() {
    assert!(!report(&[]).passed());
    assert!(!report(&[Verdict::Skipped]).passed());
}

/// The text names every check with its outcome and ends with the tally.
#[test]
fn the_text_says_each_check_and_the_tally() {
    let text = report(&[Verdict::Passed, Verdict::Failed, Verdict::NotReached]).as_text();
    assert!(text.contains("PASS        check 0 — seen"), "{text}");
    assert!(text.contains("FAIL        check 1"), "{text}");
    assert!(text.contains("NOT REACHED check 2"), "{text}");
    assert!(
        text.ends_with("the walk FAILED: 1 passed, 1 failed, 0 skipped, 1 not reached\n"),
        "{text}"
    );
}

/// A bar in evidence does not break the summary's table.
#[test]
fn a_bar_in_evidence_is_escaped_in_the_table() {
    let mut walked = report(&[Verdict::Passed]);
    walked.entries.push(Entry::Checked(Check {
        name: "a|b".to_owned(),
        verdict: Verdict::Passed,
        evidence: "x|y".to_owned(),
    }));
    let table = walked.as_markdown();
    assert!(table.contains("| PASS | a\\|b | x\\|y |"), "{table}");
}
