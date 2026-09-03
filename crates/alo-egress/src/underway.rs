//! The only type that means *alo OS may do this* — and it is already being
//! shown.
//!
//! [`Departing`](crate::Departing)'s twin, for egress with no agent behind it,
//! and it is a twin rather than the same type on purpose. A
//! [`Departing`](crate::Departing)
//! answers *whose authority is this under* with a
//! [`Grantee`](alo_capability::Grantee), and `alo-record` writes that into
//! every entry it makes about something leaving. Widening it to answer
//! *nobody* would make every one of those answers an `Option`, in the crate
//! whose whole job is saying who did what — so the two authorities stay two
//! types, and the thing they share is the indicator, which is the thing a
//! person actually looks at.
//!
//! What is the same is the guarantee. There is no constructor here;
//! [`Indicator::beginning_on_its_own`](crate::Indicator::beginning_on_its_own)
//! is the only maker of one, and it has already put the errand on the indicator
//! by the time it hands one back. *Nothing alo OS does on its own escapes the
//! indicator* is therefore carried by a type rather than by whoever writes the
//! next errand remembering.
//!
//! It is honest about its edge, as [`crate::departing`] is: nothing here can
//! stop code that opens a socket without asking for an [`Underway`] at all.
//! What it stops is the ordinary mistake — doing something on the person's
//! machine and forgetting to show it.

use std::time::SystemTime;

use crate::destination::Destination;
use crate::errand::Errand;
use crate::indicator::ShownId;
use crate::itself::OnItsOwn;

/// An errand that may happen, and is on the indicator while it does.
///
/// Deliberately not `Clone`, as [`Departing`](crate::Departing) is not: an
/// errand that can be copied is one that can be ended twice, and the second
/// ending would take somebody else's line off the indicator. Not `Serialize`
/// either — it is an authority, not evidence.
#[derive(Debug)]
pub struct Underway {
    /// What alo OS is doing.
    on_its_own: OnItsOwn,
    /// The moment it began.
    at: SystemTime,
    /// Which line on the indicator this one is.
    shown: ShownId,
}

impl Underway {
    /// Made only by the indicator, at the moment it starts showing this.
    pub(crate) fn new(on_its_own: OnItsOwn, at: SystemTime, shown: ShownId) -> Self {
        Self {
            on_its_own,
            at,
            shown,
        }
    }

    /// What alo OS is doing.
    #[must_use]
    pub fn on_its_own(&self) -> &OnItsOwn {
        &self.on_its_own
    }

    /// Why it is doing it.
    #[must_use]
    pub fn errand(&self) -> Errand {
        self.on_its_own.errand()
    }

    /// Where it is reaching.
    #[must_use]
    pub fn destination(&self) -> &Destination {
        self.on_its_own.destination()
    }

    /// The moment it began, which is the moment it went on the indicator.
    #[must_use]
    pub fn at(&self) -> SystemTime {
        self.at
    }

    /// Which line on the indicator this is.
    #[must_use]
    pub fn shown(&self) -> ShownId {
        self.shown
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::indicator::Indicator;
    use crate::testing::in_english;
    use std::time::Duration;

    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(60 * 60 * 12)
    }

    fn fetching_a_model() -> OnItsOwn {
        OnItsOwn::for_(
            Errand::FetchingAModel,
            Destination::at("models.alo.example").unwrap(),
        )
    }

    /// An errand under way carries everything a person needs to be told about
    /// it, worked out once at the moment it went on the indicator.
    #[test]
    fn an_errand_under_way_says_what_where_and_when_without_anything_else_being_consulted() {
        let mut indicator = Indicator::default();
        let underway = indicator.beginning_on_its_own(fetching_a_model(), noon());
        assert_eq!(underway.errand(), Errand::FetchingAModel);
        assert_eq!(
            underway.destination(),
            &Destination::at("models.alo.example").unwrap()
        );
        assert_eq!(underway.at(), noon());
        assert_eq!(
            underway.on_its_own().said(&in_english()).text(),
            "alo OS is fetching a model from models.alo.example"
        );
    }

    /// **There is no way to make one except by showing it**, which is the whole
    /// guarantee: the only constructor is the indicator's, and it puts the
    /// errand on the list before it hands this back.
    #[test]
    fn nothing_alo_os_does_on_its_own_is_under_way_without_being_shown() {
        let mut indicator = Indicator::default();
        assert!(indicator.is_quiet());
        let underway = indicator.beginning_on_its_own(fetching_a_model(), noon());
        assert!(!indicator.is_quiet());
        assert_eq!(
            indicator.showing().first().map(crate::Shown::id),
            Some(underway.shown())
        );
    }
}
