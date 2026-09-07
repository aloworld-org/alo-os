//! A hard link inside a granted folder, driven through the whole journey.
//!
//! Everything else `alo-files` does about links is about **symbolic** ones: a
//! path is resolved, and a link out of a granted folder resolves to somewhere
//! the grants refuse. A **hard** link is not a link in that sense. It is a
//! second real name for one file, so `/home/anna/Invoices/notes.txt` can be a
//! name for a file that also lives in `/home/anna/.ssh`, and resolving it
//! answers with the granted name — because the granted name *genuinely is* a
//! real name for that file.
//!
//! No comparison of paths can see that, and `docs/quirks.md` has said so since
//! 2026-09-02. What a file will answer is **how many names it has**, asked of
//! the open handle, and a file with more than one is not read.
//!
//! # What this is worth, stated in proportion
//!
//! Making a hard link needs write access to the granted folder and read access
//! to the file, so this is not a way *in*: somebody who can already read the
//! file and write to the folder could read it anyway. What it is, is a way to
//! widen **what an agent sees** without widening any grant — and what an agent
//! sees can leave the machine, while the record of the turn names only a path
//! the person granted. That is the *record becomes an observation rather than a
//! claim* line of ADR 0013, met from the other side.
//!
//! Neither layer beneath this catches it. The capability model cannot, for the
//! reason above; and the kernel boundary does not either, because it decides an
//! open by walking up from the *file's own* directory entry — and a hard link's
//! entry sits in the granted folder. `alo-bounding`'s
//! `a_hard_link_is_inside_every_boundary` measures that rather than assuming
//! it. So this check is the only thing standing there, which is why it refuses
//! rather than reports.
//!
//! # Unix only, because a hard link is what is being made
//!
//! `std::fs::hard_link` exists on Windows and NTFS supports them, but `std`
//! cannot count a file's names there without a call it does not expose — so the
//! check answers *not that we can tell* on Windows and there would be nothing
//! for these tests to assert. That gap is in `docs/quirks.md`, named rather
//! than skipped over.

#![cfg(unix)]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::{Approvals, Authorised, Given, Grant, Grantee, Grants, Proposal, Reach};
use alo_files::{Did, Failed, OnThisMachine, Resolving, Touching, file_verbs, file_words};
use alo_strings::Strings;

/// What the file nobody granted holds, so that a leak is something a test can
/// look for rather than something it has to reason about.
const NOBODY_GRANTED_THIS: &str = "the bytes of a file nobody granted";

/// A fixed moment, so that expiry is arithmetic rather than a wait.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the grants and the approvals in these tests last.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The agent the tests grant things to.
fn files() -> Grantee {
    Grantee::named("@files")
}

/// A folder of this test's own, resolved — a grant is made over a resolved
/// path, which `docs/quirks.md` explains.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-another-name-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    OnThisMachine.real(&folder).unwrap().into_path_buf()
}

/// Grants to `@files` over these folders, made at noon and lasting an hour.
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

/// A path as a verb's argument arrives: text.
fn as_given(path: &Path) -> Given {
    Given::text(path.to_string_lossy().into_owned())
}

/// The strings this machine reads, with nothing translated.
fn in_english() -> Strings {
    let mut vocabulary = file_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// What a failure says on such a machine.
fn said(failed: &Failed) -> String {
    failed.said(&in_english()).into_text()
}

/// A read, done: validated, permitted, resolved, and performed.
fn looking(verb: &str, given: &[(&str, Given)], grants: &Grants) -> Did {
    let call = file_verbs().unwrap().call(verb, given).unwrap();
    let authorised = Authorised::read(&call, &files(), grants, noon()).unwrap();
    let touching = Touching::of(authorised, grants, &OnThisMachine, &in_english()).unwrap();
    Did::of(touching, grants, &in_english()).unwrap()
}

/// A change, all the way through: proposed, approved once, redeemed, resolved,
/// and performed.
fn changing(verb: &str, given: &[(&str, Given)], grants: &Grants) -> Did {
    let call = file_verbs().unwrap().call(verb, given).unwrap();
    let mut approvals = Approvals::default();
    let id = approvals.propose(Proposal::checked(&call, &files(), grants, noon(), hour()).unwrap());
    let authorised = approvals
        .approve(id, noon())
        .unwrap()
        .redeem(grants, noon())
        .unwrap();
    let touching = Touching::of(authorised, grants, &OnThisMachine, &in_english()).unwrap();
    Did::of(touching, grants, &in_english()).unwrap()
}

/// A granted folder holding a second name for a file that lives outside it.
///
/// Returns the root, the granted folder, and the granted name for the file
/// nobody granted.
fn a_granted_folder_holding_somebody_elses_file(what: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root = a_folder_of_our_own(what);
    let granted = root.join("Invoices");
    let private = root.join("Private");
    fs::create_dir_all(&granted).unwrap();
    fs::create_dir_all(&private).unwrap();
    fs::write(private.join("secret.txt"), NOBODY_GRANTED_THIS).unwrap();

    let second_name = granted.join("notes.txt");
    fs::hard_link(private.join("secret.txt"), &second_name).unwrap();
    // The fixture really is what it claims: two names, one file, and reading
    // the granted one by name gives the bytes of the other. This is what the
    // old code did, and it is why the test below is worth having.
    assert_eq!(
        fs::read_to_string(&second_name).unwrap(),
        NOBODY_GRANTED_THIS
    );

    (root, granted, second_name)
}

/// **The whole point.** The path is granted, resolves to itself, passes every
/// check the capability model can make — and is not read, because the file has
/// another name and the file cannot say where it is.
#[test]
fn a_granted_name_for_somebody_elses_file_is_not_read() {
    let (root, granted, second_name) = a_granted_folder_holding_somebody_elses_file("read");
    let grants = granting(&[&granted]);

    let did = looking("read_file", &[("file", as_given(&second_name))], &grants);

    let failed = did.failure().unwrap();
    assert!(
        matches!(failed, Failed::HasAnotherName { .. }),
        "{failed:?}"
    );
    assert!(did.answer().is_none());
    assert!(
        !format!("{did:?}").contains(NOBODY_GRANTED_THIS),
        "the ungranted file's bytes came back"
    );

    let _ = fs::remove_dir_all(&root);
}

/// It is a refusal by **the machine's side of the house**, not by the grants.
///
/// The distinction is `failed.rs`'s whole subject and it matters here more than
/// usual: the grants really did permit this path, and saying they refused it
/// would tell a security review the capability model caught something it cannot
/// catch. So this arrives as a [`Failed`] inside a [`Did`], and the
/// authorisation comes back beside it, because the call was permitted,
/// approved and attempted.
#[test]
fn it_is_recorded_as_something_the_machine_would_not_do_rather_than_a_refusal() {
    let (root, granted, second_name) = a_granted_folder_holding_somebody_elses_file("record");
    let grants = granting(&[&granted]);

    let did = looking("read_file", &[("file", as_given(&second_name))], &grants);

    assert_eq!(did.authorised().verb(), "read_file");
    assert!(did.failure().is_some());

    let _ = fs::remove_dir_all(&root);
}

/// What a person reads: it says the file has other names, that it was not read,
/// and what to do instead. Every sentence in this crate has to end somewhere a
/// person can act on.
#[test]
fn the_person_is_told_which_file_and_what_to_do_about_it() {
    let (root, granted, second_name) = a_granted_folder_holding_somebody_elses_file("said");
    let grants = granting(&[&granted]);

    let did = looking("read_file", &[("file", as_given(&second_name))], &grants);
    let words = said(did.failure().unwrap());

    assert!(words.contains("notes.txt"), "{words}");
    assert!(words.contains("was not read"), "{words}");
    assert!(words.contains("copy"), "{words}");
    // And it does not name the file nobody granted, which this crate has no
    // business knowing and could not find out anyway.
    assert!(!words.contains("secret.txt"), "{words}");

    let _ = fs::remove_dir_all(&root);
}

/// **An archive is made of bytes too**, so the same file stops it.
///
/// Leaving the file out quietly would be the other way to be wrong, and
/// `archiving.rs`'s own rule settles it: a bound refused in words beats an
/// archive that is missing a document nobody mentioned.
#[test]
fn an_archive_of_the_folder_is_refused_rather_than_made_without_it() {
    let (root, granted, _second_name) = a_granted_folder_holding_somebody_elses_file("archive");
    let keep = root.join("Archive");
    fs::create_dir_all(&keep).unwrap();
    let grants = granting(&[&granted, &keep]);

    let did = changing(
        "archive_folder",
        &[
            ("folder", as_given(&granted)),
            ("into", as_given(&keep)),
            ("name", Given::text("invoices.zip")),
        ],
        &grants,
    );

    let failed = did.failure().unwrap();
    assert!(
        matches!(failed, Failed::HasAnotherName { .. }),
        "{failed:?}"
    );
    // Whatever is left on the disk does not hold what was never granted.
    if let Ok(made) = fs::read(keep.join("invoices.zip")) {
        assert!(
            !String::from_utf8_lossy(&made).contains(NOBODY_GRANTED_THIS),
            "an archive was made holding a file nobody granted"
        );
    }

    let _ = fs::remove_dir_all(&root);
}

/// The ordinary day is untouched: a file with one name reads, and a folder —
/// which every filesystem gives at least two names, its own and the `.` inside
/// it — is listed and archived as it always was.
///
/// This is the test that would fail if the count were asked of the wrong thing,
/// and it would fail for every folder on the machine.
#[test]
fn a_folder_and_an_ordinary_file_are_untouched_by_this() {
    let root = a_folder_of_our_own("ordinary");
    let granted = root.join("Invoices");
    let keep = root.join("Archive");
    fs::create_dir_all(granted.join("2026")).unwrap();
    fs::create_dir_all(&keep).unwrap();
    fs::write(granted.join("march.pdf"), "an invoice, for March").unwrap();
    let grants = granting(&[&granted, &keep]);

    let read = looking(
        "read_file",
        &[("file", as_given(&granted.join("march.pdf")))],
        &grants,
    );
    assert_eq!(
        read.answer().unwrap().read(),
        Some("an invoice, for March"),
        "{:?}",
        read.failure()
    );

    let listed = looking("list_folder", &[("folder", as_given(&granted))], &grants);
    assert!(listed.answer().is_some(), "{:?}", listed.failure());

    // The folder has a subfolder, so on Unix it has three names by the
    // filesystem's own counting. It is still archived.
    let archived = changing(
        "archive_folder",
        &[
            ("folder", as_given(&granted)),
            ("into", as_given(&keep)),
            ("name", Given::text("invoices.zip")),
        ],
        &grants,
    );
    assert!(archived.answer().is_some(), "{:?}", archived.failure());
    assert!(keep.join("invoices.zip").is_file());

    let _ = fs::remove_dir_all(&root);
}

/// Two names **both inside the granted folder** are refused as well, and this
/// test exists to say that out loud rather than to celebrate it.
///
/// Nothing escapes in this case: both names are granted, and the refusal costs
/// somebody a file they were entitled to read. It is the price of the only
/// question a file will answer — *how many names* — with no way to ask *and
/// where are they*, which would be a scan of every filesystem the file could be
/// on. Refusing is the safe direction and `docs/quirks.md` records the cost.
#[test]
fn two_names_inside_the_same_granted_folder_are_refused_too_and_that_is_the_cost() {
    let root = a_folder_of_our_own("both-inside");
    let granted = root.join("Invoices");
    fs::create_dir_all(&granted).unwrap();
    fs::write(granted.join("march.pdf"), "an invoice, for March").unwrap();
    fs::hard_link(granted.join("march.pdf"), granted.join("march-copy.pdf")).unwrap();
    let grants = granting(&[&granted]);

    let did = looking(
        "read_file",
        &[("file", as_given(&granted.join("march.pdf")))],
        &grants,
    );

    let failed = did.failure().unwrap();
    assert!(
        matches!(failed, Failed::HasAnotherName { .. }),
        "{failed:?}"
    );

    let _ = fs::remove_dir_all(&root);
}
