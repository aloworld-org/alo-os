//! The machine every test in this crate is written against.
//!
//! One moment, one hour, one agent and one granted folder, here rather than
//! beside whichever file needed them first: this crate has two files that both
//! ask the same question — *is the grant that went down the grant that comes
//! back* — and a fixture written twice is two fixtures that can disagree about
//! what was granted.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, SystemTime};

use alo_capability::{Grant, Grants, Reach};

/// The agent these tests grant to.
pub(crate) const HERS: &str = "@files";

/// The moment these tests call noon.
pub(crate) fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long a grant made in these tests lasts.
pub(crate) fn an_hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// A machine on which this folder has been granted to [`HERS`] for an hour at
/// [`noon`].
pub(crate) fn granted_in(folder: &str) -> Grants {
    let mut grants = Grants::default();
    grants.grant(Grant::checked(HERS, Reach::Folder(folder.into()), noon(), an_hour()).unwrap());
    grants
}

/// A folder of this test's own, emptied of any earlier run.
///
/// Unix only, because the file on the disk is: `crate::keeping` is what has a
/// mode and an owner to check.
#[cfg(unix)]
pub(crate) fn a_folder_of_our_own(what: &str) -> std::path::PathBuf {
    let at = std::env::temp_dir().join(format!("alo-remembering-{what}-{}", std::process::id()));
    let _cleared = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    at
}
