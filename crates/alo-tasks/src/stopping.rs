//! What **Stop** means, and why it is three states rather than two.
//!
//! The owner's words, 2026-10-12: *Stop immediately requests cancellation and
//! reports when work has stopped, including any changes already made.*
//!
//! Three sentences, and each one is a separate promise:
//!
//! - **requests** cancellation — not *performs* it. A person presses Stop and
//!   the asking is instant; the stopping is not, because something is running.
//! - **reports when work has stopped** — so there is a moment a person can
//!   trust, distinct from the moment they pressed.
//! - **including any changes already made** — so stopping never implies
//!   nothing happened.
//!
//! # Why a `bool` would be a lie
//!
//! *Stopped: yes or no* collapses the two moments into one, and the only way
//! to implement it is to answer at the press — which reports success while the
//! work is still running. **That is worse than having no Stop**, because it
//! converts *I do not know* into *it is finished*, and a person acts on the
//! second.
//!
//! So [`Stopping`] has a middle state with no answer in it, and the type cannot
//! be asked *did it stop* without being told *not yet*.

use crate::changed::WhatChanged;

/// Where a task is between a person pressing Stop and the work ending.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stopping {
    /// Nobody has asked. The task is running or finished on its own.
    NobodyAsked,
    /// A person pressed Stop and the work has not ended yet.
    ///
    /// **The state a `bool` cannot hold.** What a person is shown here is
    /// *stopping*, never *stopped* — and what they must not be shown is
    /// nothing, which is how a window that only changes on completion behaves
    /// when the work takes a minute.
    Asked,
    /// The work has ended, and this is what it changed before it did.
    ///
    /// **The changes are carried rather than fetched.** A reader that had to
    /// ask somewhere else what happened is a reader that can be built without
    /// asking, and then a person is told *stopped* with no account of what was
    /// done to their machine.
    Stopped {
        /// What the task changed before it ended. Possibly nothing, which is a
        /// real answer and is said rather than left blank.
        changed: WhatChanged,
    },
}

impl Stopping {
    /// A task nobody has asked to stop.
    #[must_use]
    pub const fn nobody_asked() -> Self {
        Self::NobodyAsked
    }

    /// A person pressed Stop.
    ///
    /// **Pressing it twice is pressing it once.** A second press while the
    /// first is being honoured changes nothing, and must not look like a
    /// stronger request: there is no harder stop to escalate to, and offering
    /// one would be a control that lies about what it does.
    ///
    /// Pressing Stop on a task that has already stopped changes nothing
    /// either — the account of what changed is not discarded by a late press.
    #[must_use]
    pub fn asked(self) -> Self {
        match self {
            Self::NobodyAsked | Self::Asked => Self::Asked,
            already @ Self::Stopped { .. } => already,
        }
    }

    /// The work ended, having changed this.
    #[must_use]
    pub const fn stopped(changed: WhatChanged) -> Self {
        Self::Stopped { changed }
    }

    /// Whether the work has actually ended.
    ///
    /// **False while a person is waiting**, which is the whole of this type.
    #[must_use]
    pub const fn has_stopped(&self) -> bool {
        matches!(self, Self::Stopped { .. })
    }

    /// Whether a person has asked and is still waiting.
    #[must_use]
    pub const fn is_waiting(&self) -> bool {
        matches!(self, Self::Asked)
    }

    /// What was changed before the work ended, once it has.
    ///
    /// [`None`] while it has not. A caller that answered *nothing changed*
    /// here would be answering a question nobody can answer yet, and the one
    /// moment that matters is the one where the work is still going.
    #[must_use]
    pub const fn what_changed(&self) -> Option<&WhatChanged> {
        match self {
            Self::NobodyAsked | Self::Asked => None,
            Self::Stopped { changed } => Some(changed),
        }
    }
}

#[cfg(test)]
#[path = "stopping_tests.rs"]
mod tests;
