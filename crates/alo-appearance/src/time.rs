//! A time of day, which is all a schedule needs to know.
//!
//! **Not an instant, and no date.** *Use dark after six* means six o'clock where
//! the person is, every day, and a schedule written in instants would need a
//! calendar, a timezone database and a decision about what happens on the night
//! the clocks go back. What it needs instead is the hour and the minute the
//! shell is showing on its own clock, which is a thing the compositor already
//! knows.
//!
//! **Nothing here reads that clock either.** As in `alo-capability`, the moment
//! is passed in — [`crate::scheme::Following::at`] takes it — so what the shell
//! draws at half past six is testable without waiting until half past six, and
//! the settings panel previewing a schedule and the compositor obeying it cannot
//! disagree about what the schedule says.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Why an hour and a minute are not a time of day.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TimeError {
    /// An hour past 23.
    #[error("{0} is not an hour of the day — the day runs from 0 to 23")]
    NoSuchHour(u8),
    /// A minute past 59.
    #[error("{0} is not a minute of the hour — an hour runs from 0 to 59")]
    NoSuchMinute(u8),
}

/// A time of day on the clock the person is looking at.
///
/// Ordered, so *later than* is the question a schedule asks. Reads back through
/// [`TimeOfDay::checked`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "Written", into = "Written")]
pub struct TimeOfDay {
    /// The hour, 0 to 23. First, so ordering is chronological.
    hour: u8,
    /// The minute, 0 to 59.
    minute: u8,
}

impl TimeOfDay {
    /// An hour and a minute.
    ///
    /// # Errors
    /// [`TimeError`], which says the range the number is outside.
    pub fn checked(hour: u8, minute: u8) -> Result<Self, TimeError> {
        if hour > 23 {
            return Err(TimeError::NoSuchHour(hour));
        }
        if minute > 59 {
            return Err(TimeError::NoSuchMinute(minute));
        }
        Ok(Self { hour, minute })
    }

    /// A time the compiler can build, for the schedule this crate ships.
    ///
    /// Unchecked, and the only caller is [`crate::shipped`] — which is held to
    /// the same rules by a test that puts every shipped time back through
    /// [`TimeOfDay::checked`].
    pub(crate) const fn shipped(hour: u8, minute: u8) -> Self {
        Self { hour, minute }
    }

    /// The hour, 0 to 23.
    #[must_use]
    pub const fn hour(self) -> u8 {
        self.hour
    }

    /// The minute, 0 to 59.
    #[must_use]
    pub const fn minute(self) -> u8 {
        self.minute
    }
}

impl fmt::Display for TimeOfDay {
    /// Twenty-four hours with a leading zero, which is what a settings file
    /// holds. How a *person* is shown a time is their region's business and is
    /// item 9's, not this file's.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.hour, self.minute)
    }
}

/// A time of day as a settings file holds it.
#[derive(Serialize, Deserialize)]
struct Written {
    /// The hour, 0 to 23.
    hour: u8,
    /// The minute, 0 to 59.
    minute: u8,
}

impl TryFrom<Written> for TimeOfDay {
    type Error = TimeError;

    fn try_from(written: Written) -> Result<Self, Self::Error> {
        Self::checked(written.hour, written.minute)
    }
}

impl From<TimeOfDay> for Written {
    fn from(time: TimeOfDay) -> Self {
        Self {
            hour: time.hour,
            minute: time.minute,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// Both ends of the day are ordinary times, and the middle of the night is
    /// not a special case.
    #[test]
    fn the_whole_day_is_a_time_of_day() {
        assert_eq!(TimeOfDay::checked(0, 0).unwrap().to_string(), "00:00");
        assert_eq!(TimeOfDay::checked(23, 59).unwrap().to_string(), "23:59");
        assert_eq!(TimeOfDay::checked(6, 5).unwrap().to_string(), "06:05");
    }

    /// Times compare chronologically, which is the only question a schedule
    /// asks of one.
    #[test]
    fn times_run_in_the_order_the_day_does() {
        let six = TimeOfDay::checked(6, 0).unwrap();
        let half_six = TimeOfDay::checked(6, 30).unwrap();
        let seven = TimeOfDay::checked(7, 0).unwrap();
        assert!(six < half_six);
        assert!(half_six < seven);
        assert!(TimeOfDay::checked(0, 0).unwrap() < TimeOfDay::checked(23, 59).unwrap());
    }

    /// A file is a thing a person edits, so a 25th hour is refused where it is
    /// read rather than becoming a schedule that never fires.
    #[test]
    fn there_is_no_twenty_fifth_hour() {
        assert_eq!(TimeOfDay::checked(24, 0), Err(TimeError::NoSuchHour(24)));
        assert_eq!(TimeOfDay::checked(6, 60), Err(TimeError::NoSuchMinute(60)));
        assert!(serde_json::from_str::<TimeOfDay>(r#"{"hour":24,"minute":0}"#).is_err());

        let six = TimeOfDay::checked(18, 0).unwrap();
        let written = serde_json::to_string(&six).unwrap();
        assert_eq!(written, r#"{"hour":18,"minute":0}"#);
        assert_eq!(serde_json::from_str::<TimeOfDay>(&written).unwrap(), six);
    }

    /// A refusal says the range rather than the number.
    #[test]
    fn a_refusal_says_what_the_range_is() {
        assert_eq!(
            TimeOfDay::checked(24, 0).unwrap_err().to_string(),
            "24 is not an hour of the day — the day runs from 0 to 23"
        );
        assert_eq!(
            TimeOfDay::checked(6, 60).unwrap_err().to_string(),
            "60 is not a minute of the hour — an hour runs from 0 to 59"
        );
    }
}
