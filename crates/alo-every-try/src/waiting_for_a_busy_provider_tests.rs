//! A provider that is busy is waited for, and never read as having answered.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Ok is the failure being reported"
)]

use std::time::Duration;

use alo_answering::WentWrong;

use super::{Attempted, THE_WAITS, asking_until_answered, is_busy};

/// A scripted provider: each attempt takes the next of `answers`.
fn scripted(
    answers: Vec<Attempted<&'static str>>,
) -> (Result<&'static str, String>, Vec<Duration>) {
    let mut answers = answers.into_iter();
    let mut waited = Vec::new();
    let got = asking_until_answered(
        &mut || {
            answers
                .next()
                .unwrap_or(Attempted::Refused("nothing more".to_owned()))
        },
        &mut |wait| waited.push(wait),
    );
    (got, waited)
}

/// **A 429, then a 200** (the coordinator, 2026-11-20): the try waits and is
/// answered, and the busy answer is not what it got.
#[test]
fn a_429_then_a_200_is_waited_for_and_answered() {
    let (got, waited) = scripted(vec![
        Attempted::Busy("HavingTrouble(429)".to_owned()),
        Attempted::Answered("the model's line"),
    ]);
    assert_eq!(got, Ok("the model's line"));
    assert_eq!(waited, [THE_WAITS[0]]);
}

/// **Busy every time**: every wait is taken, then the try counts as not
/// asked — never as an answer.
#[test]
fn busy_every_time_is_not_asked_after_every_wait() {
    let busy = || Attempted::Busy("HavingTrouble(429)".to_owned());
    let (got, waited) = scripted((0..=THE_WAITS.len()).map(|_| busy()).collect());
    let why = got.expect_err("never an answer");
    assert!(why.contains("HavingTrouble(429)"), "{why}");
    assert_eq!(waited, THE_WAITS);
    let total: Duration = THE_WAITS.iter().sum();
    assert!(
        total >= Duration::from_secs(120),
        "a few minutes: {total:?}"
    );
    assert!(total <= Duration::from_secs(300), "and no more: {total:?}");
}

/// **A refusal that waiting cannot mend is not waited for**: a key not
/// accepted, or a provider out of credit.
#[test]
fn a_refusal_is_not_waited_for() {
    let (got, waited) = scripted(vec![Attempted::Refused("KeyNotAccepted".to_owned())]);
    assert!(got.is_err());
    assert!(waited.is_empty());
}

/// Which troubles are a busy provider: a 429, a 5xx, or too long a wait — and
/// not a refusal, a key, or credit run out.
#[test]
fn what_is_a_busy_provider() {
    assert!(is_busy(&WentWrong::HavingTrouble(429)));
    assert!(is_busy(&WentWrong::HavingTrouble(500)));
    assert!(is_busy(&WentWrong::HavingTrouble(503)));
    assert!(is_busy(&WentWrong::TookTooLong));
    assert!(!is_busy(&WentWrong::HavingTrouble(418)));
    assert!(!is_busy(&WentWrong::RanOut));
    assert!(!is_busy(&WentWrong::KeyNotAccepted));
    assert!(!is_busy(&WentWrong::NothingUsable));
}
