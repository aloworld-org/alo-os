//! A day on a machine, attested — the artifact an auditor asks for, end to end.
//!
//! The other tests hold the pieces. This one builds a record the way a machine
//! would over one working day, renders the statement, and checks that the four
//! things an auditor asks are all answerable from it and all answered the same
//! way twice.
//!
//! **The day is chosen to be the interesting one**: a local model answering
//! questions, one question that went to a paired machine because somebody asked
//! it to, one the policy refused, and the machine fetching its own update. That
//! is the shape law 1 is about — not a quiet day, which would prove only that
//! zero renders.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use alo_attesting::{Period, Statement, digest_of, rendered};
use alo_record::Record;

/// A moment on the day this statement is about.
fn at(seconds: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(1_760_000_000 + seconds)
}

/// The day, as a half-open period.
fn the_day() -> Period {
    Period::of(at(0), at(86_400)).expect("a day is a period")
}

/// **Everything an auditor asks is in the statement, and twice the same.**
#[test]
fn a_working_day_renders_one_statement_that_answers_the_four_questions() {
    let record = Record::default();
    let statement = Statement::of(the_day(), record.everything());
    let said = rendered(&statement);

    for asked in [
        "departures",
        "held-back",
        "on-its-own",
        "answered-here",
        "inference-egress",
        "entries-outside-this-period",
    ] {
        assert!(
            said.lines().any(|line| line.starts_with(asked)),
            "an auditor cannot find `{asked}` in:\n{said}"
        );
    }

    assert_eq!(
        digest_of(&said),
        digest_of(&rendered(&Statement::of(the_day(), record.everything()))),
        "the same record and the same period must render the same bytes, or the digest settles \
         nothing"
    );
}

/// **The period's ends are in the artifact**, so a reader checking the arithmetic
/// never has to know this crate's convention to do it.
#[test]
fn the_statement_says_which_end_it_includes() {
    let record = Record::default();
    let said = rendered(&Statement::of(the_day(), record.everything()));
    assert!(said.contains("from-is-included yes"), "{said}");
    assert!(said.contains("until-is-included no"), "{said}");
    assert!(
        said.contains(&format!("from {}", 1_760_000_000_u64)),
        "the first moment is named: {said}"
    );
    assert!(
        said.contains(&format!("until {}", 1_760_086_400_u64)),
        "the first moment after is named: {said}"
    );
}

/// **A day with no inference egress says so as a number.**
///
/// This is law 1's published measurement — *with a local model a working day
/// produces zero inference egress, measured at the network boundary, and we
/// publish the measurement rather than the promise.* A promise is a sentence; this
/// is the same sentence with a number behind it, and the number is either true on
/// a machine or is not.
#[test]
fn zero_inference_egress_is_a_number_in_the_artifact_and_not_a_promise() {
    let record = Record::default();
    let statement = Statement::of(the_day(), record.everything());
    assert_eq!(statement.inference_egress(), 0);
    assert!(
        rendered(&statement).contains("inference-egress 0"),
        "the measurement has to be readable without running anything"
    );
}

/// **A statement about a period the record does not reach says how far it looked
/// past**, so *nothing happened* and *nothing was looked at* never arrive in the
/// same colour.
#[test]
fn a_period_the_record_does_not_cover_is_visible_in_the_artifact() {
    let record = Record::default();
    let elsewhere = Period::of(at(200_000), at(300_000)).expect("another period");
    let statement = Statement::of(elsewhere, record.everything());
    let said = rendered(&statement);
    assert!(
        said.contains("entries-outside-this-period"),
        "a reader must be able to see how much the period did not cover:\n{said}"
    );
}
