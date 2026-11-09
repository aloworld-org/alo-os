//! **The grants a door is asked against, as the person's level is now**
//! ([ADR 0149](../../../docs/decisions/0149-full-access-is-everything-the-person-can-do.md)).
//!
//! The owner, 2026-11-08: *"We should give it always full access and see all
//! the folders if the users gives it full access in the settings"* — and of
//! what full access reaches, *"Yes, everything I can reach."*
//!
//! # Read at the moment, every time
//!
//! The level is read from the person's own file
//! (`/var/lib/alo/how-code-runs.toml`, through
//! [`crate::running_a_command::CommandsHere::the_level_now`], the one reader)
//! at each of the three moments something is decided: a read, a proposal, and
//! the approval that runs it. Only an explicit full access gives one: **nothing
//! chosen and a file that will not read are the sealed box**, never higher.
//!
//! At full access the door gets a **copy** of the list made for that moment
//! (`alo_capability::Grants::at_full_access_for`); at every other level it gets
//! the list itself, untouched. Nothing is kept, so there is no stale full access
//! to forget to clear: a person who switches it off between a proposal and
//! their OK is asked about against a list without it, and the change is refused
//! unless a grant covers it — as a revoked grant is.
//!
//! # And it is this machine's own agent's alone
//!
//! The copy names the turn's own agent. A turn from a paired machine (ADR 0003)
//! never comes through here: its verbs are asked against this machine's grants
//! for that machine's principal, in `crate::reaching`, and full access is a
//! choice this machine's person made about their own agent.
//!
//! # The account is the boundary
//!
//! What full access lets through is then carried out by this service, which
//! runs as the person: a file the person's account cannot open is a file the
//! verb cannot open, and the machine says so. The kernel boundary a turn is held
//! in (ADR 0013, `alo-bounding`) is built from the paths the call names, not
//! from the grants, so it still holds each execution to exactly what it touches
//! — full access widens which paths a call may name, never what one call reaches.

use std::borrow::Cow;

use alo_capability::{Grantee, Grants, HowCodeRuns};

use crate::running_a_command::CommandsHere;

/// The list a door is asked against at this moment: `grants` at full access
/// for `agent` when the person's level is full access, and `grants` itself at
/// every other level, when nothing was chosen, and when the level will not
/// read.
#[must_use]
pub fn the_grants_as_the_level_is_now<'g>(
    grants: &'g Grants,
    here: &CommandsHere,
    agent: &Grantee,
) -> Cow<'g, Grants> {
    match here.the_level_now() {
        HowCodeRuns::AtFullTrust => Cow::Owned(grants.at_full_access_for(agent)),
        HowCodeRuns::AskingEachTime | HowCodeRuns::InASealedBox => Cow::Borrowed(grants),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_grants_as_the_level_is_now_tests.rs"]
mod tests;
