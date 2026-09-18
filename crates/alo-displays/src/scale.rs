//! How large everything is drawn on one screen, and what a screen nobody has
//! ever chosen for is set to.
//!
//! # A scale is a whole per cent, and fractional means what it says
//!
//! A [`Scale`] is a per cent between [`Scale::LEAST`] and [`Scale::MOST`].
//! 125%, 150% and 175% are the sizes a modern panel actually wants, and each of
//! them is *fractional*: not a whole multiple of the pixels the machine has.
//! The compositor draws them where it can say so, and where it cannot, alo OS
//! uses the nearest whole multiple and **says that it did**
//! ([`Support::nearest`]) rather than quietly drawing everything at the wrong
//! size.
//!
//! # A screen nobody has ever seen is not 100%
//!
//! A machine that starts every new screen at 100% is a machine that hands
//! somebody a 4K panel covered in text four millimetres high. So the first size
//! is **worked out from what the screen reports about itself**: its pixels and
//! how large its glass is.
//!
//! [`Scale::worked_out_for`] is that arithmetic, and it is arithmetic rather
//! than a table of models:
//!
//! 1. how many pixels there are per inch across the screen;
//! 2. against 96 of them, which is the density every desktop system has drawn
//!    its 100% against for thirty years;
//! 3. rounded to the nearest quarter, because a quarter is the step every
//!    settings panel offers and a size between two of them is a size nobody can
//!    put back by hand;
//! 4. held between [`Scale::LEAST`] and [`Scale::MOST`].
//!
//! A screen that reports no size at all cannot be worked out — there is nothing
//! to divide by — and is 100%, which is the one case the criticism above
//! applies to and the only honest answer when the machine is told nothing.

use serde::{Deserialize, Serialize};

use alo_strings::{Filling, Said, Strings, Word};

use crate::reported::{Millimetres, Resolution};
use crate::unreadable::NotRead;
use crate::words;

/// The density every system's 100% has meant, in pixels per inch.
const A_HUNDRED_PER_CENT: u64 = 96;

/// A tenth of a millimetre short of an inch, times a hundred: the number that
/// turns millimetres into inches without leaving integers.
const HUNDREDTHS_OF_AN_INCH: u64 = 2540;

/// The step a size is rounded to, in per cent.
const THE_STEP: u16 = 25;

/// Why a number is not a size a screen can be set to.
///
/// There is no `Display`: the only road to words is [`ScaleError::said`], and
/// what a settings file that did not read writes is [`NotRead`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScaleError {
    /// What was asked for.
    asked: u16,
}

impl ScaleError {
    /// What was asked for.
    #[must_use]
    pub const fn asked(self) -> u16 {
        self.asked
    }

    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(self) -> Word {
        words::SIZE_OUT_OF_RANGE
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(
            &self.word().key(),
            &Filling::of("asked", format!("{}%", self.asked))
                .and("least", format!("{}%", Scale::LEAST))
                .and("most", format!("{}%", Scale::MOST)),
        )
    }
}

/// How large everything is drawn on one screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u16", into = "u16")]
pub struct Scale {
    /// The per cent.
    per_cent: u16,
}

impl Scale {
    /// The smallest a screen can be set to, in per cent. Below this, the shell's
    /// own measurements stop fitting the surfaces it draws.
    pub const LEAST: u16 = 100;

    /// The largest, in per cent. Above this there is not room on the smallest
    /// screen alo OS lays a dock out for (`alo_dock::Screen::the_smallest`).
    pub const MOST: u16 = 300;

    /// This many per cent.
    ///
    /// # Errors
    /// [`ScaleError`] outside [`Scale::LEAST`] and [`Scale::MOST`].
    pub const fn per_cent(per_cent: u16) -> Result<Self, ScaleError> {
        if per_cent < Self::LEAST || per_cent > Self::MOST {
            return Err(ScaleError { asked: per_cent });
        }
        Ok(Self { per_cent })
    }

    /// One pixel drawn for each the screen has.
    #[must_use]
    pub const fn a_hundred() -> Self {
        Self {
            per_cent: Self::LEAST,
        }
    }

    /// The per cent.
    #[must_use]
    pub const fn as_per_cent(self) -> u16 {
        self.per_cent
    }

    /// Whether this is not a whole multiple of the screen's own pixels.
    #[must_use]
    pub const fn is_fractional(self) -> bool {
        !self.per_cent.is_multiple_of(100)
    }

    /// How wide something this many pixels across is once it is drawn at this
    /// size, in the pixels the arrangement is laid out in.
    ///
    /// Rounded down: a screen that claimed half a pixel more room than it has
    /// would overlap the one beside it.
    #[must_use]
    pub const fn laid_out(self, pixels: u32) -> u32 {
        let across = (pixels as u64) * 100 / (self.per_cent as u64);
        // A screen is never wider laid out than it is in pixels, because the
        // least size is 100%, so this cannot lose anything.
        across as u32
    }

    /// The size alo OS gives a screen it has never been asked about.
    ///
    /// Worked out from the screen's own pixels and its own glass, by the four
    /// steps in this file's header. A screen that reports no size is
    /// [`Scale::a_hundred`], because there is nothing to work it out from.
    #[must_use]
    pub fn worked_out_for(pixels: Resolution, glass: Option<Millimetres>) -> Self {
        let Some(glass) = glass else {
            return Self::a_hundred();
        };
        let across = u64::from(pixels.width());
        let millimetres = u64::from(glass.width());
        // Pixels per inch, times a hundred, so that nothing is a fraction.
        let density = across * HUNDREDTHS_OF_AN_INCH / millimetres;
        let wanted = density / A_HUNDRED_PER_CENT;
        let step = u64::from(THE_STEP);
        let rounded = (wanted + step / 2) / step * step;
        let held = rounded.clamp(u64::from(Self::LEAST), u64::from(Self::MOST));
        Self {
            // Held between two `u16` constants immediately above.
            per_cent: held as u16,
        }
    }
}

impl TryFrom<u16> for Scale {
    type Error = NotRead;

    fn try_from(per_cent: u16) -> Result<Self, Self::Error> {
        Self::per_cent(per_cent).map_err(|refused| NotRead::about(refused.word()))
    }
}

impl From<Scale> for u16 {
    fn from(scale: Scale) -> Self {
        scale.per_cent
    }
}

/// Which sizes the compositor underneath can actually draw.
///
/// Asked of the compositor once and handed in, because it is a fact about the
/// machine rather than a decision this crate makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Support {
    /// Any of them, including the fractional ones.
    Fractional,
    /// Whole multiples of the screen's own pixels only.
    WholeMultiplesOnly,
}

impl Support {
    /// The nearest size to this one that can actually be drawn.
    ///
    /// The same size where fractional sizes are supported. Otherwise the
    /// nearest whole multiple, never below [`Scale::LEAST`] — 150% rounds up to
    /// 200% rather than down to 100%, because text that is too large can be
    /// read and text that is too small cannot.
    #[must_use]
    pub const fn nearest(self, wanted: Scale) -> Scale {
        match self {
            Self::Fractional => wanted,
            Self::WholeMultiplesOnly => {
                let rounded = (wanted.per_cent + 50) / 100 * 100;
                if rounded < Scale::LEAST {
                    Scale::a_hundred()
                } else {
                    Scale { per_cent: rounded }
                }
            }
        }
    }
}

/// A size that could not be drawn as it was chosen, and what was drawn instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rounded {
    /// What the arrangement asked for.
    asked: Scale,
    /// What the machine can draw.
    used: Scale,
}

impl Rounded {
    /// This size, rounded to what the compositor can draw — or nothing at all,
    /// when it can draw what was asked for.
    #[must_use]
    pub const fn of(asked: Scale, support: Support) -> Option<Self> {
        let used = support.nearest(asked);
        if used.per_cent == asked.per_cent {
            None
        } else {
            Some(Self { asked, used })
        }
    }

    /// What the arrangement asked for.
    #[must_use]
    pub const fn asked(self) -> Scale {
        self.asked
    }

    /// What the machine drew instead.
    #[must_use]
    pub const fn used(self) -> Scale {
        self.used
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_screen, in_english};

    /// **Fractional means what it says**: the three sizes a modern panel wants
    /// are sizes, and each of them is fractional.
    #[test]
    fn the_sizes_a_modern_panel_wants_are_sizes_and_are_fractional() {
        for per_cent in [125, 150, 175] {
            let scale = Scale::per_cent(per_cent).unwrap();
            assert_eq!(scale.as_per_cent(), per_cent);
            assert!(scale.is_fractional(), "{per_cent}");
        }
        assert!(!Scale::a_hundred().is_fractional());
        assert!(!Scale::per_cent(200).unwrap().is_fractional());
    }

    /// **A screen the machine has never seen is not 100%.** A 27-inch 4K panel
    /// is 175% and a 24-inch 1080p one is 100%, worked out from what each
    /// reports rather than looked up.
    #[test]
    fn a_screen_nobody_has_seen_gets_a_size_worked_out_from_what_it_reports() {
        let four_k = a_screen((3840, 2160), Some((596, 336)));
        assert_eq!(
            Scale::worked_out_for(four_k.0, four_k.1),
            Scale::per_cent(175).unwrap()
        );

        let ordinary = a_screen((1920, 1080), Some((531, 299)));
        assert_eq!(
            Scale::worked_out_for(ordinary.0, ordinary.1),
            Scale::a_hundred()
        );

        let laptop = a_screen((1920, 1080), Some((294, 165)));
        assert_eq!(
            Scale::worked_out_for(laptop.0, laptop.1),
            Scale::per_cent(175).unwrap()
        );

        let dense = a_screen((3840, 2160), Some((345, 194)));
        assert_eq!(
            Scale::worked_out_for(dense.0, dense.1),
            Scale::per_cent(Scale::MOST).unwrap(),
            "held at the largest size rather than run past it"
        );
    }

    /// **A screen that reports no size is the one case that is 100%**, because
    /// there is nothing to divide by and inventing a number would be worse.
    #[test]
    fn a_screen_that_reports_no_size_is_a_hundred() {
        let unknown = a_screen((3840, 2160), None);
        assert_eq!(
            Scale::worked_out_for(unknown.0, unknown.1),
            Scale::a_hundred()
        );
    }

    /// **Where the compositor cannot draw a fractional size, the nearest whole
    /// multiple is used and it is not the smaller one.**
    #[test]
    fn a_machine_that_cannot_draw_a_fractional_size_rounds_up_rather_than_down() {
        let one_and_a_half = Scale::per_cent(150).unwrap();
        assert_eq!(Support::Fractional.nearest(one_and_a_half), one_and_a_half);
        assert_eq!(
            Support::WholeMultiplesOnly.nearest(one_and_a_half),
            Scale::per_cent(200).unwrap()
        );
        assert_eq!(
            Support::WholeMultiplesOnly.nearest(Scale::per_cent(125).unwrap()),
            Scale::a_hundred()
        );
        assert_eq!(
            Support::WholeMultiplesOnly.nearest(Scale::per_cent(300).unwrap()),
            Scale::per_cent(300).unwrap()
        );
    }

    /// **A size that was rounded says so**, and a size that was not says
    /// nothing at all — a sentence about every screen every morning is a
    /// sentence nobody reads.
    #[test]
    fn a_size_that_was_rounded_says_so_and_one_that_was_not_says_nothing() {
        let one_and_a_half = Scale::per_cent(150).unwrap();
        assert_eq!(Rounded::of(one_and_a_half, Support::Fractional), None);
        let rounded = Rounded::of(one_and_a_half, Support::WholeMultiplesOnly).unwrap();
        assert_eq!(rounded.asked(), one_and_a_half);
        assert_eq!(rounded.used(), Scale::per_cent(200).unwrap());
        assert_eq!(
            Rounded::of(Scale::a_hundred(), Support::WholeMultiplesOnly),
            None
        );
    }

    /// A size outside the range is refused where it is given, and the refusal
    /// names the range so a person knows what to type instead.
    #[test]
    fn a_size_no_screen_can_be_set_to_is_refused_and_names_the_range() {
        assert_eq!(Scale::per_cent(99).unwrap_err().asked(), 99);
        assert_eq!(Scale::per_cent(301).unwrap_err().asked(), 301);
        assert!(Scale::per_cent(Scale::LEAST).is_ok());
        assert!(Scale::per_cent(Scale::MOST).is_ok());

        let strings = in_english();
        let said = Scale::per_cent(400).unwrap_err().said(&strings);
        assert!(said.text().contains("400%"), "{said}");
        assert!(said.text().contains("100%"), "{said}");
        assert!(said.text().contains("300%"), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
    }

    /// **A size out of a settings file is checked again on the way in**, and a
    /// file that did not read writes the key of the same refusal.
    #[test]
    fn a_size_out_of_a_settings_file_is_checked_again() {
        assert_eq!(
            serde_json::from_str::<Scale>("150").unwrap(),
            Scale::per_cent(150).unwrap()
        );
        let refused = serde_json::from_str::<Scale>("400").unwrap_err();
        assert!(
            refused.to_string().contains("displays.size-out-of-range"),
            "{refused}"
        );
    }

    /// How wide a screen is once it is drawn at this size, which is what puts
    /// one screen beside another.
    #[test]
    fn a_screen_takes_less_room_the_larger_everything_on_it_is_drawn() {
        assert_eq!(Scale::a_hundred().laid_out(3840), 3840);
        assert_eq!(Scale::per_cent(200).unwrap().laid_out(3840), 1920);
        assert_eq!(Scale::per_cent(150).unwrap().laid_out(3840), 2560);
    }
}
