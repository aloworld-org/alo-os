//! What this crate's own tests are written against.
//!
//! Two things, and both of them are *this machine* rather than a description of
//! one: a directory on the disk the tests are running on, and the two sides as
//! this process could really have them.
//!
//! **The person is whoever is running the tests.** Everything here touches a
//! real filesystem and a real socket, so a fixture that named a person this
//! process is not would be a fixture whose every happy path is a refusal. The
//! agent is a login beside it, chosen so that it is neither root nor the person
//! whoever runs these tests happens to be.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::caller::{Caller, Uid};
use crate::side::Sides;
use crate::unix::{our_group, us};

/// The login the fixture gives the agent when the person is not it.
const AN_AGENT: u32 = 989;

/// The one it gives the agent when the person happens to be [`AN_AGENT`].
const ANOTHER_AGENT: u32 = 990;

/// This machine, with whoever is running the tests as the person.
pub(crate) fn ourselves() -> Sides {
    let person = us().unwrap();
    let agent = if person.raw() == AN_AGENT {
        ANOTHER_AGENT
    } else {
        AN_AGENT
    };
    Sides::of(person, Uid::of(agent).unwrap(), our_group().unwrap()).unwrap()
}

/// A caller running as the user this process is, which is the person.
pub(crate) fn calling_as_the_person() -> Caller {
    Caller::known(
        i32::try_from(std::process::id()).unwrap(),
        us().unwrap(),
        our_group().unwrap(),
    )
}

/// A folder of this test's own, on the disk the tests are running on.
///
/// `alo-keeping` and `alo-files` have the same fixture and for the same reason:
/// a test about a real socket has to be about a real directory, and two of them
/// must not meet.
pub(crate) fn a_directory_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-agentd-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    drop(std::fs::remove_dir_all(&folder));
    std::fs::create_dir_all(&folder).unwrap();
    folder
}
