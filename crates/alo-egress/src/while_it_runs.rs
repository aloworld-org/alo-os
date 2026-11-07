//! The only way a line about an application during a person's command comes
//! off the indicator, and the only evidence one was shown.
//!
//! [`Underway`](crate::Underway)'s twin for the third kind of line. There is no
//! constructor here: [`Indicator::beginning_during_their_command`](crate::Indicator::beginning_during_their_command)
//! makes one, and it has already put the line on the indicator by the time it
//! hands it back. So a record entry that says *this application sent this much
//! during your command* can be made only from a line somebody could see.

use std::time::SystemTime;

use crate::during::DuringTheirCommand;
use crate::indicator::ShownId;

/// An application on the indicator, sending during a person's command.
///
/// Not `Clone`, so one line is ended exactly once, and not `Serialize`: it is
/// a handle on a line, not evidence of anything.
#[derive(Debug)]
pub struct WhileItRuns {
    /// Which application, and what it is called.
    during: DuringTheirCommand,
    /// The moment it went on the indicator.
    at: SystemTime,
    /// Which line it is.
    shown: ShownId,
}

impl WhileItRuns {
    /// Made only by the indicator, at the moment it starts showing this.
    pub(crate) fn new(during: DuringTheirCommand, at: SystemTime, shown: ShownId) -> Self {
        Self { during, at, shown }
    }

    /// Which application, and what it is called.
    #[must_use]
    pub fn during(&self) -> &DuringTheirCommand {
        &self.during
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
