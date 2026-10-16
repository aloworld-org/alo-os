//! Why a command was not run, which is always a refusal.
//!
//! Every variant here is a thing that did not happen. There is no variant for
//! *it ran and went wrong*, because a command that ran and failed is
//! [`crate::WhatHappened`] with a status in it — a program returning 1 is news
//! for the person, not a fault of this crate's.
//!
//! # A failure to take the box away is the one that leaves the machine changed
//!
//! [`NotRun::TheBoxWouldNotGoAway`] is last for the reason `alo-bounding` gives
//! about the same situation: the entry and the control group stay, because a
//! box with no entry for it would be a box the kernel stopped looking at. It is
//! reported rather than swallowed, and the caller's record should carry it.

use std::path::PathBuf;

use thiserror::Error;

/// Why a command was not run inside a box.
#[derive(Debug, Error)]
pub enum NotRun {
    /// The program named is not an absolute path.
    ///
    /// A relative path would be resolved against whatever directory this
    /// service happens to be in, which is not a thing a person approved. ADR
    /// 0001's rule about approving the sentence shown reaches this far: a
    /// command whose meaning depends on where it is run from is not the command
    /// anybody read.
    #[error("the command must be an absolute path, and {0} is not")]
    NotAnAbsoluteProgram(PathBuf),

    /// The program named is not there, or is not a file this machine would run.
    #[error("there is no program at {path}: {why}")]
    NoSuchProgram {
        /// Where the program was expected.
        path: PathBuf,
        /// What the machine said about it.
        #[source]
        why: std::io::Error,
    },

    /// The run named no place it may reach.
    ///
    /// Not an empty box but a refusal, because a box around nothing cannot even
    /// open the program it was asked to run — `alo-bounding`'s `places_of` says
    /// the same about a turn, for the same reason.
    #[error("a run must name at least one place it may reach")]
    NothingToReach,

    /// The level the person chose is not one this crate runs.
    ///
    /// **Only the sealed box is run here.** Asking each time and full trust are
    /// runs with everything the person may do, which needs no box and no
    /// control group; a caller that sent one of those here has confused the
    /// level with the mechanism, and a silent fallback to *no box* would be the
    /// worst possible reading of ADR 0064.
    #[error("this runs the sealed box, and the person chose {chose} instead")]
    NotTheSealedBox {
        /// What the person chose, named as the level rather than as a sentence.
        chose: &'static str,
    },

    /// The boundary, the control group or the bound would not be made.
    #[error("the box could not be built: {0}")]
    TheBoxWouldNotClose(#[source] alo_bounding::NotBounded),

    /// The program that puts a command inside its box would not start.
    #[error("the command could not be started inside its box: {0}")]
    ItWouldNotStart(#[source] std::io::Error),

    /// Waiting for the command failed, so what it did is unknown.
    #[error("the command was started and then could not be waited for: {0}")]
    ItCouldNotBeWaitedFor(#[source] std::io::Error),

    /// The command ran, and the box could not be taken away afterwards.
    ///
    /// **The machine is left changed by this one**, which is why it carries what
    /// happened: a caller still has an account of the run to record, and
    /// discarding it to report the cleanup failure would lose the more
    /// important of the two.
    #[error("the command ran, and its box could not be taken away: {why}")]
    TheBoxWouldNotGoAway {
        /// What the command did, which happened and is still true.
        what_happened: Box<crate::WhatHappened>,
        /// Why the box is still there.
        #[source]
        why: alo_bounding::NotBounded,
    },
}
