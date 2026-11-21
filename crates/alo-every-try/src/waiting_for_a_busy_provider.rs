//! **A provider that is busy is waited for** (the coordinator, 2026-11-20):
//! a 429 or a 5xx, or a provider that took too long, is asked again after a
//! wait — longer each time, a few minutes in all — before the try counts as
//! "could not be asked". A busy answer is never read as the model's.
//!
//! Measured 2026-11-20: with several every-try runs at once, most tries came
//! back `HavingTrouble(429)` on the new key. The provider road
//! (`alo_asking`) reports the status and not the provider's headers, so the
//! waits are a declared schedule rather than its `Retry-After`; the
//! workflow reads the rate-limit headers once a run and shows them.

use std::time::Duration;

use alo_answering::WentWrong;

/// How one attempt at a question went.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attempted<T> {
    /// It was answered.
    Answered(T),
    /// The provider was busy, and may answer later.
    Busy(String),
    /// Refused in a way waiting cannot mend.
    Refused(String),
}

/// **The waits between attempts**: 5, 10, 20, 40, 60 and 60 seconds — three
/// minutes and a quarter in all, then the try is not asked.
pub const THE_WAITS: [Duration; 6] = [
    Duration::from_secs(5),
    Duration::from_secs(10),
    Duration::from_secs(20),
    Duration::from_secs(40),
    Duration::from_secs(60),
    Duration::from_secs(60),
];

/// Ask through `attempt` until answered, waiting through `wait`.
///
/// # Errors
/// Why it was not answered.
pub fn asking_until_answered<T>(
    attempt: &mut dyn FnMut() -> Attempted<T>,
    wait: &mut dyn FnMut(Duration),
) -> Result<T, String> {
    let mut waits = THE_WAITS.iter();
    loop {
        match attempt() {
            Attempted::Answered(answer) => return Ok(answer),
            Attempted::Refused(why) => return Err(why),
            Attempted::Busy(why) => match waits.next() {
                Some(pause) => wait(*pause),
                None => return Err(format!("the provider was busy every time: {why}")),
            },
        }
    }
}

/// Whether `went_wrong` is a busy provider.
#[must_use]
pub const fn is_busy(went_wrong: &WentWrong) -> bool {
    match went_wrong {
        WentWrong::HavingTrouble(status) => *status == 429 || (*status >= 500 && *status <= 599),
        WentWrong::TookTooLong => true,
        _ => false,
    }
}

#[cfg(test)]
#[path = "waiting_for_a_busy_provider_tests.rs"]
mod tests;
