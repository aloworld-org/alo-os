//! Nothing that leaves reaches the rented tool unless it is on the indicator.
//!
//! Installing, looking for updates and updating are `alo_egress::Errand`s, and
//! the only maker of an `alo_egress::Underway` is the indicator, which has
//! already shown the line by the time it hands one back. So each act here takes
//! the `Underway` by reference — the line is still showing while the tool is
//! reached — and `held_to` refuses one for **another errand or another
//! place**. An installation carried out under a line saying the machine is
//! signing somebody in, or reaching a different host from the one named, would
//! be the indicator saying something untrue; this is the check that it cannot.
//!
//! That refusal is [`NotShown`], and it keeps its English: it cannot happen
//! because of anything on the machine, only because the code asking was
//! written wrong, and whoever reads it is fixing that code. `alo-keeping-up`'s
//! `NotACheck` is the same decision.

use alo_egress::{Destination, Errand, Underway};

use crate::refusing::NotDone;

/// An act was asked for under a line on the indicator that does not describe
/// it.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error(
    "{wanted:?} at {place:?} was asked for while the indicator showed {showing:?} at {at:?}, and \
     only a line describing the act itself may carry it out"
)]
pub struct NotShown {
    /// The errand the act is.
    wanted: Errand,
    /// Where the act reaches.
    place: Destination,
    /// What the indicator was actually showing.
    showing: Errand,
    /// Where the line on the indicator said it was reaching.
    at: Destination,
}

impl NotShown {
    /// What the indicator was actually showing.
    #[must_use]
    pub const fn showing(&self) -> Errand {
        self.showing
    }
}

/// Why an act that leaves the machine did not happen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stopped {
    /// Refused, for a reason a person reads.
    Refused(NotDone),
    /// Asked for under the wrong line — see the top of this file.
    NotShown(NotShown),
}

impl From<NotDone> for Stopped {
    fn from(refused: NotDone) -> Self {
        Self::Refused(refused)
    }
}

impl From<NotShown> for Stopped {
    fn from(not_shown: NotShown) -> Self {
        Self::NotShown(not_shown)
    }
}

/// That this line on the indicator is this errand, reaching this place.
///
/// # Errors
/// [`NotShown`] when it is any other errand, or the same errand anywhere else.
pub(crate) fn held_to(
    during: &Underway,
    wanted: Errand,
    place: &Destination,
) -> Result<(), NotShown> {
    if during.errand() == wanted && during.destination() == place {
        return Ok(());
    }
    Err(NotShown {
        wanted,
        place: place.clone(),
        showing: during.errand(),
        at: during.destination().clone(),
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_egress::{Indicator, OnItsOwn};
    use std::time::{Duration, SystemTime};

    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    fn flathub() -> Destination {
        Destination::at("dl.flathub.org").unwrap()
    }

    #[test]
    fn the_line_describing_the_act_carries_it() {
        let mut indicator = Indicator::default();
        let underway = indicator.beginning_on_its_own(
            OnItsOwn::for_(Errand::InstallingAnApplication, flathub()),
            noon(),
        );
        assert!(held_to(&underway, Errand::InstallingAnApplication, &flathub()).is_ok());
    }

    /// **Another errand's line, or the same errand's line naming another
    /// place, carries nothing.**
    #[test]
    fn another_errand_or_another_place_carries_nothing() {
        let mut indicator = Indicator::default();
        let signing_in =
            indicator.beginning_on_its_own(OnItsOwn::for_(Errand::SigningIn, flathub()), noon());
        let refused =
            held_to(&signing_in, Errand::InstallingAnApplication, &flathub()).unwrap_err();
        assert_eq!(refused.showing(), Errand::SigningIn);

        let elsewhere = indicator.beginning_on_its_own(
            OnItsOwn::for_(
                Errand::InstallingAnApplication,
                Destination::at("apps.acme.example").unwrap(),
            ),
            noon(),
        );
        assert!(held_to(&elsewhere, Errand::InstallingAnApplication, &flathub()).is_err());
        assert!(held_to(&elsewhere, Errand::UpdatingAnApplication, &flathub()).is_err());
    }
}
