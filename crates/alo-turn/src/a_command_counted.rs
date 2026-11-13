//! **A command alo runs, in a control group of its own that the kernel
//! counts** — what the machine hands a turn for the length of one command
//! ([ADR 0148](../../../docs/decisions/0148-the-kernel-counts-what-an-application-sends-during-a-persons-command.md),
//! as amended 2026-11-13).
//!
//! The owner, 2026-11-13: *"Yes, count alo's commands"* — bytes only, no
//! addresses, no contents, only for the seconds alo's command runs, and only
//! alo's command, never the person's. This trait is the machine's half of that,
//! and each of those words is a rule an implementation keeps:
//!
//! - **a group made for this command and nothing else**, so the count is the
//!   command's and the person's own programs are in other groups the count
//!   never names;
//! - **begun before the command starts and ended when it ends** — by
//!   [`ACommandCounted::ended`], which takes the value, so a count cannot be
//!   read after it has ended or ended twice;
//! - **a number of bytes**, and nothing a caller could ask about where.
//!
//! What the person sees is not decided here: `crate::while_alos_command_runs`
//! puts the line up from what this answers.

use std::io;
use std::process::Child;

/// One command's count, from before the command starts until it ends.
pub trait ACommandCounted {
    /// Start the command, by calling `start`, from **inside** the command's
    /// own control group, so it is counted from its first instruction.
    ///
    /// # Errors
    /// Why the command could not be started inside its group, or whatever
    /// `start` answered. Either way nothing was started outside it.
    fn starting(&mut self, start: &mut dyn FnMut() -> io::Result<Child>) -> io::Result<Child>;

    /// How many bytes the command has sent off this machine so far, or
    /// [`None`] if the count could not be read at this moment.
    fn so_far(&self) -> Option<u64>;

    /// End the count, and answer what it came to: from this moment nothing
    /// the group sends is looked at, and the group is taken away.
    ///
    /// [`None`] when the last reading could not be taken.
    fn ended(self: Box<Self>) -> Option<u64>;
}
