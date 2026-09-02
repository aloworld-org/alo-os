//! The calls, grants, words and moments the approval tests are written against.
//!
//! One folder, one archive, one file and one clock reading, shared by
//! [`crate::proposal`], [`crate::approvals`], [`crate::approval`] and
//! [`crate::authorised`] — four files asking questions about the same journey,
//! and four copies of the fixtures would eventually be four different journeys.
//!
//! **The two verbs are declared from words**, like every real verb since item
//! 9g, and [`reading`] is the vocabulary that holds them beside this crate's
//! own. What a fixture verb says is therefore looked up exactly as a shell
//! would look it up.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use alo_strings::{Strings, Word};

use crate::arg::{Arg, Given, Takes};
use crate::call::Call;
use crate::grant::{Grant, Grantee};
use crate::grants::Grants;
use crate::reach::Reach;
use crate::verb::{Effect, Requires, Verb};

/// What `move_file` does.
pub(crate) const MOVING: Word = Word::saying(
    "testing.verb.move-file.purpose",
    "move a file into a folder",
);
/// **The sentence a person approves before the fixture file is moved.**
pub(crate) const MOVING_SENTENCE: Word =
    Word::saying("testing.verb.move-file.sentence", "move {file} into {into}");
/// What `move_file` moves.
pub(crate) const MOVING_FILE: Word =
    Word::saying("testing.verb.move-file.argument.file", "the file to move");
/// Where `move_file` moves it.
pub(crate) const MOVING_INTO: Word = Word::saying(
    "testing.verb.move-file.argument.into",
    "the folder it goes into",
);

/// What `list_folder` does.
pub(crate) const LISTING: Word = Word::saying(
    "testing.verb.list-folder.purpose",
    "list what is in a folder",
);
/// What a person is shown when `list_folder` runs.
pub(crate) const LISTING_SENTENCE: Word = Word::saying(
    "testing.verb.list-folder.sentence",
    "list what is in {folder}",
);
/// What `list_folder` lists.
pub(crate) const LISTING_FOLDER: Word = Word::saying(
    "testing.verb.list-folder.argument.folder",
    "the folder to list",
);

/// Everything the two fixture verbs can say.
pub(crate) const THE_WORDS: [Word; 7] = [
    MOVING,
    MOVING_SENTENCE,
    MOVING_FILE,
    MOVING_INTO,
    LISTING,
    LISTING_SENTENCE,
    LISTING_FOLDER,
];

/// This crate's words and the fixture verbs', with nothing translated.
pub(crate) fn reading() -> Strings {
    crate::testing::speaking(&THE_WORDS)
}

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
        MOVING,
        Effect::Change,
        vec![
            Arg::taking("file", MOVING_FILE, Takes::Path),
            Arg::taking("into", MOVING_INTO, Takes::Path),
        ],
        Requires::grants_over(["file", "into"]),
        MOVING_SENTENCE,
    )
    .unwrap()
}

/// A read: it answers inside the turn and is never proposed.
pub(crate) fn list_folder() -> Verb {
    Verb::checked(
        "list_folder",
        LISTING,
        Effect::Read,
        vec![Arg::taking("folder", LISTING_FOLDER, Takes::Path)],
        Requires::grants_over(["folder"]),
        LISTING_SENTENCE,
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
