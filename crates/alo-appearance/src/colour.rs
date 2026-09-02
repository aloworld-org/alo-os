//! A colour, and the one way it is written down.
//!
//! Three channels and no fourth: a background that could be partly transparent
//! would be a background with something behind it, and there is nothing behind
//! the background.
//!
//! **A colour is written the way a person writes one** — `#102A43`, the way
//! every design tool and every stylesheet in the world spells it — so a settings
//! file stays a file somebody can read and edit. It is checked where it is read,
//! so `#12345` never becomes a colour nobody chose.

use std::fmt;

use alo_strings::{Filling, Said, Strings, Word};
use serde::{Deserialize, Serialize};

use crate::unreadable::NotRead;
use crate::words;

/// Why a piece of text is not a colour.
///
/// Both say the shape a colour has, because somebody seeing one of these is
/// looking at a file they typed into and wants to know what to type instead.
///
/// There is no `Display`: the only road to words is [`ColourError::said`]. What
/// a settings file that did not read writes instead is [`NotRead`], which is the
/// key of the refusal rather than an English sentence nothing could translate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColourError {
    /// Not a hash and six digits.
    NotAColour(String),
    /// A character that is not a hexadecimal digit.
    NotADigit(char),
}

impl ColourError {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(&self) -> Word {
        match *self {
            Self::NotAColour(_) => words::NOT_A_COLOUR,
            Self::NotADigit(_) => words::NOT_A_DIGIT,
        }
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = match self {
            Self::NotAColour(text) => Filling::of("text", text.clone()),
            Self::NotADigit(character) => Filling::of("character", character.to_string()),
        };
        strings.say(&self.word().key(), &filling)
    }
}

/// One colour.
///
/// Serialises as the text a person would write, and reads back through
/// [`Colour::written`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Colour {
    /// How much red, 0 to 255.
    red: u8,
    /// How much green, 0 to 255.
    green: u8,
    /// How much blue, 0 to 255.
    blue: u8,
}

impl Colour {
    /// A colour from its three channels.
    #[must_use]
    pub const fn of(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    /// A colour as it is written: a hash and six hexadecimal digits.
    ///
    /// # Errors
    /// [`ColourError`], which says the shape a colour is written in.
    pub fn written(text: &str) -> Result<Self, ColourError> {
        let digits = text
            .strip_prefix('#')
            .ok_or_else(|| ColourError::NotAColour(text.to_owned()))?;
        let mut chars = digits.chars();
        let red = channel(&mut chars, text)?;
        let green = channel(&mut chars, text)?;
        let blue = channel(&mut chars, text)?;
        if chars.next().is_some() {
            return Err(ColourError::NotAColour(text.to_owned()));
        }
        Ok(Self::of(red, green, blue))
    }

    /// How much red.
    #[must_use]
    pub const fn red(self) -> u8 {
        self.red
    }

    /// How much green.
    #[must_use]
    pub const fn green(self) -> u8 {
        self.green
    }

    /// How much blue.
    #[must_use]
    pub const fn blue(self) -> u8 {
        self.blue
    }
}

/// One channel: two hexadecimal digits taken off the front.
fn channel(chars: &mut impl Iterator<Item = char>, text: &str) -> Result<u8, ColourError> {
    let high = digit(
        chars
            .next()
            .ok_or(ColourError::NotAColour(text.to_owned()))?,
    )?;
    let low = digit(
        chars
            .next()
            .ok_or(ColourError::NotAColour(text.to_owned()))?,
    )?;
    Ok(high.saturating_mul(16).saturating_add(low))
}

/// One hexadecimal digit, 0 to 15.
fn digit(character: char) -> Result<u8, ColourError> {
    let value = character
        .to_digit(16)
        .ok_or(ColourError::NotADigit(character))?;
    // `to_digit(16)` answers with at most 15, so the conversion cannot fail;
    // it is written as a conversion anyway because a cast that is only correct
    // while an argument stays 16 is a cast waiting for somebody to change it.
    u8::try_from(value).map_err(|_| ColourError::NotADigit(character))
}

impl fmt::Display for Colour {
    /// Upper case, because that is how the design tokens are written down.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
    }
}

impl TryFrom<String> for Colour {
    type Error = NotRead;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Self::written(&text).map_err(|refused| NotRead::about(refused.word()))
    }
}

impl From<Colour> for String {
    fn from(colour: Colour) -> Self {
        colour.to_string()
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

    /// What is written is what comes back, in the spelling the design tokens
    /// use.
    #[test]
    fn a_colour_survives_being_written_down() {
        let navy = Colour::written("#102A43").unwrap();
        assert_eq!((navy.red(), navy.green(), navy.blue()), (0x10, 0x2A, 0x43));
        assert_eq!(navy.to_string(), "#102A43");
        assert_eq!(
            Colour::written("#102a43").unwrap(),
            navy,
            "case is not part of it"
        );
        assert_eq!(serde_json::to_string(&navy).unwrap(), r##""#102A43""##);
        assert_eq!(
            serde_json::from_str::<Colour>(r##""#102A43""##).unwrap(),
            navy
        );
    }

    /// Black and white are the ends of every channel and are written like
    /// anything else.
    #[test]
    fn the_two_ends_are_ordinary() {
        assert_eq!(Colour::written("#000000").unwrap(), Colour::of(0, 0, 0));
        assert_eq!(
            Colour::written("#FFFFFF").unwrap(),
            Colour::of(255, 255, 255)
        );
    }

    /// **A file is a thing a person edits**, so everything that is not a colour
    /// is refused where it is read rather than becoming a colour nobody chose.
    #[test]
    fn what_is_not_a_colour_is_refused() {
        assert_eq!(
            Colour::written("102A43"),
            Err(ColourError::NotAColour("102A43".to_owned())),
            "the hash is not decoration"
        );
        assert_eq!(
            Colour::written("#12345"),
            Err(ColourError::NotAColour("#12345".to_owned())),
            "five digits is not a colour, and is not silently five"
        );
        assert_eq!(
            Colour::written("#1234567"),
            Err(ColourError::NotAColour("#1234567".to_owned())),
            "and neither is seven, cut down to six"
        );
        assert_eq!(Colour::written("#10ZZ43"), Err(ColourError::NotADigit('Z')));
        assert!(serde_json::from_str::<Colour>(r#""blue""#).is_err());
    }

    /// A refusal says the shape rather than the mistake, because the person is
    /// mid-edit and wants the next thing to type.
    #[test]
    fn a_refusal_says_what_a_colour_looks_like() {
        let strings = in_english();
        assert_eq!(
            Colour::written("blue").unwrap_err().said(&strings).text(),
            "a colour is a hash and six hexadecimal digits, as in #102A43 — blue is not"
        );
        assert_eq!(
            Colour::written("#10ZZ43")
                .unwrap_err()
                .said(&strings)
                .text(),
            "Z is not a hexadecimal digit — a colour uses 0 to 9 and A to F, as in #102A43"
        );
    }

    /// **And it says it in the language the person reads**, with what they
    /// actually typed in the middle of it — which is not translated, because it
    /// came off their own settings file.
    #[test]
    fn a_refusal_is_read_in_the_readers_language() {
        let strings = translated(&[(
            words::NOT_A_COLOUR,
            "eine Farbe ist ein Rautezeichen und sechs Hexadezimalziffern, etwa #102A43 — {text} \
             ist keine",
        )]);
        let said = Colour::written("blau").unwrap_err().said(&strings);
        assert_eq!(
            said.text(),
            "eine Farbe ist ein Rautezeichen und sechs Hexadezimalziffern, etwa #102A43 — blau \
             ist keine"
        );
        assert!(said.is_translated());
        assert!(said.unfilled().is_empty());
    }

    /// **A settings file that did not read writes the key, not a sentence.**
    /// A deserialiser has no `Strings` and never will, so what it writes is what
    /// whoever reports the file looks up to show the same words a panel shows.
    #[test]
    fn a_file_that_did_not_read_names_the_string_rather_than_saying_it() {
        let refused = serde_json::from_str::<Colour>(r#""blue""#).unwrap_err();
        assert!(
            refused
                .to_string()
                .contains("appearance.colour.not-a-colour"),
            "{refused}"
        );
    }
}
