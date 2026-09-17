//! **How long is left, and when this machine will not say.**
//!
//! A time remaining is a guess about the future made out of two numbers, and
//! the wrong thing to do with a shaky guess is to show it to four significant
//! figures. Everyone has seen *2 hours 14 minutes* become *31 minutes* by
//! opening a browser, and what that teaches a person is to stop reading the
//! number — which costs them the one moment it would have been useful.
//!
//! # The rule, in one place so a test can name it
//!
//! A time is shown when **all** of these are true, and otherwise the machine
//! says how full the battery is and nothing else:
//!
//! 1. the machine is running **on** the battery — a time to empty while it is
//!    charging is a number about a situation that is not happening;
//! 2. there are at least [`ENOUGH_READINGS`] readings;
//! 3. they span at least [`LONG_ENOUGH`], so that a burst of activity is not
//!    the whole sample;
//! 4. the largest and smallest draw across them differ by no more than
//!    [`STEADY_ENOUGH`] hundredths of the largest.
//!
//! Rule 4 is the one that does the work. A machine whose draw doubled while
//! this was being measured has no steady draw to extrapolate from, and the
//! honest answer is that this machine does not know.

use std::time::Duration;

use crate::reading::Reading;

/// How many readings before this machine will say anything.
pub const ENOUGH_READINGS: usize = 3;

/// How long they must span.
pub const LONG_ENOUGH: Duration = Duration::from_secs(60);

/// How much the draw may vary across them, in hundredths of the largest.
pub const STEADY_ENOUGH: u64 = 25;

/// **How long is left, or why this machine will not say.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HowLong {
    /// About this long, at the draw of the last few minutes.
    About(Duration),
    /// The machine is not running on the battery.
    NotOnTheBattery,
    /// Not enough has been seen yet.
    NotSeenEnoughYet,
    /// What the machine is doing is not steady enough to say anything about.
    NotSteadyEnough,
    /// This machine does not report what is going through the battery at all.
    ThisMachineDoesNotSay,
}

impl HowLong {
    /// The time, where there is one to show.
    #[must_use]
    pub const fn shown(self) -> Option<Duration> {
        match self {
            Self::About(how_long) => Some(how_long),
            _ => None,
        }
    }
}

/// **How long is left, from the last few readings** — oldest first.
///
/// The rule is at the top of this file and the test below names each part of it.
#[must_use]
pub fn how_long_is_left(readings: &[Reading]) -> HowLong {
    let Some(last) = readings.last() else {
        return HowLong::NotSeenEnoughYet;
    };
    if !last.charging().is_on_the_battery() {
        return HowLong::NotOnTheBattery;
    }
    if readings.len() < ENOUGH_READINGS {
        return HowLong::NotSeenEnoughYet;
    }
    let Some(first) = readings.first() else {
        return HowLong::NotSeenEnoughYet;
    };
    if last.since(first) < LONG_ENOUGH {
        return HowLong::NotSeenEnoughYet;
    }

    let draws: Vec<u64> = readings.iter().filter_map(Reading::through_it).collect();
    if draws.len() != readings.len() {
        return HowLong::ThisMachineDoesNotSay;
    }
    let (Some(most), Some(least)) = (draws.iter().max().copied(), draws.iter().min().copied())
    else {
        return HowLong::ThisMachineDoesNotSay;
    };
    if most == 0 {
        return HowLong::NotSteadyEnough;
    }
    if (most - least).saturating_mul(100) / most > STEADY_ENOUGH {
        return HowLong::NotSteadyEnough;
    }

    // What is left, at the average draw. Charge is in hundredths of a battery
    // whose whole size this machine may not report, so the arithmetic is done
    // in the one unit every machine does report: the share of the battery, and
    // how fast that share is going.
    let gone = u64::from(first.charge().hundredths())
        .saturating_sub(u64::from(last.charge().hundredths()));
    if gone == 0 {
        return HowLong::NotSteadyEnough;
    }
    let over = last.since(first).as_secs();
    let left = u64::from(last.charge().hundredths());
    HowLong::About(Duration::from_secs(left.saturating_mul(over) / gone))
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::reading::{Charge, Charging};
    use std::time::SystemTime;

    fn a_reading(per_cent: u8, seconds: u64, draw: Option<i64>, charging: Charging) -> Reading {
        Reading::taken(
            Charge::reported(per_cent).expect("a charge"),
            charging,
            draw,
            SystemTime::UNIX_EPOCH + Duration::from_secs(seconds),
        )
    }

    fn steady_readings() -> Vec<Reading> {
        vec![
            a_reading(60, 0, Some(-1_000), Charging::Discharging),
            a_reading(55, 300, Some(-1_050), Charging::Discharging),
            a_reading(50, 600, Some(-1_000), Charging::Discharging),
        ]
    }

    /// **Rule 1: a time to empty is not shown while the machine is charging**,
    /// because it is a number about something that is not happening.
    #[test]
    fn nothing_is_said_about_a_battery_that_is_filling() {
        let mut readings = steady_readings();
        readings.push(a_reading(51, 900, Some(1_000), Charging::Charging));
        assert_eq!(how_long_is_left(&readings), HowLong::NotOnTheBattery);
        assert!(how_long_is_left(&readings).shown().is_none());
    }

    /// **Rules 2 and 3: too few readings, or too short a span, and this machine
    /// says nothing.**
    #[test]
    fn nothing_is_said_before_enough_has_been_seen() {
        assert_eq!(how_long_is_left(&[]), HowLong::NotSeenEnoughYet);

        let readings = steady_readings();
        let two = readings.get(..2).expect("three readings");
        assert_eq!(how_long_is_left(two), HowLong::NotSeenEnoughYet);

        let quick = vec![
            a_reading(60, 0, Some(-1_000), Charging::Discharging),
            a_reading(59, 10, Some(-1_000), Charging::Discharging),
            a_reading(58, 20, Some(-1_000), Charging::Discharging),
        ];
        assert_eq!(how_long_is_left(&quick), HowLong::NotSeenEnoughYet);
    }

    /// **Rule 4: a draw that moved by more than a quarter is not steady**, and
    /// the machine says how full the battery is instead of inventing an hour.
    #[test]
    fn nothing_is_said_when_what_the_machine_is_doing_moved() {
        let swinging = vec![
            a_reading(60, 0, Some(-1_000), Charging::Discharging),
            a_reading(55, 300, Some(-3_000), Charging::Discharging),
            a_reading(50, 600, Some(-1_100), Charging::Discharging),
        ];
        assert_eq!(how_long_is_left(&swinging), HowLong::NotSteadyEnough);
    }

    /// **A machine that does not report its draw is not guessed about.**
    #[test]
    fn a_machine_that_says_nothing_about_its_draw_is_not_extrapolated_from() {
        let quiet = vec![
            a_reading(60, 0, None, Charging::Discharging),
            a_reading(55, 300, None, Charging::Discharging),
            a_reading(50, 600, None, Charging::Discharging),
        ];
        assert_eq!(how_long_is_left(&quiet), HowLong::ThisMachineDoesNotSay);
    }

    /// **And when all four hold, the time is the one the arithmetic gives.**
    ///
    /// Half a battery went in ten minutes here — ten per cent over six hundred
    /// seconds — so the fifty per cent left is another fifty minutes.
    #[test]
    fn a_steady_machine_is_told_how_long_is_left() {
        assert_eq!(
            how_long_is_left(&steady_readings()),
            HowLong::About(Duration::from_secs(3_000))
        );
        assert_eq!(
            how_long_is_left(&steady_readings()).shown(),
            Some(Duration::from_secs(3_000))
        );
    }

    /// **A battery that has not moved at all is not steady enough either**: no
    /// charge has gone, so there is nothing to divide by and no honest guess to
    /// make.
    #[test]
    fn a_battery_that_has_not_moved_gives_no_time() {
        let still = vec![
            a_reading(60, 0, Some(-1_000), Charging::Discharging),
            a_reading(60, 300, Some(-1_000), Charging::Discharging),
            a_reading(60, 600, Some(-1_000), Charging::Discharging),
        ];
        assert_eq!(how_long_is_left(&still), HowLong::NotSteadyEnough);
    }
}
