//! The only way a line about a command alo is running comes off the indicator,
//! and the only evidence one was shown.
//!
//! [`WhileAloWorks`](crate::WhileAloWorks)'s twin for the fifth kind of line.
//! No constructor here:
//! [`Indicator::beginning_alos_command`](crate::Indicator::beginning_alos_command)
//! makes one, after it has put the line up.

use std::time::SystemTime;

use crate::alos_command::AlosCommand;
use crate::indicator::ShownId;

/// A command alo is running, on the indicator.
///
/// Not `Clone`, so one line is ended exactly once.
#[derive(Debug)]
pub struct WhileAlosCommandRuns {
    /// Which command, and which line.
    command: AlosCommand,
    /// The moment it went on the indicator.
    at: SystemTime,
    /// Which line it is.
    shown: ShownId,
}

impl WhileAlosCommandRuns {
    /// Made only by the indicator, at the moment it starts showing this.
    pub(crate) fn new(command: AlosCommand, at: SystemTime, shown: ShownId) -> Self {
        Self { command, at, shown }
    }

    /// Which command, and which line.
    #[must_use]
    pub fn command(&self) -> &AlosCommand {
        &self.command
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
