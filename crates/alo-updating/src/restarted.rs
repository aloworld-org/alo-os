//! After a restart: which build booted, and — when it is a different one — the
//! entry in the record saying the machine updated, from which build to which.
//!
//! **Written at the first start on the new build, because that is when it is
//! true.** An update staged and never restarted into has not happened to the
//! machine. So nothing is recorded when the base is told to stage a build;
//! what is recorded is the machine starting on one.
//!
//! **In this order, and the order is the guarantee.** The record is given the
//! entry first, and only then is the new build kept as the last known. A
//! machine that stops between the two writes the entry again at its next start
//! — a duplicate a reader can see is the same update — rather than keeping the
//! new build first and losing the entry, which would be a change nobody could
//! find in the record.
//!
//! **No agent is behind it, and none is named.** The entry is
//! `alo_record::Happened::Updated`, which has no field for one.

use std::path::Path;
use std::time::SystemTime;

use alo_keeping::Writing;
use alo_keeping_up::Since;
use alo_record::Entry;

use crate::last_known;
use crate::refusing::{NotRead, NotRecorded};
use crate::status;
use crate::the_base::Base;

/// Where the build last known is kept on an alo OS machine, beside the record.
pub const THE_LAST_KNOWN_BUILD: &str = "/var/lib/alo/last-known-build";

/// Compare the build booted with the one last known, write down an update if
/// it is one, and keep the build booted as the last known.
///
/// `at` is the moment written on the entry — the moment this start was noticed,
/// handed in, because nothing in this crate reads the clock on its own.
///
/// # Errors
/// [`NotRecorded`]. Nothing is kept as the last known unless the record took
/// the entry first.
pub fn after_a_restart(
    base: &impl Base,
    last_known_at: &Path,
    record: &mut Writing,
    at: SystemTime,
) -> Result<Since, NotRecorded> {
    let last_known = last_known::read(last_known_at)?;
    let deployments = status::deployments(base).map_err(NotRecorded::NotRead)?;
    let since = Since::between(last_known.as_ref(), &deployments)
        .map_err(|_| NotRecorded::NotRead(NotRead::NotRunningABuild))?;
    match &since {
        Since::Unchanged(_) => return Ok(since),
        Since::Updated { from, to } => record
            .keep(&Entry::updated(from.as_str(), to.as_str(), at))
            .map_err(NotRecorded::NotKept)?,
        Since::FirstKnown(_) => {}
    }
    last_known::keep(last_known_at, since.now())?;
    Ok(since)
}
