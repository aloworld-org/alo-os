//! **One shell command alo was asked to run where no verb fits**
//! ([ADR 0149](../../../docs/decisions/0149-full-access-is-everything-the-person-can-do.md)).
//!
//! The owner, 2026-11-08: *where no verb exists, alo does it by running a
//! command at full access (without asking) and at* ask me each time *(after the
//! OK). "There is no verb for that" is never an answer a person sees at either
//! level.*
//!
//! # This is not a verb, and law 2 is why that matters
//!
//! `CLAUDE.md`'s law 2: *no `exec`, no shell and no "advanced" escape hatch is
//! ever added to that list* — the broker's list of verbs — *running code is a
//! separate grant that only the person gives, at one of three levels (ADR
//! 0064)*. A command is that second half. It is not on [`crate::Verbs`], it is
//! not a [`crate::Call`], it never passes [`crate::Authorised`], and no grant
//! reaches it: what decides whether it runs is the level the person chose for
//! running code, [`crate::HowCodeRuns`], read from their own file at the moment
//! it is asked. A model can ask; only the person's choice lets it happen.
//!
//! # What is refused before anything else is asked
//!
//! Three shapes, refused here so that nothing downstream has to remember them:
//! an empty command, a command longer than [`THE_LONGEST_COMMAND`] bytes, and a
//! command that is not one line. The last is about the person rather than the
//! shell: what a person approves is the sentence, and a sentence with a line
//! break in it is drawn as two lines, the second of which reads as something
//! else.

use alo_strings::{Filling, Said, Strings};
use serde::Serialize;

use crate::words;

/// The program a command is handed to, by its full path.
///
/// Named once, here, because the record writes it down and the crate that
/// starts the program starts it: two spellings would be a record naming a
/// program that is not the one that ran.
pub const THE_SHELL: &str = "/bin/sh";

/// The flag that hands [`THE_SHELL`] one line to run.
pub const ONE_LINE_TO_THE_SHELL: &str = "-c";

/// The longest command, in bytes, alo will put to the shell or to the person.
///
/// A bound rather than a limit anybody reached: a model asked to do one thing
/// writes a line, and a command longer than a screen is one nobody could read
/// before approving it.
pub const THE_LONGEST_COMMAND: usize = 4096;

/// One command, as it will be shown to the person and handed to the shell.
///
/// Made only by [`ACommand::of`], so a value of this type is always one
/// non-empty line no longer than [`THE_LONGEST_COMMAND`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ACommand(String);

/// Why a line could not be a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotACommand {
    /// It had nothing in it but spaces.
    Empty,
    /// It was longer than [`THE_LONGEST_COMMAND`] bytes.
    TooLong,
    /// It had a line break or a NUL in it.
    NotOneLine,
}

impl NotACommand {
    /// What this says, in the language the person reads.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        match self {
            Self::Empty => strings.say(&words::A_COMMAND_THAT_IS_EMPTY.key(), &Filling::nothing()),
            Self::TooLong => strings.say(
                &words::A_COMMAND_THAT_IS_TOO_LONG.key(),
                &Filling::of("most", THE_LONGEST_COMMAND.to_string()),
            ),
            Self::NotOneLine => strings.say(
                &words::A_COMMAND_THAT_IS_NOT_ONE_LINE.key(),
                &Filling::nothing(),
            ),
        }
    }
}

impl ACommand {
    /// The command in `line`, as it was written.
    ///
    /// Not trimmed: what the person approves and what the shell runs are the
    /// same bytes.
    ///
    /// # Errors
    /// [`NotACommand`], saying which of the three shapes it was.
    pub fn of(line: &str) -> Result<Self, NotACommand> {
        if line.trim().is_empty() {
            return Err(NotACommand::Empty);
        }
        if line.len() > THE_LONGEST_COMMAND {
            return Err(NotACommand::TooLong);
        }
        if line.contains(['\n', '\r', '\0']) {
            return Err(NotACommand::NotOneLine);
        }
        Ok(Self(line.to_owned()))
    }

    /// The command, exactly as it will run.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.0
    }

    /// The program the command starts with — its first word — for a sentence
    /// that names what was not run.
    #[must_use]
    pub fn program(&self) -> &str {
        self.0.split_whitespace().next().unwrap_or(THE_SHELL)
    }

    /// What the person is asked to approve: the command, whole.
    #[must_use]
    pub fn sentence(&self, strings: &Strings) -> Said {
        strings.say(
            &words::RUN_A_COMMAND.key(),
            &Filling::of("command", self.0.clone()),
        )
    }

    /// What the shell is handed after [`THE_SHELL`], as the record keeps it.
    #[must_use]
    pub fn arguments(&self) -> [String; 2] {
        [ONE_LINE_TO_THE_SHELL.to_owned(), self.0.clone()]
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
#[path = "a_command_tests.rs"]
mod tests;
