//! **Whether a command is running in a terminal**, which is what decides if
//! closing it asks first.
//!
//! # What *running* means here, and why it is this
//!
//! The owner's decision of 2026-11-01: *Close ends the terminal at once when
//! the shell is idle; when a command is still running, it first asks.* So the
//! question is whether closing would end something the person started.
//!
//! **A command is running when the terminal's foreground process group is not
//! the shell's own.** A shell with job control — every interactive shell on a
//! terminal — puts each command it starts in a process group of its own and
//! hands the terminal to that group, and takes the terminal back when the
//! command ends. So the kernel's answer to *who has this terminal* (`tcgetpgrp`)
//! is the shell exactly when the person is at a prompt, and something else
//! exactly while a command holds the terminal. That is the definition, read
//! from the kernel at the moment Close is pressed, never inferred from what the
//! screen shows.
//!
//! # What it does not see, said rather than hidden
//!
//! - **A job sent to the background with `&`** does not hold the terminal, so
//!   it does not count: the shell is in front and the person is at a prompt.
//!   Closing ends it as closing any terminal does. This is the same line every
//!   terminal that asks before closing draws.
//! - **A shell without job control** runs its commands in its own group, so
//!   every command reads as idle. The person's own shell on alo's terminal is
//!   interactive and has job control; a shell started with it turned off is the
//!   exception, and it closes without asking.
//!
//! # When the terminal will not say
//!
//! [`IsACommandRunning::NobodyCanTell`]: the kernel refused the question. That
//! is not *idle*, and a caller that cannot tell **asks** rather than ends
//! something it could not see — the cost of asking is one press, the cost of
//! guessing wrong is a person's work.

/// Whether a command holds the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsACommandRunning {
    /// The shell is in front: the person is at a prompt.
    No,
    /// Another process group holds the terminal.
    Yes,
    /// The terminal would not say who holds it.
    NobodyCanTell,
}

impl IsACommandRunning {
    /// The answer, from the group the terminal has in front and the shell's
    /// own process number — which is its group's too, because a shell with
    /// job control leads its own group.
    #[must_use]
    pub fn of(in_front: Option<u32>, the_shell: u32) -> Self {
        match in_front {
            None => Self::NobodyCanTell,
            Some(group) if group == the_shell => Self::No,
            Some(_) => Self::Yes,
        }
    }

    /// Whether closing should ask first: anything but a shell known to be at a
    /// prompt.
    #[must_use]
    pub const fn closing_asks_first(self) -> bool {
        !matches!(self, Self::No)
    }
}

#[cfg(test)]
#[path = "whether_a_command_is_running_tests.rs"]
mod tests;
