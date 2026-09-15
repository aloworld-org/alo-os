//! Yesterday's machine: the build this machine ran before, when it was
//! replaced, whether it is still on the disk, and whether going back to it can
//! be offered.
//!
//! This is what the recovery screen draws, and nothing here draws it (the
//! shell plan's). It is read at the moment it is asked for, from the two
//! witnesses `alo_keeping_up::Before` puts together: the base's status, asked
//! now, and the record's last `updated` or `rolled-back` entry — which also
//! says **when** the build before was replaced, because that entry is the first
//! start on the build that replaced it.
//!
//! **A record that cannot be read leaves *when* unknown, and nothing else.**
//! The recovery screen matters most on a machine where something is wrong, and
//! a damaged record is no reason to withhold a return the base can make. So
//! the build before is still named and going back still decided, from the base
//! alone, and [`Yesterday::record_not_read`] says why the moment is missing.
//! What cannot be read is the base's status: nothing is decided about a
//! machine this cannot name.

use std::path::Path;
use std::time::SystemTime;

use alo_keeping::{NotKept, Reading};
use alo_keeping_up::{Before, CannotGoBack, Changed, Digest, GoingBack};
use alo_record::{Entry, Happened};

use crate::refusing::NotRead;
use crate::status;
use crate::the_base::Base;

/// The build before, and going back to it, as the machine answers now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Yesterday {
    /// The build running now.
    running: Digest,
    /// The build before it, if any is known.
    before: Option<Before>,
    /// When it was replaced, if the record says.
    replaced_at: Option<SystemTime>,
    /// Going back, or why it is not offered.
    going_back: Result<GoingBack, CannotGoBack>,
    /// Why the record could not be read, if it could not.
    record_not_read: Option<NotKept>,
}

/// Yesterday's machine, from the base's status now and the record at
/// `record_at`.
///
/// # Errors
/// [`NotRead`] when the base's status cannot be read, or names no build.
pub fn yesterday(base: &impl Base, record_at: &Path) -> Result<Yesterday, NotRead> {
    let deployments = status::deployments(base)?;
    let running = deployments
        .running()
        .map_err(|_| NotRead::NotRunningABuild)?
        .digest()
        .clone();
    let (last_change, record_not_read) = match Reading::at(record_at) {
        Ok(reading) => (
            reading.record().everything().filter_map(a_change).last(),
            None,
        ),
        Err(why) => (None, Some(why)),
    };
    let last_changed = last_change.as_ref().map(|(changed, _)| changed);
    let before = Before::of(&deployments, last_changed).map_err(|_| NotRead::NotRunningABuild)?;
    let replaced_at = match (&before, &last_change) {
        (Some(before), Some((_, at))) if before.replaced_as_the_record_says() => Some(*at),
        _ => None,
    };
    Ok(Yesterday {
        going_back: GoingBack::offered(&deployments, last_changed),
        running,
        before,
        replaced_at,
        record_not_read,
    })
}

/// A change of build the record says happened, and when — or [`None`] for any
/// other entry, or one whose builds are not whole digests.
fn a_change(entry: &Entry) -> Option<(Changed, SystemTime)> {
    let (from, to) = match entry.happened() {
        Happened::Updated { from, to } | Happened::RolledBack { from, to } => (from, to),
        _ => return None,
    };
    let changed = Changed::between(
        Digest::read(from.as_str()).ok()?,
        Digest::read(to.as_str()).ok()?,
    );
    Some((changed, entry.at()))
}

impl Yesterday {
    /// The build running now.
    #[must_use]
    pub fn running(&self) -> &Digest {
        &self.running
    }

    /// The build this machine ran before, and whether it is still on the disk.
    #[must_use]
    pub fn before(&self) -> Option<&Before> {
        self.before.as_ref()
    }

    /// When the build before was replaced by the one running, when the record
    /// says.
    #[must_use]
    pub fn replaced_at(&self) -> Option<SystemTime> {
        self.replaced_at
    }

    /// Going back, ready to offer — or, instead of an offer, why it cannot be
    /// done.
    ///
    /// # Errors
    /// [`CannotGoBack`], decided before anything is offered.
    pub fn going_back(&self) -> Result<&GoingBack, &CannotGoBack> {
        self.going_back.as_ref()
    }

    /// Why the record could not be read, when that is why
    /// [`Yesterday::replaced_at`] is unknown.
    #[must_use]
    pub fn record_not_read(&self) -> Option<&NotKept> {
        self.record_not_read.as_ref()
    }
}
