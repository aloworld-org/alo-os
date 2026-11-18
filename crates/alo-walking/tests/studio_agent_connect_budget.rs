//! One-shot operation deadline, not an installed daemon or a reusable service.
//!
//! The real binary must exit on timeout so any blocked connect worker dies
//! with the process. Tests use finite operations and join them by observation.

#![expect(clippy::expect_used, reason = "test thread/channel setup must succeed")]

use alo_walking::studio_agent_launch::before_deadline;
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

#[test]
fn one_operation_preserves_the_exact_success_or_error() {
    let attempts = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&attempts);
    let result = before_deadline(Instant::now() + Duration::from_secs(2), move || {
        observed.fetch_add(1, Ordering::SeqCst);
        Ok(vec![7_u8, 9])
    });
    assert_eq!(result, Ok(vec![7, 9]));
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    let result = before_deadline(Instant::now() + Duration::from_secs(2), || {
        Err::<(), _>("test-only original connect refusal".to_owned())
    });
    assert_eq!(result, Err("test-only original connect refusal".to_owned()));
}

#[test]
fn an_expired_budget_never_attempts_the_operation() {
    let attempts = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&attempts);
    assert!(
        before_deadline(Instant::now(), move || {
            observed.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .is_err()
    );
    assert_eq!(attempts.load(Ordering::SeqCst), 0);
}

#[test]
fn deadline_returns_failure_without_waiting_for_or_repeating_a_slow_connect() {
    let (finished, completion) = mpsc::channel();
    let attempts = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&attempts);
    let began = Instant::now();
    let result = before_deadline(began + Duration::from_millis(100), move || {
        observed.fetch_add(1, Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(600));
        finished.send(()).expect("test observer remains alive");
        Ok("too late")
    });
    assert!(
        result.is_err(),
        "late completion cannot become a successful connection"
    );
    assert!(
        began.elapsed() < Duration::from_millis(500),
        "timeout must not join a blocked worker"
    );
    completion
        .recv_timeout(Duration::from_secs(3))
        .expect("finite test worker finishes");
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        1,
        "no retries or second connection"
    );
}
