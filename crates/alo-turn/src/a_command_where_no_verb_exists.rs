//! **A command, where no verb fits: run, put to the person, or refused — and
//! written down every time**
//! ([ADR 0149](../../../docs/decisions/0149-full-access-is-everything-the-person-can-do.md)).
//!
//! The owner, 2026-11-08: *where no verb exists, alo does it by running a
//! command at full access (without asking) and at* ask me each time *(after the
//! OK). "There is no verb for that" is never an answer a person sees at either
//! level.*
//!
//! # What this turn decides, and what it does not
//!
//! **It does not read the level.** Which of the three happens is the person's
//! choice in their own file, read by the service at the moment a command is
//! asked for; this crate is handed the answer as which door is called. What it
//! does decide is what every door here has to be true of:
//!
//! - **every run and every refusal is written down** before it is answered —
//!   `alo_record::Entry::ran_a_command` and `Entry::a_command_was_not_run`,
//!   through the one road to the record every other door uses;
//! - **a command at *ask me each time* is a change waiting**, under the same
//!   numbers as every other change (`alo_capability::a_command_waiting`), and
//!   one approval is one run;
//! - **what ran carries the place kept before it**, when the service kept one,
//!   so Put back is asked from the run itself.
//!
//! # And it starts nothing
//!
//! The command is run by whatever the caller hands in — on a real machine
//! `alo_running::run_as_the_person`, ADR 0117's one crate that starts a
//! program. A turn that started programs itself would be a second such place.

use std::time::{Duration, SystemTime};

use alo_capability::{
    ACommand, ItDidNotStart, ProposalId, THE_SHELL, WaitingCommand, WhatACommandDid,
};
use alo_record::Entry;
use alo_strings::{Filling, Said};

use crate::became::Became;
use crate::refusing::NotDone;
use crate::turning::Turning;

/// What runs a command, handed in by the caller: on a real machine
/// `alo_running::run_as_the_person`.
pub type Running<'r> = dyn FnMut(&ACommand) -> Result<WhatACommandDid, ItDidNotStart> + 'r;

impl Turning<'_, '_> {
    /// **Run a command now** — what full access means where no verb fits.
    ///
    /// Written down before it is answered, with the place kept before it when
    /// there is one. Nothing is asked of the grants: the person's choice of
    /// full access is what lets it run, and the account's own permissions are
    /// its boundary (ADR 0149, decision 1).
    ///
    /// # Errors
    /// [`NotDone::ACommandDidNotStart`] when the shell would not start, written
    /// down as a command that was not run; [`NotDone::NotRecorded`] when what
    /// happened could not be written down; [`NotDone::TurnClosed`] if something
    /// earlier could not be.
    pub fn running_a_command(
        &mut self,
        command: &ACommand,
        run: &mut Running<'_>,
        now: SystemTime,
    ) -> Result<WhatACommandDid, NotDone> {
        self.still_open()?;
        self.ran(command, run, now)
    }

    /// **Put a command to the person** — what *ask me each time* means where
    /// no verb fits. Nothing runs, and nothing is written down: a question is
    /// not a thing that happened, and its answer is.
    ///
    /// # Errors
    /// [`NotDone::NeverAsked`] for a question that would stand for no time, and
    /// the refusal is written down; [`NotDone::TurnClosed`] as everywhere.
    pub fn proposing_a_command(
        &mut self,
        command: ACommand,
        standing: Duration,
        now: SystemTime,
    ) -> Result<ProposalId, NotDone> {
        self.still_open()?;
        let grantee = self.grantee().clone();
        match self
            .approvals_mut()
            .propose_a_command(command, &grantee, now, standing)
        {
            Ok(id) => {
                self.remembering(id, Became::Waiting);
                Ok(id)
            }
            Err(why) => {
                let said = why.said(self.strings());
                self.a_command_was_not_run(&said, now)?;
                Err(NotDone::NeverAsked(why))
            }
        }
    }

    /// **The person said yes to a waiting command, so it runs — once.**
    ///
    /// The approval is spent by being answered: the same number a second time
    /// is [`NotDone::NotAnswered`], and so is a number a change waits under.
    ///
    /// # Errors
    /// [`NotDone::NotAnswered`] for a number no command waits under or one that
    /// stood too long, and otherwise as [`Turning::running_a_command`].
    pub fn approving_a_command(
        &mut self,
        id: ProposalId,
        run: &mut Running<'_>,
        now: SystemTime,
    ) -> Result<WhatACommandDid, NotDone> {
        self.still_open()?;
        let approved = self.approvals_mut().approve_a_command(id, now)?;
        self.remembering(id, Became::Approved);
        self.ran(approved.command(), run, now)
    }

    /// A command that was asked for and is not going to run, written down with
    /// the sentence the agent and the person are told.
    ///
    /// For the refusals this turn does not make itself: the level the person
    /// chose said no, or the line was not a command at all. The sentence
    /// arrives already rendered, so the record and the screen are one value.
    ///
    /// # Errors
    /// [`NotDone::NotRecorded`] when it could not be written down — and the
    /// caller answers with that, not with the refusal it was carrying.
    pub fn a_command_was_not_run(&mut self, why: &Said, now: SystemTime) -> Result<(), NotDone> {
        self.still_open()?;
        let entry = Entry::a_command_was_not_run(self.grantee(), THE_SHELL, why.text(), now);
        self.writing_down(entry)
    }

    /// A waiting command the person's level no longer lets run, taken off the
    /// list and written down with the sentence they are told — what happens
    /// when they say yes after moving to the sealed box.
    ///
    /// # Errors
    /// [`NotDone::NotAnswered`] when no command waits under the number, and
    /// [`NotDone::NotRecorded`] when it could not be written down.
    pub fn a_command_taken_back(
        &mut self,
        id: ProposalId,
        why: &Said,
        now: SystemTime,
    ) -> Result<(), NotDone> {
        self.still_open()?;
        if self.approvals_mut().decline_a_command(id).is_none() {
            return Err(NotDone::NotAnswered(
                alo_capability::AnswerError::NothingWaiting {
                    number: id.as_u64(),
                },
            ));
        }
        self.remembering(id, Became::Declined);
        self.a_command_was_not_run(why, now)
    }

    /// The commands this turn is waiting for the person to answer.
    pub fn commands_waiting_at(&self, now: SystemTime) -> impl Iterator<Item = &WaitingCommand> {
        self.approvals().commands_waiting_at(now)
    }

    /// One command this turn put to the person, by number, whether or not the
    /// question still stands — [`None`] once it has been answered.
    #[must_use]
    pub fn command_proposed(&self, id: ProposalId) -> Option<&WaitingCommand> {
        self.approvals().command_of(id)
    }

    /// The person said no to a waiting command: written down, and nothing ran.
    ///
    /// `pub(crate)`, for [`Turning::declining`], which answers a *no* whichever
    /// list the number is on. `None` when no command waits under it.
    pub(crate) fn declining_a_command(
        &mut self,
        id: ProposalId,
        now: SystemTime,
    ) -> Option<Result<(), NotDone>> {
        self.approvals_mut().decline_a_command(id)?;
        self.remembering(id, Became::Declined);
        let said = self.strings().say(
            &alo_capability::words::NOT_DONE_YOU_SAID_NO.key(),
            &Filling::nothing(),
        );
        let entry = Entry::a_command_was_not_run(self.grantee(), THE_SHELL, said.text(), now);
        Some(self.writing_down(entry))
    }

    /// Run it, and write down what it did — or that it did not start.
    fn ran(
        &mut self,
        command: &ACommand,
        run: &mut Running<'_>,
        now: SystemTime,
    ) -> Result<WhatACommandDid, NotDone> {
        match run(command) {
            Ok(did) => {
                // **No grants and no box**: `allowed` is empty because nothing
                // was granted, and the kernel was not holding it because at
                // this level nothing alo runs is boxed (ADR 0149).
                let entry = Entry::ran_a_command(
                    self.grantee(),
                    THE_SHELL,
                    &command.arguments(),
                    &[],
                    did.returned(),
                    false,
                    now,
                );
                let entry = match self.kept_as() {
                    Some(named) => entry.kept_as(named),
                    None => entry,
                };
                self.writing_down(entry)?;
                Ok(did)
            }
            Err(not_started) => {
                let said = self.strings().say(
                    &alo_capability::words::NOTHING_RAN_WOULD_NOT_START.key(),
                    &Filling::nothing(),
                );
                let entry =
                    Entry::a_command_was_not_run(self.grantee(), THE_SHELL, said.text(), now);
                self.writing_down(entry)?;
                Err(NotDone::ACommandDidNotStart(not_started))
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "a_command_where_no_verb_exists_tests.rs"]
mod tests;
