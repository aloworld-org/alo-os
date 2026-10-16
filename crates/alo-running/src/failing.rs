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

use alo_capability::words;
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
    /// **The machine is left changed by this one**, and the command **ran** —
    /// which is why it carries how the command ended. `WhereCommandsRun::ran`
    /// writes the run down before reporting this, because a cleanup failure
    /// recorded as *a command was not run* would be a record saying something
    /// that did not happen about something that did.
    ///
    /// *The first version of this carried the whole account and was reached
    /// through the refusal road, which would have written exactly that lie.*
    #[error("the command ran and returned {ended}, and its box could not be taken away: {why}")]
    TheBoxWouldNotGoAway {
        /// How the command ended, which happened and is still true.
        ended: std::process::ExitStatus,
        /// Why the box is still there.
        #[source]
        why: alo_bounding::NotBounded,
    },

    /// What happened could not be written down, so it did not happen.
    ///
    /// **A protection ADR 0064 calls always on is not one that may be skipped
    /// because a disk was full.** A run whose record could not be written is
    /// refused, which is `alo-bounding`'s rule about a boundary arriving here:
    /// *a turn whose boundary cannot be applied does not run.*
    ///
    /// For a run that already happened there is nothing to undo, so this is
    /// reported after the attempt rather than instead of it — and a caller
    /// meeting one has a machine whose record is incomplete, which is the fact
    /// a review wants.
    ///
    /// **Carried by its `Debug` rather than through `#[source]`**, because
    /// `alo_keeping::NotKept` is a person-facing type with a `said(strings)`
    /// method and implements neither `Display` nor `Error`. The sentence a
    /// person reads comes from that method; this text is for whoever administers
    /// the machine, which is what every other message in this file is for.
    #[error("what happened could not be written down, so the run is refused: {0:?}")]
    ItWasNotWrittenDown(alo_keeping::NotKept),
}

impl NotRun {
    /// What a person is told when a command did not run.
    ///
    /// **Four sentences for eight refusals, and the grouping is the point.** A
    /// person asking *why did nothing happen* needs to know whether the request
    /// was unusable, whether their own choice stopped it, or whether the machine
    /// failed — and does not need eight shades of the third. A surface that
    /// wants the detail has the error itself; this is the sentence.
    ///
    /// **The words live in `alo-capability`, not here, and that is deliberate.**
    /// `alo-saying` is the machine's one vocabulary and it ships every crate
    /// that declares a word, which puts that crate inside the process an agent's
    /// turn runs in. This crate holds the only `exec` in the repository
    /// ([ADR 0117](../../../docs/decisions/0117-the-thing-that-starts-a-program-is-not-the-thing-that-bounds-a-turn.md)),
    /// so a word declared here would put that `exec` on a turn's dependency
    /// path. `alo-letting-go` guards the same hazard with a test of its own, for
    /// its one privileged remover. `alo-capability` already declares the words
    /// for the three levels and is already in that process, which is where
    /// these belong: they are about the capability, not about the mechanism.
    #[must_use]
    pub const fn the_word(&self) -> alo_strings::Word {
        match self {
            // The request could not be a command on this machine.
            Self::NotAnAbsoluteProgram(_) | Self::NoSuchProgram { .. } | Self::NothingToReach => {
                words::NOTHING_RAN_NOT_A_COMMAND
            }
            // The person's own choice, and it is not a fault.
            Self::NotTheSealedBox { .. } => words::NOTHING_RAN_NOT_THAT_WAY,
            // The machine could not build the box, so it ran nothing.
            Self::TheBoxWouldNotClose(_) => words::NOTHING_RAN_NO_BOX,
            // The box was built and the command still did not run, or ran and
            // was lost. `TheBoxWouldNotGoAway` is here because the command
            // *did* run — what failed was taking the box away afterwards — and
            // there is no fifth sentence for it: the person's news is the run,
            // which `WhatHappened` carries, and this sentence is only reached
            // where a caller asks this type what to say.
            Self::ItWouldNotStart(_)
            | Self::ItCouldNotBeWaitedFor(_)
            | Self::TheBoxWouldNotGoAway { .. }
            | Self::ItWasNotWrittenDown(_) => words::NOTHING_RAN_WOULD_NOT_START,
        }
    }
}
