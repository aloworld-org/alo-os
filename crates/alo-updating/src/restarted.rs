//! After a restart: which build booted, and — when it is a different one — the
//! entry in the record saying the machine updated or went back, from which
//! build to which.
//!
//! **Written at the first start on the new build, because that is when it is
//! true.** An update staged, or a return set, and never restarted into has not
//! happened to the machine. So nothing is recorded when the base is told; what
//! is recorded is the machine starting on a build.
//!
//! **An update or a return, by what the person asked.** Both are a different
//! build starting. It is a return exactly when that build is the one noted by
//! [`crate::go_back`] before the base was told, and an update otherwise.
//!
//! **In this order, and the order is the guarantee.** The record is given the
//! entry first; only then is the new build kept as the last known; and only
//! then is the note of where the person chose to go back cleared. A machine
//! that stops between any two writes the entry again at its next start — a
//! duplicate a reader can see is the same change — or clears a note that
//! already did its work, rather than losing the entry or writing a return as an
//! update.
//!
//! **A note that no longer names anything is cleared.** Once the machine starts
//! on a different build, a note is either the return that just happened or a
//! return the base never made; either way it is done. A note naming the build
//! running with nothing changed is the tail of a return already written, and is
//! cleared too. A note naming another build while nothing changed is kept: the
//! person set a return and has not restarted yet.
//!
//! **No agent is behind either entry, and none is named.**

use std::time::SystemTime;

use alo_keeping::Writing;
use alo_keeping_up::Since;
use alo_record::Entry;

use crate::across_restarts::AcrossRestarts;
use crate::last_known;
use crate::one_build;
use crate::refusing::{NotRead, NotRecorded};
use crate::status;
use crate::the_base::Base;

/// Compare the build booted with the one last known, write down an update or
/// a return if it is one, and keep the build booted as the last known.
///
/// `at` is the moment written on the entry — the moment this start was noticed,
/// handed in, because nothing in this crate reads the clock on its own.
///
/// # Errors
/// [`NotRecorded`]. Nothing is kept as the last known unless the record took
/// the entry first.
pub fn after_a_restart(
    base: &impl Base,
    kept: &AcrossRestarts,
    record: &mut Writing,
    at: SystemTime,
) -> Result<Since, NotRecorded> {
    let last_known = last_known::read(kept.last_known())?;
    let going_back_to =
        one_build::read(kept.going_back_to()).map_err(|trouble| NotRecorded::GoingBackNotRead {
            path: trouble.path,
            why: trouble.why,
        })?;
    let deployments = status::deployments(base).map_err(NotRecorded::NotRead)?;
    let since = Since::between(last_known.as_ref(), going_back_to.as_ref(), &deployments)
        .map_err(|_| NotRecorded::NotRead(NotRead::NotRunningABuild))?;
    match &since {
        Since::Unchanged(now) => {
            if going_back_to.as_ref() == Some(now) {
                clear_the_note(kept)?;
            }
            return Ok(since);
        }
        Since::Updated { from, to } => record
            .keep(&Entry::updated(from.as_str(), to.as_str(), at))
            .map_err(NotRecorded::NotKept)?,
        Since::RolledBack { from, to } => record
            .keep(&Entry::rolled_back(from.as_str(), to.as_str(), at))
            .map_err(NotRecorded::NotKept)?,
        Since::FirstKnown(_) => {}
    }
    last_known::keep(kept.last_known(), since.now())?;
    if going_back_to.is_some() {
        clear_the_note(kept)?;
    }
    Ok(since)
}

/// Clear the note of the build the person chose to go back to.
fn clear_the_note(kept: &AcrossRestarts) -> Result<(), NotRecorded> {
    one_build::clear(kept.going_back_to()).map_err(|trouble| NotRecorded::GoingBackNotCleared {
        path: trouble.path,
        why: trouble.why,
    })
}
