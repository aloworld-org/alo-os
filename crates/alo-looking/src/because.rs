//! Why a check is happening — and the whole list is two, because both of them
//! are somebody's act.
//!
//! `docs/features.md` promises **updates that never interrupt**, and every
//! operating system says that. What makes the promise keepable here is not a
//! gentler schedule: it is that **there is no schedule**. A check happens when
//! a person asks for one, or once when the machine starts, and there is no
//! third member of this enumeration and nothing in this crate that could reach
//! one.
//!
//! **Neither of these is made here.** A [`Because`] is handed to
//! [`crate::look`] by whatever is doing the asking — the shell when somebody
//! presses the button, and the unit that runs once at a start — so a check has
//! an author, and the author is named in what the check produced
//! ([`crate::Found::because`]) and in what is kept of it
//! ([`crate::Kept`]).
//!
//! # Why there is no *every so often*
//!
//! Because a machine that checks while somebody is working is a machine that
//! is doing something unasked on a network the person is paying attention to,
//! and law 1 makes every such errand visible — so a timer here would put a line
//! on somebody's indicator, repeatedly, for a question nobody asked. The
//! promise is *the choice is when, never whether to be told*
//! (`alo_keeping_up::WhenItApplies`), and *when* belongs to the two acts below.
//!
//! A machine that is never restarted and whose person never asks therefore
//! never checks. That is the correct answer rather than a gap: **a check is a
//! departure**, and a machine nobody is using has no business making one.

use serde::{Deserialize, Serialize};

/// Why this machine is asking whether there is an update.
///
/// Serialises so the kept answer says which of the two produced it; a surface
/// that shows *an update is ready* reads it back to know whether the person
/// asked for this answer or the machine found it on its way up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Because {
    /// Somebody asked this machine to look.
    ThePersonAsked,
    /// This machine started, and looked once on the way up.
    ThisMachineStarted,
}

impl Because {
    /// Both of them, in the order this file declares them.
    ///
    /// The acceptance walks this list, so a third added here arrives with a
    /// test rather than beside one.
    pub const EVERY: [Self; 2] = [Self::ThePersonAsked, Self::ThisMachineStarted];

    /// Whether a person is waiting for this answer.
    ///
    /// The one thing anything else needs to tell them apart, and it is asked
    /// rather than matched on so that a caller cannot accidentally treat a
    /// start as somebody standing in front of the machine.
    #[must_use]
    pub const fn somebody_is_waiting(self) -> bool {
        matches!(self, Self::ThePersonAsked)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **There are two, and both are somebody's act.** A member meaning *every
    /// so often* is what this test is written to fail on.
    #[test]
    fn there_are_two_occasions_and_neither_is_a_watchers() {
        assert_eq!(Because::EVERY.len(), 2);
        assert!(Because::ThePersonAsked.somebody_is_waiting());
        assert!(!Because::ThisMachineStarted.somebody_is_waiting());
    }

    /// It is written down as the words a reader recognises, because the kept
    /// answer carries it across a restart.
    #[test]
    fn each_occasion_is_written_down_as_itself() {
        for because in Because::EVERY {
            let written = serde_json::to_string(&because).unwrap();
            assert_eq!(serde_json::from_str::<Because>(&written).unwrap(), because);
        }
        assert_eq!(
            serde_json::to_string(&Because::ThePersonAsked).unwrap(),
            "\"the-person-asked\""
        );
    }
}
