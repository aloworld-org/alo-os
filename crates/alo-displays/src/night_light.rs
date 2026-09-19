//! Night light: when the screens are warmed, how warm, and what is said about
//! it.
//!
//! Two things, and they are separate because a person changes them separately:
//! **when** ([`Nightly`] — never, a schedule, or the sun) and **how warm**
//! ([`Warmth`]). Somebody who finds the evening too orange turns the warmth
//! down without losing the schedule they set, and somebody who moves house
//! changes where they are without losing the warmth they liked.
//!
//! # It is off until somebody asks for it
//!
//! [`NightLight::as_shipped`] is *never*, at a warmth that is only a starting
//! point for the moment it is switched on. A machine that arrived with its
//! screens already tinted would be a machine whose owner's first impression of
//! the colour they paid for was wrong, and nobody would know why.
//!
//! # And it is applied per screen
//!
//! What this file answers is one [`Tonight`] for the machine — the clock and
//! the sun are the same in every direction a person can turn their head. It
//! reaches a screen through [`crate::Wearing`], beside that screen's background
//! and the edge its dock sits on, so the shell warms each output as it draws it
//! rather than laying one tinted sheet over a desk whose screens are at
//! different sizes and in different places.
//!
//! There is no *except this one*. A person doing colour work has a calibrated
//! screen and turns night light off; a setting that warmed one screen and not
//! the one beside it would be two screens that disagree about what white is,
//! which is the thing that person is trying to avoid.

use serde::{Deserialize, Serialize};

use crate::between::Between;
use crate::moment::Moment;
use crate::nightly::{Nightly, Now};
use crate::notes::Note;
use crate::sun::Whereabouts;
use crate::unreadable::NotRead;
use crate::warmth::Warmth;

/// The warmth a machine offers the first time somebody switches night light
/// on, in kelvin.
///
/// Warm enough to be worth switching on and far enough from
/// [`Warmth::WARMEST`] that the first thing somebody sees is not the strongest
/// thing the machine can do.
const A_STARTING_POINT: u16 = 3400;

/// Night light, as a person has it set.
///
/// Reads back through [`Nightly::asked`], so a file asking for both a schedule
/// and the sun is refused where it is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Written", into = "Written")]
pub struct NightLight {
    /// When the screens are warmed.
    when: Nightly,
    /// How warm they are drawn while they are.
    warmth: Warmth,
}

impl NightLight {
    /// Night light as a machine arrives: off, at a warmth nothing is using yet.
    #[must_use]
    pub const fn as_shipped() -> Self {
        Self {
            when: Nightly::Never,
            warmth: Warmth::shipped(A_STARTING_POINT),
        }
    }

    /// Night light on at this warmth, at these times.
    #[must_use]
    pub const fn of(when: Nightly, warmth: Warmth) -> Self {
        Self { when, warmth }
    }

    /// When the screens are warmed.
    #[must_use]
    pub const fn when(self) -> Nightly {
        self.when
    }

    /// How warm they are drawn while they are.
    #[must_use]
    pub const fn warmth(self) -> Warmth {
        self.warmth
    }

    /// The same, at another warmth.
    #[must_use]
    pub const fn at_this_warmth(self, warmth: Warmth) -> Self {
        Self { warmth, ..self }
    }

    /// The same, at other times.
    #[must_use]
    pub const fn at_these_times(self, when: Nightly) -> Self {
        Self { when, ..self }
    }

    /// What night light is doing at this moment.
    #[must_use]
    pub fn at(self, moment: Moment) -> Tonight {
        let now = self.when.at(moment);
        Tonight {
            warmth: if now.is_on() {
                self.warmth
            } else {
                Warmth::neutral()
            },
            note: match now {
                Now::On | Now::Off => None,
                Now::OffWhileTheSunDoesNotSet => Some(Note::TheSunDoesNotSet),
                Now::OnWhileTheSunDoesNotRise => Some(Note::TheSunDoesNotRise),
            },
        }
    }
}

impl Default for NightLight {
    /// What a machine arrives with, which is off.
    fn default() -> Self {
        Self::as_shipped()
    }
}

/// What night light is doing at one moment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tonight {
    /// The warmth every screen is drawn at — [`Warmth::neutral`] when night
    /// light is not on, so that there is one answer rather than an answer and
    /// an absence.
    warmth: Warmth,
    /// What the person is told, where there is anything to tell.
    note: Option<Note>,
}

impl Tonight {
    /// The warmth every screen is drawn at.
    #[must_use]
    pub const fn warmth(&self) -> Warmth {
        self.warmth
    }

    /// Whether the screens are warmed at all.
    #[must_use]
    pub const fn is_on(&self) -> bool {
        !self.warmth.changes_nothing()
    }

    /// What the person is told about it, where there is anything to tell.
    #[must_use]
    pub const fn note(&self) -> Option<&Note> {
        self.note.as_ref()
    }
}

/// Night light as a settings file holds it, flat: a warmth, and at most one of
/// two times and a place.
///
/// Flat rather than nested because it is read by people in an editor. *How
/// warm* and *when* are one question to somebody looking at their own file, and
/// a `when` with a `between` inside it would be a level of nesting that exists
/// only because the code has two types.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct Written {
    /// How warm the screens are drawn while night light is on.
    warmth: Warmth,
    /// The two times a person set, where they set two.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    between: Option<Between>,
    /// Where they say they are, where they said.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sunset_at: Option<Whereabouts>,
}

impl TryFrom<Written> for NightLight {
    type Error = NotRead;

    fn try_from(written: Written) -> Result<Self, Self::Error> {
        Ok(Self {
            when: Nightly::asked(written.between, written.sunset_at)
                .map_err(|refused| NotRead::about(refused.word()))?,
            warmth: written.warmth,
        })
    }
}

impl From<NightLight> for Written {
    fn from(night_light: NightLight) -> Self {
        let (between, sunset_at) = match night_light.when {
            Nightly::Never => (None, None),
            Nightly::Between(between) => (Some(between), None),
            Nightly::FromSunsetAt(where_they_are) => (None, Some(where_they_are)),
        };
        Self {
            warmth: night_light.warmth,
            between,
            sunset_at,
        }
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

    /// Ten in the evening until seven in the morning.
    fn the_evening() -> Nightly {
        Nightly::Between(
            Between::these_two_times(
                TimeOfDay::written("22:00").unwrap(),
                TimeOfDay::written("07:00").unwrap(),
            )
            .unwrap(),
        )
    }

    /// **A machine arrives with its screens as they are**, and the warmth it
    /// carries is a starting point rather than something in force.
    #[test]
    fn a_machine_arrives_with_night_light_off() {
        let shipped = NightLight::as_shipped();
        assert_eq!(shipped, NightLight::default());
        assert_eq!(shipped.when(), Nightly::Never);
        assert!(!shipped.warmth().changes_nothing());
        for hour in ["00:00", "12:00", "23:30"] {
            let tonight = shipped.at(at(MIDSUMMER, hour, 0));
            assert!(!tonight.is_on(), "{hour}");
            assert!(tonight.warmth().changes_nothing(), "{hour}");
            assert_eq!(tonight.note(), None, "{hour}");
        }
    }

    /// **The shipped warmth is a warmth**, held to the same range as one a
    /// person types — which is what makes the unchecked door this file uses
    /// safe to have.
    #[test]
    fn the_shipped_warmth_is_inside_the_range_everybody_else_is_held_to() {
        assert_eq!(
            Warmth::kelvin(NightLight::as_shipped().warmth().as_kelvin()),
            Ok(NightLight::as_shipped().warmth())
        );
    }

    /// **When it is on, it is on at the warmth the person chose**, and when it
    /// is not, every colour is exactly where it was.
    #[test]
    fn the_warmth_in_force_is_the_one_that_was_chosen_and_otherwise_nothing() {
        let chosen = Warmth::kelvin(2700).unwrap();
        let night = NightLight::of(the_evening(), chosen);
        let warmed = night.at(at(MIDSUMMER, "23:00", 0));
        assert!(warmed.is_on());
        assert_eq!(warmed.warmth(), chosen);
        assert_eq!(warmed.note(), None);

        let daytime = night.at(at(MIDSUMMER, "12:00", 0));
        assert!(!daytime.is_on());
        assert_eq!(daytime.warmth(), Warmth::neutral());
    }

    /// **When and how warm are changed separately**, because a person changes
    /// them separately.
    #[test]
    fn when_and_how_warm_are_two_settings() {
        let night = NightLight::of(the_evening(), Warmth::kelvin(2700).unwrap());
        let cooler = night.at_this_warmth(Warmth::kelvin(4500).unwrap());
        assert_eq!(cooler.when(), night.when());
        assert_eq!(cooler.warmth().as_kelvin(), 4500);

        let by_the_sun = night.at_these_times(Nightly::FromSunsetAt(
            Whereabouts::typed(51.5, -0.1).unwrap(),
        ));
        assert_eq!(by_the_sun.warmth(), night.warmth());
        assert!(matches!(by_the_sun.when(), Nightly::FromSunsetAt(_)));
    }

    /// **A polar day is said**, and the sentence a person meets is one this
    /// crate declares rather than a silence.
    #[test]
    fn a_sun_that_does_not_set_is_said_and_the_screens_are_left_alone() {
        let tromso = NightLight::of(
            Nightly::FromSunsetAt(Whereabouts::typed(69.6492, 18.9553).unwrap()),
            Warmth::kelvin(2700).unwrap(),
        );
        let tonight = tromso.at(at(MIDSUMMER, "23:00", 120));
        assert!(!tonight.is_on());
        assert_eq!(tonight.note(), Some(&Note::TheSunDoesNotSet));
        let said = tonight.note().unwrap().said(&in_english());
        assert!(said.text().contains("does not set"), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
        assert!(!said.is_a_bug(), "{said}");
    }

    /// **Every way of setting night light survives a settings file**, and the
    /// file is flat: a warmth, and at most one of two times and a place.
    #[test]
    fn every_way_of_setting_it_survives_a_file() {
        for night in [
            NightLight::as_shipped(),
            NightLight::of(the_evening(), Warmth::kelvin(2700).unwrap()),
            NightLight::of(
                Nightly::FromSunsetAt(Whereabouts::typed(51.5074, -0.1278).unwrap()),
                Warmth::kelvin(4500).unwrap(),
            ),
        ] {
            let written = serde_json::to_string(&night).unwrap();
            assert_eq!(serde_json::from_str::<NightLight>(&written).unwrap(), night);
        }
        assert_eq!(
            serde_json::to_string(&NightLight::as_shipped()).unwrap(),
            r#"{"warmth":3400}"#,
            "night light that is off is a warmth and nothing else"
        );
        assert_eq!(
            serde_json::to_string(&NightLight::of(
                the_evening(),
                Warmth::kelvin(2700).unwrap()
            ))
            .unwrap(),
            r#"{"warmth":2700,"between":{"from":"22:00","to":"07:00"}}"#
        );
    }

    /// **A file asking for both a schedule and the sun is refused where it is
    /// read**, and so is a key this shape does not have.
    #[test]
    fn a_hand_edited_file_that_asks_for_both_is_refused() {
        let refused = serde_json::from_str::<NightLight>(
            r#"{"warmth":2700,"between":{"from":"22:00","to":"07:00"},"sunset-at":{"latitude":51.5,"longitude":0.0}}"#,
        )
        .unwrap_err();
        assert!(
            refused
                .to_string()
                .contains("displays.a-schedule-or-the-sun"),
            "{refused}"
        );
        assert!(
            serde_json::from_str::<NightLight>(r#"{"warmth":2700,"screen":"eDP-1"}"#).is_err(),
            "a key this shape does not have is refused where it is read"
        );
        assert!(
            serde_json::from_str::<NightLight>(r#"{"between":{"from":"22:00","to":"07:00"}}"#)
                .is_err(),
            "and night light with no warmth at all is not night light"
        );
    }
}
