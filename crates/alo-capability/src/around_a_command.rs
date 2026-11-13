//! **What happens around a command alo runs, while it runs**
//! ([ADR 0148](../../../docs/decisions/0148-the-kernel-counts-what-an-application-sends-during-a-persons-command.md),
//! as amended 2026-11-13).
//!
//! The owner, 2026-11-13: *"Yes, count alo's commands."* So a command alo runs
//! where no verb fits (ADR 0149) starts inside a control group of its own, the
//! kernel counts what that group sends while it runs, and the egress indicator
//! shows it. Two things have to happen **inside** the run to make that true,
//! and the crate that starts the program (`alo-running`) is the only one that
//! is inside it:
//!
//! - **where it starts**: the program is started by whoever holds the control
//!   group, so that it is born inside it rather than moved in after it may
//!   already have sent something — [`AroundACommand::starting`];
//! - **a look now and then** while it runs, so the indicator goes up when the
//!   count first moves rather than after the command has ended —
//!   [`AroundACommand::looking`].
//!
//! This trait is the agreement between the two, and it lives here because both
//! sides already depend on this crate and neither should depend on the other.
//! It decides nothing: what is counted, and what the person sees, is
//! `alo-turn`'s, and how a control group is made is the machine's.

use std::io;
use std::process::Child;

/// What is done around one command alo runs, while it runs.
pub trait AroundACommand {
    /// Start the command, by calling `start`, from wherever it is to begin.
    ///
    /// Whatever `start` starts belongs to where this thread is when it is
    /// called, which is the whole of the point: an implementation that counts
    /// the command puts this thread in the command's control group, calls
    /// `start`, and brings the thread back out, so the command is inside from
    /// its first instruction.
    ///
    /// # Errors
    /// Whatever `start` answered, or why the command could not be started
    /// where it was to begin — and then nothing was started.
    fn starting(&mut self, start: &mut dyn FnMut() -> io::Result<Child>) -> io::Result<Child>;

    /// Look at the command, as it runs. Called again and again until it ends,
    /// a few times a second, and never after.
    fn looking(&mut self);
}

/// Nothing around the command: it starts where the caller is, and nothing
/// looks at it — what a command run outside alo's turns has, such as one run
/// as administrator by its own service.
#[derive(Debug, Default, Clone, Copy)]
pub struct NothingAround;

impl AroundACommand for NothingAround {
    fn starting(&mut self, start: &mut dyn FnMut() -> io::Result<Child>) -> io::Result<Child> {
        start()
    }

    fn looking(&mut self) {}
}
