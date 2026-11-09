//! **A command asked for as administrator: run once the person gave their
//! password, or refused in words — and written down either way, with what
//! they did**
//! ([ADR 0150](../../../docs/decisions/0150-administrator-actions-ask-the-persons-password-each-time.md)).
//!
//! The owner, 2026-11-08: administrator actions *"Ask my password each
//! time"*. Whatever runs the command is handed in, as it is for a command run
//! as the person (`crate::a_command_where_no_verb_exists`); on a machine that
//! is `alo-agentd` asking `alo-running-as-administrator`, which asks polkit,
//! which asks the person. This file decides what is written down:
//!
//! - **a run** is `alo_record::Entry::ran_a_command`, stamped
//!   [`AsAnAdministrator::TheyGaveTheirPassword`];
//! - **a refusal** is `Entry::a_command_was_not_run` with the sentence the
//!   person is told, stamped with what they did — cancelled, not accepted,
//!   nothing could ask — so the record tells the reasons apart as the person
//!   was told them apart.
//!
//! And a refusal is never a fallback: nothing here runs the command as the
//! person instead.

use std::time::SystemTime;

use alo_capability::{ACommand, ProposalId, THE_SHELL, WhatACommandDid};
use alo_record::{AsAnAdministrator, Entry};
use alo_strings::Said;

use crate::became::Became;
use crate::refusing::NotDone;
use crate::turning::Turning;

/// What runs a command as administrator, handed in by the caller: the
/// command's run, or the sentence the person is told and what they did.
pub type RunningAsAdministrator<'r> =
    dyn FnMut(&ACommand) -> Result<WhatACommandDid, (Said, AsAnAdministrator)> + 'r;

impl Turning<'_, '_> {
    /// **Run a command as administrator now**, at full access: the person is
    /// asked for their password by whatever `run` asks, and what happened is
    /// written down before it is answered.
    ///
    /// # Errors
    /// [`NotDone::NotRecorded`] when it could not be written down, and
    /// [`NotDone::TurnClosed`] as everywhere. A refusal is not an error: it is
    /// the inner `Err`, the sentence the person and the agent are told.
    pub fn running_as_administrator(
        &mut self,
        command: &ACommand,
        run: &mut RunningAsAdministrator<'_>,
        now: SystemTime,
    ) -> Result<Result<WhatACommandDid, Said>, NotDone> {
        self.still_open()?;
        self.ran_as_administrator(command, run, now)
    }

    /// **The person said yes to a waiting command that needs administrator
    /// rights**, so their password is asked for and it runs — once.
    ///
    /// # Errors
    /// [`NotDone::NotAnswered`] for a number no command waits under or one that
    /// stood too long, and otherwise as [`Turning::running_as_administrator`].
    pub fn approving_a_command_as_administrator(
        &mut self,
        id: ProposalId,
        run: &mut RunningAsAdministrator<'_>,
        now: SystemTime,
    ) -> Result<Result<WhatACommandDid, Said>, NotDone> {
        self.still_open()?;
        let approved = self.approvals_mut().approve_a_command(id, now)?;
        self.remembering(id, Became::Approved);
        self.ran_as_administrator(approved.command(), run, now)
    }

    /// Run it as administrator, and write down what happened.
    fn ran_as_administrator(
        &mut self,
        command: &ACommand,
        run: &mut RunningAsAdministrator<'_>,
        now: SystemTime,
    ) -> Result<Result<WhatACommandDid, Said>, NotDone> {
        match run(command) {
            Ok(did) => {
                let entry = Entry::ran_a_command(
                    self.grantee(),
                    THE_SHELL,
                    &command.arguments(),
                    &[],
                    did.returned(),
                    false,
                    now,
                )
                .as_an_administrator(AsAnAdministrator::TheyGaveTheirPassword);
                let entry = match self.kept_as() {
                    Some(named) => entry.kept_as(named),
                    None => entry,
                };
                self.writing_down(entry)?;
                Ok(Ok(did))
            }
            Err((said, how)) => {
                let entry =
                    Entry::a_command_was_not_run(self.grantee(), THE_SHELL, said.text(), now)
                        .as_an_administrator(how);
                self.writing_down(entry)?;
                Ok(Err(said))
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "a_command_as_administrator_tests.rs"]
mod tests;
