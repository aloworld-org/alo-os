//! How big the text is.
//!
//! **This is an accessibility setting before it is a taste one.** EN 301 549 —
//! the standard an EU public-sector desktop is procured against — carries WCAG's
//! requirement that text can be resized to 200% without loss of content or
//! function. So 200% is not a number somebody picked as generous: it is the
//! floor this crate is not allowed to sit above, and there is a test that says
//! so rather than a comment hoping somebody remembers.
//!
//! **The scale is a whole percentage.** A settings panel offers steps, because a
//! slider that lands on 113% is a slider nobody wanted; which steps those are is
//! the panel's business, and a person who types a number into the file gets the
//! number they typed.

use std::fmt;

use alo_strings::{Filling, Said, Strings, Word};
use serde::{Deserialize, Serialize};

use crate::unreadable::NotRead;
use crate::words;

/// The smallest text alo OS will draw, as a percentage.
const SMALLEST: u16 = 75;

/// The largest text alo OS will draw, as a percentage. Above the 200% EN 301 549
/// requires, because a person who needs 200% is not always a person who needs
/// exactly 200%.
const LARGEST: u16 = 300;

/// Text at the size the shell was designed at.
const ORDINARY: u16 = 100;

/// Why a size cannot be used.
///
/// Both name the end of the range they are outside, because a person setting
/// this may be doing it because they cannot read the screen as it is.
///
/// There is no `Display`: the only road to words is [`TextError::said`], and
/// what a settings file that did not read writes is [`NotRead`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextError {
    /// Smaller than the shell can be read at: what was asked for, then the
    /// smallest there is.
    TooSmall(u16, u16),
    /// Larger than the shell has room for: what was asked for, then the largest
    /// there is.
    TooLarge(u16, u16),
}

impl TextError {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::TooSmall(_, _) => words::TOO_SMALL,
            Self::TooLarge(_, _) => words::TOO_LARGE,
        }
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    ///
    /// Both numbers go in without a percent sign on them, and the sign is part
    /// of the sentence — so a language that writes *200 %* with a space, or puts
    /// the sign in front, can. Where a number goes is a translator's; what the
    /// number *is* is not.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        let filling = match self {
            Self::TooSmall(percent, smallest) => {
                Filling::of("percent", percent.to_string()).and("smallest", smallest.to_string())
            }
            Self::TooLarge(percent, largest) => {
                Filling::of("percent", percent.to_string()).and("largest", largest.to_string())
            }
        };
        strings.say(&self.word().key(), &filling)
    }
}

/// How big the text is, as a percentage of the size the shell was designed at.
///
/// Reads back through [`TextScale::percent`], so a file cannot ask for text
/// nobody could read or a menu that does not fit on the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u16", into = "u16")]
pub struct TextScale {
    /// The percentage, between [`SMALLEST`] and [`LARGEST`].
    percent: u16,
}

impl TextScale {
    /// The size the shell was designed at.
    #[must_use]
    pub const fn ordinary() -> Self {
        Self { percent: ORDINARY }
    }

    /// Text at this percentage.
    ///
    /// # Errors
    /// [`TextError`], which names the end of the range the size is outside.
    pub fn percent(percent: u16) -> Result<Self, TextError> {
        if percent < SMALLEST {
            return Err(TextError::TooSmall(percent, SMALLEST));
        }
        if percent > LARGEST {
            return Err(TextError::TooLarge(percent, LARGEST));
        }
        Ok(Self { percent })
    }

    /// The percentage.
    #[must_use]
    pub const fn as_percent(self) -> u16 {
        self.percent
    }

    /// The smallest and the largest a settings panel may offer.
    #[must_use]
    pub const fn range() -> (u16, u16) {
        (SMALLEST, LARGEST)
    }
}

impl Default for TextScale {
    fn default() -> Self {
        Self::ordinary()
    }
}

impl fmt::Display for TextScale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.percent)
    }
}

impl TryFrom<u16> for TextScale {
    type Error = NotRead;

    fn try_from(percent: u16) -> Result<Self, Self::Error> {
        Self::percent(percent).map_err(|refused| NotRead::about(refused.word()))
    }
}

impl From<TextScale> for u16 {
    fn from(scale: TextScale) -> Self {
        scale.percent
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

    /// **EN 301 549 is a test, not a sentence.** The standard an EU public-sector
    /// desktop is procured against requires text to reach 200%, so 200% is a
    /// size this model must accept for as long as that is true.
    #[test]
    fn text_reaches_the_two_hundred_percent_the_standard_requires() {
        assert_eq!(TextScale::percent(200).unwrap().as_percent(), 200);
        let (_, largest) = TextScale::range();
        assert!(
            largest >= 200,
            "EN 301 549 requires 200%, and the ceiling is {largest}%"
        );
    }

    /// The ordinary size is the one the shell was drawn at, and it is what a
    /// machine has before anybody changes it.
    #[test]
    fn the_ordinary_size_is_a_hundred_percent() {
        assert_eq!(TextScale::ordinary().as_percent(), 100);
        assert_eq!(TextScale::default(), TextScale::ordinary());
        assert_eq!(TextScale::ordinary().to_string(), "100%");
    }

    /// Both ends are refused, and both refusals say where the end is — because
    /// somebody setting this may not be able to read the screen as it is.
    #[test]
    fn a_size_outside_the_range_is_refused_and_says_the_range() {
        let (smallest, largest) = TextScale::range();
        assert_eq!(
            TextScale::percent(smallest.saturating_sub(1)),
            Err(TextError::TooSmall(smallest.saturating_sub(1), smallest))
        );
        assert_eq!(
            TextScale::percent(largest.saturating_add(1)),
            Err(TextError::TooLarge(largest.saturating_add(1), largest))
        );
        let strings = in_english();
        assert_eq!(
            TextScale::percent(0).unwrap_err().said(&strings).text(),
            "0% is smaller than this screen can be read at — 75% is as small as it goes"
        );
        assert_eq!(
            TextScale::percent(1000).unwrap_err().said(&strings).text(),
            "1000% is larger than the shell has room for — 300% is as large as it goes"
        );
        assert!(TextScale::percent(smallest).is_ok());
        assert!(TextScale::percent(largest).is_ok());
    }

    /// **The percent sign belongs to the sentence, not to the number.** A
    /// language that writes a space before it — or puts it in front — can, and
    /// this is the test that says the numbers arrive bare.
    #[test]
    fn the_percent_sign_is_the_translators_to_place() {
        let strings = translated(&[(
            words::TOO_SMALL,
            "{percent} % ist kleiner, als dieser Bildschirm lesbar ist — {smallest} % ist das \
             Kleinste",
        )]);
        let said = TextScale::percent(10).unwrap_err().said(&strings);
        assert_eq!(
            said.text(),
            "10 % ist kleiner, als dieser Bildschirm lesbar ist — 75 % ist das Kleinste"
        );
        assert!(said.is_translated());
        assert!(said.unfilled().is_empty());
    }

    /// A file is a thing a person edits, so the range is checked again where it
    /// is read.
    #[test]
    fn a_file_cannot_ask_for_unreadable_text() {
        let scale = TextScale::percent(150).unwrap();
        assert_eq!(serde_json::to_string(&scale).unwrap(), "150");
        assert_eq!(serde_json::from_str::<TextScale>("150").unwrap(), scale);
        assert!(serde_json::from_str::<TextScale>("0").is_err());
        assert!(serde_json::from_str::<TextScale>("9999").is_err());
    }
}
