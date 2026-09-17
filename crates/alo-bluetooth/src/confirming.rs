//! **What a device asks for when it is being paired with, shown in full.**
//!
//! Bluetooth pairing has several shapes and they are not interchangeable. A
//! headset with no screen and no keys shows nothing and expects nothing. A
//! keyboard shows nothing and expects six digits to be typed on it. A phone
//! shows six digits and expects a person to see the same six here and say yes.
//!
//! **alo OS shows what the device asked for, in full, and waits.** Not a
//! shortened code, not *pairing…* with the digits somewhere behind a detail
//! view, and never a yes this machine said on a person's behalf. A confirmation
//! nobody read is not a confirmation, and this is the exact place where somebody
//! else's device is asking to become part of a machine.

use crate::words::{self, Word};

/// How many digits a passkey has, always, with the leading zeros kept.
pub const DIGITS: usize = 6;

/// **What the device requires before it will pair.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Asked {
    /// Nothing: the device has no way to show or take a code.
    Nothing,
    /// Six digits this machine shows, which a person types on the device.
    TypeThisOnIt(u32),
    /// Six digits both ends show, which a person compares and confirms.
    TheSameOnBoth(u32),
    /// A code the device's maker printed, which a person types here.
    ItsOwnCode,
}

impl Asked {
    /// **The digits, written the way they must be shown**: six of them, with
    /// the leading zeros, because `012345` shown as `12345` is a person typing
    /// the wrong thing and being told the device refused.
    #[must_use]
    pub fn shown(&self) -> Option<String> {
        match self {
            Self::TypeThisOnIt(digits) | Self::TheSameOnBoth(digits) => {
                Some(format!("{digits:0>width$}", width = DIGITS))
            }
            Self::Nothing | Self::ItsOwnCode => None,
        }
    }

    /// What a person is asked.
    #[must_use]
    pub const fn word(&self) -> Word {
        match self {
            Self::Nothing => words::PAIR_WITH_THIS,
            Self::TypeThisOnIt(_) => words::TYPE_THIS_ON_IT,
            Self::TheSameOnBoth(_) => words::THE_SAME_ON_BOTH,
            Self::ItsOwnCode => words::ITS_OWN_CODE,
        }
    }

    /// **Whether a person has to answer before this can go ahead**, which is
    /// every one of them. The method exists to be read: there is no shape of
    /// pairing this machine completes on its own, including the shape where the
    /// device asks for nothing.
    #[must_use]
    pub const fn needs_a_person(&self) -> bool {
        true
    }
}

/// **What a person said.** There is no third value, and no default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    /// They said yes to what they were shown.
    Yes,
    /// They said no, or they were never asked.
    No,
}

impl Answer {
    /// Whether the pairing goes ahead.
    #[must_use]
    pub const fn is_yes(self) -> bool {
        matches!(self, Self::Yes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Six digits are shown as six digits**, leading zeros and all.
    #[test]
    fn a_passkey_keeps_the_zeros_a_person_has_to_type() {
        assert_eq!(
            Asked::TypeThisOnIt(12_345).shown().as_deref(),
            Some("012345")
        );
        assert_eq!(Asked::TheSameOnBoth(0).shown().as_deref(), Some("000000"));
        assert_eq!(
            Asked::TheSameOnBoth(999_999).shown().as_deref(),
            Some("999999")
        );
        assert_eq!(Asked::Nothing.shown(), None);
        assert_eq!(Asked::ItsOwnCode.shown(), None);
    }

    /// **Every shape of pairing waits for a person**, including the one where
    /// the device asks for nothing at all.
    #[test]
    fn nothing_pairs_without_somebody_saying_yes() {
        for asked in [
            Asked::Nothing,
            Asked::TypeThisOnIt(1),
            Asked::TheSameOnBoth(1),
            Asked::ItsOwnCode,
        ] {
            assert!(asked.needs_a_person(), "{asked:?} would pair on its own");
            assert!(!asked.word().says().is_empty());
        }
        assert!(Answer::Yes.is_yes());
        assert!(!Answer::No.is_yes());
    }
}
