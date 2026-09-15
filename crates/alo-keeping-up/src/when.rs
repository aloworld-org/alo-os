//! When an update applies — the one thing about an update a person chooses.
//!
//! **The choice is *when*, never *whether to be told*.** Two things are left
//! out of this type on purpose, one on each side:
//!
//! - **No *never*.** A setting that turns updates off entirely is a checkbox
//!   that leaves a machine unpatched, and on a fleet that is the worst
//!   liability there is. A person who does not want an update now leaves it
//!   waiting for the next restart, which is a restart they make.
//! - **No *by itself*.** Nothing here applies an update unasked. There is no
//!   member meaning *automatically*, *overnight* or *immediately*, because each
//!   of those is a moment somebody other than the person picked.
//!
//! Read back as well as written down, because a person's choice is kept; and a
//! kept choice naming a member that does not exist is refused rather than read
//! as the nearest one.

use serde::{Deserialize, Serialize};

use crate::never::Cause;
use crate::words::{self, Word};

/// When a ready update applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WhenItApplies {
    /// The next time the person restarts the machine themselves. What happens
    /// when they have chosen nothing.
    #[default]
    AtTheNextRestart,
    /// Now, because the person asked to restart and apply it.
    NowBecauseThePersonAsked,
}

impl WhenItApplies {
    /// Every choice there is.
    pub const EVERY: [Self; 2] = [Self::AtTheNextRestart, Self::NowBecauseThePersonAsked];

    /// The words a person chooses this by.
    #[must_use]
    pub fn word(self) -> Word {
        match self {
            Self::AtTheNextRestart => words::APPLY_AT_THE_NEXT_RESTART,
            Self::NowBecauseThePersonAsked => words::RESTART_AND_APPLY_NOW,
        }
    }

    /// Who causes the restart that applies the update — the person, whichever
    /// they chose.
    ///
    /// This is what is handed to [`crate::THE_RULE`] when the restart happens,
    /// and it cannot be [`Cause::AnUpdate`]: there is no choice here under
    /// which an update restarts the machine.
    #[must_use]
    pub fn cause_of_the_restart(self) -> Cause {
        match self {
            Self::AtTheNextRestart | Self::NowBecauseThePersonAsked => Cause::ThePerson,
        }
    }
}
