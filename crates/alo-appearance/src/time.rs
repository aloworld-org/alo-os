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

use alo_strings::{Filling, Said, Strings, Word};
use serde::{Deserialize, Serialize};

use crate::unreadable::NotRead;
use crate::words;

/// Why an hour and a minute are not a time of day.
///
/// There is no `Display`: the only road to words is [`TimeError::said`], and
/// what a settings file that did not read writes is [`NotRead`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeError {
    /// An hour past 23.
    NoSuchHour(u8),
    /// A minute past 59.
    NoSuchMinute(u8),
}

impl TimeError {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::NoSuchHour(_) => words::NO_SUCH_HOUR,
            Self::NoSuchMinute(_) => words::NO_SUCH_MINUTE,
        }
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        let filling = match self {
            Self::NoSuchHour(hour) => Filling::of("hour", hour.to_string()),
            Self::NoSuchMinute(minute) => Filling::of("minute", minute.to_string()),
        };
        strings.say(&self.word().key(), &filling)
    }
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
    type Error = NotRead;

    fn try_from(written: Written) -> Result<Self, Self::Error> {
        Self::checked(written.hour, written.minute)
            .map_err(|refused| NotRead::about(refused.word()))
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
    use crate::testing::{in_english, translated};

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

    /// A refusal says the range rather than the number, in the language the
    /// person reads.
    #[test]
    fn a_refusal_says_what_the_range_is() {
        let strings = in_english();
        assert_eq!(
            TimeOfDay::checked(24, 0).unwrap_err().said(&strings).text(),
            "24 is not an hour of the day — the day runs from 0 to 23"
        );
        assert_eq!(
            TimeOfDay::checked(6, 60).unwrap_err().said(&strings).text(),
            "60 is not a minute of the hour — an hour runs from 0 to 59"
        );

        let auf_deutsch = translated(&[(
            words::NO_SUCH_HOUR,
            "{hour} ist keine Stunde des Tages — der Tag läuft von 0 bis 23",
        )]);
        let said = TimeOfDay::checked(24, 0).unwrap_err().said(&auf_deutsch);
        assert_eq!(
            said.text(),
            "24 ist keine Stunde des Tages — der Tag läuft von 0 bis 23"
        );
        assert!(said.is_translated());
        assert!(said.unfilled().is_empty());
    }

    /// A settings file holding a twenty-fifth hour writes the key of the
    /// refusal, because a deserialiser has no `Strings` to ask.
    #[test]
    fn a_file_that_did_not_read_names_the_string_rather_than_saying_it() {
        let refused = serde_json::from_str::<TimeOfDay>(r#"{"hour":24,"minute":0}"#).unwrap_err();
        assert!(
            refused.to_string().contains("appearance.time.no-such-hour"),
            "{refused}"
        );
    }
}
