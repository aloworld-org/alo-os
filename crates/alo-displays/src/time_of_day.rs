//! A time on the clock the person is looking at, which is all a night-light
//! schedule needs to know.
//!
//! **Not an instant, and no date.** *Warm the screens after ten* means ten
//! o'clock where the person is, every evening. A schedule written in instants
//! would need a calendar, a list of every timezone in the world and a decision
//! about the night the clocks go back; what it needs instead is the hour and
//! the minute the shell is already showing on its own clock.
//!
//! **Nothing here reads that clock.** The moment is handed in
//! ([`crate::Moment`]), as it is everywhere else in this repository, so that a
//! settings panel previewing a schedule and the compositor obeying it cannot
//! disagree about what the schedule says — and so that *what happens at half
//! past ten* is testable without waiting until half past ten.
//!
//! # Why this crate spells a time of day itself
//!
//! `alo-appearance` has a time of day too, for the schedule that moves a
//! machine between light and dark, and this crate reads that crate for other
//! things. It is not reused here for two reasons that are the same reason.
//! ADR 0038 gives a settings file to the crate that **declares its shape**, and
//! a value in `displays.toml` refused in another crate's words would show
//! somebody editing their screens file a sentence filed under appearance. The
//! second is arithmetic: the sun answers in minutes since midnight
//! ([`crate::sun`]), and a time this crate builds from its own remainders needs
//! a door that cannot fail — which a type belonging to another crate cannot
//! offer without opening that door to everybody.
//!
//! # It is written the way a person writes one
//!
//! `22:00`, on a twenty-four hour clock, because a settings file is a file
//! somebody opens in an editor. How a *person* is shown a time elsewhere
//! belongs to their region rather than to their language, and is not one of
//! this crate's strings.

use std::fmt;

use alo_strings::{Filling, Said, Strings, Word};
use serde::{Deserialize, Serialize};

use crate::unreadable::NotRead;
use crate::words;

/// How many minutes there are in a day, which is what a time of day is
/// compared as.
pub(crate) const MINUTES_IN_A_DAY: u16 = 24 * 60;

/// How many minutes there are in an hour.
const MINUTES_IN_AN_HOUR: u16 = 60;

/// Why a piece of text is not a time of day.
///
/// One refusal rather than three, because *not a time* is one thing to a person
/// mid-edit: `25:00`, `half ten` and `22-00` are the same mistake, and the
/// sentence they need is the shape rather than the diagnosis.
///
/// There is no `Display`: the only road to words is [`NotATime::said`], and
/// what a settings file that did not read writes is [`NotRead`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotATime {
    /// What was offered as a time.
    text: String,
}

impl NotATime {
    /// What was offered as a time.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(&self) -> Word {
        words::NOT_A_TIME
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    ///
    /// The text goes in **quoted**, because what is wrong with it is often a
    /// space or a character that is invisible on its own.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(
            &self.word().key(),
            &Filling::of("name", format!("{:?}", self.text)),
        )
    }
}

/// A time on the clock the person is looking at.
///
/// Ordered, so *later than* is the question a schedule asks. Reads back through
/// [`TimeOfDay::written`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct TimeOfDay {
    /// The hour, 0 to 23. First, so ordering is chronological.
    hour: u8,
    /// The minute, 0 to 59.
    minute: u8,
}

impl TimeOfDay {
    /// A time as it is written: `22:00`, on a twenty-four hour clock.
    ///
    /// # Errors
    /// [`NotATime`], which says the shape a time is written in.
    pub fn written(text: &str) -> Result<Self, NotATime> {
        let refused = || NotATime {
            text: text.to_owned(),
        };
        let (hour, minute) = text.split_once(':').ok_or_else(refused)?;
        if !(1..=2).contains(&hour.len()) || minute.len() != 2 {
            return Err(refused());
        }
        if !hour
            .bytes()
            .chain(minute.bytes())
            .all(|byte| byte.is_ascii_digit())
        {
            return Err(refused());
        }
        let hour: u8 = hour.parse().map_err(|_| refused())?;
        let minute: u8 = minute.parse().map_err(|_| refused())?;
        if hour > 23 || minute > 59 {
            return Err(refused());
        }
        Ok(Self { hour, minute })
    }

    /// This many minutes after midnight, wrapped into the day.
    ///
    /// The one door the arithmetic in [`crate::sun`] and [`crate::moment`] come
    /// through, and one reason this type is this crate's own: a time worked out
    /// from a remainder cannot be out of range, and a refusal nothing can reach
    /// is a branch nobody can test.
    pub(crate) const fn after_midnight(minutes: u16) -> Self {
        let through = minutes % MINUTES_IN_A_DAY;
        Self {
            // Both are remainders of a day and of an hour, so each is inside a
            // byte: the hour is at most 23 and the minute at most 59.
            hour: (through / MINUTES_IN_AN_HOUR) as u8,
            minute: (through % MINUTES_IN_AN_HOUR) as u8,
        }
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

    /// How many minutes after midnight this is, which is how two times are
    /// compared and how a stretch of the clock is measured.
    #[must_use]
    pub const fn minutes_after_midnight(self) -> u16 {
        (self.hour as u16) * MINUTES_IN_AN_HOUR + (self.minute as u16)
    }
}

impl fmt::Display for TimeOfDay {
    /// Twenty-four hours with a leading zero, which is what a settings file
    /// holds and what [`TimeOfDay::written`] reads back.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:02}:{:02}", self.hour, self.minute)
    }
}

impl TryFrom<String> for TimeOfDay {
    type Error = NotRead;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Self::written(&text).map_err(|refused| NotRead::about(refused.word()))
    }
}

impl From<TimeOfDay> for String {
    fn from(time: TimeOfDay) -> Self {
        time.to_string()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// What is written is what comes back, in the spelling a settings file
    /// holds.
    #[test]
    fn a_time_survives_being_written_down() {
        let ten = TimeOfDay::written("22:00").unwrap();
        assert_eq!((ten.hour(), ten.minute()), (22, 0));
        assert_eq!(ten.to_string(), "22:00");
        assert_eq!(TimeOfDay::written("9:05").unwrap().to_string(), "09:05");
        assert_eq!(serde_json::to_string(&ten).unwrap(), r#""22:00""#);
        assert_eq!(
            serde_json::from_str::<TimeOfDay>(r#""22:00""#).unwrap(),
            ten
        );
    }

    /// **Everything that is not a time is refused where it is read**, rather
    /// than becoming a schedule nobody chose.
    #[test]
    fn what_is_not_a_time_is_refused() {
        for written in [
            "",
            "2200",
            "25:00",
            "22:60",
            "22:0",
            "22:000",
            "half ten",
            "22-00",
            "+9:05",
            " 9:05",
            "22:0a",
            "٢٢:٠٠",
        ] {
            assert_eq!(
                TimeOfDay::written(written).unwrap_err().text(),
                written,
                "{written} was read as a time"
            );
        }
        assert!(TimeOfDay::written("00:00").is_ok());
        assert!(TimeOfDay::written("23:59").is_ok());
    }

    /// A refusal says the shape rather than the mistake, because the person is
    /// mid-edit and wants the next thing to type — and it quotes what they
    /// wrote, because the fault is often a character nobody can see.
    #[test]
    fn a_refusal_says_what_a_time_looks_like_and_quotes_what_was_written() {
        let strings = in_english();
        let said = TimeOfDay::written("25:00").unwrap_err().said(&strings);
        assert!(said.text().contains("22:00"), "{said}");
        assert!(said.text().contains("\"25:00\""), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
        assert!(!said.is_a_bug(), "{said}");
    }

    /// **A time out of a settings file is checked again on the way in**, and a
    /// file that did not read writes the key of the same refusal.
    #[test]
    fn a_time_out_of_a_settings_file_is_checked_again() {
        let refused = serde_json::from_str::<TimeOfDay>(r#""25:00""#).unwrap_err();
        assert!(
            refused.to_string().contains("displays.not-a-time"),
            "{refused}"
        );
    }

    /// Minutes after midnight are the one measure two times are compared by,
    /// and the door this crate's own arithmetic comes through wraps rather than
    /// refusing.
    #[test]
    fn a_time_is_minutes_after_midnight_and_the_arithmetic_wraps() {
        assert_eq!(
            TimeOfDay::written("00:00")
                .unwrap()
                .minutes_after_midnight(),
            0
        );
        assert_eq!(
            TimeOfDay::written("22:00")
                .unwrap()
                .minutes_after_midnight(),
            1320
        );
        assert_eq!(
            TimeOfDay::written("23:59")
                .unwrap()
                .minutes_after_midnight(),
            1439
        );
        assert_eq!(TimeOfDay::after_midnight(1320).to_string(), "22:00");
        assert_eq!(TimeOfDay::after_midnight(0).to_string(), "00:00");
        assert_eq!(
            TimeOfDay::after_midnight(MINUTES_IN_A_DAY).to_string(),
            "00:00",
            "a day on is the same time again"
        );
        assert!(TimeOfDay::written("07:00").unwrap() < TimeOfDay::written("22:00").unwrap());
    }
}
