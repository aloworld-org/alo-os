//! Light and dark, and the schedule that moves between them.
//!
//! **Two schemes, as equals.** The design brief says to design light and dark as
//! equals rather than treating one as the real one and the other as a mode, and
//! the model says the same thing by having no default hidden in it: what a
//! machine starts in is stated once, in [`crate::shipped`], where it can be read.
//!
//! **A schedule is two times, not a duration.** *Dark after six, light at seven
//! in the morning* is what a person says, and it is what a settings panel shows
//! back to them. The other spelling — dark for thirteen hours — is the same
//! information written so that nobody can check it at a glance.
//!
//! **A schedule crosses midnight, because that is the ordinary case.** Dark from
//! 18:00 to 07:00 spans the night, so the comparison wraps; the same two fields
//! the other way round — dark from 07:00 to 18:00 — is a person who works
//! nights, and it works without a second kind of schedule.

use alo_strings::{Filling, Said, Strings, Word};
use serde::{Deserialize, Serialize};

use crate::time::TimeOfDay;
use crate::unreadable::NotRead;
use crate::words;

/// Why two times are not a schedule.
///
/// There is no `Display`: the only road to words is [`ScheduleError::said`], and
/// what a settings file that did not read writes is [`NotRead`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleError {
    /// The same time twice.
    TheSameMoment(TimeOfDay),
}

impl ScheduleError {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::TheSameMoment(_) => words::THE_SAME_MOMENT,
        }
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    ///
    /// The time goes in as the settings file writes it — `18:00`, on a
    /// twenty-four hour clock — because that is what a person editing the file
    /// is looking at. How a *person* is shown a time elsewhere belongs to their
    /// region rather than to their language, and is not one of this crate's
    /// strings.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        let filling = match self {
            Self::TheSameMoment(time) => Filling::of("time", time.to_string()),
        };
        strings.say(&self.word().key(), &filling)
    }
}

/// Light or dark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scheme {
    /// Dark on a light ground.
    Light,
    /// Light on a dark ground.
    Dark,
}

/// When the machine turns dark, and when it turns light again.
///
/// Reads back through [`Schedule::checked`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Written", into = "Written")]
pub struct Schedule {
    /// When it turns dark.
    dark_from: TimeOfDay,
    /// When it turns light again.
    light_from: TimeOfDay,
}

impl Schedule {
    /// Dark from one time, light again from the other.
    ///
    /// # Errors
    /// [`ScheduleError::TheSameMoment`] when the two are the same time, which
    /// would be a schedule that says nothing about any minute of the day.
    pub fn checked(dark_from: TimeOfDay, light_from: TimeOfDay) -> Result<Self, ScheduleError> {
        if dark_from == light_from {
            return Err(ScheduleError::TheSameMoment(dark_from));
        }
        Ok(Self {
            dark_from,
            light_from,
        })
    }

    /// A schedule the compiler can build, for the one this crate ships.
    ///
    /// Unchecked, and the only caller is [`crate::shipped`] — which is held to
    /// the same rules by a test that puts it back through [`Schedule::checked`].
    pub(crate) const fn shipped(dark_from: TimeOfDay, light_from: TimeOfDay) -> Self {
        Self {
            dark_from,
            light_from,
        }
    }

    /// When it turns dark.
    #[must_use]
    pub fn dark_from(self) -> TimeOfDay {
        self.dark_from
    }

    /// When it turns light again.
    #[must_use]
    pub fn light_from(self) -> TimeOfDay {
        self.light_from
    }

    /// Which scheme this time of day falls in.
    #[must_use]
    pub fn at(self, now: TimeOfDay) -> Scheme {
        let dark = if self.dark_from < self.light_from {
            now >= self.dark_from && now < self.light_from
        } else {
            // The ordinary case: the dark stretch runs through midnight, so it
            // is everything from the evening onwards *or* everything before the
            // morning.
            now >= self.dark_from || now < self.light_from
        };
        if dark { Scheme::Dark } else { Scheme::Light }
    }
}

/// A schedule as a settings file holds it.
#[derive(Serialize, Deserialize)]
struct Written {
    /// When it turns dark.
    dark_from: TimeOfDay,
    /// When it turns light again.
    light_from: TimeOfDay,
}

impl TryFrom<Written> for Schedule {
    type Error = NotRead;

    fn try_from(written: Written) -> Result<Self, Self::Error> {
        Self::checked(written.dark_from, written.light_from)
            .map_err(|refused| NotRead::about(refused.word()))
    }
}

impl From<Schedule> for Written {
    fn from(schedule: Schedule) -> Self {
        Self {
            dark_from: schedule.dark_from,
            light_from: schedule.light_from,
        }
    }
}

/// What decides the scheme: a person's standing choice, or the clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Following {
    /// This scheme, whatever the time is.
    Always(Scheme),
    /// The clock, by this schedule.
    TheClock(Schedule),
}

impl Following {
    /// Which scheme is showing at this time of day.
    ///
    /// The time is passed in rather than read, so what the shell will do at half
    /// past six is answerable at any hour of the day — including in a settings
    /// panel previewing the schedule somebody is halfway through setting.
    #[must_use]
    pub fn at(self, now: TimeOfDay) -> Scheme {
        match self {
            Self::Always(scheme) => scheme,
            Self::TheClock(schedule) => schedule.at(now),
        }
    }
}

impl From<Scheme> for Following {
    fn from(scheme: Scheme) -> Self {
        Self::Always(scheme)
    }
}

impl From<Schedule> for Following {
    fn from(schedule: Schedule) -> Self {
        Self::TheClock(schedule)
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

    /// A time, spelled shortly enough to read a table of them.
    fn at(hour: u8, minute: u8) -> TimeOfDay {
        TimeOfDay::checked(hour, minute).unwrap()
    }

    /// A standing choice ignores the clock entirely, at every hour of the day.
    #[test]
    fn a_standing_choice_ignores_the_clock() {
        for hour in 0..24 {
            assert_eq!(Following::from(Scheme::Dark).at(at(hour, 0)), Scheme::Dark);
            assert_eq!(
                Following::from(Scheme::Light).at(at(hour, 30)),
                Scheme::Light
            );
        }
    }

    /// **Dark after six.** The ordinary schedule runs through midnight, and
    /// every hour of the day falls on the side a person would say it does.
    #[test]
    fn dark_after_six_covers_the_night() {
        let evening = Following::from(Schedule::checked(at(18, 0), at(7, 0)).unwrap());
        let expected = [
            (at(17, 59), Scheme::Light),
            (at(18, 0), Scheme::Dark),
            (at(23, 59), Scheme::Dark),
            (at(0, 0), Scheme::Dark),
            (at(3, 0), Scheme::Dark),
            (at(6, 59), Scheme::Dark),
            (at(7, 0), Scheme::Light),
            (at(12, 0), Scheme::Light),
        ];
        for (now, scheme) in expected {
            assert_eq!(evening.at(now), scheme, "at {now}");
        }
    }

    /// The same two fields the other way round is somebody who works nights,
    /// and it needs no second kind of schedule.
    #[test]
    fn a_schedule_that_does_not_cross_midnight_works_too() {
        let nights = Schedule::checked(at(7, 0), at(18, 0)).unwrap();
        assert_eq!(nights.at(at(6, 59)), Scheme::Light);
        assert_eq!(nights.at(at(7, 0)), Scheme::Dark);
        assert_eq!(nights.at(at(17, 59)), Scheme::Dark);
        assert_eq!(nights.at(at(18, 0)), Scheme::Light);
    }

    /// **A schedule that says nothing is refused**, where a person sets it and
    /// again where a file is read: turning dark and light at the same minute
    /// would leave the shell to guess which one it meant.
    #[test]
    fn a_schedule_that_never_changes_is_refused() {
        assert_eq!(
            Schedule::checked(at(18, 0), at(18, 0)),
            Err(ScheduleError::TheSameMoment(at(18, 0)))
        );
        let said = Schedule::checked(at(18, 0), at(18, 0))
            .unwrap_err()
            .said(&in_english());
        assert_eq!(
            said.text(),
            "give two different times — a day that turns dark and light at 18:00 is a day that \
             never changes"
        );
        assert!(said.unfilled().is_empty());

        let refused = serde_json::from_str::<Schedule>(
            r#"{"dark_from":{"hour":18,"minute":0},"light_from":{"hour":18,"minute":0}}"#,
        )
        .unwrap_err();
        assert!(
            refused
                .to_string()
                .contains("appearance.schedule.the-same-moment"),
            "a file that did not read names the string rather than saying it: {refused}"
        );
    }

    /// The same refusal, read on a machine that has been translated — with the
    /// time still written the way the settings file writes it.
    #[test]
    fn the_refusal_is_read_in_the_readers_language() {
        let strings = translated(&[(
            words::THE_SAME_MOMENT,
            "geben Sie zwei verschiedene Zeiten an — ein Tag, der um {time} dunkel und hell wird, \
             ändert sich nie",
        )]);
        let said = Schedule::checked(at(18, 0), at(18, 0))
            .unwrap_err()
            .said(&strings);
        assert_eq!(
            said.text(),
            "geben Sie zwei verschiedene Zeiten an — ein Tag, der um 18:00 dunkel und hell wird, \
             ändert sich nie"
        );
        assert!(said.is_translated());
    }

    /// Both kinds survive a settings file unchanged.
    #[test]
    fn what_decides_the_scheme_survives_being_written_down() {
        let each = [
            Following::from(Scheme::Dark),
            Following::from(Schedule::checked(at(18, 0), at(7, 0)).unwrap()),
        ];
        for following in each {
            let written = serde_json::to_string(&following).unwrap();
            assert_eq!(
                serde_json::from_str::<Following>(&written).unwrap(),
                following
            );
        }
    }

    /// The two times come back out for a settings panel to show.
    #[test]
    fn a_schedule_says_both_of_its_times() {
        let schedule = Schedule::checked(at(18, 30), at(7, 15)).unwrap();
        assert_eq!(schedule.dark_from().to_string(), "18:30");
        assert_eq!(schedule.light_from().to_string(), "07:15");
    }
}
