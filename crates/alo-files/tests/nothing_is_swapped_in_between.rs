//! What happens when the disk changes underneath a call that was already
//! allowed — on Linux, where there is something to be done about it.
//!
//! `docs/quirks.md` recorded the gap this file is about: a path is resolved,
//! the grants are asked about where it really leads, and then the file is
//! opened **by that name a second time**. Anything with write access to a
//! folder on the way can put a link there in between, and the second lookup
//! goes somewhere nobody granted. The same shape a second time: a destination
//! is checked for and then renamed onto.
//!
//! # Why these are deterministic and not races
//!
//! A test that raced two threads and hoped would pass on a slow machine and
//! quietly stop testing anything on a fast one. The window here is a real one
//! in the code rather than a timing accident — [`Touching::of`] ends the
//! deciding and [`Did::of`] begins the acting — so a test can stand in the
//! middle of it, make the substitution an attacker would have to win a race to
//! make, and then let the call continue. What is checked is what the call does
//! against a disk that has already changed, which is the whole question.
//!
//! Each substitution is shown to be real before the call resumes: the test
//! reads the path **by name**, the way the old code did, and asserts that this
//! is now the file nobody granted. So a failure here cannot be the fixture
//! having quietly not worked.
//!
//! Two of these were checked against the code as it was — by putting
//! `File::open` back for one run — and they failed: a link put where the file
//! was, and a folder exchanged on the way to it. Some of the rest pass either
//! way, because something else already refused them, and one passes because the
//! gap it is about is still open. Each says which it is where it stands, so
//! that nobody reads this file and believes more of it than is true.
//!
//! # What is still open, and it is in here
//!
//! A read resolves its whole path in one syscall. A rename cannot, and
//! `crates/alo-files/src/opening.rs` argues why that is a question about how
//! wide a turn's kernel boundary is rather than a syscall nobody reached for.
//! So a move out of a folder exchanged after the grants said yes still takes
//! the file from where the link leads, and there is a test here that says so
//! and fails on the day it stops being true.
//!
//! Linux only, and deliberately. `openat2` and `renameat2` are what closes
//! these, `alo OS` runs on Linux, and a test that skipped itself elsewhere
//! would be a test nobody notices has stopped running. The portable half keeps
//! the behaviour it had, and `docs/quirks.md` still says what that costs.

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::{
    Approvals, Authorised, Given, Grant, Grantee, Grants, Proposal, Reach, Refused,
};
use alo_files::{Did, Failed, OnThisMachine, Resolving, Touching, file_verbs, file_words};
use alo_strings::Strings;

/// What a file nobody granted holds, so that a leak is a thing a test can look
/// for rather than a thing it has to reason about.
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
        "alo-swapped-{}-{what}-{}",
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

/// **The first half of a read**: everything the capability model can decide.
///
/// The call is validated, permitted, resolved, and asked about again where the
/// paths really lead. Nothing has touched a disk on the call's behalf, and the
/// substitution the test is about goes after this.
fn checked_read(verb: &str, given: &[(&str, Given)], grants: &Grants) -> Touching {
    let call = file_verbs().unwrap().call(verb, given).unwrap();
    let authorised = Authorised::read(&call, &files(), grants, noon()).unwrap();
    Touching::of(authorised, grants, &OnThisMachine, &in_english()).unwrap()
}

/// **The first half of a change**: the same, with the approval a change needs.
fn checked_change(verb: &str, given: &[(&str, Given)], grants: &Grants) -> Touching {
    let call = file_verbs().unwrap().call(verb, given).unwrap();
    let mut approvals = Approvals::default();
    let id = approvals.propose(Proposal::checked(&call, &files(), grants, noon(), hour()).unwrap());
    let authorised = approvals
        .approve(id, noon())
        .unwrap()
        .redeem(grants, noon())
        .unwrap();
    Touching::of(authorised, grants, &OnThisMachine, &in_english()).unwrap()
}

/// **The second half**: the machine actually doing it, against whatever the
/// disk holds now.
fn then_done(touching: Touching, grants: &Grants) -> Did {
    Did::of(touching, grants, &in_english()).unwrap()
}

/// What the capability model said when it would not even get this far.
fn refused_at_the_check(verb: &str, given: &[(&str, Given)], grants: &Grants) -> Refused {
    let call = file_verbs().unwrap().call(verb, given).unwrap();
    let authorised = Authorised::read(&call, &files(), grants, noon()).unwrap();
    Touching::of(authorised, grants, &OnThisMachine, &in_english()).unwrap_err()
}

/// A folder holding a file nobody granted, next to the granted one.
fn somewhere_nobody_granted(root: &Path) -> PathBuf {
    let elsewhere = root.join("Elsewhere");
    fs::create_dir_all(&elsewhere).unwrap();
    fs::write(elsewhere.join("march.pdf"), NOBODY_GRANTED_THIS).unwrap();
    elsewhere
}

/// Whatever the call answered or complained about, as one string, so that a
/// test can assert the ungranted bytes are in none of it.
fn everything_it_said(did: &Did) -> String {
    match (did.answer(), did.failure()) {
        (Some(answer), _) => format!("{answer:?}"),
        (None, Some(failed)) => format!("{failed:?} {}", said(failed)),
        (None, None) => String::new(),
    }
}

/// **The last component, swapped after the grants said yes.**
///
/// The file that was resolved and approved is replaced by a link to one that
/// was not. Opening by name would follow it; opening from a handle with
/// `O_NOFOLLOW` refuses, and the person is told the machine would not read it.
#[test]
fn a_link_put_where_the_file_was_is_refused_rather_than_read() {
    let root = a_folder_of_our_own("last-component");
    let invoices = root.join("Invoices");
    fs::create_dir_all(&invoices).unwrap();
    let file = invoices.join("march.pdf");
    fs::write(&file, "an invoice, for March").unwrap();
    let elsewhere = somewhere_nobody_granted(&root);

    let grants = granting(&[&invoices]);
    let touching = checked_read("read_file", &[("file", as_given(&file))], &grants);

    // The moment an attacker would have to win a race for.
    fs::remove_file(&file).unwrap();
    symlink(elsewhere.join("march.pdf"), &file).unwrap();
    // The substitution really is in place: by name, this path is now the file
    // nobody granted, which is exactly what the old code would have read.
    assert_eq!(fs::read_to_string(&file).unwrap(), NOBODY_GRANTED_THIS);

    let did = then_done(touching, &grants);

    let failed = did.failure().unwrap();
    assert!(
        matches!(failed, Failed::TheMachineSaidNo { .. }),
        "{failed:?}"
    );
    assert!(did.answer().is_none());
    assert!(
        !everything_it_said(&did).contains(NOBODY_GRANTED_THIS),
        "the ungranted file's bytes came back: {}",
        everything_it_said(&did)
    );
    // And the person is told about the path they named, in words.
    assert!(said(failed).contains("march.pdf"), "{}", said(failed));

    let _ = fs::remove_dir_all(&root);
}

/// **A folder on the way, swapped after the grants said yes** — the half that
/// `O_NOFOLLOW` on the file alone does not cover.
///
/// `march.pdf` is never touched. `Invoices` becomes a link to a folder nobody
/// granted, and the file of that name inside it is what the second lookup would
/// find. Every component is opened from the handle for the one before it, so
/// the walk stops at `Invoices` and nothing under it is ever opened.
#[test]
fn a_folder_on_the_way_swapped_for_a_link_is_refused_rather_than_walked_through() {
    let root = a_folder_of_our_own("on-the-way");
    let invoices = root.join("Invoices");
    fs::create_dir_all(&invoices).unwrap();
    let file = invoices.join("march.pdf");
    fs::write(&file, "an invoice, for March").unwrap();
    let elsewhere = somewhere_nobody_granted(&root);

    let grants = granting(&[&invoices]);
    let touching = checked_read("read_file", &[("file", as_given(&file))], &grants);

    // The folder itself is exchanged, and the file inside it is untouched.
    fs::rename(&invoices, root.join("Invoices.moved-aside")).unwrap();
    symlink(&elsewhere, &invoices).unwrap();
    assert_eq!(fs::read_to_string(&file).unwrap(), NOBODY_GRANTED_THIS);

    let did = then_done(touching, &grants);

    let failed = did.failure().unwrap();
    assert!(
        matches!(failed, Failed::TheMachineSaidNo { .. }),
        "{failed:?}"
    );
    assert!(
        !everything_it_said(&did).contains(NOBODY_GRANTED_THIS),
        "a link on the way was followed: {}",
        everything_it_said(&did)
    );

    let _ = fs::remove_dir_all(&root);
}

/// The same substitution against the verb that walks.
///
/// **This one does not need the handle walk and is here anyway.** The archive
/// verb asks what the folder is before it walks it, and a folder that has
/// become a link is not a folder — so this was already refused, and it still
/// is. It is written down because *what stops it* is now two things rather than
/// one, and a change to either should be met by a test that noticed.
#[test]
fn a_folder_on_the_way_swapped_for_a_link_is_refused_before_an_archive_is_finished() {
    let root = a_folder_of_our_own("archiving");
    let invoices = root.join("Invoices");
    let keep = root.join("Archive");
    fs::create_dir_all(&invoices).unwrap();
    fs::create_dir_all(&keep).unwrap();
    fs::write(invoices.join("march.pdf"), "an invoice, for March").unwrap();
    let elsewhere = somewhere_nobody_granted(&root);

    let grants = granting(&[&invoices, &keep]);
    let touching = checked_change(
        "archive_folder",
        &[
            ("folder", as_given(&invoices)),
            ("into", as_given(&keep)),
            ("name", Given::text("invoices.zip")),
        ],
        &grants,
    );

    fs::rename(&invoices, root.join("Invoices.moved-aside")).unwrap();
    symlink(&elsewhere, &invoices).unwrap();

    let did = then_done(touching, &grants);

    assert!(
        !everything_it_said(&did).contains(NOBODY_GRANTED_THIS),
        "{}",
        everything_it_said(&did)
    );
    // Whatever was made, it does not hold what was never granted.
    let made = keep.join("invoices.zip");
    if made.exists() {
        let bytes = fs::read(&made).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(
            !text.contains(NOBODY_GRANTED_THIS),
            "an archive was made of a folder nobody granted"
        );
    }

    let _ = fs::remove_dir_all(&root);
}

/// **A destination that appears after the grants said yes is still not
/// replaced.**
///
/// The name is free when the call is approved and taken by the time it runs.
/// One call both refuses and moves, so there is no moment in between for this
/// to be decided in — and what was already there keeps its own bytes.
///
/// The window the old code lost was microseconds wide and not one a test can
/// stand in, so this passes either way. What it holds down is the guarantee
/// rather than the fix: if the check ever moves earlier again — into the
/// deciding half, where a test *can* stand in it — this is what fails.
#[test]
fn a_destination_that_appears_after_the_check_is_not_replaced() {
    let root = a_folder_of_our_own("appeared");
    let invoices = root.join("Invoices");
    fs::create_dir_all(&invoices).unwrap();
    let file = invoices.join("march.pdf");
    fs::write(&file, "an invoice, for March").unwrap();
    let taken = invoices.join("march-2026.pdf");

    let grants = granting(&[&invoices]);
    let touching = checked_change(
        "rename_file",
        &[
            ("file", as_given(&file)),
            ("name", Given::text("march-2026.pdf")),
        ],
        &grants,
    );

    // Free when it was approved, taken by the time it runs.
    fs::write(&taken, "somebody else's invoice").unwrap();

    let did = then_done(touching, &grants);

    let failed = did.failure().unwrap();
    assert!(matches!(failed, Failed::AlreadyThere { .. }), "{failed:?}");
    assert!(
        said(failed).contains("choose another name"),
        "{}",
        said(failed)
    );
    // Nothing was replaced and nothing was lost: both files are as they were.
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        "an invoice, for March",
        "the file was moved anyway"
    );
    assert_eq!(
        fs::read_to_string(&taken).unwrap(),
        "somebody else's invoice",
        "the file that was already there was replaced"
    );

    let _ = fs::remove_dir_all(&root);
}

/// A name taken by a **folder**, or by a link that leads nowhere at all, is a
/// name that is taken. Neither is replaced, and the refusal is the one a person
/// reads about a name being in use.
#[test]
fn a_name_held_by_a_folder_or_by_a_link_to_nothing_is_a_name_that_is_taken() {
    for (what, make) in [
        (
            "folder",
            (|at: &Path| fs::create_dir_all(at).unwrap()) as fn(&Path),
        ),
        (
            "dangling-link",
            (|at: &Path| symlink(at.with_file_name("nothing-at-all.pdf"), at).unwrap())
                as fn(&Path),
        ),
    ] {
        let root = a_folder_of_our_own(what);
        let invoices = root.join("Invoices");
        fs::create_dir_all(&invoices).unwrap();
        let file = invoices.join("march.pdf");
        fs::write(&file, "an invoice, for March").unwrap();

        let grants = granting(&[&invoices]);
        let touching = checked_change(
            "rename_file",
            &[
                ("file", as_given(&file)),
                ("name", Given::text("march-2026.pdf")),
            ],
            &grants,
        );

        let taken = invoices.join("march-2026.pdf");
        make(&taken);

        let did = then_done(touching, &grants);
        let failed = did.failure().unwrap();
        assert!(
            matches!(failed, Failed::AlreadyThere { .. }),
            "{what}: {failed:?}"
        );
        assert_eq!(fs::read_to_string(&file).unwrap(), "an invoice, for March");
        assert!(
            fs::symlink_metadata(&taken).is_ok(),
            "{what}: what was there is gone"
        );

        let _ = fs::remove_dir_all(&root);
    }
}

/// **The one this does not close, measured rather than described.**
///
/// A read resolves its whole path inside one syscall that refuses a link at
/// every component. A rename cannot: `renameat2` has no such flag, and the
/// alternative — handles on the two folders — means opening them, and the
/// folder a move takes a file *out of* is not a place its call named, so a
/// turn's boundary refuses that open. `crates/alo-files/src/opening.rs` argues
/// it and `docs/quirks.md` keeps it.
///
/// So this is what actually happens today: the folder is exchanged after the
/// grants said yes, and the move takes the file from where the link leads.
/// Asserting it is not approval of it. It is the difference between a gap
/// somebody measured and a gap somebody assumed, and the day the gap closes
/// this test fails and whoever closed it has to say so here, in
/// `docs/quirks.md`, and in `ROADMAP.md`.
#[test]
fn a_move_out_of_a_folder_swapped_for_a_link_is_not_yet_refused() {
    let root = a_folder_of_our_own("moving-out");
    let invoices = root.join("Invoices");
    let keep = root.join("Archive");
    fs::create_dir_all(&invoices).unwrap();
    fs::create_dir_all(&keep).unwrap();
    let file = invoices.join("march.pdf");
    fs::write(&file, "an invoice, for March").unwrap();
    let elsewhere = somewhere_nobody_granted(&root);

    let grants = granting(&[&invoices, &keep]);
    let touching = checked_change(
        "move_file",
        &[("file", as_given(&file)), ("into", as_given(&keep))],
        &grants,
    );

    fs::rename(&invoices, root.join("Invoices.moved-aside")).unwrap();
    symlink(&elsewhere, &invoices).unwrap();

    let did = then_done(touching, &grants);

    assert!(
        did.failure().is_none(),
        "the gap this test measures has closed — say so in docs/quirks.md, in \
         crates/alo-files/src/opening.rs, and in ROADMAP.md, and make this a \
         refusal: {:?}",
        did.failure()
    );
    assert_eq!(
        fs::read_to_string(keep.join("march.pdf")).unwrap(),
        NOBODY_GRANTED_THIS,
        "the gap this test measures has closed"
    );

    let _ = fs::remove_dir_all(&root);
}

/// **A rename does close it**, because a rename's two folders are one folder,
/// and the file it is given is what gets moved rather than what a name leads
/// to now. The final component is never followed by `renameat2`, so a link put
/// where the file was is moved as the link it is — and what it pointed at stays
/// where it is.
#[test]
fn a_rename_moves_the_link_that_was_put_there_and_not_what_it_points_at() {
    let root = a_folder_of_our_own("renaming-a-link");
    let invoices = root.join("Invoices");
    fs::create_dir_all(&invoices).unwrap();
    let file = invoices.join("march.pdf");
    fs::write(&file, "an invoice, for March").unwrap();
    let elsewhere = somewhere_nobody_granted(&root);

    let grants = granting(&[&invoices]);
    let touching = checked_change(
        "rename_file",
        &[
            ("file", as_given(&file)),
            ("name", Given::text("march-2026.pdf")),
        ],
        &grants,
    );

    fs::remove_file(&file).unwrap();
    symlink(elsewhere.join("march.pdf"), &file).unwrap();

    let did = then_done(touching, &grants);

    // Either it refused, or it moved the link. What it must not have done is
    // reach through the link to the file nobody granted.
    assert_eq!(
        fs::read_to_string(elsewhere.join("march.pdf")).unwrap(),
        NOBODY_GRANTED_THIS,
        "the file nobody granted was moved"
    );
    assert!(
        elsewhere.join("march.pdf").is_file(),
        "the file nobody granted left the folder it was in"
    );
    let _ = did;

    let _ = fs::remove_dir_all(&root);
}

/// **A link that was there all along is still the grants' refusal to make.**
///
/// This is what closing the race must not change. A path that leads outside a
/// grant at the moment it is resolved is refused by the capability model, in
/// its own words, before anything touches a disk — not by the kernel, and not
/// as *the machine would not*. A security review reads those two facts
/// differently and they are different facts.
#[test]
fn a_link_that_was_always_there_is_still_refused_by_the_grants_and_not_by_the_kernel() {
    let root = a_folder_of_our_own("always-there");
    let invoices = root.join("Invoices");
    fs::create_dir_all(&invoices).unwrap();
    let elsewhere = somewhere_nobody_granted(&root);
    let file = invoices.join("march.pdf");
    symlink(elsewhere.join("march.pdf"), &file).unwrap();

    let grants = granting(&[&invoices]);
    let refused = refused_at_the_check("read_file", &[("file", as_given(&file))], &grants);
    let words = refused.said(&in_english()).into_text();

    assert!(words.contains("really leads"), "{words}");
    assert!(!words.contains(NOBODY_GRANTED_THIS), "{words}");

    let _ = fs::remove_dir_all(&root);
}

/// The ordinary day, on the machine alo OS runs on: nothing was substituted,
/// and all three of these do what they were asked to.
#[test]
fn with_nothing_swapped_the_verbs_still_do_what_they_were_asked_to() {
    let root = a_folder_of_our_own("ordinary");
    let invoices = root.join("Invoices");
    let keep = root.join("Archive");
    fs::create_dir_all(&invoices).unwrap();
    fs::create_dir_all(&keep).unwrap();
    let file = invoices.join("march.pdf");
    fs::write(&file, "an invoice, for March").unwrap();

    let grants = granting(&[&invoices, &keep]);

    let read = then_done(
        checked_read("read_file", &[("file", as_given(&file))], &grants),
        &grants,
    );
    assert_eq!(read.answer().unwrap().read(), Some("an invoice, for March"));

    let renamed = then_done(
        checked_change(
            "rename_file",
            &[
                ("file", as_given(&file)),
                ("name", Given::text("march-2026.pdf")),
            ],
            &grants,
        ),
        &grants,
    );
    let now_at = invoices.join("march-2026.pdf");
    assert_eq!(renamed.answer().unwrap().now_at(), Some(now_at.as_path()));
    assert!(!file.exists());

    let moved = then_done(
        checked_change(
            "move_file",
            &[("file", as_given(&now_at)), ("into", as_given(&keep))],
            &grants,
        ),
        &grants,
    );
    let ended_at = keep.join("march-2026.pdf");
    assert_eq!(moved.answer().unwrap().now_at(), Some(ended_at.as_path()));
    assert_eq!(
        fs::read_to_string(&ended_at).unwrap(),
        "an invoice, for March"
    );
    assert!(!now_at.exists());

    let _ = fs::remove_dir_all(&root);
}

/// A file that went away between the deciding and the acting is *gone*, and not
/// the machine's own words about it — the one error kind this crate reads
/// rather than repeats, and it survives being reached through a handle.
#[test]
fn a_file_that_went_away_between_the_two_halves_is_answered_as_gone() {
    let root = a_folder_of_our_own("went-away");
    let invoices = root.join("Invoices");
    fs::create_dir_all(&invoices).unwrap();
    let file = invoices.join("march.pdf");
    fs::write(&file, "an invoice, for March").unwrap();

    let grants = granting(&[&invoices]);
    let touching = checked_read("read_file", &[("file", as_given(&file))], &grants);
    fs::remove_file(&file).unwrap();

    let did = then_done(touching, &grants);
    let failed = did.failure().unwrap();
    assert!(matches!(failed, Failed::Gone { .. }), "{failed:?}");
    assert!(said(failed).contains("march.pdf"), "{}", said(failed));

    let _ = fs::remove_dir_all(&root);
}
