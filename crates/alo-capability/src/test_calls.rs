//! The calls, grants and moments the approval tests are written against.
//!
//! One folder, one archive, one file and one clock reading, shared by
//! [`crate::proposal`], [`crate::approvals`], [`crate::approval`] and
//! [`crate::authorised`] — four files asking questions about the same journey,
//! and four copies of the fixtures would eventually be four different journeys.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::arg::{Arg, Given, Takes};
use crate::call::Call;
use crate::grant::{Grant, Grantee};
use crate::grants::Grants;
use crate::reach::Reach;
use crate::verb::{Effect, Requires, Verb};

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
pub(crate) fn move_file() -> Verb {
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
pub(crate) fn list_folder() -> Verb {
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

/// The change every approval test follows: March's invoice into the archive.
pub(crate) fn archiving_march() -> Call {
    moving("/home/anna/Invoices/march.pdf")
}

/// A second, different change, for asking whether one approval answers another.
pub(crate) fn archiving_april() -> Call {
    moving("/home/anna/Invoices/april.pdf")
}

/// One file into the archive, by name.
fn moving(file: &str) -> Call {
    Call::of(
        &move_file(),
        &[
            ("file", Given::text(file)),
            ("into", Given::text("/home/anna/Archive")),
        ],
    )
    .unwrap()
}

/// The read every read test follows.
pub(crate) fn listing_invoices() -> Call {
    Call::of(
        &list_folder(),
        &[("folder", Given::text("/home/anna/Invoices"))],
    )
    .unwrap()
}

/// Grants to `@files` over these folders, made at noon and lasting an hour.
pub(crate) fn granting(reaches: &[&str]) -> Grants {
    let mut grants = Grants::default();
    for reach in reaches {
        grants.grant(
            Grant::checked(
                "@files",
                Reach::Folder(PathBuf::from(reach)),
                noon(),
                hour(),
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
