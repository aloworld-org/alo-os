//! Asking whether there is an update, as an errand everybody can see.
//!
//! A check for an update is the classic place telemetry rides along: the
//! machine reaches out on its own, nobody asked, and nobody sees. On this
//! machine it is [`Errand::CheckingForAnUpdate`] — one of the three reasons alo
//! OS reaches the network by itself, on the same indicator as everything an
//! agent causes.
//!
//! **The answer is held to the errand, not merely described beside it.** An
//! [`Offered`] can only be made from an [`Underway`], and the only maker of an
//! [`Underway`] is `alo_egress::Indicator::beginning_on_its_own`, which has
//! already put the errand on the indicator by the time it hands one back. So
//! code that fetched an answer without being shown has nothing to turn that
//! answer into: there is no other constructor, and no `Deserialize` that would
//! be one by the back door.
//!
//! **And held to *this* errand.** An [`Underway`] for signing in or fetching a
//! model is proof that *something* was shown, not that a check was, and a line
//! on the indicator that said *fetching a model* while the machine asked about
//! updates would be the indicator telling a lie. [`Offered::heard`] refuses it.

use alo_egress::{Destination, Errand, OnItsOwn, Underway};

use crate::digest::Digest;

/// The errand of checking for an update at this place.
///
/// What a caller hands `alo_egress::Indicator::beginning_on_its_own` before it
/// asks anything; the [`Underway`] that comes back is what [`Offered::heard`]
/// takes. Which place is not decided here — it is the installer plan's
/// registry, or an organisation's own mirror, and neither is a permission.
#[must_use]
pub fn a_check_at(from: Destination) -> OnItsOwn {
    OnItsOwn::for_(Errand::CheckingForAnUpdate, from)
}

/// The build the place updates come from offers.
///
/// Deliberately not `Deserialize`: one read back off a disk would be an offer
/// nobody was shown being asked for.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Offered {
    /// The build offered.
    digest: Digest,
}

/// An answer about updates, offered as heard during an errand that was not a
/// check for one.
///
/// Not a sentence a person reads, and deliberately in English: it cannot
/// happen because of anything on the machine, only because the code asking was
/// written wrong, and whoever reads it is fixing that code.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error(
    "an answer about updates was offered as heard while the indicator showed {during:?}, and \
     only a check for an update that is on the indicator may produce one"
)]
pub struct NotACheck {
    /// What the indicator was actually showing.
    during: Errand,
}

impl NotACheck {
    /// What the indicator was actually showing.
    #[must_use]
    pub fn during(&self) -> Errand {
        self.during
    }
}

impl Offered {
    /// The build offered, heard while a check for an update is on the
    /// indicator.
    ///
    /// Takes the [`Underway`] by reference, so the answer is heard while the
    /// line is still showing; ending the errand consumes it, and after that
    /// there is nothing to hear an answer with.
    ///
    /// # Errors
    /// [`NotACheck`] when the errand under way is not a check for an update.
    pub fn heard(during: &Underway, digest: Digest) -> Result<Self, NotACheck> {
        match during.errand() {
            Errand::CheckingForAnUpdate => Ok(Self { digest }),
            other @ (Errand::SigningIn | Errand::FetchingAModel) => {
                Err(NotACheck { during: other })
            }
        }
    }

    /// The build offered.
    #[must_use]
    pub fn digest(&self) -> &Digest {
        &self.digest
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The errand is a check for an update, reaching the place it was given.
    ///
    /// What it looks like on the indicator, and that an answer is refused from
    /// any other errand, are in `tests/what_an_update_may_never_do.rs`: an
    /// indicator needs a moment, and this crate's own source never names the
    /// clock, not even in a test.
    #[test]
    fn a_check_is_the_update_errand_at_the_place_it_was_given() {
        let updates = Destination::at("updates.alo.example").unwrap();
        let check = a_check_at(updates.clone());
        assert_eq!(check.errand(), Errand::CheckingForAnUpdate);
        assert_eq!(check.destination(), &updates);
    }
}
