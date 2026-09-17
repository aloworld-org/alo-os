//! The passphrase typed at every start, on a machine whose chip cannot hold a
//! key.
//!
//! Twelve characters, where [`crate::Pin`] needs six.
//! [ADR 0054](../../../docs/decisions/0054-the-disk-is-sealed-to-this-machine-and-opened-with-a-pin.md)
//! makes the argument and `crate::pin` restates it: there is no chip counting
//! the wrong answers here. Whoever has the volume's header has as many guesses
//! as they have electricity, and what stands between them and the disk is the
//! key derivation and the passphrase's own length.
//!
//! Twelve is a floor and not advice. This crate cannot tell a person that
//! twelve ordinary words would be better, because this crate says nothing to
//! anybody; that sentence is task 6's to write and task 7's to translate.

use std::fmt;

/// How many characters a passphrase is at least.
///
/// Twelve, because nothing rate limits an attacker holding the header. See this
/// module's own reasoning before changing it.
pub const A_PASSPHRASE_IS_AT_LEAST: usize = 12;

/// The passphrase a person typed, twice.
///
/// No [`Clone`], no [`PartialEq`] and no [`std::fmt::Display`], for the reasons
/// [`crate::Pin`] has none.
pub struct Passphrase {
    /// Exactly what the person typed, untrimmed: a passphrase may begin or end
    /// with a space, and tidying it up would lock them out of their own disk on
    /// the next start.
    typed: String,
}

/// The two boxes did not make a passphrase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotAPassphrase {
    /// Nothing was typed.
    Nothing,
    /// Fewer characters than a passphrase is. Says how many are needed and
    /// never how many were typed.
    TooShort {
        /// How many characters a passphrase is at least.
        at_least: usize,
    },
    /// The two typings were not the same.
    TheTwoDoNotMatch,
}

impl fmt::Display for NotAPassphrase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nothing => f.write_str("no passphrase was typed"),
            Self::TooShort { at_least } => {
                write!(f, "a passphrase is at least {at_least} characters")
            }
            Self::TheTwoDoNotMatch => {
                f.write_str("the two typings of the passphrase were not the same")
            }
        }
    }
}

impl std::error::Error for NotAPassphrase {}

impl Passphrase {
    /// The passphrase, from the two boxes a person typed it into.
    ///
    /// # Errors
    /// [`NotAPassphrase`]: nothing typed, too short, or the two not the same.
    pub fn typed(first: &str, again: &str) -> Result<Self, NotAPassphrase> {
        if first.is_empty() {
            return Err(NotAPassphrase::Nothing);
        }
        if first.chars().count() < A_PASSPHRASE_IS_AT_LEAST {
            return Err(NotAPassphrase::TooShort {
                at_least: A_PASSPHRASE_IS_AT_LEAST,
            });
        }
        if first != again {
            return Err(NotAPassphrase::TheTwoDoNotMatch);
        }
        Ok(Self {
            typed: first.to_owned(),
        })
    }

    /// Exactly what the person typed.
    ///
    /// The one way out, and it exists because the rented tool has to be given
    /// it. Nothing in this crate calls it.
    #[must_use]
    pub fn as_the_person_typed_it(&self) -> &str {
        &self.typed
    }
}

impl fmt::Debug for Passphrase {
    /// Says that there is a passphrase and never what it is.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Passphrase(not put in this line)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A passphrase long enough, typed the same twice, is what was typed —
    /// spaces at either end included.
    #[test]
    fn a_passphrase_is_exactly_what_was_typed() {
        let typed = " correct horse battery ";
        let passphrase = Passphrase::typed(typed, typed);
        assert!(matches!(&passphrase, Ok(it) if it.as_the_person_typed_it() == typed));
    }

    /// Nothing, too short, and the two not the same: each refused for what it
    /// is.
    #[test]
    fn nothing_and_too_short_and_a_mistyping_are_each_refused() {
        assert_eq!(
            Passphrase::typed("", "").err(),
            Some(NotAPassphrase::Nothing)
        );
        assert_eq!(
            Passphrase::typed("elevenchars", "elevenchars").err(),
            Some(NotAPassphrase::TooShort {
                at_least: A_PASSPHRASE_IS_AT_LEAST
            })
        );
        assert_eq!(
            Passphrase::typed("twelvecharsx", "twelvecharsy").err(),
            Some(NotAPassphrase::TheTwoDoNotMatch)
        );
    }

    /// A passphrase is longer than a PIN, and that is the decision rather than
    /// an accident of two numbers.
    #[test]
    fn a_passphrase_is_longer_than_a_pin() {
        const { assert!(A_PASSPHRASE_IS_AT_LEAST > crate::A_PIN_IS_AT_LEAST) };
    }

    /// **A passphrase is not in the line that mentions it.**
    #[test]
    fn a_passphrase_is_never_in_its_own_debug() {
        let said = Passphrase::typed("correcthorsebattery", "correcthorsebattery")
            .map(|it| format!("{it:?}"))
            .unwrap_or_default();
        assert!(!said.is_empty(), "a long passphrase typed twice is one");
        assert!(!said.contains("correcthorse"), "{said}");
    }

    /// The refusals say something a person could be told, and name no secret.
    #[test]
    fn each_refusal_says_what_is_wrong_without_saying_the_passphrase() {
        assert_eq!(
            NotAPassphrase::TooShort {
                at_least: A_PASSPHRASE_IS_AT_LEAST
            }
            .to_string(),
            "a passphrase is at least 12 characters"
        );
        assert_eq!(
            NotAPassphrase::Nothing.to_string(),
            "no passphrase was typed"
        );
    }
}
