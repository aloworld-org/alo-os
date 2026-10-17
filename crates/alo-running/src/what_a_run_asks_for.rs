//! A command, and everywhere it may reach — checked before any of it happens.
//!
//! This is the whole of a run's input, and [`ARun::checked`] is the whole of its
//! validation. Both are here so that a caller cannot build a run that was never
//! looked at: the fields are private, the one way in checks, and what comes out
//! is a value the lifecycle can take without asking any more questions.
//!
//! # Everything is refused before anything is made
//!
//! `alo-bounding`'s `places_of` refuses too many paths before it looks any of
//! them up, and says why: *a call that cannot be bounded is refused without the
//! machine being asked about paths it named.* The same order holds here one
//! level up. A run with a relative program, no places, or the wrong level is
//! refused before a control group exists, before an entry is written, and
//! before the machine is asked about a single path.
//!
//! So a refused run leaves the machine exactly as it was. That is worth having
//! as a property rather than as a sequence of early returns that happen to be
//! in the right order, and `a_refused_run_changes_nothing` asserts it.

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use alo_capability::HowCodeRuns;

use crate::failing::NotRun;

/// A command to run inside a sealed box, with everywhere it may reach.
///
/// Made only by [`ARun::checked`], so a value of this type is a run whose
/// program is an absolute path to something that exists and which names at
/// least one place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ARun {
    /// The command, which [`ARun::checked`] has established is an absolute path
    /// to something this machine has.
    program: PathBuf,
    /// What is passed to it, never parsed and never expanded.
    arguments: Vec<OsString>,
    /// The folders the kernel will let it reach, at least one.
    may_reach: Vec<PathBuf>,
}

impl ARun {
    /// A run, if what was asked for is a run at all.
    ///
    /// `program` is an absolute path to the command. `arguments` are passed as
    /// they are — this crate never parses them, never expands them and never
    /// hands them to a shell, because a shell is the one thing law 2 names.
    /// `may_reach` are the folders the kernel will let it open, and one of them
    /// has to be the one the program itself is in or the box cannot even start
    /// it.
    ///
    /// # Errors
    /// [`NotRun::NotTheSealedBox`] if the person chose another level, which is
    /// checked **first** because it is the only one of these that is about what
    /// a person decided rather than about whether a request is well formed.
    /// [`NotRun::NotAnAbsoluteProgram`] for a relative program,
    /// [`NotRun::NoSuchProgram`] when there is nothing there, and
    /// [`NotRun::NothingToReach`] when no place was named.
    pub fn checked(
        chose: HowCodeRuns,
        program: PathBuf,
        arguments: Vec<OsString>,
        may_reach: Vec<PathBuf>,
    ) -> Result<Self, NotRun> {
        // Exhaustive rather than a `matches!` and a second match, so that a
        // fourth level added to ADR 0064 cannot reach here without somebody
        // saying what this crate does about it. There is no `unreachable!`,
        // because the compiler walking the list is the point.
        let wrong_level = match chose {
            HowCodeRuns::InASealedBox => None,
            HowCodeRuns::AskingEachTime => Some("asking each time"),
            HowCodeRuns::AtFullTrust => Some("full trust"),
        };
        if let Some(chose) = wrong_level {
            return Err(NotRun::NotTheSealedBox { chose });
        }
        if !program.is_absolute() {
            return Err(NotRun::NotAnAbsoluteProgram(program));
        }
        if may_reach.is_empty() {
            return Err(NotRun::NothingToReach);
        }
        // Asked of the machine last, and only once everything that can be
        // decided without it has been. `metadata` rather than `exists` so that
        // *why* it is not there reaches the person: a permission error and an
        // absence are different news.
        program
            .metadata()
            .map_err(|why| NotRun::NoSuchProgram {
                path: program.clone(),
                why,
            })
            .map(|_| Self {
                program,
                arguments,
                may_reach,
            })
    }

    /// The command itself.
    #[must_use]
    pub fn program(&self) -> &Path {
        &self.program
    }

    /// What is passed to it, unchanged.
    #[must_use]
    pub fn arguments(&self) -> &[OsString] {
        &self.arguments
    }

    /// Everywhere the kernel will let it reach.
    #[must_use]
    pub fn may_reach(&self) -> &[PathBuf] {
        &self.may_reach
    }

    /// This run, as an account of what happened, given how it ended.
    ///
    /// One place rather than two, because `WhereCommandsRun::ran` builds it on
    /// two different roads — the command ending, and the command ending beside
    /// a box that would not come away — and two copies of this would be two
    /// accounts able to disagree about one run.
    ///
    /// `the_kernel_was_holding_it` is true because this crate runs nothing it
    /// cannot bound. [`crate::WhatHappened`] says why it is a field at all.
    #[must_use]
    pub fn as_it_ended(&self, ended: std::process::ExitStatus) -> crate::WhatHappened {
        crate::WhatHappened {
            program: self.program.clone(),
            arguments: self.arguments.clone(),
            was_allowed: self.may_reach.clone(),
            ended,
            the_kernel_was_holding_it: true,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "what_a_run_asks_for_tests.rs"]
mod tests;
