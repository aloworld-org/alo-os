//! Which genuine tokens have been used, so one approval is one execution.
//!
//! ADR 0001 §5 and the gate's *one approval causes exactly one execution*. The
//! turn's own list already refuses an approval answered twice; this is the
//! same promise held on the other side of the privilege boundary, where a
//! request that crossed the door twice would otherwise be carried out twice.
//!
//! # Bounded by the lifetime, not by a count
//!
//! A token is remembered for [`crate::LIFETIME`] after it was issued and then
//! forgotten, because after that it is refused as lapsed whether or not it is
//! remembered — so forgetting it changes no answer. Only a token the key
//! proved genuine is ever remembered, so nobody without the key can fill this.
//!
//! A broker that restarts forgets everything here, and that is safe for the
//! reason it is safe to forget a lapsed token: the process that verifies holds
//! a key made when it started, and nothing issued under the previous one is
//! genuine any longer.

use std::collections::BTreeMap;
use std::time::SystemTime;

use crate::approving::{LIFETIME, Token, seconds};

/// How far ahead of the broker's clock a token may say it was issued.
///
/// The turn and the broker read one machine's clock, so this is the width of a
/// second boundary between two reads of it and nothing more.
const AHEAD: u64 = 2;

/// Why a genuine token was not accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotSpendable {
    /// It has already been used.
    AlreadySpent,
    /// It is older than [`crate::LIFETIME`], or claims a moment that has not
    /// happened yet.
    Lapsed,
}

/// The genuine tokens used within the last [`crate::LIFETIME`].
#[derive(Debug, Default)]
pub struct Spent {
    /// Each proof, and the moment its token was issued.
    proofs: BTreeMap<[u8; 32], u64>,
}

impl Spent {
    /// Spend a token the key has already proven genuine, or say why not.
    ///
    /// A token refused as lapsed is not remembered, and never needs to be.
    ///
    /// # Errors
    /// [`NotSpendable`].
    pub fn spend(&mut self, token: &Token, now: SystemTime) -> Result<(), NotSpendable> {
        let now = seconds(now);
        let lifetime = LIFETIME.as_secs();
        self.proofs
            .retain(|_, issued| now.saturating_sub(*issued) <= lifetime);

        if token.issued() > now.saturating_add(AHEAD)
            || now.saturating_sub(token.issued()) > lifetime
        {
            return Err(NotSpendable::Lapsed);
        }
        if self.proofs.contains_key(token.proof()) {
            return Err(NotSpendable::AlreadySpent);
        }
        self.proofs.insert(*token.proof(), token.issued());
        Ok(())
    }

    /// How many tokens are remembered.
    #[cfg(test)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.proofs.len()
    }

    /// Whether none is.
    #[cfg(test)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.proofs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approving::ApprovingKey;
    use crate::arguments::Switch;
    use crate::verbs::SystemVerb;
    use std::time::Duration;

    /// Noon on a day in 2025.
    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    /// A token issued at `at`.
    fn issued_at(at: SystemTime, approval: u64) -> Token {
        ApprovingKey::of(&[1; 32]).issue(&SystemVerb::SetRadio(Switch::On), approval, at)
    }

    /// **Once.** The second use of the same token is refused.
    #[test]
    fn a_token_is_spent_once() {
        let mut spent = Spent::default();
        let token = issued_at(noon(), 1);
        assert_eq!(spent.spend(&token, noon()), Ok(()));
        assert_eq!(spent.spend(&token, noon()), Err(NotSpendable::AlreadySpent));
        assert_eq!(
            spent.spend(&token, noon() + Duration::from_secs(30)),
            Err(NotSpendable::AlreadySpent)
        );
    }

    /// **Not for long, and not from the future.**
    #[test]
    fn a_token_too_old_or_from_the_future_has_lapsed() {
        let mut spent = Spent::default();
        let old = issued_at(noon(), 1);
        let later = noon() + LIFETIME + Duration::from_secs(1);
        assert_eq!(spent.spend(&old, later), Err(NotSpendable::Lapsed));

        let tomorrow = issued_at(noon() + Duration::from_secs(86_400), 2);
        assert_eq!(spent.spend(&tomorrow, noon()), Err(NotSpendable::Lapsed));
        assert!(spent.is_empty());
    }

    /// Exactly at the lifetime it is still good, and a second boundary is
    /// tolerated.
    #[test]
    fn the_edges_are_generous_by_exactly_what_they_say() {
        let mut spent = Spent::default();
        assert_eq!(
            spent.spend(&issued_at(noon(), 1), noon() + LIFETIME),
            Ok(())
        );
        let ahead = issued_at(noon() + Duration::from_secs(AHEAD), 2);
        assert_eq!(spent.spend(&ahead, noon()), Ok(()));
    }

    /// **What is remembered is bounded by the lifetime**: a minute later, a
    /// new token finds the old ones forgotten.
    #[test]
    fn what_is_remembered_is_forgotten_once_it_could_only_lapse() {
        let mut spent = Spent::default();
        for approval in 0..10 {
            assert_eq!(spent.spend(&issued_at(noon(), approval), noon()), Ok(()));
        }
        assert_eq!(spent.len(), 10);
        let later = noon() + LIFETIME + Duration::from_secs(5);
        assert_eq!(spent.spend(&issued_at(later, 11), later), Ok(()));
        assert_eq!(spent.len(), 1);
    }
}
