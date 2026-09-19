//! How warm a screen is drawn, and what that does to every colour on it.
//!
//! # A warmth is a colour temperature, and the range is closed at both ends
//!
//! [`Warmth::NEUTRAL`] is 6500 K, which is not a setting so much as a fact:
//! it is the white the colour space every screen in this system draws in is
//! defined against, so it is the one temperature at which night light changes
//! **nothing**. Everything below it takes green and blue away and leaves red
//! alone, which is what warming an image is.
//!
//! [`Warmth::WARMEST`] is 2000 K, and the floor is not arbitrary: below about
//! 1900 K the curve below has no blue left to take and stops being a
//! description of anything, so the warmest setting is a little above the point
//! where the arithmetic gives out.
//!
//! **And the second thing that bounds it is the agent.** ADR 0010 reserves
//! terracotta, and warming pulls every hue on a screen towards orange — so
//! *terracotta means the agent* has to be re-measured at every warmth a person
//! can choose, and `tests/terracotta_still_means_the_agent_under_night_light.rs`
//! is that measurement. It is not what puts the floor at 2000 K today: at the
//! warmest setting the nearest accent is still 9.5 ΔE\*ab from the agent's
//! colour, against a floor of 5.0. It is what would refuse a warmer one.
//!
//! # And the arithmetic is one curve, normalised at the neutral point
//!
//! [`Warming`] turns a warmth into what each of a screen's three channels is
//! multiplied by. The curve is the published approximation of what a black body
//! at a temperature looks like; what is this crate's own is that it is
//! **divided through by its own value at [`Warmth::NEUTRAL`]**, so that neutral
//! is exactly one for each channel rather than nearly one. A night light whose
//! *off* was a faint tint would be a night light nobody could turn off.
//!
//! Nothing here draws. What a compositor does with three numbers is the shell's
//! ([`crate::Wearing`] is where a screen is handed its warmth), and no device
//! is opened in this crate (ADR 0011).

use serde::{Deserialize, Serialize};

use alo_appearance::Colour;
use alo_strings::{Filling, Said, Strings, Word};

use crate::unreadable::NotRead;
use crate::words;

/// The first of the three numbers the published approximation is written with,
/// for how much green a black body sends.
const GREEN_RISES_BY: f64 = 99.470_802_586_1;

/// And what that rise is measured from.
const GREEN_STARTS_AT: f64 = 161.119_568_166_1;

/// The same two for blue, which is the channel a warm screen has least of.
const BLUE_RISES_BY: f64 = 138.517_731_223_1;

/// And what blue's rise is measured from.
const BLUE_STARTS_AT: f64 = 305.044_792_730_7;

/// The temperature the approximation is written in hundreds of kelvin, and the
/// offset blue's term is taken from.
const BLUE_IS_TAKEN_FROM: f64 = 10.0;

/// A hundred kelvin, which is the step the approximation counts in.
const A_HUNDRED_KELVIN: f64 = 100.0;

/// The largest a channel can be.
const A_WHOLE_CHANNEL: f64 = 255.0;

/// Why a number is not a warmth a screen can be drawn at.
///
/// There is no `Display`: the only road to words is [`WarmthError::said`], and
/// what a settings file that did not read writes is [`NotRead`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WarmthError {
    /// What was asked for, in kelvin.
    asked: u16,
}

impl WarmthError {
    /// What was asked for, in kelvin.
    #[must_use]
    pub const fn asked(self) -> u16 {
        self.asked
    }

    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(self) -> Word {
        words::NOT_A_WARMTH
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(
            &self.word().key(),
            &Filling::of("asked", format!("{} K", self.asked))
                .and("least", format!("{} K", Warmth::WARMEST))
                .and("most", format!("{} K", Warmth::NEUTRAL)),
        )
    }
}

/// How warm a screen is drawn, as a colour temperature in kelvin.
///
/// Ordered by the number, so a smaller one is a warmer screen. Reads back
/// through [`Warmth::kelvin`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u16", into = "u16")]
pub struct Warmth {
    /// The colour temperature, in kelvin.
    kelvin: u16,
}

impl Warmth {
    /// The warmest a screen can be drawn, in kelvin. See this file's header for
    /// the two things that put the floor here.
    pub const WARMEST: u16 = 2000;

    /// The temperature at which night light changes nothing, in kelvin: the
    /// white the colour space every screen here draws in is defined against.
    pub const NEUTRAL: u16 = 6500;

    /// This many kelvin.
    ///
    /// # Errors
    /// [`WarmthError`] outside [`Warmth::WARMEST`] and [`Warmth::NEUTRAL`].
    pub const fn kelvin(kelvin: u16) -> Result<Self, WarmthError> {
        if kelvin < Self::WARMEST || kelvin > Self::NEUTRAL {
            return Err(WarmthError { asked: kelvin });
        }
        Ok(Self { kelvin })
    }

    /// The warmth at which nothing is warmed at all.
    #[must_use]
    pub const fn neutral() -> Self {
        Self {
            kelvin: Self::NEUTRAL,
        }
    }

    /// A warmth the compiler can build, for the one this crate ships.
    ///
    /// Unchecked, and the only caller is [`crate::NightLight::as_shipped`] —
    /// which is held to the same range by a test that puts the shipped warmth
    /// back through [`Warmth::kelvin`].
    pub(crate) const fn shipped(kelvin: u16) -> Self {
        Self { kelvin }
    }

    /// The colour temperature, in kelvin.
    #[must_use]
    pub const fn as_kelvin(self) -> u16 {
        self.kelvin
    }

    /// Whether this is the warmth that leaves every colour where it was.
    #[must_use]
    pub const fn changes_nothing(self) -> bool {
        self.kelvin == Self::NEUTRAL
    }
}

impl TryFrom<u16> for Warmth {
    type Error = NotRead;

    fn try_from(kelvin: u16) -> Result<Self, Self::Error> {
        Self::kelvin(kelvin).map_err(|refused| NotRead::about(refused.word()))
    }
}

impl From<Warmth> for u16 {
    fn from(warmth: Warmth) -> Self {
        warmth.kelvin
    }
}

/// What each of a screen's three channels is multiplied by at one warmth.
///
/// Never above one in any channel: warming a screen takes light away and never
/// adds it, so a warmed white is a warmer white rather than a brighter one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Warming {
    /// What red is multiplied by, which is one everywhere in this range.
    red: f64,
    /// What green is multiplied by.
    green: f64,
    /// What blue is multiplied by, which is the channel that goes first.
    blue: f64,
}

impl Warming {
    /// What this warmth does to a screen's three channels.
    #[must_use]
    pub fn at(warmth: Warmth) -> Self {
        let (green, blue) = a_black_body(f64::from(warmth.as_kelvin()));
        let (neutral_green, neutral_blue) = a_black_body(f64::from(Warmth::NEUTRAL));
        Self {
            // Red is the whole channel at every temperature in this range, so
            // it is one here rather than a ratio that is one.
            red: 1.0,
            green: (green / neutral_green).clamp(0.0, 1.0),
            blue: (blue / neutral_blue).clamp(0.0, 1.0),
        }
    }

    /// What red is multiplied by.
    #[must_use]
    pub const fn red(self) -> f64 {
        self.red
    }

    /// What green is multiplied by.
    #[must_use]
    pub const fn green(self) -> f64 {
        self.green
    }

    /// What blue is multiplied by.
    #[must_use]
    pub const fn blue(self) -> f64 {
        self.blue
    }

    /// This colour as a screen at this warmth shows it.
    #[must_use]
    pub fn applied_to(self, colour: Colour) -> Colour {
        Colour::of(
            dimmed(colour.red(), self.red),
            dimmed(colour.green(), self.green),
            dimmed(colour.blue(), self.blue),
        )
    }
}

/// How much green and how much blue a black body at this temperature sends,
/// each out of [`A_WHOLE_CHANNEL`].
///
/// Red is left out because it is the whole channel everywhere between
/// [`Warmth::WARMEST`] and [`Warmth::NEUTRAL`]: the approximation only takes
/// red away above 6600 K, which is warmer than this crate offers.
fn a_black_body(kelvin: f64) -> (f64, f64) {
    let hundreds = kelvin / A_HUNDRED_KELVIN;
    let green = GREEN_RISES_BY.mul_add(hundreds.ln(), -GREEN_STARTS_AT);
    let blue = BLUE_RISES_BY.mul_add((hundreds - BLUE_IS_TAKEN_FROM).ln(), -BLUE_STARTS_AT);
    (
        green.clamp(0.0, A_WHOLE_CHANNEL),
        blue.clamp(0.0, A_WHOLE_CHANNEL),
    )
}

/// One channel, with this much of it left, as the byte a screen takes.
fn dimmed(channel: u8, left: f64) -> u8 {
    let kept = (f64::from(channel) * left)
        .round()
        .clamp(0.0, A_WHOLE_CHANNEL);
    // Held between zero and a whole channel immediately above.
    kept as u8
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_appearance::Token;

    use super::*;
    use crate::testing::in_english;

    /// **Neutral changes nothing, exactly.** Every colour the palette has comes
    /// back as itself, byte for byte — a night light whose *off* left a tint
    /// would be a night light nobody could turn off.
    #[test]
    fn the_neutral_warmth_leaves_every_colour_exactly_where_it_was() {
        let warming = Warming::at(Warmth::neutral());
        assert!(Warmth::neutral().changes_nothing());
        for token in Token::ALL {
            assert_eq!(
                warming.applied_to(token.colour()),
                token.colour(),
                "{} moved at the neutral warmth",
                token.word().says()
            );
        }
        assert_eq!(
            warming.applied_to(Colour::of(255, 255, 255)),
            Colour::of(255, 255, 255)
        );
        assert_eq!(warming.applied_to(Colour::of(0, 0, 0)), Colour::of(0, 0, 0));
    }

    /// **Warming takes light away and never adds it**, and it takes it away in
    /// the order a warm light does: blue first, then green, and never red.
    #[test]
    fn warming_takes_blue_first_and_green_after_and_never_touches_red() {
        for kelvin in [Warmth::WARMEST, 2500, 3400, 4500, 5500, Warmth::NEUTRAL] {
            let warming = Warming::at(Warmth::kelvin(kelvin).unwrap());
            assert!((warming.red() - 1.0).abs() < f64::EPSILON, "{kelvin}");
            assert!(warming.blue() <= warming.green(), "{kelvin}");
            assert!(warming.green() <= 1.0, "{kelvin}");
            assert!(warming.blue() >= 0.0, "{kelvin}");
            let white = warming.applied_to(Colour::of(255, 255, 255));
            assert_eq!(white.red(), 255, "{kelvin}");
            assert!(white.blue() <= white.green(), "{kelvin}");
        }
    }

    /// **A warmer screen is warmer than a less warm one**, at every step, which
    /// is what makes a slider in Settings mean anything.
    #[test]
    fn a_warmer_screen_has_less_blue_than_a_cooler_one() {
        let mut blue_before = f64::NEG_INFINITY;
        let mut green_before = f64::NEG_INFINITY;
        let mut kelvin = Warmth::WARMEST;
        while kelvin <= Warmth::NEUTRAL {
            let warming = Warming::at(Warmth::kelvin(kelvin).unwrap());
            assert!(warming.blue() > blue_before, "{kelvin}");
            assert!(warming.green() > green_before, "{kelvin}");
            blue_before = warming.blue();
            green_before = warming.green();
            kelvin = kelvin.saturating_add(100);
        }
        assert!(
            Warming::at(Warmth::kelvin(Warmth::WARMEST).unwrap()).blue() < 0.1,
            "the warmest a screen goes has all but no blue in it"
        );
    }

    /// A warmth outside the range is refused where it is given, and the refusal
    /// names both ends so a person knows what to type instead.
    #[test]
    fn a_warmth_no_screen_can_be_drawn_at_is_refused_and_names_the_range() {
        assert_eq!(Warmth::kelvin(1999).unwrap_err().asked(), 1999);
        assert_eq!(Warmth::kelvin(6501).unwrap_err().asked(), 6501);
        assert_eq!(Warmth::kelvin(0).unwrap_err().asked(), 0);
        assert!(Warmth::kelvin(Warmth::WARMEST).is_ok());
        assert!(Warmth::kelvin(Warmth::NEUTRAL).is_ok());
        assert!(!Warmth::kelvin(Warmth::WARMEST).unwrap().changes_nothing());

        let strings = in_english();
        let said = Warmth::kelvin(10_000).unwrap_err().said(&strings);
        assert!(said.text().contains("10000 K"), "{said}");
        assert!(said.text().contains("2000 K"), "{said}");
        assert!(said.text().contains("6500 K"), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
        assert!(!said.is_a_bug(), "{said}");
    }

    /// **A warmth out of a settings file is checked again on the way in**, and
    /// a file that did not read writes the key of the same refusal.
    #[test]
    fn a_warmth_out_of_a_settings_file_is_checked_again() {
        assert_eq!(
            serde_json::from_str::<Warmth>("3400").unwrap(),
            Warmth::kelvin(3400).unwrap()
        );
        assert_eq!(serde_json::to_string(&Warmth::neutral()).unwrap(), "6500");
        let refused = serde_json::from_str::<Warmth>("1000").unwrap_err();
        assert!(
            refused.to_string().contains("displays.not-a-warmth"),
            "{refused}"
        );
    }
}
