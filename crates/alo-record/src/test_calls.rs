//! The calls, grants and moments the record's tests are written against.
//!
//! One folder, one archive, one file and one clock reading, shared by
//! [`crate::what`], [`crate::happened`], [`crate::entry`], [`crate::record`]
//! and [`crate::explain`] — five files asking questions about the same
//! afternoon, and five copies of the fixtures would eventually be five
//! different afternoons.
//!
//! Deliberately a second copy of `alo-capability`'s own test fixtures rather
//! than a shared one. Making them shared would mean shipping them in the
//! capability crate's public surface, and a set of grants that exists for the
//! convenience of tests is not something a released crate should be able to
//! hand out.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use alo_capability::{
    Arg, Call, Effect, Given, Grant, Grantee, Grants, Proposal, Reach, Requires, Takes, Verb,
};

/// A fixed moment, so that expiry is arithmetic rather than a wait.
pub(crate) fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the grants and the questions in these tests last.
pub(crate) fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The agent the tests grant things to.
pub(crate) fn files() -> Grantee {
    Grantee::named("@files")
}

/// A change: moving one file into one folder, so two things must be granted.
fn move_file() -> Verb {
    Verb::checked(
        "move_file",
        "move a file into a folder",
        Effect::Change,
        vec![
            Arg::taking("file", "the file to move", Takes::Path),
            Arg::taking("into", "the folder it goes into", Takes::Path),
        ],
        Requires::grants_over(["file", "into"]),
        "move {file} into {into}",
    )
    .unwrap()
}

/// A read: it answers inside the turn and is never proposed.
fn list_folder() -> Verb {
    Verb::checked(
        "list_folder",
        "list what is in a folder",
        Effect::Read,
        vec![Arg::taking("folder", "the folder to list", Takes::Path)],
        Requires::grants_over(["folder"]),
        "list what is in {folder}",
    )
    .unwrap()
}

/// The change every test follows: March's invoice into the archive.
pub(crate) fn archiving_march() -> Call {
    Call::of(
        &move_file(),
        &[
            ("file", Given::text("/home/anna/Invoices/march.pdf")),
            ("into", Given::text("/home/anna/Archive")),
        ],
    )
    .unwrap()
}

/// The read every test follows.
pub(crate) fn listing_invoices() -> Call {
    Call::of(
        &list_folder(),
        &[("folder", Given::text("/home/anna/Invoices"))],
    )
    .unwrap()
}

/// Grants to `@files` over these folders, made at noon and lasting a day —
/// long enough that a test about a span is not accidentally a test about
/// expiry.
pub(crate) fn granting(reaches: &[&str]) -> Grants {
    let mut grants = Grants::default();
    for reach in reaches {
        grants.grant(
            Grant::checked(
                "@files",
                Reach::Folder(PathBuf::from(reach)),
                noon(),
                hour() * 24,
            )
            .unwrap(),
        );
    }
    grants
}

/// Everything the archiving change needs granted.
pub(crate) fn granting_both() -> Grants {
    granting(&["/home/anna/Invoices", "/home/anna/Archive"])
}

/// The archiving change, put to a person and standing for a day.
pub(crate) fn proposing(call: &Call, grants: &Grants) -> Proposal {
    Proposal::checked(call, &files(), grants, noon(), hour() * 24).unwrap()
}
