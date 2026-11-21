//! **Which status a task is in**: one of the six things a task window says it is
//! doing, named so that whatever draws one can match on it with no catch-all.
//!
//! The words are `crate::words`'; this is only *which* of them, answered by
//! [`crate::ATask::its_status`], so a new status is a compile error wherever a
//! status is drawn rather than a status drawn like another.

/// One of a task's statuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheStatus {
    /// *alo is working*.
    AloIsWorking,
    /// *Waiting for your permission*.
    WaitingForYou,
    /// *Complete*.
    Complete,
    /// *Stopping…*.
    Stopping,
    /// *Stopped*.
    Stopped,
    /// *Stop requested · not confirmed*.
    StopNotConfirmed,
}

impl TheStatus {
    /// Every status, in the order a task can pass through them.
    pub const EVERY: [Self; 6] = [
        Self::AloIsWorking,
        Self::WaitingForYou,
        Self::Complete,
        Self::Stopping,
        Self::Stopped,
        Self::StopNotConfirmed,
    ];

    /// The words this status is said in.
    #[must_use]
    pub const fn word(self) -> alo_strings::Word {
        match self {
            Self::AloIsWorking => crate::words::ALO_IS_WORKING,
            Self::WaitingForYou => crate::words::WAITING_FOR_YOUR_PERMISSION,
            Self::Complete => crate::words::COMPLETE,
            Self::Stopping => crate::words::STOPPING,
            Self::Stopped => crate::words::STOPPED,
            Self::StopNotConfirmed => crate::words::STOP_NOT_CONFIRMED,
        }
    }

    /// Which status these words are, if they are one.
    #[must_use]
    pub fn of(word: alo_strings::Word) -> Option<Self> {
        Self::EVERY.into_iter().find(|status| status.word() == word)
    }
}

#[cfg(test)]
#[path = "status_tests.rs"]
mod tests;
