//! Whether anybody has vouched for the build a place offers — decided when the
//! offer is heard, told to the person before they choose, and never mistaken
//! for the answer the machine's own signature policy gives.
//!
//! # Why an offer carries this at all
//!
//! Because *an update is ready* is a sentence a person acts on, and the act
//! stages the build under `--enforce-container-sigpolicy`
//! ([`crate::Staging`], [ADR 0036](../../../docs/decisions/0036-the-image-is-signed-by-a-key-a-person-holds.md)).
//! A build nothing vouches for is refused there, safely, with the machine
//! unchanged — and a person who was told *an update is ready* and then told
//! *it could not be prepared* has been told two true things and learned
//! nothing, at the one moment a machine sold on sovereignty cannot afford to be
//! vague.
//!
//! So the doubt travels **with the offer**, ahead of the choice, rather than
//! arriving as a refusal afterwards.
//!
//! # What this is not
//!
//! **It is not a signature check, and nothing here verifies anything.** What a
//! place holds beside a build is evidence that somebody signed it, not proof
//! that this machine will accept it: the key, the identity and the rule are the
//! machine's signature policy's, and that policy is asked by the base at the
//! moment a build is staged. This type is the difference between *there is
//! reason to believe this machine would take it* and *there is none* — never
//! between *valid* and *invalid*.
//!
//! Read the other way round it would be dangerous: a second, weaker verifier
//! that looked authoritative is exactly the thing somebody later trusts instead
//! of the policy. [`Vouching::ThePlaceVouchesForIt`] therefore promises nothing
//! and changes nothing about what is staged. The only behaviour it has is the
//! sentence a person reads before they choose.
//!
//! # Where it comes from
//!
//! `alo-looking`, out of the names the place already answered with during the
//! one check that was on the indicator — no second question, no extra
//! departure. Which names count is that crate's, because it is a fact about
//! places rather than about updates.

use serde::Serialize;

/// Whether the place offering a build has vouched for it.
///
/// **Not** whether this machine's signature policy accepts it; see this file's
/// header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Vouching {
    /// The place holds something vouching for this build that this machine's
    /// own signature policy would read.
    ThePlaceVouchesForIt,
    /// It holds nothing of the kind, so this machine has no reason to believe
    /// the build would be accepted.
    ///
    /// The reading everything falls back to wherever the answer is missing
    /// rather than negative, because the cost of the two mistakes is not equal:
    /// a person told an offer is unconfirmed when it is signed reads one
    /// cautious sentence, and a person told an offer is confirmed when nothing
    /// signed it is the person this whole file exists for.
    NobodyHasVouchedForIt,
}

impl Vouching {
    /// Both of them, in the order this file declares them.
    ///
    /// A third cannot arrive without this file changing, which is deliberate:
    /// *probably*, *partly* and *not checked yet* are each a way of telling a
    /// person something they cannot act on.
    pub const EVERY: [Self; 2] = [Self::ThePlaceVouchesForIt, Self::NobodyHasVouchedForIt];

    /// Whether anybody has vouched for it.
    #[must_use]
    pub fn is_vouched_for(self) -> bool {
        matches!(self, Self::ThePlaceVouchesForIt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One of the two, and the reading of each is the one its name says.
    #[test]
    fn a_build_is_vouched_for_or_it_is_not() {
        assert!(Vouching::ThePlaceVouchesForIt.is_vouched_for());
        assert!(!Vouching::NobodyHasVouchedForIt.is_vouched_for());
        assert_eq!(Vouching::EVERY.len(), 2);
    }

    /// **Written down, it says which**, so a check kept on a disk and read
    /// back by a surface carries the doubt rather than losing it.
    #[test]
    fn it_is_written_down_in_words_rather_than_as_a_flag_nobody_can_read() {
        assert_eq!(
            serde_json::to_string(&Vouching::NobodyHasVouchedForIt).unwrap_or_default(),
            "\"nobody-has-vouched-for-it\""
        );
        assert_eq!(
            serde_json::to_string(&Vouching::ThePlaceVouchesForIt).unwrap_or_default(),
            "\"the-place-vouches-for-it\""
        );
    }
}
