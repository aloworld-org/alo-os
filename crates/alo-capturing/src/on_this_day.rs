//! The moment a picture was taken, as a calendar date.
//!
//! A picture of the screen is saved under the moment it was taken and under
//! nothing else ([`crate::naming`]), so something has to turn a machine's clock
//! into a year, a month and a day. This is that, and it is the only arithmetic
//! of its kind in this crate.
//!
//! # There is one road in, and it starts at a clock
//!
//! [`OnThisDay::at`] takes a [`SystemTime`] and the machine's offset from
//! universal time, and there is no other constructor: no `from_str`, no
//! deserialiser, no six numbers a caller can hand over. That is not tidiness.
//! A date somebody could write by hand is a file name somebody could write by
//! hand, and the plan's promise is that a picture's name says **nothing about
//! what was on the screen** — which is only a property of the type if the type
//! cannot be told anything but a moment.
//!
//! # Why the offset is handed in rather than looked up
//!
//! A person in Auckland taking a picture at nine in the morning should find it
//! under that morning's date, not under yesterday's. Which offset a machine is
//! at is a setting alo OS does not yet have a crate for, and a library that
//! read one out of the environment would be reading a setting nobody chose.
//! So the moment and the offset both arrive from the caller, exactly as `now`
//! does everywhere else in this repository, and nothing here reads a clock.
//!
//! # The numbers are what a calendar says, and they are only ever formatted
//!
//! Every field is an `i64`, which is what the arithmetic below produces and
//! what [`std::fmt`] is handed. Narrowing them to a byte apiece would buy
//! nothing and would need a lossy cast or a refusal for a month that cannot
//! exist, and a refusal nothing can reach is a branch nobody can test.

use std::time::{SystemTime, UNIX_EPOCH};

/// How many seconds a day is, on a machine's clock.
const A_DAY: i64 = 24 * 60 * 60;

/// How many seconds an hour is.
const AN_HOUR: i64 = 60 * 60;

/// How many seconds a minute is.
const A_MINUTE: i64 = 60;

/// The moment a picture was taken, on the calendar the person reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OnThisDay {
    /// The year.
    year: i64,
    /// The month, from one to twelve.
    month: i64,
    /// The day of the month, from one.
    day: i64,
    /// The hour, from zero to twenty-three.
    hour: i64,
    /// The minute, from zero to fifty-nine.
    minute: i64,
    /// The second, from zero to fifty-nine.
    second: i64,
}

impl OnThisDay {
    /// This moment, on the calendar of a machine this far from universal time.
    ///
    /// The offset is in minutes east, so central European summer time is `120`
    /// and Newfoundland's winter is `-210`. Minutes rather than hours because
    /// several places in the world are not a whole number of hours from
    /// universal time, and a library that could not say where somebody was is a
    /// library that puts their picture under the wrong day.
    #[must_use]
    pub fn at(when: SystemTime, minutes_east_of_universal_time: i32) -> Self {
        let seconds = match when.duration_since(UNIX_EPOCH) {
            Ok(since) => i64::try_from(since.as_secs()).unwrap_or(i64::MAX),
            Err(before) => i64::try_from(before.duration().as_secs())
                .unwrap_or(i64::MAX)
                .saturating_neg(),
        };
        let here = seconds.saturating_add(i64::from(minutes_east_of_universal_time) * A_MINUTE);
        let (year, month, day) = the_date_of(here.div_euclid(A_DAY));
        let through_the_day = here.rem_euclid(A_DAY);
        Self {
            year,
            month,
            day,
            hour: through_the_day / AN_HOUR,
            minute: (through_the_day % AN_HOUR) / A_MINUTE,
            second: through_the_day % A_MINUTE,
        }
    }

    /// The year.
    #[must_use]
    pub const fn year(self) -> i64 {
        self.year
    }

    /// The month, from one to twelve.
    #[must_use]
    pub const fn month(self) -> i64 {
        self.month
    }

    /// The day of the month, from one.
    #[must_use]
    pub const fn day(self) -> i64 {
        self.day
    }

    /// The hour, from zero to twenty-three.
    #[must_use]
    pub const fn hour(self) -> i64 {
        self.hour
    }

    /// The minute, from zero to fifty-nine.
    #[must_use]
    pub const fn minute(self) -> i64 {
        self.minute
    }

    /// The second, from zero to fifty-nine.
    #[must_use]
    pub const fn second(self) -> i64 {
        self.second
    }
}

/// The year, month and day this many days after the first of January 1970.
///
/// Howard Hinnant's `civil_from_days`, which is the shortest correct answer to
/// a question every calendar gets wrong somewhere: it is exact for every day a
/// machine's clock can hold, leap years and leap centuries included, and it has
/// no table in it. The shift of 719 468 days moves the epoch to the first of
/// March in the year zero, which is what makes a leap day the last day of a
/// year rather than a hole in the middle of one.
fn the_date_of(days_since_the_epoch: i64) -> (i64, i64, i64) {
    let shifted = days_since_the_epoch.saturating_add(719_468);
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let months_from_march = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * months_from_march + 2) / 5 + 1;
    let month = if months_from_march < 10 {
        months_from_march + 3
    } else {
        months_from_march - 9
    };
    (if month <= 2 { year + 1 } else { year }, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// A moment, as a machine's clock has it.
    fn at(seconds_since_the_epoch: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds_since_the_epoch)
    }

    /// **The epoch is the first of January 1970**, which is the one date every
    /// reader can check against by hand.
    #[test]
    fn the_epoch_is_the_first_of_january_nineteen_seventy() {
        let day = OnThisDay::at(UNIX_EPOCH, 0);
        assert_eq!(
            (day.year(), day.month(), day.day()),
            (1970, 1, 1),
            "{day:?}"
        );
        assert_eq!((day.hour(), day.minute(), day.second()), (0, 0, 0));
    }

    /// **A moment on a real machine is the date on a real calendar.**
    /// 1 789 560 000 seconds after the epoch is noon on the sixteenth of
    /// September 2026, universal time.
    #[test]
    fn a_moment_is_the_date_a_calendar_says_it_is() {
        let day = OnThisDay::at(at(1_789_560_000), 0);
        assert_eq!((day.year(), day.month(), day.day()), (2026, 9, 16));
        assert_eq!((day.hour(), day.minute(), day.second()), (12, 0, 0));
    }

    /// **The offset is what decides which day it is.** The same instant is one
    /// day in Auckland and another in Honolulu, and a picture has to appear
    /// under the date the person taking it would say.
    #[test]
    fn the_offset_decides_which_day_a_moment_falls_on() {
        let moment = at(1_789_560_000);
        let universal = OnThisDay::at(moment, 0);
        assert_eq!((universal.month(), universal.day(), universal.hour()), {
            (9, 16, 12)
        });

        // Auckland in September is twelve hours east: the next day has begun.
        let auckland = OnThisDay::at(moment, 12 * 60);
        assert_eq!((auckland.month(), auckland.day(), auckland.hour()), {
            (9, 17, 0)
        });

        // Honolulu is ten hours west: still early on the same morning.
        let honolulu = OnThisDay::at(moment, -10 * 60);
        assert_eq!((honolulu.month(), honolulu.day(), honolulu.hour()), {
            (9, 16, 2)
        });
    }

    /// **An offset that is not a whole number of hours works too.** India is
    /// five and a half hours east, Nepal five and three quarters, and a library
    /// that could only say hours would put a third of the world's pictures
    /// under the wrong minute, and some of them under the wrong day.
    #[test]
    fn an_offset_that_is_not_a_whole_hour_is_a_place_people_live() {
        let day = OnThisDay::at(at(1_789_560_000), 5 * 60 + 30);
        assert_eq!((day.month(), day.day()), (9, 16));
        assert_eq!((day.hour(), day.minute()), (17, 30));
    }

    /// **Leap days are days.** The twenty-ninth of February 2024 and the first
    /// of March 2000 — a leap year, and the leap century that the simple rule
    /// gets wrong — are both exactly where a calendar puts them.
    #[test]
    fn leap_days_and_leap_centuries_land_where_a_calendar_puts_them() {
        // 1 709 208 000 is 2024-02-29T12:00:00Z.
        let leap_day = OnThisDay::at(at(1_709_208_000), 0);
        assert_eq!((leap_day.year(), leap_day.month(), leap_day.day()), {
            (2024, 2, 29)
        });

        // 951 868 800 is 2000-03-01T00:00:00Z, the day after a leap day that
        // the every-hundred-years rule alone would have removed.
        let after_the_century = OnThisDay::at(at(951_868_800), 0);
        assert_eq!(
            (
                after_the_century.year(),
                after_the_century.month(),
                after_the_century.day()
            ),
            (2000, 3, 1)
        );

        // 4_107_542_400 is 2100-03-01T00:00:00Z: 2100 is *not* a leap year, and
        // the day before it is the twenty-eighth of February.
        let not_a_leap_year = OnThisDay::at(at(4_107_542_400), 0);
        assert_eq!(
            (
                not_a_leap_year.year(),
                not_a_leap_year.month(),
                not_a_leap_year.day()
            ),
            (2100, 3, 1)
        );
        let the_day_before = OnThisDay::at(at(4_107_542_400 - 86_400), 0);
        assert_eq!((the_day_before.month(), the_day_before.day()), (2, 28));
    }

    /// **A day is twenty-four hours of its own**, and every one of them lands
    /// on the day it belongs to — the last second of a day is that day's, and
    /// the next second is the next day's first.
    #[test]
    fn the_last_second_of_a_day_is_that_days() {
        // 1 789 603 199 is 2026-09-16T23:59:59Z.
        let last = OnThisDay::at(at(1_789_603_199), 0);
        assert_eq!((last.day(), last.hour(), last.minute(), last.second()), {
            (16, 23, 59, 59)
        });
        let next = OnThisDay::at(at(1_789_603_200), 0);
        assert_eq!((next.day(), next.hour(), next.minute(), next.second()), {
            (17, 0, 0, 0)
        });
    }

    /// **A clock set before the epoch does not panic and does not wrap.**
    /// Machines with a dead battery come up in 1970 or earlier, and a picture
    /// taken on one is still saved somewhere sensible rather than taking the
    /// shell down.
    #[test]
    fn a_clock_from_before_the_epoch_is_still_a_date() {
        let before = UNIX_EPOCH - Duration::from_secs(86_400);
        let day = OnThisDay::at(before, 0);
        assert_eq!((day.year(), day.month(), day.day()), (1969, 12, 31));
    }
}
