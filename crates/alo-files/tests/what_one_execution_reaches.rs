//! The six verbs held to the width of a boundary, and to a real filesystem.
//!
//! `alo-bounding-map` says one turn's entry holds
//! [`PLACES`](alo_bounding_map::PLACES) places, and the argument for that number
//! is written in that crate: *the widest call names two paths, and a change
//! creates a name in the folder each of them sits in.* It is an argument about
//! **these six verbs**, made in a crate that cannot see them — nothing there can
//! hold it to the closed list, and queue item 26b said so and left the test
//! owed.
//!
//! This is that test. Both crates are portable and depend on nothing, so it runs
//! on every host the workspace is built on rather than only on the one that has
//! a kernel to attach a boundary to. What it asserts is the two halves of the
//! claim:
//!
//! - **From the declarations**, which is the half that holds for a verb nobody
//!   has called yet: a verb reaches at most one place per path argument, plus
//!   one folder if it creates anything, and there are fewer of those than a
//!   boundary holds.
//! - **From executions on a real disk**, which is the half that checks the first
//!   one is measuring the right thing: each of the six, resolved and asked
//!   about, reaching exactly the places `reaching.rs` says it does.
//!
//! A seventh file verb that needed five places would fail here rather than on
//! somebody's machine, as `NotBounded::TooManyPlaces` — a turn refused for a
//! reason nobody could act on.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_bounding_map::PLACES;
use alo_capability::{
    Approvals, Authorised, Effect, Given, Grant, Grantee, Grants, Proposal, Reach, Takes, Verb,
    Verbs,
};
use alo_files::{OnThisMachine, Reaching, Resolving, Touching, file_verbs, file_words};
use alo_strings::Strings;

/// The strings this machine reads: this crate's and the deciding crate's, in
/// one vocabulary, as a shell would have them.
fn in_english() -> Strings {
    let mut vocabulary = file_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A fixed moment, so that expiry is arithmetic rather than a wait.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the grants and the questions in these tests last.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The agent the tests grant things to.
fn files() -> Grantee {
    Grantee::named("@files")
}

/// The file verbs, on a list.
fn verbs() -> Verbs {
    file_verbs().unwrap()
}

/// A path as a verb's argument would arrive: text.
fn as_given(path: &Path) -> Given {
    Given::text(path.to_string_lossy().into_owned())
}

/// A folder of this test's own, resolved, so what is granted is spelled the way
/// this machine spells it — `on_this_machine.rs` says why that matters.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-reaching-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    OnThisMachine.real(&folder).unwrap().into_path_buf()
}

/// Grants to `@files` over each of these folders.
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

/// One call these tests make, and everywhere it should then reach.
///
/// A struct rather than a tuple because the three parts are a verb, what a model
/// sent for each of its arguments, and an answer — and a reader of a failure
/// message needs to know which of the three is which.
struct Call<'a> {
    /// The verb, as an agent would name it.
    verb: &'a str,
    /// What was given for each argument.
    given: Vec<(&'a str, Given)>,
    /// Everywhere this execution should reach, in order.
    places: Vec<&'a Path>,
}

/// How many path arguments a verb declares.
fn paths_named(verb: &Verb) -> usize {
    verb.args()
        .iter()
        .filter(|arg| matches!(arg.takes(), Takes::Path))
        .count()
}

/// **Every verb this machine offers fits inside one turn's boundary**, argued
/// from the declaration rather than from a call somebody happened to make.
///
/// The bound is one place per path the call names, plus at most one more for the
/// folder a created name goes in — *at most* one because what a file verb
/// creates is an `Option` in `doing.rs` rather than a list, so no verb can
/// invent two names however it is written.
#[test]
fn no_verb_this_machine_offers_needs_more_places_than_a_boundary_holds() {
    for verb in verbs().all() {
        let creates = usize::from(verb.effect() == Effect::Change);
        let widest = paths_named(verb) + creates;
        assert!(
            widest <= PLACES,
            "{} could reach {widest} places and a turn's boundary holds {PLACES}",
            verb.name()
        );
    }
}

/// **And every one of them reaches something**, which is the other end of the
/// same width: `alo-bounding` refuses to bound a turn to nowhere, so a file verb
/// naming no path at all would be one that could never run inside a boundary.
#[test]
fn every_verb_this_machine_offers_reaches_at_least_one_place() {
    for verb in verbs().all() {
        assert!(
            paths_named(verb) >= 1,
            "{} names no path, so a turn running it could be bounded to nothing",
            verb.name()
        );
    }
}

/// **The three reads reach the one place they named.** On a real filesystem,
/// from a declared verb through the grants and the resolver to the places a
/// kernel would be told about.
#[test]
fn the_reads_reach_the_folder_or_the_file_they_were_given() {
    let root = a_folder_of_our_own("reads");
    let invoices = root.join("Invoices");
    fs::create_dir_all(&invoices).unwrap();
    let march = invoices.join("march.pdf");
    fs::write(&march, b"an invoice").unwrap();
    let grants = granting(&[&invoices]);
    let strings = in_english();

    let each = [
        Call {
            verb: "list_folder",
            given: vec![("folder", as_given(&invoices))],
            places: vec![invoices.as_path()],
        },
        Call {
            verb: "read_file",
            given: vec![("file", as_given(&march))],
            places: vec![march.as_path()],
        },
        Call {
            verb: "find_in_folder",
            given: vec![
                ("folder", as_given(&invoices)),
                ("named", Given::text("march")),
                ("most", Given::number(10)),
            ],
            places: vec![invoices.as_path()],
        },
    ];

    for asked in each {
        let call = verbs().call(asked.verb, &asked.given).unwrap();
        let authorised = Authorised::read(&call, &files(), &grants, noon()).unwrap();
        let touching = Touching::of(authorised, &grants, &OnThisMachine, &strings).unwrap();
        let reaching = Reaching::of(&touching).unwrap();

        assert_eq!(
            reaching.places().collect::<Vec<_>>(),
            asked.places,
            "{} reaches somewhere else",
            asked.verb
        );
        assert!(reaching.len() <= PLACES);
    }

    let _ = fs::remove_dir_all(&root);
}

/// **The three changes reach two places each, and the second is a folder.**
/// A rename reaches the folder the file sits in, a move and an archive the
/// folder they write into — and in both of those the folder was already named,
/// so two places rather than three.
#[test]
fn the_changes_reach_what_they_name_and_the_folder_they_create_in() {
    let root = a_folder_of_our_own("changes");
    let invoices = root.join("Invoices");
    let archive = root.join("Archive");
    fs::create_dir_all(&invoices).unwrap();
    fs::create_dir_all(&archive).unwrap();
    let march = invoices.join("march.pdf");
    fs::write(&march, b"an invoice").unwrap();
    let grants = granting(&[&invoices, &archive]);
    let strings = in_english();

    let each = [
        Call {
            verb: "rename_file",
            given: vec![
                ("file", as_given(&march)),
                ("name", Given::text("march-final.pdf")),
            ],
            places: vec![march.as_path(), invoices.as_path()],
        },
        Call {
            verb: "move_file",
            given: vec![("file", as_given(&march)), ("into", as_given(&archive))],
            places: vec![march.as_path(), archive.as_path()],
        },
        Call {
            verb: "archive_folder",
            given: vec![
                ("folder", as_given(&invoices)),
                ("into", as_given(&archive)),
                ("name", Given::text("invoices.zip")),
            ],
            places: vec![invoices.as_path(), archive.as_path()],
        },
    ];

    for asked in each {
        let call = verbs().call(asked.verb, &asked.given).unwrap();
        let mut approvals = Approvals::default();
        let id =
            approvals.propose(Proposal::checked(&call, &files(), &grants, noon(), hour()).unwrap());
        let authorised = approvals
            .approve(id, noon())
            .unwrap()
            .redeem(&grants, noon())
            .unwrap();
        let touching = Touching::of(authorised, &grants, &OnThisMachine, &strings).unwrap();
        let reaching = Reaching::of(&touching).unwrap();

        assert_eq!(
            reaching.places().collect::<Vec<_>>(),
            asked.places,
            "{} reaches somewhere else",
            asked.verb
        );
        assert!(reaching.len() <= PLACES);
    }

    let _ = fs::remove_dir_all(&root);
}

/// **A boundary is never made over a file that is not there yet.** The archive
/// a call would write does not exist when the turn is bounded, so what goes in
/// is the folder above it — and a reach naming the archive itself would be a
/// place nothing on the machine could answer for.
#[test]
fn nothing_a_call_would_create_is_reached_as_a_place_of_its_own() {
    let root = a_folder_of_our_own("creating");
    let invoices = root.join("Invoices");
    let archive = root.join("Archive");
    fs::create_dir_all(&invoices).unwrap();
    fs::create_dir_all(&archive).unwrap();
    let grants = granting(&[&invoices, &archive]);

    let call = verbs()
        .call(
            "archive_folder",
            &[
                ("folder", as_given(&invoices)),
                ("into", as_given(&archive)),
                ("name", Given::text("invoices.zip")),
            ],
        )
        .unwrap();
    let mut approvals = Approvals::default();
    let id =
        approvals.propose(Proposal::checked(&call, &files(), &grants, noon(), hour()).unwrap());
    let authorised = approvals
        .approve(id, noon())
        .unwrap()
        .redeem(&grants, noon())
        .unwrap();
    let touching = Touching::of(authorised, &grants, &OnThisMachine, &in_english()).unwrap();
    let reaching = Reaching::of(&touching).unwrap();

    let would_be = archive.join("invoices.zip");
    assert!(!would_be.exists(), "the test wrote the file it is about");
    assert!(
        !reaching.holds(&would_be),
        "a turn was bounded to a place nothing on the machine could name"
    );
    assert!(reaching.holds(&archive));

    let _ = fs::remove_dir_all(&root);
}
