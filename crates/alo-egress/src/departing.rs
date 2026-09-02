//! The only type that means *this may leave* — and it is already being shown.
//!
//! Law 1 has two halves, and the second one is the one systems lose: a policy
//! that permits an egress and an indicator that displays it are easy to write
//! separately, and easy to get out of step the first time somebody adds a code
//! path in a hurry. So they are not separate here. A [`Departing`] is what a
//! caller must hold before it opens a connection, it has no constructor of its
//! own, and the only thing that can make one is
//! [`Indicator::beginning`](crate::Indicator::beginning) — which has already put
//! it on the indicator by the time it hands one back.
//!
//! This is the same shape as [`alo_capability::Authorised`], for the same
//! reason. *No agent-caused egress escapes the indicator* is a guarantee
//! `CLAUDE.md` makes in public, and a guarantee carried by a type is one that
//! stays true when somebody who has not read this file adds the next verb.
//!
//! It is honest about its edge, as [`alo_capability::arg`] is: nothing here can
//! stop code that opens a socket without asking for a [`Departing`] at all.
//! What it stops is the ordinary mistake — permitting an egress and forgetting
//! to show it — because there is no way to get the permission without the
//! showing.

use std::time::SystemTime;

use alo_capability::Grantee;

use crate::destination::Destination;
use crate::indicator::ShownId;
use crate::leaving::{Leaving, Why};

/// An egress that may happen, and is on the indicator while it does.
///
/// Deliberately not `Clone`: a departure that can be copied is a departure that
/// can be ended twice, and the second ending would take somebody else's line
/// off the indicator. Deliberately not `Serialize` either — it is an authority,
/// not evidence, and evidence is `alo-record`'s.
#[derive(Debug)]
pub struct Departing {
    /// What is leaving.
    leaving: Leaving,
    /// The moment it began.
    at: SystemTime,
    /// Which line on the indicator is this one.
    shown: ShownId,
}

impl Departing {
    /// Made only by the indicator, at the moment it starts showing this.
    pub(crate) fn new(leaving: Leaving, at: SystemTime, shown: ShownId) -> Self {
        Self { leaving, at, shown }
    }

    /// What is leaving.
    #[must_use]
    pub fn leaving(&self) -> &Leaving {
        &self.leaving
    }

    /// Whose authority it is under.
    #[must_use]
    pub fn agent(&self) -> &Grantee {
        self.leaving.agent()
    }

    /// Where it is going.
    #[must_use]
    pub fn destination(&self) -> &Destination {
        self.leaving.destination()
    }

    /// Why it is leaving.
    #[must_use]
    pub fn why(&self) -> Why {
        self.leaving.why()
    }

    /// The moment it began — the moment the policy was asked, which is the
    /// moment it was allowed to happen.
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
    use crate::policy::EgressPolicy;
    use crate::testing::in_english;
    use std::time::Duration;

    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(60 * 60 * 12)
    }

    fn leaving() -> Leaving {
        Leaving::because(
            &Grantee::named("@files"),
            Why::Fetching,
            Destination::at("alo.example").unwrap(),
        )
    }

    /// A departure carries everything a person and a record need to be told
    /// about it, worked out once at the moment it was allowed rather than
    /// reconstructed afterwards.
    #[test]
    fn a_departure_says_who_where_why_and_when_without_anything_else_being_consulted() {
        let mut indicator = Indicator::default();
        let departing = indicator
            .beginning(&EgressPolicy::Anywhere, leaving(), noon())
            .unwrap();
        assert_eq!(departing.agent(), &Grantee::named("@files"));
        assert_eq!(departing.why(), Why::Fetching);
        assert_eq!(
            departing.destination(),
            &Destination::at("alo.example").unwrap()
        );
        assert_eq!(departing.at(), noon());
        assert_eq!(
            departing.leaving().said(&in_english()).text(),
            leaving().said(&in_english()).text()
        );
    }
}
