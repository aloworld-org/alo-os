//! **What each file change says it changes and leaves alone, held to what it
//! does**, on a real filesystem.
//!
//! The owner's decision of 2026-11-01: each change verb says, in one or two
//! short sentences on the approval card, what it changes and what it leaves
//! alone, and each sentence is backed by a test that runs the verb and checks
//! the claim. These are those tests for `rename_file`, `move_file` and
//! `archive_folder`. Each starts by holding the verb to its exact words, so a
//! sentence that is reworded without its test is a failing build, and then
//! runs the verb the whole way — proposed, approved once, redeemed, resolved
//! and done — and checks every file around it, by bytes and modification time,
//! before and after.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::{Approvals, Given, Grant, Grantee, Grants, Proposal, Reach};
use alo_files::{Did, OnThisMachine, Resolving as _, Touching, file_verbs, file_words, words};
use alo_strings::{Strings, Word};

/// A fixed moment, so that expiry is arithmetic rather than a wait.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the grants and the approvals here last.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// A folder of this test's own, resolved.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-says-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    OnThisMachine.real(&folder).unwrap().into_path_buf()
}

/// Grants to `@files` over these folders.
fn granting(folders: &[&Path]) -> Grants {
    let mut grants = Grants::default();
    for folder in folders {
        grants.grant(
            Grant::checked(
                "@files",
                Reach::Folder((*folder).to_path_buf()),
                noon(),
                hour(),
            )
            .unwrap(),
        );
    }
    grants
}

/// A path as an argument arrives: text.
fn as_given(path: &Path) -> Given {
    Given::text(path.to_string_lossy().into_owned())
}

/// Every word this machine reads here.
fn in_english() -> Strings {
    let mut vocabulary = file_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A change, all the way through, or the sentence that refused it.
fn changing(verb: &str, given: &[(&str, Given)], grants: &Grants) -> Result<Did, String> {
    let call = file_verbs().unwrap().call(verb, given).unwrap();
    let mut approvals = Approvals::default();
    let id = approvals.propose(
        Proposal::checked(&call, &Grantee::named("@files"), grants, noon(), hour()).unwrap(),
    );
    let authorised = approvals
        .approve(id, noon())
        .unwrap()
        .redeem(grants, noon())
        .unwrap();
    let touching = Touching::of(authorised, grants, &OnThisMachine, &in_english())
        .map_err(|why| why.said(&in_english()).into_text())?;
    let did = Did::of(touching, grants, &in_english())
        .map_err(|why| why.said(&in_english()).into_text())?;
    // **A refusal the machine made is carried inside what was done**, as the
    // record keeps it; here it is a refusal.
    match did.failure() {
        Some(why) => Err(why.said(&in_english()).into_text()),
        None => Ok(did),
    }
}

/// What is under a folder: every file's bytes and modification time, and
/// every folder, by its path from `root`.
fn everything_under(root: &Path) -> BTreeMap<PathBuf, Option<(Vec<u8>, SystemTime)>> {
    let mut found = BTreeMap::new();
    let mut waiting = vec![root.to_path_buf()];
    while let Some(folder) = waiting.pop() {
        for entry in fs::read_dir(&folder).unwrap() {
            let path = entry.unwrap().path();
            let what = fs::symlink_metadata(&path).unwrap();
            let from_root = path.strip_prefix(root).unwrap().to_path_buf();
            if what.is_dir() {
                waiting.push(path);
                found.insert(from_root, None);
            } else {
                found.insert(
                    from_root,
                    Some((fs::read(&path).unwrap(), what.modified().unwrap())),
                );
            }
        }
    }
    found
}

/// The verb's own words, as it declares them.
fn what_it_says(verb: &str) -> (Option<&'static str>, Option<&'static str>) {
    let verbs = file_verbs().unwrap();
    let verb = verbs.of(verb).unwrap();
    (
        verb.what_it_changes().map(Word::says),
        verb.what_it_leaves().map(Word::says),
    )
}

/// A home with an Invoices folder holding the file a change is about and
/// two others, one of them in a folder inside it, and an Archive folder.
fn a_home(what: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root = a_folder_of_our_own(what);
    let invoices = root.join("Invoices");
    let archive = root.join("Archive");
    fs::create_dir_all(invoices.join("2026")).unwrap();
    fs::create_dir_all(&archive).unwrap();
    fs::write(invoices.join("march.pdf"), b"the March invoice").unwrap();
    fs::write(invoices.join("april.pdf"), b"the April invoice").unwrap();
    fs::write(invoices.join("2026/may.pdf"), b"the May invoice").unwrap();
    fs::write(archive.join("old.pdf"), b"an old invoice").unwrap();
    (root, invoices, archive)
}

/// **`rename_file`: "Gives one file a new name, in the folder it is in." and
/// "What is in it stays the same, and no other file is changed or
/// replaced."**
#[test]
fn renaming_does_what_it_says_and_leaves_what_it_says() {
    assert_eq!(
        what_it_says("rename_file"),
        (
            Some("Gives one file a new name, in the folder it is in."),
            Some("What is in it stays the same, and no other file is changed or replaced.")
        ),
        "the words changed without this test"
    );
    assert_eq!(
        Some(words::RENAME_FILE_CHANGES.says()),
        what_it_says("rename_file").0
    );
    let (root, invoices, _) = a_home("renaming");
    let grants = granting(&[&invoices]);
    let before = everything_under(&root);

    changing(
        "rename_file",
        &[
            ("file", as_given(&invoices.join("march.pdf"))),
            ("name", Given::text("invoice-march.pdf")),
        ],
        &grants,
    )
    .unwrap();

    let mut after = everything_under(&root);
    // One file, in the same folder, under the new name.
    let renamed = after
        .remove(Path::new("Invoices/invoice-march.pdf"))
        .unwrap()
        .unwrap();
    let mut expected = before.clone();
    let was = expected
        .remove(Path::new("Invoices/march.pdf"))
        .unwrap()
        .unwrap();
    // What is in it stays the same.
    assert_eq!(renamed.0, was.0, "what is in the file changed");
    // And no other file is changed.
    assert_eq!(
        after, expected,
        "another file was changed, added or removed"
    );

    // No file is replaced: a name already taken is refused and changes nothing.
    let before = everything_under(&root);
    let refused = changing(
        "rename_file",
        &[
            ("file", as_given(&invoices.join("april.pdf"))),
            ("name", Given::text("invoice-march.pdf")),
        ],
        &grants,
    );
    assert!(refused.is_err(), "a file was renamed over another");
    assert_eq!(
        everything_under(&root),
        before,
        "a refused rename changed something"
    );
    let _ = fs::remove_dir_all(&root);
}

/// **`move_file`: "Moves one file into another folder, keeping its name."
/// and "What is in it stays the same, and no other file is changed or
/// replaced."**
#[test]
fn moving_does_what_it_says_and_leaves_what_it_says() {
    assert_eq!(
        what_it_says("move_file"),
        (
            Some("Moves one file into another folder, keeping its name."),
            Some("What is in it stays the same, and no other file is changed or replaced.")
        ),
        "the words changed without this test"
    );
    let (root, invoices, archive) = a_home("moving");
    let grants = granting(&[&invoices, &archive]);
    let before = everything_under(&root);

    changing(
        "move_file",
        &[
            ("file", as_given(&invoices.join("march.pdf"))),
            ("into", as_given(&archive)),
        ],
        &grants,
    )
    .unwrap();

    let mut after = everything_under(&root);
    let moved = after
        .remove(Path::new("Archive/march.pdf"))
        .unwrap()
        .unwrap();
    let mut expected = before.clone();
    let was = expected
        .remove(Path::new("Invoices/march.pdf"))
        .unwrap()
        .unwrap();
    assert_eq!(moved.0, was.0, "what is in the file changed");
    assert_eq!(
        after, expected,
        "another file was changed, added or removed"
    );

    // A file of that name already in the folder is not replaced.
    fs::write(invoices.join("old.pdf"), b"a newer old invoice").unwrap();
    let before = everything_under(&root);
    let refused = changing(
        "move_file",
        &[
            ("file", as_given(&invoices.join("old.pdf"))),
            ("into", as_given(&archive)),
        ],
        &grants,
    );
    assert!(refused.is_err(), "a file was moved over another");
    assert_eq!(
        everything_under(&root),
        before,
        "a refused move changed something"
    );
    let _ = fs::remove_dir_all(&root);
}

/// **`archive_folder`: "Makes one new file: an archive of the folder." and
/// "The folder and everything in it stay as they are, and no file is
/// replaced."**
#[test]
fn archiving_does_what_it_says_and_leaves_what_it_says() {
    assert_eq!(
        what_it_says("archive_folder"),
        (
            Some("Makes one new file: an archive of the folder."),
            Some("The folder and everything in it stay as they are, and no file is replaced.")
        ),
        "the words changed without this test"
    );
    let (root, invoices, archive) = a_home("archiving");
    let grants = granting(&[&invoices, &archive]);
    let before = everything_under(&root);

    changing(
        "archive_folder",
        &[
            ("folder", as_given(&invoices)),
            ("into", as_given(&archive)),
            ("name", Given::text("invoices.zip")),
        ],
        &grants,
    )
    .unwrap();

    let mut after = everything_under(&root);
    // One new file, and it is an archive.
    let made = after
        .remove(Path::new("Archive/invoices.zip"))
        .unwrap()
        .unwrap();
    assert!(
        made.0.starts_with(b"PK\x03\x04"),
        "the new file is not an archive"
    );
    // The folder and everything in it, and everything else, as they were.
    assert_eq!(after, before, "the folder or another file was changed");

    // No file is replaced: an archive name already taken is refused.
    let before = everything_under(&root);
    let refused = changing(
        "archive_folder",
        &[
            ("folder", as_given(&invoices)),
            ("into", as_given(&archive)),
            ("name", Given::text("invoices.zip")),
        ],
        &grants,
    );
    assert!(refused.is_err(), "an archive was written over a file");
    assert_eq!(
        everything_under(&root),
        before,
        "a refused archive changed something"
    );
    let _ = fs::remove_dir_all(&root);
}

/// **Every change this crate declares says what it changes and what it
/// leaves alone**, and every read says neither — so a new change verb cannot
/// arrive without its words and its test.
#[test]
fn every_file_change_says_both_and_every_read_says_neither() {
    let verbs = file_verbs().unwrap();
    let tested = ["rename_file", "move_file", "archive_folder"];
    for verb in verbs.all() {
        let says = (
            verb.what_it_changes().is_some(),
            verb.what_it_leaves().is_some(),
        );
        if verb.effect().waits_for_approval() {
            assert_eq!(says, (true, true), "{} does not say both", verb.name());
            assert!(
                tested.contains(&verb.name()),
                "{} says what it changes and nothing here holds it to that",
                verb.name()
            );
        } else {
            assert_eq!(
                says,
                (false, false),
                "{} reads and says it changes",
                verb.name()
            );
        }
    }
}
