//! The whole journey, on a real filesystem: a declared verb, a call, a grant,
//! an approval where one is needed, the real path, and the machine actually
//! doing it.
//!
//! The crate's own tests take each half apart. This file is the other half of
//! that bargain — every step in the order `alo-agentd` will run them in, ending
//! in files that really moved on the disk the tests are running on.
//!
//! It is not the hardware verification `CLAUDE.md` asks for: that is a
//! certified machine, and this is whatever the tests were run on.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#![expect(
    clippy::indexing_slicing,
    reason = "an archive read back by the offsets its format states is the test; a wrong offset should fail here"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::{
    Approvals, Arg, Authorised, Effect, Given, Grant, Grantee, Grants, Proposal, Reach, Requires,
    Takes, Value, Verb, Verbs,
};
use alo_files::{
    Answer, Did, Failed, Kind, Named, OnThisMachine, Real, Resolving, Touching, file_verbs,
    file_words,
};
use alo_strings::{Strings, Word};

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
/// path, which `docs/quirks.md` explains and Windows insists on.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-doing-{}-{what}-{}",
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

/// This crate's words, with nothing translated — the machine a shell that has
/// loaded no translations is running, and the one every step below refuses on.
/// The strings this machine reads: **every crate's**, in one vocabulary.
///
/// That is the arrangement a shell has — `alo_strings::Vocabulary` holds one
/// area per crate — and it is what these tests need, because a refusal a person
/// meets here can have been worded by either crate: the file half words *this
/// path really leads elsewhere*, and the capability model words *nobody granted
/// it*.
fn in_english() -> Strings {
    let mut vocabulary = file_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
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
fn changing(verb: &str, given: &[(&str, Given)], grants: &Grants) -> Result<Did, String> {
    let call = file_verbs().unwrap().call(verb, given).unwrap();
    let mut approvals = Approvals::default();
    let id = approvals.propose(Proposal::checked(&call, &files(), grants, noon(), hour()).unwrap());
    let authorised = approvals
        .approve(id, noon())
        .unwrap()
        .redeem(grants, noon())
        .unwrap();
    let touching = Touching::of(authorised, grants, &OnThisMachine, &in_english())
        .map_err(|why| why.said(&in_english()).into_text())?;
    Did::of(touching, grants, &in_english()).map_err(|why| why.said(&in_english()).into_text())
}

/// The ordinary day, from a declared verb to what is really in the folder.
#[test]
fn a_read_answers_with_what_is_really_in_the_folder() {
    let root = a_folder_of_our_own("listing");
    let invoices = root.join("Invoices");
    fs::create_dir_all(invoices.join("2026")).unwrap();
    fs::write(invoices.join("march.pdf"), b"an invoice").unwrap();

    let grants = granting(&[&invoices]);
    let did = looking("list_folder", &[("folder", as_given(&invoices))], &grants);

    let listing = did.answer().unwrap().listed().unwrap();
    let names: Vec<_> = listing.things().iter().map(Named::name).collect();
    assert_eq!(names, ["2026", "march.pdf"]);
    assert_eq!(
        listing.things().first().map(Named::kind),
        Some(Kind::Folder)
    );
    assert!(!listing.was_cut_short());
    // A read needs no approval, and the record will say so by the absence.
    assert!(did.authorised().from_approval().is_none());
    assert_eq!(did.authorised().verb(), "list_folder");

    let _ = fs::remove_dir_all(&root);
}

/// Reading and finding, on the same folder, answering about the same files.
#[test]
fn reading_and_finding_answer_about_the_files_that_are_there() {
    let root = a_folder_of_our_own("reading");
    let invoices = root.join("Invoices");
    fs::create_dir_all(invoices.join("2026")).unwrap();
    fs::write(invoices.join("2026/march.pdf"), "an invoice, for March").unwrap();
    fs::write(invoices.join("taxes.txt"), "not an invoice").unwrap();

    let grants = granting(&[&invoices]);

    let found = looking(
        "find_in_folder",
        &[
            ("folder", as_given(&invoices)),
            ("named", Given::text("march")),
            ("most", Given::number(10)),
        ],
        &grants,
    );
    let search = found.answer().unwrap().found().unwrap();
    assert_eq!(search.files().len(), 1, "{:?}", search.files());
    assert!(!search.was_cut_short());

    let file = search.files().first().unwrap().clone();
    let read = looking("read_file", &[("file", as_given(&file))], &grants);
    assert_eq!(read.answer().unwrap().read(), Some("an invoice, for March"));

    let _ = fs::remove_dir_all(&root);
}

/// A change moves a real file, once, after one approval — and what comes back
/// is what the record is written from.
#[test]
fn a_change_moves_a_real_file_after_one_approval() {
    let root = a_folder_of_our_own("moving");
    let invoices = root.join("Invoices");
    let archive = root.join("Archive");
    fs::create_dir_all(&invoices).unwrap();
    fs::create_dir_all(&archive).unwrap();
    let file = invoices.join("march.pdf");
    fs::write(&file, b"an invoice").unwrap();

    let grants = granting(&[&invoices, &archive]);
    let did = changing(
        "move_file",
        &[("file", as_given(&file)), ("into", as_given(&archive))],
        &grants,
    )
    .unwrap();

    assert_eq!(
        did.answer().unwrap().now_at(),
        Some(archive.join("march.pdf").as_path())
    );
    assert!(!file.exists(), "the file is still in the folder it left");
    assert_eq!(fs::read(archive.join("march.pdf")).unwrap(), b"an invoice");
    assert!(did.authorised().from_approval().is_some());
    assert_eq!(did.authorised().against().len(), 2);

    let _ = fs::remove_dir_all(&root);
}

/// **A grant covers where a file goes, not only where it comes from.**
///
/// The document offered at invocation is a grant over one file (ADR 0001 §4).
/// Renaming under one would put a file at a name nobody granted, so the grants
/// are asked about what would be created and this is refused — as a refusal by
/// the grants, into the record beside every other one, with nothing touched.
#[test]
fn a_change_that_would_create_a_name_nobody_granted_is_refused() {
    let root = a_folder_of_our_own("creating");
    let file = root.join("march.pdf");
    fs::write(&file, b"an invoice").unwrap();

    let mut grants = Grants::default();
    grants.grant(Grant::checked("@files", Reach::File(file.clone()), noon(), hour()).unwrap());

    let refused = changing(
        "rename_file",
        &[
            ("file", as_given(&file)),
            ("name", Given::text("march-2026.pdf")),
        ],
        &grants,
    )
    .unwrap_err();

    assert!(refused.contains("march-2026.pdf"), "{refused}");
    assert!(refused.contains("has not been granted"), "{refused}");
    assert!(
        refused.contains("not only where it comes from"),
        "{refused}"
    );
    // Nothing happened: the file is where it was, under the name it had.
    assert_eq!(fs::read(&file).unwrap(), b"an invoice");
    assert!(!root.join("march-2026.pdf").exists());

    let _ = fs::remove_dir_all(&root);
}

/// The same rename, under a grant over the folder, is exactly what a person
/// meant to allow — so the refusal above is about reach and not about renaming.
#[test]
fn the_same_rename_under_a_grant_over_the_folder_is_done() {
    let root = a_folder_of_our_own("renaming");
    let file = root.join("march.pdf");
    fs::write(&file, b"an invoice").unwrap();

    let grants = granting(&[&root]);
    let did = changing(
        "rename_file",
        &[
            ("file", as_given(&file)),
            ("name", Given::text("march-2026.pdf")),
        ],
        &grants,
    )
    .unwrap();

    assert_eq!(
        did.answer().unwrap().now_at(),
        Some(root.join("march-2026.pdf").as_path())
    );
    assert!(!file.exists());
    assert_eq!(
        fs::read(root.join("march-2026.pdf")).unwrap(),
        b"an invoice"
    );

    let _ = fs::remove_dir_all(&root);
}

/// An archive of a granted folder is one file, in the format alo OS makes, with
/// the folder inside it — read back here by the offsets the format states.
#[test]
fn an_archive_is_a_zip_that_holds_the_folder() {
    let root = a_folder_of_our_own("archiving");
    let invoices = root.join("Invoices");
    let keep = root.join("Archive");
    fs::create_dir_all(invoices.join("2026")).unwrap();
    fs::create_dir_all(&keep).unwrap();
    fs::write(invoices.join("2026/march.pdf"), b"an invoice").unwrap();
    fs::write(invoices.join("taxes.txt"), b"not an invoice").unwrap();

    let grants = granting(&[&invoices, &keep]);
    let did = changing(
        "archive_folder",
        &[
            ("folder", as_given(&invoices)),
            ("into", as_given(&keep)),
            ("name", Given::text("invoices-2026.zip")),
        ],
        &grants,
    )
    .unwrap();

    let archived = did.answer().unwrap().archived().unwrap();
    assert_eq!(archived.at(), keep.join("invoices-2026.zip"));
    assert_eq!(archived.things(), 3, "two files and the folder holding one");
    assert_eq!(archived.left_out(), 0);

    let inside = inside_the_archive(archived.at());
    assert_eq!(
        inside,
        ["2026/", "taxes.txt", "2026/march.pdf"],
        "an archive holds the folder as it was — every folder before the things \
         inside it, spelled the way the format spells it"
    );

    let _ = fs::remove_dir_all(&root);
}

/// An archive whose name does not say what it is, is refused before anything is
/// written: a zip called `invoices` is a file whose name lies about it.
#[test]
fn an_archive_called_something_that_is_not_a_zip_is_refused() {
    let root = a_folder_of_our_own("misnamed");
    let invoices = root.join("Invoices");
    let keep = root.join("Archive");
    fs::create_dir_all(&invoices).unwrap();
    fs::create_dir_all(&keep).unwrap();
    fs::write(invoices.join("march.pdf"), b"an invoice").unwrap();

    let grants = granting(&[&invoices, &keep]);
    let did = changing(
        "archive_folder",
        &[
            ("folder", as_given(&invoices)),
            ("into", as_given(&keep)),
            ("name", Given::text("invoices-2026")),
        ],
        &grants,
    )
    .unwrap();

    let failed = did.failure().unwrap();
    assert!(matches!(failed, Failed::NotAZipName { .. }), "{failed:?}");
    assert_eq!(
        fs::read_dir(&keep).unwrap().count(),
        0,
        "something was written"
    );

    let _ = fs::remove_dir_all(&root);
}

/// **The authorisation comes back whether or not the machine managed it.** A
/// call that was permitted, approved and attempted is a thing that happened,
/// and the record is written from it either way — a full disk is not a refusal
/// by the grants and must never be recorded as one.
#[test]
fn a_machine_that_could_not_do_it_still_hands_back_what_ran() {
    let root = a_folder_of_our_own("gone");
    let file = root.join("march.pdf");
    fs::write(&file, b"an invoice").unwrap();

    let grants = granting(&[&root]);
    let call = file_verbs()
        .unwrap()
        .call("read_file", &[("file", as_given(&file))])
        .unwrap();
    let authorised = Authorised::read(&call, &files(), &grants, noon()).unwrap();
    let touching = Touching::of(authorised, &grants, &OnThisMachine, &in_english()).unwrap();

    // Between the check and the doing, the file goes away. This is the race
    // `docs/quirks.md` records, seen from the side where it is harmless.
    fs::remove_file(&file).unwrap();

    let did = Did::of(touching, &grants, &in_english()).unwrap();
    let failed = did.failure().unwrap();
    assert!(matches!(failed, Failed::Gone { .. }), "{failed:?}");
    assert!(did.answer().is_none());

    let (authorised, outcome) = did.into_parts();
    assert_eq!(authorised.verb(), "read_file");
    assert_eq!(authorised.under(), &files());
    assert!(outcome.is_err());

    let _ = fs::remove_dir_all(&root);
}

/// The file half does the file verbs. A verb belonging to somewhere else is not
/// quietly performed here, and it is not an error of the capability model
/// either — it is this half saying it is not the one being asked.
#[test]
fn a_verb_that_is_not_a_file_verb_is_not_performed_here() {
    let opening = Verb::checked(
        "open_application",
        Word::saying("example.open-application.purpose", "open an application"),
        Effect::Change,
        vec![Arg::taking(
            "application",
            Word::saying(
                "example.open-application.application",
                "the application to open",
            ),
            Takes::Application,
        )],
        Requires::grants_over(["application"]),
        Word::saying("example.open-application.sentence", "open {application}"),
    )
    .unwrap();
    let mut verbs = Verbs::default();
    verbs.declare(opening).unwrap();
    let call = verbs
        .call(
            "open_application",
            &[("application", Given::text("org.gnome.Files"))],
        )
        .unwrap();

    let mut grants = Grants::default();
    grants.grant(
        Grant::checked(
            "@files",
            Reach::Application("org.gnome.Files".to_owned()),
            noon(),
            hour(),
        )
        .unwrap(),
    );
    let mut approvals = Approvals::default();
    let id =
        approvals.propose(Proposal::checked(&call, &files(), &grants, noon(), hour()).unwrap());
    let authorised = approvals
        .approve(id, noon())
        .unwrap()
        .redeem(&grants, noon())
        .unwrap();
    let touching = Touching::of(authorised, &grants, &OnThisMachine, &in_english()).unwrap();

    let did = Did::of(touching, &grants, &in_english()).unwrap();
    let failed = did.failure().unwrap();
    assert!(matches!(failed, Failed::NotAFileVerb { .. }), "{failed:?}");
    let said = failed.said(&in_english());
    assert!(said.text().contains("open_application"), "{said}");
    assert!(!said.is_a_bug(), "{said}");
}

/// A read never becomes a change on the way through, and what a verb touched is
/// what it said it would touch.
#[test]
fn a_read_touches_only_what_it_named() {
    let root = a_folder_of_our_own("touched");
    let invoices = root.join("Invoices");
    fs::create_dir_all(&invoices).unwrap();
    fs::write(invoices.join("march.pdf"), b"an invoice").unwrap();

    let grants = granting(&[&invoices]);
    let call = file_verbs()
        .unwrap()
        .call("list_folder", &[("folder", as_given(&invoices))])
        .unwrap();
    assert!(!call.waits_for_approval());
    let authorised = Authorised::read(&call, &files(), &grants, noon()).unwrap();
    let touching = Touching::of(authorised, &grants, &OnThisMachine, &in_english()).unwrap();

    assert_eq!(touching.all().count(), 1);
    assert_eq!(
        touching.real("folder").map(Real::as_path),
        Some(fs::canonicalize(&invoices).unwrap().as_path())
    );
    assert_eq!(
        touching.call().value("folder"),
        Some(&Value::Path(invoices.clone()))
    );

    let did = Did::of(touching, &grants, &in_english()).unwrap();
    assert!(matches!(did.answer(), Some(Answer::Listed(_))));
    // The folder is exactly as it was: a read changed nothing.
    assert_eq!(fs::read_dir(&invoices).unwrap().count(), 1);

    let _ = fs::remove_dir_all(&root);
}

/// The names in an archive, read out of the directory the format keeps at the
/// end of it.
///
/// A reader rather than a comparison against bytes we wrote: if the sizes,
/// offsets and counts in the archive disagree with each other, this walks off
/// the end and the test fails, which is the property worth asserting about a
/// format nothing else here reads.
fn inside_the_archive(at: &Path) -> Vec<String> {
    let bytes = fs::read(at).unwrap();
    let end = bytes.len() - 22;
    assert_eq!(
        u32::from_le_bytes(bytes[end..end + 4].try_into().unwrap()),
        0x0605_4b50,
        "the record at the end of the archive is not where the format says"
    );
    let how_many = u16::from_le_bytes(bytes[end + 10..end + 12].try_into().unwrap());
    let mut at = u32::from_le_bytes(bytes[end + 16..end + 20].try_into().unwrap()) as usize;

    let mut names = Vec::new();
    for _ in 0..how_many {
        assert_eq!(
            u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()),
            0x0201_4b50,
            "an entry in the directory is not where the one before it said"
        );
        let long = u16::from_le_bytes(bytes[at + 28..at + 30].try_into().unwrap()) as usize;
        let extra = u16::from_le_bytes(bytes[at + 30..at + 32].try_into().unwrap()) as usize;
        let comment = u16::from_le_bytes(bytes[at + 32..at + 34].try_into().unwrap()) as usize;
        let local = u32::from_le_bytes(bytes[at + 42..at + 46].try_into().unwrap()) as usize;
        assert_eq!(
            u32::from_le_bytes(bytes[local..local + 4].try_into().unwrap()),
            0x0403_4b50,
            "an entry points at something that is not a file header"
        );
        names.push(String::from_utf8(bytes[at + 46..at + 46 + long].to_vec()).unwrap());
        at += 46 + long + extra + comment;
    }
    names
}
