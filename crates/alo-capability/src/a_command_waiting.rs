//! **A command waiting for the person's OK**, under the same numbers every other
//! change waits under.
//!
//! At *ask me each time* (ADR 0064's second level), a command alo asks to run
//! where no verb fits is not run: it is put to the person, as a change is, and
//! runs once they say yes ([ADR
//! 0149](../../../docs/decisions/0149-full-access-is-everything-the-person-can-do.md)).
//!
//! # Why it is not a [`crate::Proposal`]
//!
//! A proposal is a [`crate::Call`] — a verb on the closed list, validated, with
//! grants asked when it is made and again when it runs — and the only way from
//! one to running is [`crate::Authorised`]. A command is none of those, and law
//! 2 is the reason it must stay none of them: a command that became a `Call`
//! would be `exec` on the broker's list by another name. So it waits beside the
//! proposals rather than among them.
//!
//! # And why it shares their numbers
//!
//! What the person answers is a number and the sentence beside it — the
//! terminal's *Run this command? [y/N]*, the alo Bar's *Allow once* — and both
//! already answer by number on the person's door. A second numbering would let
//! one number mean two things. So [`crate::Approvals`] hands out one number
//! from one counter whichever list it goes on, a number is never reused, and
//! answering takes it off its list in the same act: **one approval, one
//! execution**, and a stale or repeated number finds nothing.

use std::time::{Duration, SystemTime};

use alo_strings::{Said, Strings};
use serde::Serialize;

use crate::a_command::ACommand;
use crate::approvals::ProposalId;
use crate::grant::Grantee;

/// A command put to the person, and the number it is answered by.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct WaitingCommand {
    /// What to answer it by — the same numbers [`crate::Waiting`] uses.
    pub id: ProposalId,
    /// The command, exactly as it would run.
    command: ACommand,
    /// Whose request it is.
    grantee: Grantee,
    /// When the question stops standing.
    lapses: SystemTime,
}

impl WaitingCommand {
    /// A command waiting under `id` until `lapses`. Made only by
    /// [`crate::Approvals::propose_a_command`].
    pub(crate) fn new(
        id: ProposalId,
        command: ACommand,
        grantee: Grantee,
        lapses: SystemTime,
    ) -> Self {
        Self {
            id,
            command,
            grantee,
            lapses,
        }
    }

    /// The command, exactly as it would run.
    #[must_use]
    pub fn command(&self) -> &ACommand {
        &self.command
    }

    /// Whose request it is.
    #[must_use]
    pub fn grantee(&self) -> &Grantee {
        &self.grantee
    }

    /// What the person is asked to approve.
    #[must_use]
    pub fn sentence(&self, strings: &Strings) -> Said {
        self.command.sentence(strings)
    }

    /// Whether the question still stands at this moment — on the same side of
    /// the boundary as [`crate::Proposal::is_waiting_at`].
    #[must_use]
    pub fn is_waiting_at(&self, now: SystemTime) -> bool {
        self.lapses > now
    }

    /// How long is left to answer, or `None` once it has lapsed.
    #[must_use]
    pub fn lapses_in(&self, now: SystemTime) -> Option<Duration> {
        self.lapses
            .duration_since(now)
            .ok()
            .filter(|left| !left.is_zero())
    }

    /// The approval, spent: what [`crate::Approvals::approve_a_command`] hands
    /// back.
    pub(crate) fn approved(self) -> ApprovedCommand {
        ApprovedCommand {
            id: self.id,
            command: self.command,
            grantee: self.grantee,
        }
    }
}

/// A command the person said yes to, which may now run once.
///
/// **Not `Clone`**, for [`crate::Approved`]'s reason: one approval is one
/// execution, and a value that could be copied could be run twice.
#[derive(Debug, PartialEq, Eq)]
pub struct ApprovedCommand {
    /// The number it was approved under.
    id: ProposalId,
    /// The command, exactly as it was approved.
    command: ACommand,
    /// Whose request it was.
    grantee: Grantee,
}

impl ApprovedCommand {
    /// The number it was approved under.
    #[must_use]
    pub fn id(&self) -> ProposalId {
        self.id
    }

    /// The command, exactly as it was approved.
    #[must_use]
    pub fn command(&self) -> &ACommand {
        &self.command
    }

    /// Whose request it was.
    #[must_use]
    pub fn grantee(&self) -> &Grantee {
        &self.grantee
    }
}
