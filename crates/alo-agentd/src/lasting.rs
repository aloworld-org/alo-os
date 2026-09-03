//! How long a turn lasts, and how long a change waits for an answer.
//!
//! Two numbers a machine is told about itself, and they are one type because
//! everything that is true of one is true of the other: both are lengths of
//! time in whole seconds, both are refused at zero, and both have the same
//! ceiling for the same reason.
//!
//! # No time at all is not a length of time
//!
//! `alo_capability::Grant::checked` already refuses `Duration::ZERO`, so a
//! machine described with a turn of nought would start, open its socket, accept
//! the first agent and then stop with [`NotServed::NoTurn`] — the service
//! failing at the first caller for something that was wrong before it started.
//! Refusing it here is the same rule made where somebody can act on it, and it
//! is `alo_keeping::Keeping`'s shape rather than a new one: the value is a
//! [`NonZeroU64`], and the one road a zero can still arrive by is
//! [`Lasting::of_seconds`], where a description lands.
//!
//! [`NotServed::NoTurn`]: crate::NotServed::NoTurn
//!
//! # And a day is the longest either may be
//!
//! This is alo OS's number rather than an organisation's, and the difference
//! from `alo_keeping::Keeping` — which deliberately ships no number of days,
//! because how long an organisation may keep evidence has a legal answer in some
//! places and a cultural one in others — is what the two numbers are *about*.
//!
//! How long a change waits for an answer is a promise the capability model
//! makes. `CLAUDE.md`: *what a person approves is that sentence, and an approval
//! is never a session.* A proposal that stands for a week is an approval given
//! on Monday running on Friday's machine, against a folder whose contents
//! nobody looked at since; a turn's own grant outliving the day is the ambient
//! authority ADR 0001 §2 exists to prevent, arriving through a configuration
//! file. Neither is a policy an organisation gets to loosen, so the ceiling is
//! here and it is refused in words rather than clamped — a machine that quietly
//! shortened what somebody wrote would be running under a description nobody
//! wrote.
//!
//! A day is also what keeps the arithmetic behind these honest: `Grant::checked`
//! adds the length to the moment a turn began and refuses what it cannot add, so
//! a description holding `u64::MAX` would otherwise be a service that starts and
//! then refuses every turn.

use std::num::NonZeroU64;
use std::time::Duration;

use crate::refusing::NotDescribed;

/// The longest a turn may last or a proposal may stand, in seconds.
///
/// Twenty-four hours. See this file's own documentation for why the number is
/// alo OS's rather than an organisation's.
pub const AT_MOST: u64 = 24 * 60 * 60;

/// A length of time this machine was told, in whole seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Lasting(NonZeroU64);

impl Lasting {
    /// This many whole seconds, if that is a length of time.
    ///
    /// `what` is the key in the description this number came from, so that a
    /// refusal names what somebody has to go and edit rather than describing it
    /// — `alo_shortcuts::DefaultsError`'s rule, one crate on: the reader is
    /// standing the machine up and wants the line, not the concept.
    ///
    /// # Errors
    ///
    /// [`NotDescribed::NoTimeAtAll`] for zero and [`NotDescribed::TooLong`]
    /// above [`AT_MOST`]. Both are described in this file's own documentation.
    pub const fn of_seconds(seconds: u64, what: &'static str) -> Result<Self, NotDescribed> {
        if seconds > AT_MOST {
            return Err(NotDescribed::TooLong {
                what,
                seconds,
                at_most: AT_MOST,
            });
        }
        match NonZeroU64::new(seconds) {
            None => Err(NotDescribed::NoTimeAtAll { what }),
            Some(seconds) => Ok(Self(seconds)),
        }
    }

    /// How many whole seconds, for whoever is reporting what this machine is.
    #[must_use]
    pub const fn seconds(self) -> u64 {
        self.0.get()
    }

    /// The same length of time, as the rest of the workspace takes it.
    #[must_use]
    pub const fn duration(self) -> Duration {
        Duration::from_secs(self.0.get())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The key a test's refusals are said to have come from.
    const A_KEY: &str = "agent.turn-seconds";

    /// An ordinary length of time comes back as the duration the rest of the
    /// workspace takes, and says how it was written.
    #[test]
    fn a_length_of_time_is_the_seconds_it_was_written_as() {
        let quarter_of_an_hour = Lasting::of_seconds(900, A_KEY).unwrap();
        assert_eq!(quarter_of_an_hour.seconds(), 900);
        assert_eq!(quarter_of_an_hour.duration(), Duration::from_secs(900));
    }

    /// **No time at all is refused where somebody can fix it**, rather than at
    /// the first agent that connects. The refusal names the key in the file.
    #[test]
    fn no_time_at_all_is_refused_and_names_the_key() {
        let refused = Lasting::of_seconds(0, A_KEY).unwrap_err();
        assert!(matches!(refused, NotDescribed::NoTimeAtAll { what } if what == A_KEY));
        assert!(refused.to_string().contains(A_KEY), "{refused}");
    }

    /// **A second is a length of time**, and the refusal is of zero rather than
    /// of anything a person might think too short. How long a turn should be is
    /// theirs.
    #[test]
    fn one_second_is_a_length_of_time() {
        assert_eq!(Lasting::of_seconds(1, A_KEY).unwrap().seconds(), 1);
    }

    /// The ceiling is a day, and the day itself is on the right side of it.
    #[test]
    fn a_whole_day_is_allowed_and_a_second_more_is_not() {
        assert_eq!(
            Lasting::of_seconds(AT_MOST, A_KEY).unwrap().seconds(),
            86_400
        );

        let refused = Lasting::of_seconds(AT_MOST + 1, A_KEY).unwrap_err();
        assert!(matches!(refused, NotDescribed::TooLong { .. }));
    }

    /// **What is too long is refused rather than shortened.** A machine that
    /// quietly clamped a week down to a day would be running under a
    /// description nobody wrote, and the person who wrote the week would have
    /// no way to find out.
    #[test]
    fn something_too_long_is_refused_rather_than_clamped() {
        let refused = Lasting::of_seconds(7 * AT_MOST, A_KEY).unwrap_err();
        let said = refused.to_string();
        assert!(said.contains("604800"), "{said}");
        assert!(said.contains("86400"), "{said}");
        assert!(said.contains("an approval is never a session"), "{said}");
    }

    /// **The largest number a description can hold is refused here** rather
    /// than by the arithmetic inside a turn, so a machine described with it
    /// never starts.
    #[test]
    fn the_largest_number_there_is_is_refused_here() {
        assert!(matches!(
            Lasting::of_seconds(u64::MAX, A_KEY).unwrap_err(),
            NotDescribed::TooLong { .. }
        ));
    }

    /// Two lengths of time compare as the numbers they are, which is what lets
    /// a description be checked against itself: a proposal that stands longer
    /// than the turn it was made in is a thing somebody may want to notice.
    #[test]
    fn lengths_of_time_compare_as_numbers() {
        let shorter = Lasting::of_seconds(300, A_KEY).unwrap();
        let longer = Lasting::of_seconds(900, A_KEY).unwrap();
        assert!(shorter < longer);
    }
}
