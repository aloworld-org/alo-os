//! **What a person sees while a command alo runs is running**: the count
//! begun, the line up while it sends, and both gone when it ends
//! ([ADR 0148](../../../docs/decisions/0148-the-kernel-counts-what-an-application-sends-during-a-persons-command.md),
//! as amended 2026-11-13).
//!
//! Law 1 is *every network egress an agent causes is visible at the moment it
//! happens and afterwards in a record*. A command alo runs where no verb fits
//! (ADR 0149) sends from its own process, so until 2026-11-13 nothing it sent
//! reached the indicator at all. This is what reaches it now:
//!
//! | the machine | while it runs | when it ends |
//! |---|---|---|
//! | counts it, and it sends | *curl, a command alo is running, is sending* — up from the first look that finds a byte | the line comes off; the record keeps the bytes |
//! | counts it, and it sends nothing | **nothing** — the line stays dark | the record keeps `0` |
//! | cannot count it | *alo is running curl, and this machine cannot count what it sends* — up for the whole run, before it starts | the line comes off; the record keeps no number, because none was counted |
//!
//! **A command that sent and ended between two looks is still told.** The
//! last reading is taken when it ends; if it found bytes and no line went up,
//! the line goes up and comes off in the same moment, so whoever watches the
//! indicator is told it happened rather than nothing at all — and the record
//! has the number either way.
//!
//! **The count ends when the command ends, however it ends** — by itself, by a
//! signal, or stopped after two minutes — because [`WhileItRuns::ended`] is
//! called once the run returns, whatever it returned, and takes the count with
//! it. Nothing here is ever looked at again.
//!
//! **Only the command's own group is counted.** What the machine hands in is a
//! group made for this one command ([`crate::a_command_counted`]); the
//! person's own programs, and the applications their own commands count, are
//! in other groups this never names.

use std::io;
use std::process::Child;
use std::time::SystemTime;

use alo_capability::AroundACommand;
use alo_egress::{AlosCommand, Counting, Indicator, WhileAlosCommandRuns};

use crate::a_command_counted::ACommandCounted;

/// One command alo is running, as the indicator shows it and the count keeps
/// it, from before it starts until it ends.
pub(crate) struct WhileItRuns<'s> {
    /// The command's count — [`None`] on a machine that cannot count it.
    counted: Option<Box<dyn ACommandCounted + 's>>,
    /// What is leaving this machine.
    indicator: &'s mut Indicator,
    /// The command's first word, as the line names it.
    program: String,
    /// The line, while one is up.
    line: Option<WhileAlosCommandRuns>,
    /// The last number the count gave, kept in case the final reading cannot
    /// be taken.
    so_far: Option<u64>,
    /// The moment the turn is at, which every line is stamped with.
    now: SystemTime,
}

impl<'s> WhileItRuns<'s> {
    /// Before the command alo runs as `program` starts: counted by `counted`,
    /// or uncounted where that is [`None`], and shown on `indicator`.
    pub(crate) fn begun(
        counted: Option<Box<dyn ACommandCounted + 's>>,
        indicator: &'s mut Indicator,
        program: &str,
        now: SystemTime,
    ) -> Self {
        Self {
            counted,
            indicator,
            program: program.to_owned(),
            line: None,
            so_far: None,
            now,
        }
    }

    /// Put the line up, if it is not up already.
    fn shown(&mut self, how: Counting) {
        if self.line.is_none() {
            self.line = Some(
                self.indicator
                    .beginning_alos_command(AlosCommand::of(&self.program, how), self.now),
            );
        }
    }

    /// The command has ended, whatever it ended with: the count is ended and
    /// the line comes off. Answers how many bytes it sent, or [`None`] where
    /// nothing counted it.
    pub(crate) fn ended(mut self) -> Option<u64> {
        // A final reading that could not be taken falls back to the last
        // look's; with neither, nobody can say, and the answer is none rather
        // than a zero nothing measured.
        let last = self.so_far;
        let sent = self
            .counted
            .take()
            .and_then(|counted| counted.ended().or(last));
        // **Told even if it was missed between two looks.**
        if sent.is_some_and(|bytes| bytes > 0) {
            self.shown(Counting::Counted);
        }
        if let Some(line) = self.line.take() {
            self.indicator.ended_alos_command(line);
        }
        sent
    }
}

impl AroundACommand for WhileItRuns<'_> {
    fn starting(&mut self, start: &mut dyn FnMut() -> io::Result<Child>) -> io::Result<Child> {
        match self.counted.as_mut() {
            Some(counted) => counted.starting(start),
            None => {
                // **Up before it starts**, because nothing will say when it
                // begins sending.
                self.shown(Counting::Uncounted);
                start()
            }
        }
    }

    fn looking(&mut self) {
        let Some(counted) = self.counted.as_ref() else {
            return;
        };
        if let Some(bytes) = counted.so_far() {
            self.so_far = Some(bytes);
            if bytes > 0 {
                self.shown(Counting::Counted);
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "while_alos_command_runs_tests.rs"]
mod tests;
