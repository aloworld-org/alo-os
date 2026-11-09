//! **Full access: everything the person's account can reach**
//! ([ADR 0149](../../../docs/decisions/0149-full-access-is-everything-the-person-can-do.md)).
//!
//! The decision, as the owner accepted it: *"**Full access** (ADR 0064's level
//! 3, renamed for the person) reaches **everything the person's account can
//! reach**. There are no folder grants at this level; the account's own
//! permissions are the boundary."* Asked whether that meant every file and
//! folder the person can reach, the owner answered *"Yes, everything I can
//! reach."*
//!
//! # Where it is decided, and why there
//!
//! Every question about whether an agent may touch something already comes to
//! [`Grants`]: a read, a proposal, the redemption of an approval, where a path
//! really leads, the name a rename would create, the application an act names.
//! So full access is answered **on the list itself**, by the questions in this
//! file, and every one of those doors says yes to the same things without any
//! of them having to remember to ask a second question.
//!
//! **Grants are asked first, always.** A grant that covers the ask is named,
//! exactly as before, so a record says *against which grant* wherever there
//! was one; full access answers only where no grant does, and then says so —
//! [`Permitted::ByFullAccess`] — rather than inventing a handle. Full access
//! adds; it never removes.
//!
//! # It is for one moment, and it is never kept
//!
//! [`Grants::at_full_access_for`] makes a **copy** of the list that says yes to
//! one agent. The list a service keeps never carries it, and the grants file
//! cannot (`#[serde(skip)]`): the person's level lives in its own file, and the
//! service reads it at the moment it matters — when a read is asked, when a
//! change is proposed, and again when it is approved. That is what makes
//! *switching full access off takes effect immediately* true in the same way a
//! revoked grant does: an approval redeemed after the person moved to *ask me
//! each time* is asked against a list with no full access on it, and refused
//! unless a grant covers it.
//!
//! # What it does not reach
//!
//! - **An application's grantee.** Full access is the person's choice about
//!   their agent; an application asks through [`Grants::allowing`], which this
//!   file does not touch.
//! - **Another agent.** The copy names one agent — the one this machine's
//!   person uses. A paired machine's principal (ADR 0003) is a different
//!   grantee, and acts only under grants made here.
//! - **A machine whose person declined the agent** (ADR 0009). That answer is
//!   [`crate::Agent::permitting`]'s, and it refuses before any list is read.
//! - **The camera, the microphone and the screen** ([`Ask::Facility`]). ADR
//!   0149 names files and applications; a facility stays a grant of its own.
//! - **The person's terminal**, unless the person allowed it (ADR 0111): that
//!   is a separate choice, and full access does not answer it for them.

use std::path::Path;
use std::time::SystemTime;

use crate::a_terminal_for_the_agent::ATerminalForTheAgent;
use crate::grant::Grantee;
use crate::grants::{GrantId, Grants};
use crate::persons_own::is_a_persons_own;
use crate::reach::Ask;
use crate::refusing::NotGranted;

/// What let one ask through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permitted {
    /// A grant the person made, by the handle it is revoked by.
    ByAGrant(GrantId),
    /// No grant covers it, and the person chose full access (ADR 0149).
    ByFullAccess,
}

impl Permitted {
    /// The grant, when it was one.
    #[must_use]
    pub const fn grant(self) -> Option<GrantId> {
        match self {
            Self::ByAGrant(id) => Some(id),
            Self::ByFullAccess => None,
        }
    }
}

impl Grants {
    /// This list, at full access for `agent` — for one moment.
    ///
    /// A copy, so the list a service keeps never carries full access and a
    /// stale one cannot outlive the moment it was read at. An application's
    /// grantee is never given it: the copy is then the list unchanged.
    #[must_use]
    pub fn at_full_access_for(&self, agent: &Grantee) -> Self {
        let mut at = self.clone();
        at.full_access = (!agent.is_an_application()).then(|| agent.clone());
        at
    }

    /// The agent this list says yes to at full access, and `None` at every
    /// other level.
    #[must_use]
    pub fn full_access_for(&self) -> Option<&Grantee> {
        self.full_access.as_ref()
    }

    /// Whether this agent has full access on this list.
    #[must_use]
    pub fn has_full_access(&self, grantee: &Grantee) -> bool {
        !grantee.is_an_application() && self.full_access.as_ref() == Some(grantee)
    }

    /// What lets this agent touch this thing at this moment — a grant, or full
    /// access — or why nothing does.
    ///
    /// [`Grants::permitting`] first, so a covering grant is named; then full
    /// access, for a path or an application.
    ///
    /// # Errors
    /// [`NotGranted`], the grants' own refusal, where neither answers.
    pub fn permitted(
        &self,
        grantee: &Grantee,
        ask: &Ask,
        now: SystemTime,
    ) -> Result<Permitted, NotGranted> {
        self.permitted_when(grantee, ask, now, ATerminalForTheAgent::NotAtAll)
    }

    /// The same, told what the person has allowed their agent to do with a
    /// terminal — which full access does not decide for them.
    ///
    /// # Errors
    /// As [`Grants::permitted`].
    pub fn permitted_when(
        &self,
        grantee: &Grantee,
        ask: &Ask,
        now: SystemTime,
        terminal: ATerminalForTheAgent,
    ) -> Result<Permitted, NotGranted> {
        match self.permitting_when(grantee, ask, now, terminal) {
            Ok(id) => Ok(Permitted::ByAGrant(id)),
            Err(why) => {
                if self.has_full_access(grantee) && reached_at_full_access(ask, terminal) {
                    Ok(Permitted::ByFullAccess)
                } else {
                    Err(why)
                }
            }
        }
    }

    /// What lets this agent reach a path **where it really is** — a grant, or
    /// full access — or why nothing does.
    ///
    /// # Errors
    /// As [`Grants::permitting_where_it_really_is`].
    pub fn permitted_where_it_really_is(
        &self,
        grantee: &Grantee,
        real: &Path,
        now: SystemTime,
    ) -> Result<Permitted, NotGranted> {
        match self.permitting_where_it_really_is(grantee, real, now) {
            Ok(id) => Ok(Permitted::ByAGrant(id)),
            Err(_) if self.has_full_access(grantee) => Ok(Permitted::ByFullAccess),
            Err(why) => Err(why),
        }
    }
}

/// Whether full access answers this ask: a path, or an application other than
/// a terminal the person has not allowed.
fn reached_at_full_access(ask: &Ask, terminal: ATerminalForTheAgent) -> bool {
    match ask {
        Ask::Path(_) => true,
        Ask::Application(identifier) => !is_a_persons_own(identifier) || terminal.may_open_one(),
        Ask::Facility(_) => false,
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
#[path = "full_access_tests.rs"]
mod tests;
