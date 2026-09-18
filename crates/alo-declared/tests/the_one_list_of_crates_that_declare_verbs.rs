//! The check itself, run against the crates this workspace really has.
//!
//! Everything in the crate is the method. This is the measurement: every member
//! of this workspace's own manifest, the `src/verbs.rs` each of them does or does
//! not have, and the one list `alo-declared` keeps — read off the disk this test
//! is running on, so a crate added tomorrow whose verbs nothing hands in fails
//! here rather than by a verb quietly never being asked whether a person can do
//! the same thing without the agent.
//!
//! # And the refusals, beside it
//!
//! A green check says the list and the workspace agree. It cannot say the check
//! would have noticed if they did not, and that is the half a reader has to
//! believe rather than see — so every finding this crate can produce is put in
//! front of it here against a fixture, including the one that really happened
//! three times in one day: **a crate added that declares verbs, and two tests
//! elsewhere that had never heard of it.**

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use alo_by_hand::THE_WORKSPACE;
use alo_declared::{
    Finding, Held, WHERE_THE_LIST_IS, WHO_DECLARES_THEM, every_verb_this_machine_ships, held,
};

/// This repository, from the crate this test is in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the repository is where this crate says it is")
}

/// A file of this repository, read, or [`None`] where there is no such file.
fn off_the_disk(named: &str) -> Option<String> {
    fs::read_to_string(the_repository().join(named)).ok()
}

/// The workspace manifest, which must be there for any of this to mean anything.
fn the_manifest() -> String {
    off_the_disk(THE_WORKSPACE).expect("this repository has a workspace manifest")
}

/// **Every crate of this workspace that declares verbs is on the one list, and
/// every crate on it still declares verbs** — against this repository, on the
/// disk it is checked out on.
///
/// This is the test the three refusals of 2026-09-17 were each missing: one
/// place that fails, naming the crate and the file, instead of two tests in two
/// other crates failing for a change neither of them is about.
#[test]
fn the_one_list_is_the_crates_this_workspace_has() {
    match held(WHO_DECLARES_THEM, &the_manifest(), &off_the_disk) {
        Ok(what) => {
            assert_eq!(
                what.declaring(),
                what.listed(),
                "a crate that declares verbs was neither on the list nor named, \
                 and the check counted it anyway"
            );
            assert!(
                what.declaring() > 1,
                "this workspace declares verbs in {} crate(s), which means the \
                 check passed over a manifest it never read",
                what.declaring()
            );
        }
        Err(findings) => {
            let listed: Vec<String> = findings.iter().map(ToString::to_string).collect();
            panic!(
                "{} thing(s) do not add up between the crates of this workspace \
                 that declare verbs and the one list that hands them in:\n\n- {}",
                listed.len(),
                listed.join("\n\n- ")
            );
        }
    }
}

/// **Nothing was lost and nothing was shared.** The one list holds exactly as
/// many verbs as the crates hold between them, which is only true while no call
/// was dropped on the way in and no two crates declared the same verb.
///
/// A count on its own would pass while one crate was silently dropped and
/// another grew. What makes this one say something is that it is assembled a
/// second way, declaring each registry entry separately. No test keeps a copy.
#[test]
fn the_machine_ships_what_the_crates_declare_between_them() {
    let each = alo_declared::shipped::each_crates_verbs().expect("each crate declares");
    assert_eq!(
        each.len(),
        WHO_DECLARES_THEM.len(),
        "this test asks a different number of crates than the one list names"
    );
    let verbs = every_verb_this_machine_ships().expect("every crate's verbs declare");
    assert!(!verbs.is_empty(), "this machine ships no verbs at all");
    assert_eq!(
        verbs.len(),
        each.iter().map(|(_, verbs)| verbs.len()).sum::<usize>(),
        "a crate's verbs were dropped on the way onto the one list, or two \
         crates declared the same verb"
    );
}

/// **The check refuses a real crate left off the real list** — the same
/// measurement as the first test, with one name taken out of the list itself.
///
/// Every refusal below this is shown against a fixture, which is what makes each
/// of them cheap to write and easy to disbelieve: a check could pass its
/// fixtures and still never look at this repository. This is the one that says
/// the real reading refuses, on the real disk, for the real reason.
#[test]
fn a_real_crate_left_off_the_real_list_is_refused() {
    let (dropped, rest) = WHO_DECLARES_THEM
        .split_first()
        .expect("the one list names at least one crate");
    let findings = held(rest, &the_manifest(), &off_the_disk)
        .expect_err("a crate of this repository was left off the list and the check passed");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::ACrateNothingHandsIn { crate_name } if crate_name == dropped
        )),
        "`{dropped}` declares verbs in this repository, nothing handed them in, \
         and the check did not name it: {findings:?}"
    );
}

/// A workspace with verbs in two of its four crates, written the way this one is
/// written. `alo-capability` is the registry the verbs are declared into and
/// declares none of its own; `alo-declared` has no `src/verbs.rs` at all.
fn a_workspace(named: &str) -> Option<String> {
    match named {
        "Cargo.toml" => Some(
            "[workspace]\nmembers = [\n  \"crates/alo-files\",\n  \"crates/alo-printing\",\n  \
             \"crates/alo-capability\",\n  \"crates/alo-declared\",\n]\n"
                .to_owned(),
        ),
        "crates/alo-files/src/verbs.rs" | "crates/alo-printing/src/verbs.rs" => {
            Some("pub fn declare_into(verbs: &mut Verbs) {}".to_owned())
        }
        "crates/alo-capability/src/verbs.rs" => {
            Some("pub fn declare(&mut self, verb: Verb) {}".to_owned())
        }
        _ => None,
    }
}

/// The one list that fixture is about.
const LISTED: [&str; 2] = ["alo-files", "alo-printing"];

/// The check over the fixture, as what it held or the findings it produced.
fn checking(listed: &[&str]) -> Result<Held, Vec<Finding>> {
    held(
        listed,
        &a_workspace("Cargo.toml").expect("the fixture has a manifest"),
        &a_workspace,
    )
}

/// A workspace that adds up is held, and counted the way it actually sits.
///
/// It runs first for the reason every refusal below depends on: a check that
/// refused everything would pass all of them and mean nothing.
#[test]
fn a_workspace_that_adds_up_is_held_and_counted() {
    let what = checking(&LISTED)
        .unwrap_or_else(|findings| panic!("a sound workspace was refused: {findings:?}"));
    assert_eq!(what.declaring(), 2);
    assert_eq!(what.listed(), 2);
}

/// **A crate that declares verbs and is on no list — named, with the one file to
/// add it in.**
///
/// The finding this whole crate exists for, and the one that really happened:
/// `alo-converting` and then `alo-capturing` arrived, and two tests in two other
/// crates failed for a change neither of them was about. What a person needs at
/// that moment is the crate's name and a file to open, so both are measured here
/// rather than assumed from the sentence.
#[test]
fn a_crate_that_declares_verbs_and_is_on_no_list_is_named_with_the_file() {
    let with_another = |named: &str| match named {
        "Cargo.toml" => Some(
            "[workspace]\nmembers = [\n  \"crates/alo-files\",\n  \"crates/alo-printing\",\n  \
             \"crates/alo-capturing\",\n]\n"
                .to_owned(),
        ),
        "crates/alo-capturing/src/verbs.rs" => {
            Some("pub fn declare_into(verbs: &mut Verbs) {}".to_owned())
        }
        other => a_workspace(other),
    };
    let findings = held(
        &LISTED,
        &with_another("Cargo.toml").expect("the fixture has a manifest"),
        &with_another,
    )
    .expect_err("a crate that declares verbs and is on no list was accepted");

    let named = findings
        .iter()
        .find(|finding| {
            matches!(
                finding,
                Finding::ACrateNothingHandsIn { crate_name } if crate_name == "alo-capturing"
            )
        })
        .unwrap_or_else(|| {
            panic!(
                "a whole crate's verbs would have gone unasked how a person does \
                 the same thing by hand, and the check did not name it: {findings:?}"
            )
        })
        .to_string();
    assert!(
        named.contains("alo-capturing"),
        "the finding does not name the crate: {named}"
    );
    assert!(
        named.contains(WHERE_THE_LIST_IS),
        "the finding does not name the one file to add it in, which is the whole \
         of what its reader needs: {named}"
    );
}

/// A crate on the list that declares nothing any more — renamed, or its verbs
/// moved, or it has none left. A dead name is how a list stops meaning anything.
#[test]
fn a_listed_crate_that_declares_no_verbs_is_a_finding() {
    let findings = checking(&["alo-files", "alo-printing", "alo-nothing"])
        .expect_err("a listed crate that declares no verbs was accepted");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::AListedCrateWithNoVerbs { crate_name } if crate_name == "alo-nothing"
        )),
        "the one list named a crate that declares no verbs, and nothing said so: \
         {findings:?}"
    );
}

/// A crate named twice, which makes the count add up while a crate goes
/// unchecked.
#[test]
fn a_crate_named_twice_is_refused() {
    let findings = checking(&["alo-files", "alo-files", "alo-printing"])
        .expect_err("a crate named twice on the one list was accepted");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::ACrateNamedTwice { crate_name, times }
                if crate_name == "alo-files" && *times == 2
        )),
        "a list counted one crate twice, so its count would have covered for a \
         crate it dropped: {findings:?}"
    );
}

/// And the one that would otherwise be silent: a workspace this check cannot
/// walk. An empty list of crates that declare verbs is indistinguishable from a
/// workspace where nothing declares any, and one of those is a check and the
/// other is a green light.
#[test]
fn a_workspace_that_cannot_be_read_is_checking_nothing() {
    let nothing = |_: &str| None;
    assert!(
        held(&LISTED, "", &nothing)
            .expect_err("a workspace that could not be read was accepted")
            .iter()
            .any(|finding| matches!(finding, Finding::NoWorkspaceToWalk { .. })),
        "a manifest that named nothing was taken for a workspace in which no \
         crate declares a verb"
    );

    let moved = |named: &str| match named {
        "crates/alo-files/src/verbs.rs" | "crates/alo-printing/src/verbs.rs" => None,
        other => a_workspace(other),
    };
    assert!(
        held(
            &LISTED,
            &a_workspace("Cargo.toml").expect("the fixture has a manifest"),
            &moved,
        )
        .expect_err("a convention that had moved was accepted")
        .iter()
        .any(|finding| matches!(finding, Finding::NoWorkspaceToWalk { .. })),
        "every crate's verbs moved out of src/verbs.rs and the check went on \
         passing over a workspace it no longer understood"
    );
}
