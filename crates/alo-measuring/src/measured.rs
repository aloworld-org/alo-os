//! The one door from *this may measure* to *this is what was measured*.
//!
//! [`alo_files::Touching`] is the end of everything the capability model can
//! decide about a call that names a folder: validated, permitted, and asked
//! about again — at the moment it would run — where the folder really leads.
//! A [`Measured`] is what happens next for `what_is_running` and
//! `what_is_filling`, and it is the only thing in this crate that an agent's
//! authority ever reaches.
//!
//! # What each comes down to
//!
//! `what_is_running` is two readings of the granted directory with the
//! caller's interval between them, made into rates by
//! [`crate::Reading::since`] — the interval is passed in and so is the
//! waiting, because this crate reads no clock and the daemon that carries a
//! turn is the one that knows how long it may wait. `what_is_filling` is
//! [`crate::Holding::of`] on the granted folder. Neither reads anything the
//! grant did not name: the first reads files under the directory the call
//! named, the second walks the folder the call named and follows nothing.
//!
//! # Two ways of not happening, and they are different facts
//!
//! [`Measured::of`] cannot refuse: a refusal by the grants is
//! `alo_capability::Refused`, made by [`alo_files::Touching::of`] before
//! anything here is reached. What it can do is come back with a
//! [`NotMeasured`] — the kernel's file could not be read, the folder could
//! not be counted, this is not a Linux host — and the authorisation comes
//! back either way, because either way something is written down. A call
//! that was permitted and attempted is a thing that happened, and is
//! recorded as one; what the machine made of it is the answer to whoever
//! asked, not evidence about the capability model.
//!
//! # A person needs none of this
//!
//! [`crate::Reading::now`] and [`crate::Holding::of`] are the measurements,
//! and they take no caller, no grant and no name. This file is the road an
//! **agent** takes to the same numbers — under a grant, recorded — and a
//! person's window takes none of it. The numbers are the same because they
//! are the same functions.

use std::time::Duration;

use alo_capability::Authorised;
use alo_files::{Real, Touching};

use crate::holding::Holding;
use crate::kernel::Disk;
use crate::reading::Reading;
use crate::refusing::NotMeasured;
use crate::running::Running;

/// The name `what_is_running` is declared under.
const WHAT_IS_RUNNING: &str = "what_is_running";

/// The name `what_is_filling` is declared under.
const WHAT_IS_FILLING: &str = "what_is_filling";

/// What a permitted measurement answered with.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Measurement {
    /// What is running and what each process is using, as rates over the
    /// interval that was passed in.
    Running(Running),
    /// What is filling the folder, as a tree of sizes.
    Filling(Holding),
}

/// What happened when a permitted measurement was made.
///
/// Not `Clone`, like everything else on this journey: a thing that happened
/// is not a thing that can happen again.
#[derive(Debug)]
pub struct Measured {
    /// What ran, and the authority it ran under.
    authorised: Authorised,
    /// What was measured, or why nothing could be.
    outcome: Result<Measurement, NotMeasured>,
}

impl Measured {
    /// Make the measurement a permitted call asks for.
    ///
    /// `interval` is how far apart the two readings of *what is running*
    /// are to be, and `waiting` is what passes that time — handed in
    /// together so that a test's kernel can change between the readings
    /// without anybody sleeping, and so that the interval the rates are
    /// over is the one that was waited. It is never called for
    /// *what is filling*, which is one look at the disk.
    #[must_use]
    pub fn of(touching: Touching, interval: Duration, waiting: impl FnOnce(Duration)) -> Self {
        let outcome = measured(&touching, interval, waiting);
        Self {
            authorised: touching.into_authorised(),
            outcome,
        }
    }

    /// What ran, and the authority it ran under — what the record is written
    /// from.
    #[must_use]
    pub fn authorised(&self) -> &Authorised {
        &self.authorised
    }

    /// What was measured, when something was.
    #[must_use]
    pub fn measurement(&self) -> Option<&Measurement> {
        self.outcome.as_ref().ok()
    }

    /// Why nothing was measured, when nothing was.
    #[must_use]
    pub fn not_measured(&self) -> Option<&NotMeasured> {
        self.outcome.as_ref().err()
    }

    /// The authority and the outcome, taken.
    ///
    /// The authorisation comes back whether or not the machine managed it,
    /// because a call that was permitted and attempted is a thing that
    /// happened and is recorded as one.
    pub fn into_parts(self) -> (Authorised, Result<Measurement, NotMeasured>) {
        (self.authorised, self.outcome)
    }
}

/// The measurement, made.
fn measured(
    touching: &Touching,
    interval: Duration,
    waiting: impl FnOnce(Duration),
) -> Result<Measurement, NotMeasured> {
    match touching.verb() {
        WHAT_IS_RUNNING => {
            let proc = real(touching, "proc")?.as_path();
            let earlier = Reading::of_kernel(&Disk, proc)?;
            waiting(interval);
            let later = Reading::of_kernel(&Disk, proc)?;
            later.since(&earlier, interval).map(Measurement::Running)
        }
        WHAT_IS_FILLING => {
            Holding::of(real(touching, "folder")?.as_path()).map(Measurement::Filling)
        }
        other => Err(NotMeasured::NotThisCrates {
            verb: other.to_owned(),
        }),
    }
}

/// Where this argument's path really leads.
fn real<'a>(touching: &'a Touching, argument: &str) -> Result<&'a Real, NotMeasured> {
    touching.real(argument).ok_or_else(|| NotMeasured::Missing {
        verb: touching.verb().to_owned(),
        argument: argument.to_owned(),
    })
}
