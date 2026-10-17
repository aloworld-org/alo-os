//! Stopping the command in front, which is one of the four things a person may
//! do about one that is waiting.
//!
//! `docs/features.md`: *ask, and get [the five answers] — then show the prompt,
//! inspect the connection, reveal what is blocking it, or **stop**.* This is the
//! last of those four, and the only one of them that changes anything.
//!
//! # It is the terminal's own key and not a kill
//!
//! What this sends is `SIGINT` to the **foreground process group** — byte for
//! byte what a terminal does when a person presses its interrupt key. Not
//! `SIGKILL`, and not `SIGTERM`:
//!
//! - `SIGINT` is what the command is written to expect. A shell script traps it,
//!   an editor asks about unsaved work, a compiler stops between files. **A
//!   command that wants to clean up gets to.**
//! - `SIGKILL` cannot be caught, so nothing cleans up and a half-written file
//!   stays half-written. The whole promise this sits inside is *instead of
//!   killing something that was nearly done*, and offering a kill as the first
//!   step would be answering that promise with the thing it is against.
//!
//! **A person who wants the stronger thing has a terminal**, which is law 2's
//! other half: alo OS ships one because a system that does not trust its owner
//! with a shell is a toy. What this offers is the gentle one, by name.
//!
//! # To the group, not the process
//!
//! `cat file | grep thing | less` is three processes in one foreground group,
//! and a person asking to stop *that* means all three. Signalling only the
//! leader leaves the rest reading from a pipe nobody will write to again, which
//! is a terminal that looks stopped and is not.
//!
//! # It refuses to stop the shell
//!
//! When nothing is running, the shell itself is in front — and interrupting a
//! shell at a prompt does nothing useful on a good day and ends the person's
//! session on a bad one. So the shell is refused by name, which is the same rule
//! [`crate::AShellOnAPseudoterminal::why_the_command_in_front_is_waiting`]
//! follows about what it will answer.

use crate::a_shell_on_a_pseudoterminal::AShellOnAPseudoterminal;

/// Why nothing was stopped.
///
/// Every one of these is a refusal or a machine that would not say, and none is
/// a command that resisted: a command that catches the signal and carries on has
/// been stopped as far as this is concerned, and what it does next is its own.
#[derive(Debug)]
pub enum NotStopped {
    /// The terminal would not say which process it has in front.
    ///
    /// Not an error: a pseudoterminal being torn down has no foreground group,
    /// and a caller should show nothing rather than report a fault.
    NothingInFront,

    /// The shell itself is in front, so nothing is running to stop.
    ///
    /// **Refused rather than done.** A person at a prompt who asks to stop has
    /// asked about nothing, and interrupting their shell is at best useless.
    NothingIsRunning,

    /// The machine would not deliver the signal.
    TheMachineRefused {
        /// The group it was sent to.
        group: u32,
        /// What the machine said.
        why: rustix::io::Errno,
    },
}

impl std::fmt::Display for NotStopped {
    fn fmt(&self, into: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NothingInFront => {
                write!(into, "this terminal would not say what it has in front")
            }
            Self::NothingIsRunning => write!(
                into,
                "nothing is running in this terminal, so there is nothing to stop"
            ),
            Self::TheMachineRefused { group, why } => write!(
                into,
                "this machine would not interrupt process group {group}: {why}"
            ),
        }
    }
}

impl AShellOnAPseudoterminal {
    /// Stop the command in front, the way this terminal's own interrupt key
    /// does.
    ///
    /// Sends `SIGINT` to the foreground process group. See this module's own
    /// documentation for why that signal, why the group, and why the shell is
    /// refused.
    ///
    /// # Returns
    /// The group that was interrupted, so a caller can say which.
    ///
    /// # Errors
    /// [`NotStopped`], every variant of which is a refusal or a machine that
    /// would not answer — never a command that declined to stop.
    #[cfg(target_os = "linux")]
    pub fn stop_the_command_in_front(&self) -> Result<u32, NotStopped> {
        let group = self
            .the_command_in_front()
            .ok_or(NotStopped::NothingInFront)?;
        if group == self.the_shell_itself() {
            return Err(NotStopped::NothingIsRunning);
        }

        let which = rustix::process::Pid::from_raw(i32::try_from(group).map_err(|_| {
            NotStopped::TheMachineRefused {
                group,
                why: rustix::io::Errno::RANGE,
            }
        })?)
        .ok_or(NotStopped::TheMachineRefused {
            group,
            why: rustix::io::Errno::SRCH,
        })?;

        rustix::process::kill_process_group(which, rustix::process::Signal::INT)
            .map_err(|why| NotStopped::TheMachineRefused { group, why })?;
        Ok(group)
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "stopping_the_command_in_front_tests.rs"]
mod tests;
