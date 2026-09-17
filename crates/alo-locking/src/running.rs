//! What keeps running behind a locked screen: everything that was.
//!
//! A lock covers the screen. It is not a pause, a suspend or a sign-out, and
//! a person who locks their machine to fetch a coffee expects the download to
//! have finished, the build to have run and the change they approved to have
//! happened by the time they are back. So the decision is a closed list with
//! one answer, and the answer has no second variant for a later change to
//! reach for:
//!
//! ```compile_fail
//! let paused = alo_locking::WhileLocked::Paused;
//! ```
//!
//! What this crate does to running things is therefore nothing, and that is
//! held in the shape of [`crate::Seat::locked`] too: it takes the session and
//! the agent's overlay, and it is handed no turn, no application, no download
//! and no connection that it could stop.

/// Something that was running when the machine locked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Running {
    /// An application the person had open — its windows, its work, its sound.
    Applications,
    /// A turn the person had already set going, and a change they had already
    /// approved. A change still *waiting* waits: answering it is theirs, and
    /// the lock screen cannot (`crate::Seat::approve`).
    ApprovedTurns,
    /// A file on its way onto this machine.
    Downloads,
}

/// Every kind of thing that runs, in the order this file declares them.
pub const EVERYTHING_RUNNING: [Running; 3] = [
    Running::Applications,
    Running::ApprovedTurns,
    Running::Downloads,
];

/// What locking does to it.
///
/// One variant, on purpose — the reasoning is at the top of this file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhileLocked {
    /// It carries on exactly as it was.
    KeepsRunning,
}

impl Running {
    /// What becomes of it while the machine is locked.
    #[must_use]
    pub const fn while_locked(self) -> WhileLocked {
        match self {
            Self::Applications | Self::ApprovedTurns | Self::Downloads => WhileLocked::KeepsRunning,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A locked session keeps running** — applications, approved turns and
    /// downloads, every one of them.
    #[test]
    fn everything_running_keeps_running_while_locked() {
        for running in EVERYTHING_RUNNING {
            assert_eq!(
                running.while_locked(),
                WhileLocked::KeepsRunning,
                "{running:?}"
            );
        }
    }
}
