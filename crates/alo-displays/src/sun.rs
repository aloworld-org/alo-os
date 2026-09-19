//! When the sun sets and rises where the person says they are — worked out on
//! this machine, from arithmetic, and from nothing else.
//!
//! **There is no location service here and there will not be one.** *Sunset for
//! your location* on most systems means an address sent to somebody's server,
//! or a list of the wireless networks a machine can hear sent to somebody
//! else's, in exchange for a latitude. On a machine sold on sovereignty that
//! trade is not available: what this crate knows about where somebody is, is
//! **what they typed into Settings**, and what it does with it is
//! multiplication. `tests/the_sun_is_worked_out_on_this_machine.rs` reads this
//! crate's own source and its manifest and refuses every road to a network.
//!
//! And absent a place the person typed, there is no guess: the setting offers a
//! schedule instead ([`crate::Nightly`] has no fourth arm, so *sunset* without a
//! place is not a state this crate can be in).
//!
//! # A timezone is not a place, and is not used as one
//!
//! Knowing that somebody keeps central European time says they are somewhere in
//! a stripe fifteen degrees wide and says nothing at all about how far north
//! they are — and how far north is the whole of when the sun sets. So the
//! offset the machine's clock keeps ([`crate::Moment`]) is used for exactly one
//! thing: putting a sunset worked out for the whole earth onto the clock the
//! person is looking at.
//!
//! # The arithmetic
//!
//! The published sunrise equation, which is a series approximation of where the
//! earth is in its year and how far it is leaning. It is accurate to about a
//! minute in the middle latitudes, which is a good deal finer than the question
//! — nobody notices their screens warming ninety seconds late — and it needs no
//! table, no almanac file and no network.
//!
//! The steps, in the order they run:
//!
//! 1. how many days it is since noon on the first of January 2000;
//! 2. where in its orbit the earth is on that day, and how far that is from an
//!    even circle (the three terms named at the top of `sun.rs`);
//! 3. when the sun is highest, which is not noon on anybody's clock;
//! 4. how far the earth is leaning towards the sun that day;
//! 5. how long before and after the highest point the sun is at the horizon,
//!    which is the one step that uses the latitude — and the one step that can
//!    have no answer, because near the poles the sun does not cross the horizon
//!    at all.

use serde::{Deserialize, Serialize};

use alo_strings::{Filling, Said, Strings, Word};

use crate::between::Between;
use crate::moment::Moment;
use crate::time_of_day::{MINUTES_IN_A_DAY, TimeOfDay};
use crate::unreadable::NotRead;
use crate::words;

/// Noon on the first of January 2000, as a Julian day number: where every
/// published form of this arithmetic counts days from.
const J2000: f64 = 2_451_545.0;

/// The fraction of a day the world's clocks have been held away from the sun
/// since then, by the seconds added to keep them in step with it.
const CLOCKS_HELD_BACK: f64 = 0.0008;

/// Where the earth would be in its orbit at J2000 if the orbit were an even
/// circle, in degrees.
const THE_ORBIT_AT_J2000: f64 = 357.5291;

/// How far round that even circle the earth goes in a day, in degrees.
const THE_ORBIT_IN_A_DAY: f64 = 0.985_600_28;

/// How far the real orbit runs ahead of the even circle, in degrees — the
/// first and largest of the three terms, which is the orbit being an ellipse.
const THE_EQUATION_OF_CENTRE: f64 = 1.9148;

/// The second term, which is the ellipse not being a small one.
const THE_SECOND_TERM: f64 = 0.0200;

/// The third, which is below the accuracy of everything else here and is
/// carried because the published form carries it.
const THE_THIRD_TERM: f64 = 0.0003;

/// Where in the orbit the earth is nearest the sun, in degrees.
const THE_NEAREST_POINT: f64 = 102.9372;

/// How much the highest point of the sun moves because the orbit is an
/// ellipse, in days.
const THE_ORBITS_SHARE_OF_NOON: f64 = 0.0053;

/// And how much it moves because the earth leans, in days.
const THE_LEANS_SHARE_OF_NOON: f64 = 0.0069;

/// How far the earth leans, in degrees. The whole of why there are seasons.
const THE_LEAN: f64 = 23.44;

/// How far below the horizon the middle of the sun is at the moment a person
/// would say it had set: its own width, and the bending of light through the
/// air above it.
const THE_HORIZON: f64 = -0.833;

/// How many degrees the earth turns in a day.
const A_TURN: f64 = 360.0;

/// Why a number is not somewhere on the earth.
///
/// There is no `Display`: the only road to words is [`NowhereOnEarth::said`],
/// and what a settings file that did not read writes is [`NotRead`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NowhereOnEarth {
    /// How far north or south, outside the poles — or not a number at all.
    NotALatitude(f64),
    /// How far east or west, outside the whole way round — or not a number at
    /// all.
    NotALongitude(f64),
}

impl NowhereOnEarth {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::NotALatitude(_) => words::NOT_A_LATITUDE,
            Self::NotALongitude(_) => words::NOT_A_LONGITUDE,
        }
    }

    /// What was asked for.
    #[must_use]
    pub const fn asked(self) -> f64 {
        match self {
            Self::NotALatitude(asked) | Self::NotALongitude(asked) => asked,
        }
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(
            &self.word().key(),
            &Filling::of("asked", self.asked().to_string()),
        )
    }
}

/// Where on the earth a person says they are.
///
/// **Typed, never looked up.** There is one constructor and it takes two
/// numbers somebody entered; nothing in this crate can arrive at one any other
/// way.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "Written", into = "Written")]
pub struct Whereabouts {
    /// How far north, in degrees, which is negative south of the equator.
    latitude: f64,
    /// How far east, in degrees, which is negative west of the meridian.
    longitude: f64,
}

/// Two numbers a person typed are equal or they are not: neither can be *not a
/// number*, because [`Whereabouts::typed`] refuses that before one exists.
impl Eq for Whereabouts {}

impl Whereabouts {
    /// How far north the north pole is, in degrees.
    pub const THE_POLES: f64 = 90.0;

    /// How far east the whole way round is, in degrees.
    pub const THE_WAY_ROUND: f64 = 180.0;

    /// Where the person says they are.
    ///
    /// # Errors
    /// [`NowhereOnEarth`] for a latitude past a pole, a longitude past the way
    /// round, or either given as something that is not a number.
    pub fn typed(latitude: f64, longitude: f64) -> Result<Self, NowhereOnEarth> {
        if !latitude.is_finite() || latitude.abs() > Self::THE_POLES {
            return Err(NowhereOnEarth::NotALatitude(latitude));
        }
        if !longitude.is_finite() || longitude.abs() > Self::THE_WAY_ROUND {
            return Err(NowhereOnEarth::NotALongitude(longitude));
        }
        Ok(Self {
            latitude,
            longitude,
        })
    }

    /// How far north, in degrees.
    #[must_use]
    pub const fn latitude(self) -> f64 {
        self.latitude
    }

    /// How far east, in degrees.
    #[must_use]
    pub const fn longitude(self) -> f64 {
        self.longitude
    }

    /// What the sun does here on the day this moment falls on.
    ///
    /// The two times are on the clock the moment carries, so a machine keeping
    /// summer time is answered in summer time.
    #[must_use]
    pub fn sun_on(self, moment: Moment) -> Sun {
        // A Julian day number is a count, and every count this arithmetic could
        // be given is far inside what a `f64` holds exactly.
        let days = (moment.julian_day() as f64) - J2000 + CLOCKS_HELD_BACK;
        let mean_noon = days - self.longitude / A_TURN;
        let orbit = (THE_ORBIT_AT_J2000 + THE_ORBIT_IN_A_DAY * mean_noon).rem_euclid(A_TURN);
        let centre = THE_EQUATION_OF_CENTRE * orbit.to_radians().sin()
            + THE_SECOND_TERM * (2.0 * orbit).to_radians().sin()
            + THE_THIRD_TERM * (3.0 * orbit).to_radians().sin();
        let along_the_year = (orbit + centre + A_TURN / 2.0 + THE_NEAREST_POINT).rem_euclid(A_TURN);
        let highest = J2000 + mean_noon + THE_ORBITS_SHARE_OF_NOON * orbit.to_radians().sin()
            - THE_LEANS_SHARE_OF_NOON * (2.0 * along_the_year).to_radians().sin();

        let leaning_sin = along_the_year.to_radians().sin() * THE_LEAN.to_radians().sin();
        let leaning_cos = leaning_sin.mul_add(-leaning_sin, 1.0).sqrt();
        let here = self.latitude.to_radians();
        // The divisor is never zero, which is why there is no arm here for it:
        // the lean is far short of a right angle, so `leaning_cos` cannot reach
        // zero, and no angle a `f64` holds has a cosine of exactly zero — a
        // pole comes out of the two arms below rather than as an answer that is
        // not a number.
        let how_long = (THE_HORIZON.to_radians().sin() - here.sin() * leaning_sin)
            / (here.cos() * leaning_cos);

        if how_long >= 1.0 {
            return Sun::NeverRises;
        }
        if how_long <= -1.0 {
            return Sun::NeverSets;
        }

        let either_side = how_long.acos().to_degrees() / A_TURN;
        let east = moment.minutes_east_of_universal_time();
        Sun::SetsAndRises(Between::worked_out(
            on_the_clock(highest + either_side, east),
            on_the_clock(highest - either_side, east),
        ))
    }
}

/// What the sun does here today.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sun {
    /// It sets this evening and rises again this morning, at these two times on
    /// the person's own clock.
    ///
    /// The morning is this day's rather than tomorrow's, which differ by a
    /// minute or two at most: the alternative is a second day's arithmetic for
    /// a dawn nobody is watching.
    SetsAndRises(Between),
    /// It does not set at all today — the summer half of a polar year.
    NeverSets,
    /// It does not rise at all today — the winter half of one.
    NeverRises,
}

/// The time on the person's clock that this Julian date falls at.
fn on_the_clock(julian: f64, minutes_east: i32) -> TimeOfDay {
    // A Julian date turns over at noon, so half a day on puts the fraction of
    // it at midnight, universal time.
    let through_the_day = (julian + 0.5).rem_euclid(1.0);
    let minutes = through_the_day.mul_add(f64::from(MINUTES_IN_A_DAY), f64::from(minutes_east));
    let wrapped = minutes.round().rem_euclid(f64::from(MINUTES_IN_A_DAY));
    // Held between zero and a day immediately above, so the whole number fits
    // the count of minutes a day is measured in.
    TimeOfDay::after_midnight(wrapped as u16)
}

/// Where on the earth, as a settings file holds it.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    /// How far north, in degrees.
    latitude: f64,
    /// How far east, in degrees.
    longitude: f64,
}

impl TryFrom<Written> for Whereabouts {
    type Error = NotRead;

    fn try_from(written: Written) -> Result<Self, Self::Error> {
        Self::typed(written.latitude, written.longitude)
            .map_err(|refused| NotRead::about(refused.word()))
    }
}

impl From<Whereabouts> for Written {
    fn from(whereabouts: Whereabouts) -> Self {
        Self {
            latitude: whereabouts.latitude,
            longitude: whereabouts.longitude,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use super::*;
    use crate::testing::in_english;

    /// Noon, universal time, on the day this many days after the first of
    /// January 1970.
    fn noon_on(days: u64) -> Moment {
        noon_on_a_clock(days, 0)
    }

    /// Noon **on the person's own clock** on that day, for a machine this far
    /// east of universal time — so that the day the arithmetic runs on is the
    /// day the test names, whichever clock it is kept on.
    fn noon_on_a_clock(days: u64, minutes_east: i32) -> Moment {
        let seconds = i64::try_from(days).unwrap() * 24 * 60 * 60 + 12 * 60 * 60
            - i64::from(minutes_east) * 60;
        Moment::at(
            UNIX_EPOCH + Duration::from_secs(u64::try_from(seconds).unwrap()),
            minutes_east,
        )
    }

    /// The twenty-first of June 2026, which is 20,625 days after the first of
    /// January 1970 — the longest day of the northern year.
    const MIDSUMMER: u64 = 20_625;

    /// The twenty-first of December 2026, which is 20,808 days after it.
    const MIDWINTER: u64 = 20_808;

    /// **The answer is the one an almanac prints.** London on the longest day
    /// of 2026: the sun sets at 21:21 and rises at 04:43, British summer time.
    /// A minute either way is inside what this arithmetic claims; an hour is
    /// not, and neither is the wrong day.
    #[test]
    fn a_sunset_is_the_one_an_almanac_prints() {
        let london = Whereabouts::typed(51.5074, -0.1278).unwrap();
        let Sun::SetsAndRises(night) = london.sun_on(noon_on_a_clock(MIDSUMMER, 60)) else {
            panic!("the sun sets in London in June");
        };
        assert_eq!(night.begins().hour(), 21, "{}", night.begins());
        assert!(
            (20..=23).contains(&night.begins().minute()),
            "sunset was {}",
            night.begins()
        );
        assert_eq!(night.ends().hour(), 4, "{}", night.ends());
        assert!(
            (41..=45).contains(&night.ends().minute()),
            "sunrise was {}",
            night.ends()
        );
    }

    /// **And the clock it is answered on is the person's.** The same instant,
    /// the same place, an offset of nothing: the same sunset an hour earlier.
    #[test]
    fn the_answer_is_on_the_clock_the_machine_keeps() {
        let london = Whereabouts::typed(51.5074, -0.1278).unwrap();
        let Sun::SetsAndRises(summer_time) = london.sun_on(noon_on_a_clock(MIDSUMMER, 60)) else {
            panic!("the sun sets in London in June");
        };
        let Sun::SetsAndRises(universal) = london.sun_on(noon_on(MIDSUMMER)) else {
            panic!("the sun sets in London in June");
        };
        assert_eq!(summer_time.begins().hour(), 21);
        assert_eq!(universal.begins().hour(), 20);
        assert_eq!(summer_time.begins().minute(), universal.begins().minute());
    }

    /// **Winter is not summer.** The same place six months on sets before five
    /// in the afternoon, which is the whole reason a night light on a fixed
    /// schedule is worse than one that follows the sun.
    #[test]
    fn the_shortest_day_is_not_the_longest_one() {
        let london = Whereabouts::typed(51.5074, -0.1278).unwrap();
        let Sun::SetsAndRises(night) = london.sun_on(noon_on(MIDWINTER)) else {
            panic!("the sun sets in London in December");
        };
        assert_eq!(night.begins().hour(), 15, "sunset was {}", night.begins());
        assert_eq!(night.ends().hour(), 8, "sunrise was {}", night.ends());
    }

    /// **The southern half of the world is the other way round**, which falls
    /// out of the arithmetic rather than being a case in it.
    #[test]
    fn midsummer_in_the_north_is_midwinter_in_the_south() {
        let wellington = Whereabouts::typed(-41.2866, 174.7756).unwrap();
        let Sun::SetsAndRises(june) = wellington.sun_on(noon_on_a_clock(MIDSUMMER, 720)) else {
            panic!("the sun sets in Wellington in June");
        };
        let Sun::SetsAndRises(december) = wellington.sun_on(noon_on_a_clock(MIDWINTER, 780)) else {
            panic!("the sun sets in Wellington in December");
        };
        assert!(
            june.begins() < december.begins(),
            "June sets at {} and December at {}",
            june.begins(),
            december.begins()
        );
    }

    /// **Where the sun does not set, this says so** rather than inventing a
    /// time. Tromsø is inside the Arctic circle and has weeks of each, and a
    /// machine that warmed its screens at a sunset that never came would be
    /// wrong for a fortnight at a time.
    #[test]
    fn inside_the_arctic_circle_the_sun_is_up_or_down_for_weeks() {
        let tromso = Whereabouts::typed(69.6492, 18.9553).unwrap();
        assert_eq!(
            tromso.sun_on(noon_on_a_clock(MIDSUMMER, 120)),
            Sun::NeverSets
        );
        assert_eq!(
            tromso.sun_on(noon_on_a_clock(MIDWINTER, 60)),
            Sun::NeverRises
        );
    }

    /// And standing on a pole, where there is no crossing to find at all, is
    /// the same two answers rather than a number that is not one.
    #[test]
    fn a_pole_is_answered_and_is_not_a_number_that_is_not_one() {
        let north = Whereabouts::typed(90.0, 0.0).unwrap();
        let south = Whereabouts::typed(-90.0, 0.0).unwrap();
        assert_eq!(north.sun_on(noon_on(MIDSUMMER)), Sun::NeverSets);
        assert_eq!(south.sun_on(noon_on(MIDSUMMER)), Sun::NeverRises);
        assert_eq!(north.sun_on(noon_on(MIDWINTER)), Sun::NeverRises);
        assert_eq!(south.sun_on(noon_on(MIDWINTER)), Sun::NeverSets);
    }

    /// **Somewhere that is not on the earth is refused**, in words, where it is
    /// typed — and *not a number at all* is one of those places, which is what
    /// makes two of these equal or not equal and never neither.
    #[test]
    fn somewhere_that_is_not_on_the_earth_is_refused() {
        assert_eq!(
            Whereabouts::typed(90.1, 0.0),
            Err(NowhereOnEarth::NotALatitude(90.1))
        );
        assert_eq!(
            Whereabouts::typed(0.0, -180.5),
            Err(NowhereOnEarth::NotALongitude(-180.5))
        );
        assert!(matches!(
            Whereabouts::typed(f64::NAN, 0.0),
            Err(NowhereOnEarth::NotALatitude(_))
        ));
        assert!(matches!(
            Whereabouts::typed(0.0, f64::INFINITY),
            Err(NowhereOnEarth::NotALongitude(_))
        ));
        assert!(Whereabouts::typed(90.0, 180.0).is_ok());
        assert!(Whereabouts::typed(-90.0, -180.0).is_ok());

        let strings = in_english();
        let said = Whereabouts::typed(95.0, 0.0).unwrap_err().said(&strings);
        assert!(said.text().contains("95"), "{said}");
        assert!(said.text().contains("90"), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
        assert!(!said.is_a_bug(), "{said}");
    }

    /// Somewhere survives a settings file, and somewhere that is not on the
    /// earth is refused again on the way in.
    #[test]
    fn somewhere_survives_a_file_and_is_checked_again_on_the_way_in() {
        let london = Whereabouts::typed(51.5074, -0.1278).unwrap();
        let written = serde_json::to_string(&london).unwrap();
        assert_eq!(
            serde_json::from_str::<Whereabouts>(&written).unwrap(),
            london
        );

        let refused = serde_json::from_str::<Whereabouts>(r#"{"latitude":91.0,"longitude":0.0}"#)
            .unwrap_err();
        assert!(
            refused.to_string().contains("displays.not-a-latitude"),
            "{refused}"
        );
        assert!(
            serde_json::from_str::<Whereabouts>(r#"{"latitude":51.5,"longitude":0.0,"town":"x"}"#)
                .is_err(),
            "a key this shape does not have is refused where it is read"
        );
    }
}
