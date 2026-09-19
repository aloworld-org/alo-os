//! What the machine's clock says, handed in rather than read.
//!
//! Night light is the one thing about a person's screens that depends on when
//! it is, so something has to carry *now* into this crate. This is that, and
//! it is deliberately the only road: nothing here reads a clock, an environment
//! variable or a timezone database, exactly as
//! `alo_capturing::OnThisDay` does for the same reason. A library that read a
//! setting nobody chose would be a library deciding where somebody is.
//!
//! # Three things, and no fourth
//!
//! A moment is **the day**, as the number astronomers count days by; **the time
//! of day** on the clock the person is looking at; and **how far that clock is
//! from universal time**, in minutes east. The third is what turns a sunset
//! worked out for the whole earth into a time on somebody's own clock, and it
//! is minutes rather than hours because several places in the world are not a
//! whole number of hours from universal time.
//!
//! **The offset is a fact about the machine and not a location.** It says which
//! clock the person keeps, which is a thing they set when they chose their
//! region; it does not say where they are, and this crate never treats it as
//! though it did ([`crate::sun`] takes the place a person typed, and nothing
//! else).
//!
//! # Why the day is a Julian day number
//!
//! The arithmetic that finds a sunset counts days from noon on the first of
//! January 2000, and every published form of it is written that way. Turning a
//! clock into a year, a month and a day only to turn those back into a count of
//! days would be two conversions to get back where we started, and each of them
//! is a place to be wrong about a leap year.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::time_of_day::TimeOfDay;

/// How many seconds a day is, on a machine's clock.
const SECONDS_IN_A_DAY: i64 = 24 * 60 * 60;

/// How many seconds a minute is.
const SECONDS_IN_A_MINUTE: i64 = 60;

/// The Julian day number of the first of January 1970, which is where a
/// machine's clock starts counting.
const THE_EPOCH: i64 = 2_440_588;

/// One moment, as the machine in front of the person has it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Moment {
    /// The day this falls on, locally, as a Julian day number.
    julian_day: i64,
    /// The time of day on the person's own clock.
    time: TimeOfDay,
    /// How far that clock is east of universal time, in minutes.
    minutes_east: i32,
}

impl Moment {
    /// This moment, on a machine whose clock is this far east of universal
    /// time.
    ///
    /// Central European summer time is `120`, and Newfoundland's winter is
    /// `-210`.
    #[must_use]
    pub fn at(when: SystemTime, minutes_east_of_universal_time: i32) -> Self {
        let seconds = match when.duration_since(UNIX_EPOCH) {
            Ok(since) => i64::try_from(since.as_secs()).unwrap_or(i64::MAX),
            Err(before) => i64::try_from(before.duration().as_secs())
                .unwrap_or(i64::MAX)
                .saturating_neg(),
        };
        let here = seconds.saturating_add(
            i64::from(minutes_east_of_universal_time).saturating_mul(SECONDS_IN_A_MINUTE),
        );
        let through_the_day = here.rem_euclid(SECONDS_IN_A_DAY) / SECONDS_IN_A_MINUTE;
        Self {
            julian_day: here.div_euclid(SECONDS_IN_A_DAY).saturating_add(THE_EPOCH),
            // A remainder of a day in minutes is at most 1439, which is inside
            // the `u16` the day is counted in.
            time: TimeOfDay::after_midnight(through_the_day as u16),
            minutes_east: minutes_east_of_universal_time,
        }
    }

    /// The time of day on the person's own clock.
    #[must_use]
    pub const fn time(self) -> TimeOfDay {
        self.time
    }

    /// How far that clock is east of universal time, in minutes.
    #[must_use]
    pub const fn minutes_east_of_universal_time(self) -> i32 {
        self.minutes_east
    }

    /// The day this falls on, as a Julian day number.
    ///
    /// Only [`crate::sun`] asks, and it is the number that arithmetic is
    /// written in.
    pub(crate) const fn julian_day(self) -> i64 {
        self.julian_day
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    /// The moment a machine's clock reads zero is midnight at the start of the
    /// first of January 1970, whose Julian day number is the one every almanac
    /// prints.
    #[test]
    fn the_start_of_a_machines_clock_is_the_day_an_almanac_names() {
        let moment = Moment::at(UNIX_EPOCH, 0);
        assert_eq!(moment.julian_day(), 2_440_588);
        assert_eq!(moment.time().to_string(), "00:00");
        assert_eq!(moment.minutes_east_of_universal_time(), 0);
    }

    /// **The clock the person reads is the one the day is counted on.** An hour
    /// before midnight in universal time is already tomorrow in Helsinki, and
    /// a moment that said otherwise would warm somebody's screens a day late.
    #[test]
    fn the_day_is_the_one_the_person_is_living_in() {
        let when = UNIX_EPOCH + Duration::from_secs(half_past_eleven_on_the_twentieth());
        let here = Moment::at(when, 0);
        let helsinki = Moment::at(when, 180);
        assert_eq!(here.time().to_string(), "23:30");
        assert_eq!(helsinki.time().to_string(), "02:30");
        assert_eq!(
            helsinki.julian_day(),
            here.julian_day().saturating_add(1),
            "the next day where the clock is three hours ahead"
        );
    }

    /// And the same the other way: a machine west of universal time is still on
    /// yesterday's date shortly after midnight there.
    #[test]
    fn a_clock_west_of_universal_time_is_still_on_yesterday() {
        let when = UNIX_EPOCH + Duration::from_secs(half_past_eleven_on_the_twentieth());
        let here = Moment::at(when, 0);
        let newfoundland = Moment::at(when, -210);
        assert_eq!(newfoundland.time().to_string(), "20:00");
        assert_eq!(newfoundland.julian_day(), here.julian_day());
        assert_eq!(newfoundland.minutes_east_of_universal_time(), -210);
    }

    /// A clock set before 1970 does not panic and does not wrap into the
    /// future: it is a day before the epoch, which is what it is.
    #[test]
    fn a_clock_before_the_epoch_is_a_day_before_the_epoch() {
        let before = UNIX_EPOCH - Duration::from_secs(60 * 60);
        let moment = Moment::at(before, 0);
        assert_eq!(moment.julian_day(), 2_440_587);
        assert_eq!(moment.time().to_string(), "23:00");
    }

    /// The moment two of this file's tests are written against: 2026-06-20 at
    /// half past eleven in the evening, universal time.
    ///
    /// Written as arithmetic rather than as one number so that the date it
    /// stands for can be checked by reading it: 20,624 days from the first of
    /// January 1970 is the twentieth of June 2026.
    fn half_past_eleven_on_the_twentieth() -> u64 {
        let days: u64 = 20_624;
        days * 24 * 60 * 60 + 23 * 60 * 60 + 30 * 60
    }
}
