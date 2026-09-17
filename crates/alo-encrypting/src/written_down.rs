//! That the person kept the recovery key — the only proof there is.
//!
//! [`WrittenDown`] has no constructor. Not a private one, not a `new`, not a
//! `Default`: the only value of this type that has ever existed anywhere came
//! out of [`crate::RecoveryKey::written_back`] with a typing that matched. It is
//! what [`crate::Enrolment`] is made from, so *the disk is encrypted* is a
//! sentence that cannot be written down in this crate's types without the
//! person having typed their recovery key back first.
//!
//! That is the acceptance of task 5 of
//! `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md` — *no road enrols
//! encryption without also producing the recovery key and requiring the person
//! to confirm they kept it* — held by there being no other road rather than by a
//! check somewhere on the road there is.
//!
//! # Why typing it back, and not a box that says *I have written this down*
//!
//! Because a box like that is ticked by everybody, including the person who
//! will be standing in front of a machine that will not open in eighteen
//! months' time. A person who can type 64 characters back has them. See
//! [ADR 0054](../../../docs/decisions/0054-the-disk-is-sealed-to-this-machine-and-opened-with-a-pin.md).

use std::fmt;

use crate::recovery_key::RecoveryKey;

/// The person wrote the recovery key down and typed it back.
///
/// Carries nothing. It is the fact itself, and its only maker is
/// [`crate::RecoveryKey::written_back`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WrittenDown {
    /// Nothing, deliberately. A private field with no public constructor is
    /// what makes this type unmakeable anywhere but here.
    kept: (),
}

impl WrittenDown {
    /// The one maker, reachable only from inside this crate and called from
    /// exactly one place.
    pub(crate) const fn because_they_typed_it_back() -> Self {
        Self { kept: () }
    }
}

/// What the person typed was not the key they were shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotWhatWasShown {
    /// They typed nothing, or nothing but dashes and spaces.
    Nothing,
    /// They typed something, and it was not this key.
    NotThisKey,
}

impl fmt::Display for NotWhatWasShown {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nothing => f.write_str("the recovery key was not typed back"),
            Self::NotThisKey => {
                f.write_str("what was typed back was not the recovery key that was shown")
            }
        }
    }
}

impl std::error::Error for NotWhatWasShown {}

/// The key, handed back to be shown again.
///
/// [`crate::RecoveryKey::written_back`] consumes the key whether the typing
/// matched or not, so this is the only way one survives a wrong answer — and
/// taking it out of here is a line of code that says, in as many words, that the
/// person is being shown their recovery key a second time.
#[derive(Debug)]
pub struct Again {
    /// The key they are about to be shown again.
    key: RecoveryKey,
    /// Why the typing was not it.
    why: NotWhatWasShown,
}

impl Again {
    /// The key and the reason, from the one place that makes this.
    pub(crate) const fn of(key: RecoveryKey, why: NotWhatWasShown) -> Self {
        Self { key, why }
    }

    /// Why what they typed was not the key.
    #[must_use]
    pub const fn why(&self) -> NotWhatWasShown {
        self.why
    }

    /// The key, to put on the screen again.
    #[must_use]
    pub fn shown_again(self) -> RecoveryKey {
        self.key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **`WrittenDown` is the fact and carries nothing**, so nothing about a
    /// person's recovery key travels inside it.
    #[test]
    fn the_proof_carries_nothing() {
        assert_eq!(size_of::<WrittenDown>(), 0);
        let kept = WrittenDown::because_they_typed_it_back();
        assert_eq!(format!("{kept:?}"), "WrittenDown { kept: () }");
    }

    /// The two refusals say what is wrong and name no key.
    #[test]
    fn each_refusal_says_what_is_wrong_without_saying_the_key() {
        assert_eq!(
            NotWhatWasShown::Nothing.to_string(),
            "the recovery key was not typed back"
        );
        assert!(
            NotWhatWasShown::NotThisKey
                .to_string()
                .contains("was not the recovery key")
        );
    }
}
