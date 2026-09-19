//! A stretch of the clock, which is nearly always one that runs through
//! midnight.
//!
//! Night light is on *between ten in the evening and seven in the morning*.
//! Written as two times it is a thing a person can check at a glance and a
//! settings panel can show back to them; written as a start and a length —
//! *from ten, for nine hours* — it is the same information spelled so that
//! nobody can. So it is two times.
//!
//! **The stretch wraps, because that is the ordinary case.** From 22:00 to
//! 07:00 spans the night, so the comparison goes round the end of the day; the
//! same two fields the other way round — from 07:00 to 22:00 — is somebody who
//! works nights and wants their screens warm through the afternoon, and it
//! works without a second kind of stretch.
//!
//! **A stretch that begins and ends at the same moment is refused.** It would
//! be a stretch that says nothing about any minute of the day, and there is no
//! reading of it that is not a guess at what the person meant.

use serde::{Deserialize, Serialize};

use alo_strings::{Filling, Said, Strings, Word};

use crate::time_of_day::TimeOfDay;
use crate::unreadable::NotRead;
use crate::words;

/// Why two times are not a stretch of the clock.
///
/// There is no `Display`: the only road to words is [`NotAStretch::said`], and
/// what a settings file that did not read writes is [`NotRead`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotAStretch {
    /// The time that was given twice.
    at: TimeOfDay,
}

impl NotAStretch {
    /// The time that was given twice.
    #[must_use]
    pub const fn at(self) -> TimeOfDay {
        self.at
    }

    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(self) -> Word {
        words::BEGINS_AND_ENDS_AT_ONCE
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

/// From one time of day to another, every day.
///
/// Reads back through [`Between::these_two_times`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Written", into = "Written")]
pub struct Between {
    /// When it begins.
    begins: TimeOfDay,
    /// When it ends.
    ends: TimeOfDay,
}

impl Between {
    /// From this time to that one, every day.
    ///
    /// # Errors
    /// [`NotAStretch`] when the two are the same time, which would say nothing
    /// about any minute of the day.
    pub const fn these_two_times(begins: TimeOfDay, ends: TimeOfDay) -> Result<Self, NotAStretch> {
        if begins.minutes_after_midnight() == ends.minutes_after_midnight() {
            return Err(NotAStretch { at: begins });
        }
        Ok(Self { begins, ends })
    }

    /// A stretch this crate's own arithmetic worked out, which cannot be
    /// empty because [`crate::sun`] only builds one when the sun both sets and
    /// rises.
    pub(crate) const fn worked_out(begins: TimeOfDay, ends: TimeOfDay) -> Self {
        Self { begins, ends }
    }

    /// When it begins.
    #[must_use]
    pub const fn begins(self) -> TimeOfDay {
        self.begins
    }

    /// When it ends.
    #[must_use]
    pub const fn ends(self) -> TimeOfDay {
        self.ends
    }

    /// Whether this time of day falls inside it.
    ///
    /// The moment it begins is inside and the moment it ends is outside, so two
    /// stretches laid end to end never both hold the same minute.
    #[must_use]
    pub const fn holds(self, now: TimeOfDay) -> bool {
        let now = now.minutes_after_midnight();
        let begins = self.begins.minutes_after_midnight();
        let ends = self.ends.minutes_after_midnight();
        if begins < ends {
            now >= begins && now < ends
        } else {
            // The ordinary case: the stretch runs through midnight, so it is
            // everything from the evening onwards *or* everything before the
            // morning.
            now >= begins || now < ends
        }
    }
}

/// A stretch as a settings file holds it.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    /// When it begins.
    from: TimeOfDay,
    /// When it ends.
    to: TimeOfDay,
}

impl TryFrom<Written> for Between {
    type Error = NotRead;

    fn try_from(written: Written) -> Result<Self, Self::Error> {
        Self::these_two_times(written.from, written.to)
            .map_err(|refused| NotRead::about(refused.word()))
    }
}

impl From<Between> for Written {
    fn from(between: Between) -> Self {
        Self {
            from: between.begins,
            to: between.ends,
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
    use crate::testing::in_english;

    /// A time of day, for the tests below.
    fn at(written: &str) -> TimeOfDay {
        TimeOfDay::written(written).unwrap()
    }

    /// **The ordinary stretch runs through midnight**, and every minute of the
    /// night is inside it while every minute of the working day is not.
    #[test]
    fn a_stretch_through_the_night_holds_the_night_and_nothing_else() {
        let night = Between::these_two_times(at("22:00"), at("07:00")).unwrap();
        for inside in ["22:00", "23:59", "00:00", "03:14", "06:59"] {
            assert!(night.holds(at(inside)), "{inside}");
        }
        for outside in ["07:00", "12:00", "21:59"] {
            assert!(!night.holds(at(outside)), "{outside}");
        }
        assert_eq!(night.begins(), at("22:00"));
        assert_eq!(night.ends(), at("07:00"));
    }

    /// **And the same two fields the other way round is somebody who works
    /// nights**, which needs no second kind of stretch.
    #[test]
    fn the_same_two_times_the_other_way_round_is_the_daytime() {
        let daytime = Between::these_two_times(at("07:00"), at("22:00")).unwrap();
        assert!(daytime.holds(at("12:00")));
        assert!(!daytime.holds(at("03:14")));
        assert!(daytime.holds(at("07:00")), "the moment it begins is inside");
        assert!(!daytime.holds(at("22:00")), "the moment it ends is outside");
    }

    /// **A stretch that begins and ends at the same moment is refused**, in
    /// words, rather than becoming a schedule that holds every minute or none.
    #[test]
    fn a_stretch_that_begins_and_ends_at_once_is_refused() {
        let refused = Between::these_two_times(at("22:00"), at("22:00")).unwrap_err();
        assert_eq!(refused.at(), at("22:00"));
        let said = refused.said(&in_english());
        assert!(said.text().contains("same moment"), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
        assert!(!said.is_a_bug(), "{said}");
    }

    /// A stretch survives a settings file, and one that begins and ends at the
    /// same moment is refused again on the way in.
    #[test]
    fn a_stretch_survives_a_file_and_an_empty_one_is_refused_again() {
        let night = Between::these_two_times(at("22:00"), at("07:00")).unwrap();
        let written = serde_json::to_string(&night).unwrap();
        assert_eq!(written, r#"{"from":"22:00","to":"07:00"}"#);
        assert_eq!(serde_json::from_str::<Between>(&written).unwrap(), night);

        let refused =
            serde_json::from_str::<Between>(r#"{"from":"22:00","to":"22:00"}"#).unwrap_err();
        assert!(
            refused
                .to_string()
                .contains("displays.begins-and-ends-at-once"),
            "{refused}"
        );
        assert!(
            serde_json::from_str::<Between>(r#"{"from":"22:00","to":"07:00","and":1}"#).is_err(),
            "a key this shape does not have is refused where it is read"
        );
    }
}
