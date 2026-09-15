//! The first fact kept across a restart: which build this machine was running
//! the last time it looked.
//!
//! The base says which build it booted, and not whether this start is the
//! first on it, so this file holds the build last seen — one digest, one line,
//! written and read as `one_build.rs` writes and reads every such fact.
//! `alo_keeping_up::Since` compares it with what boots.
//!
//! **Read strictly.** What is there has to be a whole digest; anything else is
//! refused rather than read as *nothing known*, for the reason
//! [`crate::NotRecorded::LastKnownNotRead`] gives.

use std::path::Path;

use alo_keeping_up::Digest;

use crate::one_build;
use crate::refusing::NotRecorded;

/// The build last known, or [`None`] if nothing has been kept at `path`.
///
/// # Errors
/// [`NotRecorded::LastKnownNotRead`] when something is there that cannot be
/// read, or is not a whole digest.
pub fn read(path: &Path) -> Result<Option<Digest>, NotRecorded> {
    one_build::read(path).map_err(|trouble| NotRecorded::LastKnownNotRead {
        path: trouble.path,
        why: trouble.why,
    })
}

/// Keep `digest` as the build last known, whole or not at all.
///
/// # Errors
/// [`NotRecorded::LastKnownNotKept`] with what the machine said.
pub fn keep(path: &Path, digest: &Digest) -> Result<(), NotRecorded> {
    one_build::keep(path, digest).map_err(|trouble| NotRecorded::LastKnownNotKept {
        path: trouble.path,
        why: trouble.why,
    })
}
