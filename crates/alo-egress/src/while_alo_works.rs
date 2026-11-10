//! The only way a line about an application alo is working in comes off the
//! indicator, and the only evidence one was shown.
//!
//! [`WhileItRuns`](crate::WhileItRuns)'s twin for the fourth kind of line. No
//! constructor here: [`Indicator::beginning_alo_at_work`](crate::Indicator::beginning_alo_at_work)
//! makes one, after it has put the line up.

use std::time::SystemTime;

use crate::at_work::AloAtWork;
use crate::indicator::ShownId;

/// An application alo is working in, on the indicator.
///
/// Not `Clone`, so one line is ended exactly once.
#[derive(Debug)]
pub struct WhileAloWorks {
    /// Which application, and which line.
    at_work: AloAtWork,
    /// The moment it went on the indicator.
    at: SystemTime,
    /// Which line it is.
    shown: ShownId,
}

impl WhileAloWorks {
    /// Made only by the indicator, at the moment it starts showing this.
    pub(crate) fn new(at_work: AloAtWork, at: SystemTime, shown: ShownId) -> Self {
        Self { at_work, at, shown }
    }

    /// Which application, and which line.
    #[must_use]
    pub fn at_work(&self) -> &AloAtWork {
        &self.at_work
    }

    /// The moment it went on the indicator.
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
