//! The PIN that releases the key the security chip is holding.
//!
//! Six characters, where [`crate::Passphrase`] needs twelve, and the difference
//! is not a compromise about how much people can be bothered to type. A PIN is
//! never what an attacker works against: the chip holds the key and counts the
//! wrong answers, so guessing happens at the chip's rate, not at a
//! password-cracker's. A passphrase in a LUKS2 header has no such counter —
//! whoever has the header has as many guesses as they have electricity — and
//! its length is the only thing standing there.
//!
//! So the two numbers are two different arguments, and
//! [ADR 0054](../../../docs/decisions/0054-the-disk-is-sealed-to-this-machine-and-opened-with-a-pin.md)
//! makes both of them.
//!
//! **Characters, not bytes.** A PIN is counted in what the person typed, which
//! on a machine that promises 24 languages is not the same as how many bytes it
//! took — six characters of Greek is six characters.

use std::fmt;

/// How many characters a PIN is at least.
///
/// Six, because the chip's own lockout is what an attacker meets. See this
/// module's own reasoning before changing it: the number is an argument, not a
/// preference.
pub const A_PIN_IS_AT_LEAST: usize = 6;

/// The PIN a person typed, twice.
///
/// No [`Clone`], no [`PartialEq`] and no [`std::fmt::Display`]: there is one way
/// out of this type and it is named for what it is.
pub struct Pin {
    /// Exactly what the person typed, untrimmed — a secret is not tidied up.
    typed: String,
}

/// The two boxes did not make a PIN.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotAPin {
    /// Nothing was typed.
    Nothing,
    /// Fewer characters than a PIN is.
    ///
    /// It says how many are needed and never how many were typed: the first is
    /// what a person has to be told, the second is a fact about their secret
    /// and has no business in a log.
    TooShort {
        /// How many characters a PIN is at least.
        at_least: usize,
    },
    /// The two typings were not the same.
    TheTwoDoNotMatch,
}

impl fmt::Display for NotAPin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nothing => f.write_str("no PIN was typed"),
            Self::TooShort { at_least } => {
                write!(f, "a PIN is at least {at_least} characters")
            }
            Self::TheTwoDoNotMatch => f.write_str("the two typings of the PIN were not the same"),
        }
    }
}

impl std::error::Error for NotAPin {}

impl Pin {
    /// The PIN, from the two boxes a person typed it into.
    ///
    /// Typing it twice is part of the type rather than part of a screen,
    /// because a machine whose disk is sealed to a PIN nobody typed correctly
    /// the first time is a machine recovered with the recovery key on its first
    /// start.
    ///
    /// # Errors
    /// [`NotAPin`]: nothing typed, too short, or the two not the same.
    pub fn typed(first: &str, again: &str) -> Result<Self, NotAPin> {
        if first.is_empty() {
            return Err(NotAPin::Nothing);
        }
        if first.chars().count() < A_PIN_IS_AT_LEAST {
            return Err(NotAPin::TooShort {
                at_least: A_PIN_IS_AT_LEAST,
            });
        }
        if first != again {
            return Err(NotAPin::TheTwoDoNotMatch);
        }
        Ok(Self {
            typed: first.to_owned(),
        })
    }

    /// Exactly what the person typed.
    ///
    /// The one way out, and it exists for one reason: the rented tool has to be
    /// given it. Nothing in this crate calls it, because nothing in this crate
    /// runs that tool — task 6 does, from the sequence
    /// [`crate::THE_ROAD`] describes.
    #[must_use]
    pub fn as_the_person_typed_it(&self) -> &str {
        &self.typed
    }
}

impl fmt::Debug for Pin {
    /// Says that there is a PIN and never what it is. Derived, this would put a
    /// person's PIN in every service log that ever printed the enrolment.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Pin(not put in this line)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A PIN long enough, typed the same twice, is a PIN — and is exactly what
    /// was typed.
    #[test]
    fn a_pin_typed_the_same_twice_is_what_was_typed() {
        let pin = Pin::typed("123456", "123456");
        assert!(matches!(&pin, Ok(pin) if pin.as_the_person_typed_it() == "123456"));
    }

    /// Six characters of Greek is six characters, not twelve bytes.
    #[test]
    fn a_pin_is_counted_in_characters_and_not_in_bytes() {
        assert!(Pin::typed("αβγδεζ", "αβγδεζ").is_ok());
        assert_eq!(
            Pin::typed("αβγδε", "αβγδε").err(),
            Some(NotAPin::TooShort {
                at_least: A_PIN_IS_AT_LEAST
            })
        );
    }

    /// Nothing, too short, and the two not the same: each refused, and each
    /// refused for what it is.
    #[test]
    fn nothing_and_too_short_and_a_mistyping_are_each_refused() {
        assert_eq!(Pin::typed("", "").err(), Some(NotAPin::Nothing));
        assert_eq!(
            Pin::typed("12345", "12345").err(),
            Some(NotAPin::TooShort {
                at_least: A_PIN_IS_AT_LEAST
            })
        );
        assert_eq!(
            Pin::typed("123456", "123457").err(),
            Some(NotAPin::TheTwoDoNotMatch)
        );
        assert_eq!(
            Pin::typed("123456", "").err(),
            Some(NotAPin::TheTwoDoNotMatch)
        );
    }

    /// **A PIN is not in the line that mentions it.**
    #[test]
    fn a_pin_is_never_in_its_own_debug() {
        let said = Pin::typed("hunter2xyz", "hunter2xyz")
            .map(|pin| format!("{pin:?}"))
            .unwrap_or_default();
        assert!(!said.is_empty(), "a ten-character PIN typed twice is a PIN");
        assert!(!said.contains("hunter2xyz"), "{said}");
    }

    /// The refusals say something a person could be told, and name no secret.
    #[test]
    fn each_refusal_says_what_is_wrong_without_saying_the_pin() {
        assert_eq!(
            NotAPin::TooShort {
                at_least: A_PIN_IS_AT_LEAST
            }
            .to_string(),
            "a PIN is at least 6 characters"
        );
        assert_eq!(NotAPin::Nothing.to_string(), "no PIN was typed");
        assert!(
            NotAPin::TheTwoDoNotMatch
                .to_string()
                .contains("not the same")
        );
    }
}
