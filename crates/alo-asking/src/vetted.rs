//! What testing a provider found, and the departure it was found with.
//!
//! [`crate::asked`]'s twin for the one request that is not a question. Law 1
//! has two halves — *visible at the moment it happens **and afterwards in a
//! record*** — and a test request is an egress like any other: somebody asked
//! for it, a key travelled with it, and a week later *what left this machine*
//! has to be able to name it. `alo_record::Entry::left` is made from an
//! `alo_egress::Departing` and from nothing else, so the departure comes back
//! here rather than being taken off the indicator inside the door, exactly as
//! it does for a question.
//!
//! ```text
//! let vetted = Vetting::provider(&provider, Some(&key))
//!     .under(&settings, &policy, &mut indicator, now)?;
//! if let Some(departing) = vetted.departing() {
//!     record.keep(Entry::left(departing));
//! }
//! match vetted.ended(&mut indicator) {
//!     Found::Answered(_) => choosing.adding(provider)?,
//!     // Shown to the person, who may say "save it anyway".
//!     Found::RefusedTheKey(_) | Found::CouldNotBeReached(_) => {}
//! }
//! ```
//!
//! # Why the departure is sometimes not there
//!
//! A provider on this machine — vLLM on `127.0.0.1`, a runtime on `localhost`
//! — is tested too, and testing it puts nothing on a wire. `alo_egress::Destination::of`
//! refuses to make a destination of this machine, for the reason
//! `alo_egress::departing` gives: an indicator carrying a line about a
//! departure that did not happen would teach people to ignore it. So a test
//! that left nothing has no departure to hand back, and [`Vetted::departing`]
//! says so with [`None`] rather than with a line nobody should record. The
//! same provider tested with the strictest policy there is still answers, which
//! is `alo_models::trying`'s own guarantee kept.

use alo_egress::{Departing, Indicator};

use crate::found::Found;

/// What a test found, and — where the request left this machine — the
/// departure that is still on the indicator and still to be written down.
///
/// Not `Clone`, because a `Departing` is not: a departure that could be copied
/// could be ended twice, and the second ending would take somebody else's line
/// off the indicator.
#[derive(Debug)]
pub struct Vetted {
    /// What left, still on the indicator — or nothing, for a provider on this
    /// machine, where nothing left.
    departing: Option<Departing>,
    /// What the request came back with.
    found: Found,
}

impl Vetted {
    /// Made by [`crate::Vetting::under`] and by nothing else.
    pub(crate) fn new(departing: Option<Departing>, found: Found) -> Self {
        Self { departing, found }
    }

    /// What the test found, before the line comes off the indicator.
    ///
    /// Borrowed rather than taken, so that keeping the record and reading the
    /// outcome are not two orders a caller has to get right.
    #[must_use]
    pub fn found(&self) -> &Found {
        &self.found
    }

    /// What left this machine — for `alo_record::Entry::left`, which is the
    /// only way what left is written down.
    ///
    /// [`None`] for a provider on this machine, where nothing left and there
    /// is no line to record.
    #[must_use]
    pub fn departing(&self) -> Option<&Departing> {
        self.departing.as_ref()
    }

    /// Whether the request left this machine at all.
    #[must_use]
    pub fn left(&self) -> bool {
        self.departing.is_some()
    }

    /// Take the line off the indicator, if there is one, and keep what was
    /// found.
    ///
    /// Consumes the departure, which is `alo_egress::Indicator::ended`'s rule:
    /// one request can never take two lines off the indicator.
    #[must_use]
    pub fn ended(self, indicator: &mut Indicator) -> Found {
        if let Some(departing) = self.departing {
            indicator.ended(departing);
        }
        self.found
    }
}
