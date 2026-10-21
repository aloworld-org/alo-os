//! What a finding says, and that it says what to do.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic naming what was measured is the failure being reported"
)]

use std::path::PathBuf;

use super::{Finding, the_line_for};
use crate::reaching::APubFn;

/// One function, for the findings to be about.
fn a_pub_fn(named: &str) -> APubFn {
    APubFn {
        in_crate: "crates/alo-example".to_owned(),
        at: PathBuf::from("crates/alo-example/src/thing.rs"),
        line: 12,
        named: named.to_owned(),
    }
}

/// **A function nothing reaches, and not in the list, is reported.**
#[test]
fn a_new_unreached_function_is_reported() {
    let found = [a_pub_fn("nobody_calls_me")];
    let findings = Finding::every(&found, &[], true);
    assert_eq!(findings.len(), 1, "{findings:?}");
    let said = findings.first().expect("one finding").to_string();
    assert!(said.contains("nobody_calls_me"), "{said}");
    assert!(said.contains("thing.rs:12"), "it says where: {said}");
    assert!(
        said.contains("give it a caller"),
        "it says what to do, and the right thing first: {said}"
    );
}

/// **A function in the list is not reported**, which is what the list is for.
#[test]
fn a_listed_function_is_accepted() {
    let found = [a_pub_fn("known")];
    let listed = [the_line_for(&a_pub_fn("known"))];
    assert_eq!(Finding::every(&found, &listed, true), [], "accepted");
}

/// **A listed function that is reached now is reported, so the list shrinks.**
///
/// The half that makes an accepted list honest rather than a graveyard.
#[test]
fn a_listed_function_that_gained_a_caller_is_reported() {
    let listed = [the_line_for(&a_pub_fn("now_called"))];
    let findings = Finding::every(&[], &listed, true);
    assert_eq!(findings.len(), 1, "{findings:?}");
    let said = findings.first().expect("one finding").to_string();
    assert!(said.contains("now_called"), "{said}");
    assert!(
        said.contains("out of date in the good direction"),
        "it reads as good news, because it is: {said}"
    );
}

/// **A sweep that matched nothing refuses rather than reporting a clean tree.**
///
/// A run that found nothing and a run that could not run look identical, and
/// only this guard tells them apart.
#[test]
fn a_sweep_that_matched_nothing_refuses() {
    let findings = Finding::every(&[], &[], false);
    assert_eq!(findings, [Finding::ItMatchedNothing]);
    let said = findings.first().expect("one finding").to_string();
    assert!(
        said.contains("Nothing about the workspace is claimed"),
        "it says its own answer is not evidence: {said}"
    );
}

/// **The list line carries no line number.**
///
/// A line number would churn the snapshot every time somebody added a comment
/// above an unrelated function, and a diff nobody reads hides the one change
/// that mattered.
#[test]
fn the_list_line_has_no_line_number() {
    let line = the_line_for(&a_pub_fn("named"));
    assert_eq!(line, "crates/alo-example/src/thing.rs\tnamed");
    assert!(!line.contains("12"), "{line}");
}
