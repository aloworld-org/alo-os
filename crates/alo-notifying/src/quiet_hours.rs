//! The stretch of the clock during which notifications wait.
//!
//! Two times, because that is how a person says it: *hold them between eleven
//! at night and seven in the morning.* Written as a start and a length —
//! *from eleven, for eight hours* — it is the same information spelled so that
//! nobody can check it at a glance.
//!
//! **The stretch wraps, because that is the ordinary case.** From 23:00 to
//! 07:00 spans the night, so the comparison goes round the end of the day; the
//! same two fields the other way round is somebody who wants quiet through
//! their working afternoon, and it works without a second kind of stretch.
//!
//! **A stretch that begins and ends at the same moment is refused.** It would
//! say nothing about any minute of the day, and there is no reading of it that
//! is not a guess at what the person meant.
//!
//! # A time of day is `alo-appearance`'s
//!
//! `alo_appearance::TimeOfDay` is the hour and the minute on the clock the
//! person is looking at, and this crate reads that crate anyway for
//! terracotta. What a stretch of the clock *means for notifications* is this
//! crate's and is here; what a time of day is, is not — and a second spelling
//! of *22:00* on one machine is a second place for the twenty-fifth hour to be
//! allowed. A value in `notifying.toml` that does not read is still said in
//! this crate's words ([`crate::FileNotRead`]), so nobody editing their
//! notification settings is shown a sentence filed under appearance.
//!
//! **Nothing here reads a clock.** The moment is handed in, as it is everywhere
//! else in this repository, so that a settings panel previewing a stretch and
//! the thing obeying it cannot disagree — and so that *what happens at half
//! past eleven* is testable without waiting until half past eleven.

use alo_appearance::TimeOfDay;
use alo_strings::{Filling, Said, Strings, Word};
use serde::{Deserialize, Serialize};

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
/// Reads back through [`QuietHours::these_two_times`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Written", into = "Written")]
pub struct QuietHours {
    /// When they begin.
    begin: TimeOfDay,
    /// When they end.
    end: TimeOfDay,
}

impl QuietHours {
    /// Quiet from this time to that one, every day.
    ///
    /// # Errors
    /// [`NotAStretch`] when the two are the same time, which would say nothing
    /// about any minute of the day.
    pub fn these_two_times(begin: TimeOfDay, end: TimeOfDay) -> Result<Self, NotAStretch> {
        if begin == end {
            return Err(NotAStretch { at: begin });
        }
        Ok(Self { begin, end })
    }

    /// When they begin.
    #[must_use]
    pub const fn begin(self) -> TimeOfDay {
        self.begin
    }

    /// When they end.
    #[must_use]
    pub const fn end(self) -> TimeOfDay {
        self.end
    }

    /// Whether this time of day falls inside them.
    ///
    /// The minute they begin is inside and the minute they end is outside, so
    /// two stretches laid end to end never both hold the same minute.
    #[must_use]
    pub fn holds(self, now: TimeOfDay) -> bool {
        if self.begin < self.end {
            now >= self.begin && now < self.end
        } else {
            // The ordinary case: the stretch runs through midnight, so it is
            // everything from the evening onwards *or* everything before the
            // morning.
            now >= self.begin || now < self.end
        }
    }
}

/// Quiet hours as a settings file holds them.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    /// When they begin.
    begin: TimeOfDay,
    /// When they end.
    end: TimeOfDay,
}

impl TryFrom<Written> for QuietHours {
    type Error = NotRead;

    fn try_from(written: Written) -> Result<Self, Self::Error> {
        Self::these_two_times(written.begin, written.end)
            .map_err(|refused| NotRead::about(refused.word()))
    }
}

impl From<QuietHours> for Written {
    fn from(hours: QuietHours) -> Self {
        Self {
            begin: hours.begin,
            end: hours.end,
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
    use crate::testing::{at, in_english};

    /// **The night is the ordinary case, and it wraps.** Eleven at night to
    /// seven in the morning holds midnight and does not hold the afternoon.
    #[test]
    fn a_stretch_through_the_night_holds_the_night() {
        let night = QuietHours::these_two_times(at(23, 0), at(7, 0)).unwrap();
        assert!(night.holds(at(23, 0)), "the minute it begins is inside");
        assert!(night.holds(at(23, 30)));
        assert!(night.holds(at(0, 0)));
        assert!(night.holds(at(6, 59)));
        assert!(!night.holds(at(7, 0)), "the minute it ends is outside");
        assert!(!night.holds(at(15, 0)));
        assert!(!night.holds(at(22, 59)));
    }

    /// **A stretch inside one day works without a second kind of stretch** —
    /// somebody who wants quiet through their working afternoon.
    #[test]
    fn a_stretch_inside_one_day_holds_that_part_of_it() {
        let afternoon = QuietHours::these_two_times(at(13, 0), at(17, 30)).unwrap();
        assert!(afternoon.holds(at(13, 0)));
        assert!(afternoon.holds(at(17, 29)));
        assert!(!afternoon.holds(at(17, 30)));
        assert!(!afternoon.holds(at(12, 59)));
        assert!(!afternoon.holds(at(2, 0)));
    }

    /// **Two stretches laid end to end never both hold the same minute**,
    /// which is what makes one of them being *inside* an answer rather than an
    /// argument.
    #[test]
    fn two_stretches_laid_end_to_end_never_share_a_minute() {
        let night = QuietHours::these_two_times(at(23, 0), at(7, 0)).unwrap();
        let day = QuietHours::these_two_times(at(7, 0), at(23, 0)).unwrap();
        for hour in 0..24_u8 {
            for minute in [0_u8, 30, 59] {
                let now = at(hour, minute);
                assert_ne!(
                    night.holds(now),
                    day.holds(now),
                    "{hour:02}:{minute:02} is in both or in neither"
                );
            }
        }
    }

    /// **Hours that begin and end at once are refused**, with the time they
    /// were both given said back.
    #[test]
    fn hours_that_begin_and_end_at_once_are_refused() {
        let refused = QuietHours::these_two_times(at(23, 0), at(23, 0)).unwrap_err();
        assert_eq!(refused.at(), at(23, 0));
        let said = refused.said(&in_english());
        assert!(!said.is_a_bug());
        assert!(said.unfilled().is_empty(), "{said}");
    }

    /// **A file that says one is refused where it is read**, rather than
    /// becoming a stretch that holds nothing or everything.
    #[test]
    fn a_file_that_says_one_is_refused_where_it_is_read() {
        let written = "begin = { hour = 23, minute = 0 }\nend = { hour = 7, minute = 0 }\n";
        let read: QuietHours = toml::from_str(written).unwrap();
        assert_eq!(
            read,
            QuietHours::these_two_times(at(23, 0), at(7, 0)).unwrap()
        );

        let same = "begin = { hour = 23, minute = 0 }\nend = { hour = 23, minute = 0 }\n";
        let refused = toml::from_str::<QuietHours>(same).unwrap_err();
        assert!(
            refused
                .to_string()
                .contains("notifying.quiet-hours.begins-and-ends-at-once"),
            "{refused}"
        );

        let no_such_hour = "begin = { hour = 24, minute = 0 }\nend = { hour = 7, minute = 0 }\n";
        assert!(toml::from_str::<QuietHours>(no_such_hour).is_err());
    }
}
