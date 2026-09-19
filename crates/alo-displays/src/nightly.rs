//! When night light is on: never, between two times a person set, or from
//! sunset to sunrise where they say they are.
//!
//! **Three arms and no fourth**, and the missing fourth is the point. There is
//! no *follow the sun* that has no place attached to it, so *sunset for your
//! location* cannot exist on a machine that was never told a location — the
//! setting offers a schedule instead, and a person who would rather not say
//! where they live loses nothing but the two minutes a year that the sun moves
//! away from their schedule.
//!
//! **And nothing here asks anybody where they are.** [`crate::sun`] is
//! arithmetic on two numbers somebody typed; this file decides only which of
//! the three arms is in force at a moment.

use alo_strings::{Filling, Said, Strings, Word};

use crate::between::Between;
use crate::moment::Moment;
use crate::sun::{Sun, Whereabouts};
use crate::words;

/// Why this is not a way of saying when night light is on.
///
/// There is no `Display`: the only road to words is [`NotNightly::said`], and
/// what a settings file that did not read writes is [`crate::NotRead`].
///
/// It carries nothing, because there is one way to be wrong here and it is
/// asking for both a schedule and the sun at once; it is non-exhaustive so that
/// only this crate can make one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct NotNightly;

impl NotNightly {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(self) -> Word {
        words::A_SCHEDULE_OR_THE_SUN
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

/// When night light is on.
///
/// It is not written down by itself: [`crate::NightLight`] holds it beside a
/// warmth, and that is the shape `displays.toml` carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nightly {
    /// Never: the screens are drawn as they are all day and all night.
    Never,
    /// Between these two times, every day, whatever the sun is doing.
    Between(Between),
    /// From sunset to sunrise where the person says they are, worked out on
    /// this machine ([`crate::sun`]).
    FromSunsetAt(Whereabouts),
}

impl Nightly {
    /// What a settings panel and a settings file both offer: at most one of a
    /// schedule and a place.
    ///
    /// Neither is night light off, which is what a machine ships with.
    ///
    /// # Errors
    /// [`NotNightly`] when both are given, because there is no reading of that
    /// which is not a guess at which one the person meant.
    pub const fn asked(
        between: Option<Between>,
        sunset_at: Option<Whereabouts>,
    ) -> Result<Self, NotNightly> {
        match (between, sunset_at) {
            (None, None) => Ok(Self::Never),
            (Some(between), None) => Ok(Self::Between(between)),
            (None, Some(where_they_are)) => Ok(Self::FromSunsetAt(where_they_are)),
            (Some(_), Some(_)) => Err(NotNightly),
        }
    }

    /// Whether night light is on at this moment, and what needs saying about
    /// it.
    #[must_use]
    pub fn at(self, moment: Moment) -> Now {
        match self {
            Self::Never => Now::Off,
            Self::Between(between) => {
                if between.holds(moment.time()) {
                    Now::On
                } else {
                    Now::Off
                }
            }
            Self::FromSunsetAt(where_they_are) => match where_they_are.sun_on(moment) {
                Sun::SetsAndRises(night) => {
                    if night.holds(moment.time()) {
                        Now::On
                    } else {
                        Now::Off
                    }
                }
                // Both arms are somebody inside a polar circle, and both are
                // told: a screen that stayed cold for six weeks of daylight and
                // a screen that warmed for six weeks of darkness are each
                // right, and each is surprising enough to deserve a sentence.
                Sun::NeverSets => Now::OffWhileTheSunDoesNotSet,
                Sun::NeverRises => Now::OnWhileTheSunDoesNotRise,
            },
        }
    }
}

/// Whether night light is on at one moment, and why, where the why is worth
/// saying.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Now {
    /// On, for the ordinary reason: it is the hour for it.
    On,
    /// Off, for the ordinary reason: it is not.
    Off,
    /// Off because the sun has not set today and will not.
    OffWhileTheSunDoesNotSet,
    /// On because the sun has not risen today and will not.
    OnWhileTheSunDoesNotRise,
}

impl Now {
    /// Whether the screens are warmed.
    #[must_use]
    pub const fn is_on(self) -> bool {
        matches!(self, Self::On | Self::OnWhileTheSunDoesNotRise)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use super::*;
    use crate::testing::in_english;
    use crate::time_of_day::TimeOfDay;

    /// The twenty-first of June 2026, as a count of days from the first of
    /// January 1970.
    const MIDSUMMER: u64 = 20_625;

    /// The twenty-first of December 2026.
    const MIDWINTER: u64 = 20_808;

    /// This time of day on that day, on a clock this far east of universal
    /// time.
    fn at(days: u64, written: &str, minutes_east: i32) -> Moment {
        let time = TimeOfDay::written(written).unwrap();
        let seconds = i64::try_from(days).unwrap() * 24 * 60 * 60
            + i64::from(time.minutes_after_midnight()) * 60
            - i64::from(minutes_east) * 60;
        Moment::at(
            UNIX_EPOCH + Duration::from_secs(u64::try_from(seconds).unwrap()),
            minutes_east,
        )
    }

    /// Somewhere in the middle of the world, for the schedule tests.
    fn london() -> Whereabouts {
        Whereabouts::typed(51.5074, -0.1278).unwrap()
    }

    /// **A machine nobody has asked warms nothing**, at any hour of any day.
    #[test]
    fn never_is_never() {
        for hour in ["00:00", "03:00", "12:00", "22:30", "23:59"] {
            assert_eq!(Nightly::Never.at(at(MIDSUMMER, hour, 0)), Now::Off);
        }
    }

    /// **A schedule is the person's and the sun is not consulted**, which is
    /// the whole of why a schedule exists: somebody who did not say where they
    /// live still gets warm screens at ten.
    #[test]
    fn a_schedule_holds_the_hours_it_was_given_and_no_others() {
        let night = Nightly::Between(
            Between::these_two_times(
                TimeOfDay::written("22:00").unwrap(),
                TimeOfDay::written("07:00").unwrap(),
            )
            .unwrap(),
        );
        assert_eq!(night.at(at(MIDSUMMER, "22:30", 60)), Now::On);
        assert_eq!(night.at(at(MIDSUMMER, "03:00", 60)), Now::On);
        assert_eq!(
            night.at(at(MIDSUMMER, "21:00", 60)),
            Now::Off,
            "the sun was still up and so was the schedule's decision"
        );
        assert_eq!(night.at(at(MIDWINTER, "21:00", 0)), Now::Off);
    }

    /// **And the sun is the sun.** In London in midsummer the screens warm
    /// after half past nine in the evening and not at nine, which is a schedule
    /// nobody could have written down and kept right all year.
    #[test]
    fn the_sun_turns_the_screens_warm_when_it_actually_sets() {
        let night = Nightly::FromSunsetAt(london());
        assert_eq!(night.at(at(MIDSUMMER, "21:00", 60)), Now::Off);
        assert_eq!(night.at(at(MIDSUMMER, "23:00", 60)), Now::On);
        assert_eq!(night.at(at(MIDSUMMER, "03:00", 60)), Now::On);
        assert_eq!(night.at(at(MIDSUMMER, "06:00", 60)), Now::Off);
        assert_eq!(
            night.at(at(MIDWINTER, "17:00", 0)),
            Now::On,
            "and in December the same hour is long after dark"
        );
    }

    /// **Where the sun does not set, the screens are not warmed and the person
    /// is told which of the two it is.** Both arms are somebody inside a polar
    /// circle, and a machine that silently did nothing for six weeks would look
    /// broken.
    #[test]
    fn a_polar_summer_and_a_polar_winter_are_each_said() {
        let tromso = Nightly::FromSunsetAt(Whereabouts::typed(69.6492, 18.9553).unwrap());
        let summer = tromso.at(at(MIDSUMMER, "23:00", 120));
        assert_eq!(summer, Now::OffWhileTheSunDoesNotSet);
        assert!(!summer.is_on());
        let winter = tromso.at(at(MIDWINTER, "12:00", 60));
        assert_eq!(winter, Now::OnWhileTheSunDoesNotRise);
        assert!(winter.is_on());
    }

    /// **A schedule and the sun at once is refused**, in words, wherever it is
    /// asked for — because there is no reading of it that is not a guess.
    #[test]
    fn a_schedule_and_the_sun_at_once_is_refused() {
        let between = Between::these_two_times(
            TimeOfDay::written("22:00").unwrap(),
            TimeOfDay::written("07:00").unwrap(),
        )
        .unwrap();
        assert_eq!(Nightly::asked(None, None), Ok(Nightly::Never));
        assert_eq!(
            Nightly::asked(Some(between), None),
            Ok(Nightly::Between(between))
        );
        assert_eq!(
            Nightly::asked(None, Some(london())),
            Ok(Nightly::FromSunsetAt(london()))
        );

        let refused = Nightly::asked(Some(between), Some(london())).unwrap_err();
        let said = refused.said(&in_english());
        assert!(said.text().contains("a schedule or the sun"), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
        assert!(!said.is_a_bug(), "{said}");
    }
}
