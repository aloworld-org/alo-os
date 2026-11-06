//! **What a person is told when the sealed box would not run their command**:
//! the plain sentence on a computer where the box cannot run programs yet, or
//! the refusal's own sentence everywhere else.
//!
//! The owner's decision of 2026-11-06: *say so plainly, keep going*
//! (`alo_capability::the_sealed_box_here` has the measurement and the plan).
//! **The plain sentence is said only when both halves were seen**: the helper
//! said the box was joined and the kernel then denied the start, and `/` is a
//! composefs image. Either half alone is something else — a box refused for
//! another reason keeps its own sentence, and a composefs machine that ran the
//! command has nothing to say.

use std::path::PathBuf;

use alo_running::NotRun;
use alo_running::the_helpers_word::StartRefused;
use alo_strings::{Filling, Strings};

/// Which sentence a refused `try` says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatItSays {
    /// The sealed box cannot run programs on this computer yet: `program` was
    /// not run, and *ask each time* is where to go instead.
    NotOnThisComputerYet {
        /// The program that was not run.
        program: PathBuf,
    },
    /// The refusal's own sentence.
    ItsOwn,
}

impl WhatItSays {
    /// Which, for `not_run` on a machine whose root `is_composefs`.
    #[must_use]
    pub fn of(not_run: &NotRun, root_is_composefs: bool) -> Self {
        match not_run {
            NotRun::TheBoxRefusedTheCommand {
                program,
                refused: StartRefused::PermissionDenied,
            } if root_is_composefs => Self::NotOnThisComputerYet {
                program: program.clone(),
            },
            _ => Self::ItsOwn,
        }
    }

    /// The sentence, in `strings`' language.
    #[must_use]
    pub fn said(&self, not_run: &NotRun, strings: &Strings) -> String {
        match self {
            Self::NotOnThisComputerYet { program } => strings
                .say(
                    &alo_capability::words::THE_SEALED_BOX_CANNOT_RUN_PROGRAMS_HERE_YET.key(),
                    &Filling::of("program", program.display().to_string()),
                )
                .into_text(),
            Self::ItsOwn => strings
                .say(&not_run.the_word().key(), &Filling::nothing())
                .into_text(),
        }
    }
}

#[cfg(test)]
#[path = "when_the_box_cannot_run_it_tests.rs"]
mod tests;
