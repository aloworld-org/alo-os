//! The check itself, run against the crates this workspace really has.
//!
//! Everything in the crate is the method. This is the measurement: every member
//! of this workspace's own manifest, the `src/words.rs` each of them does or does
//! not have, and the two lists `alo-saying` keeps — read off the disk this test
//! is running on, so a crate added tomorrow whose words nothing collects fails
//! here rather than in somebody's first sign-in in a language nobody checked.
//!
//! # And the refusals, beside it
//!
//! A green check says the crates and the vocabulary agree. It cannot say the
//! check would have noticed if they did not, and that is the half a reader has to
//! believe rather than see — so every finding this crate can produce is put in
//! front of it here against a fixture, including the one that really happened:
//! **a crate added with words nothing collects.**

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use alo_collected::{Finding, Held, THE_WORKSPACE, held, whoever_declares_words};
use alo_saying::{DELIBERATELY_APART, EVERY_LIST};

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

/// **Every crate of this workspace that declares words is collected into the one
/// vocabulary, or is named apart with the reason it cannot be** — against this
/// repository, on the disk it is checked out on.
#[test]
fn every_crate_that_declares_words_is_collected_or_named_apart() {
    match held(
        &EVERY_LIST,
        &DELIBERATELY_APART,
        &the_manifest(),
        &off_the_disk,
    ) {
        Ok(what) => {
            assert_eq!(
                what.declaring(),
                what.collected() + what.apart(),
                "a crate that declares words was neither collected nor named \
                 apart, and the check counted it anyway"
            );
            assert!(
                what.declaring() > 1,
                "this workspace declares words in {} crate(s), which means the \
                 check passed over a manifest it never read",
                what.declaring()
            );
            assert_eq!(
                what.apart(),
                1,
                "the number of crates standing outside the one vocabulary \
                 changed. That is allowed and it is not routine: every one of \
                 them is a crate whose words no translation is checked against"
            );
        }
        Err(findings) => {
            let listed: Vec<String> = findings.iter().map(ToString::to_string).collect();
            panic!(
                "{} thing(s) do not add up between the crates of this workspace \
                 that declare words and the vocabulary that collects them:\n\n- {}",
                listed.len(),
                listed.join("\n\n- ")
            );
        }
    }
}

/// **The one crate that is not collected is named, and its reason is the
/// argument** — not a note somebody can read as a category of crate that is
/// exempt.
///
/// This is the half of the task's title that no count can carry: an exception
/// list is only worth having while each line on it says what would break.
#[test]
fn the_one_crate_that_is_not_collected_is_named_with_its_reason() {
    let (named, reason) = DELIBERATELY_APART
        .first()
        .expect("one crate stands outside the one vocabulary");
    assert_eq!(*named, "alo-agentd");
    assert!(
        alo_collected::is_a_reason(reason),
        "the reason `{named}` stands outside the one vocabulary is a shrug"
    );
    assert!(
        reason.contains("Linux"),
        "the reason no longer names the fact that makes it one: {reason}"
    );
    assert!(
        !EVERY_LIST.contains(named),
        "`{named}` is both collected and named apart"
    );
    assert!(
        whoever_declares_words(&the_manifest(), &off_the_disk).contains(&(*named).to_owned()),
        "`{named}` is named as standing apart from a vocabulary and declares no \
         words at all, so the exception is about nothing"
    );
}

/// **This crate is not collected, and does not need to be** — the constraint the
/// task set, measured rather than asserted: a repository check says nothing to a
/// person, so it has no `src/words.rs` and the check above therefore expects
/// nothing of it.
#[test]
fn the_check_itself_declares_no_words() {
    assert!(
        !whoever_declares_words(&the_manifest(), &off_the_disk)
            .contains(&"alo-collected".to_owned()),
        "crates/alo-collected declares words, so it has to be collected — and a \
         repository check that says something to a person is a different crate"
    );
    assert!(
        !EVERY_LIST.contains(&"alo-collected"),
        "the one vocabulary collects a crate with no words in it"
    );
}

/// **The check refuses a real crate left off the real list** — the same
/// measurement as the first test, with one name taken out of the vocabulary's
/// own list.
///
/// Every refusal below this is shown against a fixture, which is what makes each
/// of them cheap to write and easy to disbelieve: a check could pass its
/// fixtures and still never look at this repository. This is the one that says
/// the real reading refuses, on the real disk, for the real reason.
#[test]
fn a_real_crate_left_off_the_real_list_is_refused() {
    let (dropped, rest) = EVERY_LIST
        .split_first()
        .expect("the one vocabulary collects at least one crate");
    let findings = held(rest, &DELIBERATELY_APART, &the_manifest(), &off_the_disk)
        .expect_err("a crate of this repository was left uncollected and the check passed");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::ACrateNothingCollects { crate_name } if crate_name == dropped
        )),
        "`{dropped}` declares words in this repository, nothing collected them, \
         and the check did not name it: {findings:?}"
    );
}

/// A workspace with words in two of its three crates, written the way this one is
/// written.
fn a_workspace(named: &str) -> Option<String> {
    match named {
        "Cargo.toml" => Some(
            "[workspace]\nmembers = [\n  \"crates/alo-telling\",\n  \"crates/alo-agentd\",\n  \
             \"crates/alo-collected\",\n]\n"
                .to_owned(),
        ),
        "crates/alo-telling/src/words.rs" | "crates/alo-agentd/src/words.rs" => {
            Some("pub fn declare_into(vocabulary: &mut Vocabulary) {}".to_owned())
        }
        _ => None,
    }
}

/// One crate collected, one standing apart with an argument, and one with no
/// words at all.
const COLLECTED: [&str; 1] = ["alo-telling"];

/// The reason the fixture's daemon stands outside the fixture's vocabulary,
/// which is the real one's in miniature.
const APART: [(&str, &str); 1] = [(
    "alo-agentd",
    "it is Linux and compiled out anywhere else, so a vocabulary built from it \
     would be shorter on one host than another and a translation would be \
     refused on one of them",
)];

/// The check over the fixture, as what it held or the findings it produced.
fn checking(collected: &[&str], apart: &[(&str, &str)]) -> Result<Held, Vec<Finding>> {
    held(
        collected,
        apart,
        &a_workspace("Cargo.toml").expect("the fixture has a manifest"),
        &a_workspace,
    )
}

/// A workspace that adds up is held, and counted the way it actually sits: two
/// crates with words, one collected and one apart.
///
/// It runs first for the reason every refusal below depends on: a check that
/// refused everything would pass all of them and mean nothing.
#[test]
fn a_workspace_that_adds_up_is_held_and_counted() {
    let what = checking(&COLLECTED, &APART)
        .unwrap_or_else(|findings| panic!("a sound workspace was refused: {findings:?}"));
    assert_eq!(what.declaring(), 2);
    assert_eq!(what.collected(), 1);
    assert_eq!(what.apart(), 1);
}

/// **A crate added with words nothing collects** — the finding this whole crate
/// exists for, and the one that really happened to `alo-overlay`.
#[test]
fn a_crate_whose_words_nothing_collects_is_the_finding() {
    let with_an_overlay = |named: &str| match named {
        "Cargo.toml" => Some(
            "[workspace]\nmembers = [\n  \"crates/alo-telling\",\n  \"crates/alo-agentd\",\n  \
             \"crates/alo-overlay\",\n]\n"
                .to_owned(),
        ),
        "crates/alo-overlay/src/words.rs" => {
            Some("pub fn declare_into(vocabulary: &mut Vocabulary) {}".to_owned())
        }
        other => a_workspace(other),
    };
    let findings = held(
        &COLLECTED,
        &APART,
        &with_an_overlay("Cargo.toml").expect("the fixture has a manifest"),
        &with_an_overlay,
    )
    .expect_err("a crate whose words nothing collects was accepted");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::ACrateNothingCollects { crate_name } if crate_name == "alo-overlay"
        )),
        "a crate's whole vocabulary would have reached people as missing keys and \
         the check passed: {findings:?}"
    );
}

/// A crate the vocabulary collects that declares nothing any more — renamed, or
/// its words moved, or it has none left. A dead name is how a list stops meaning
/// anything.
#[test]
fn a_collected_crate_that_no_longer_declares_is_a_finding() {
    let findings = checking(&["alo-telling", "alo-remembering"], &APART)
        .expect_err("a collected crate that declares nothing was accepted");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::ACollectedCrateThatSaysNothing { crate_name }
                if crate_name == "alo-remembering"
        )),
        "the vocabulary named a crate that has no words, and nothing said so: \
         {findings:?}"
    );
}

/// An exception with a shrug beside it. The name reads exactly like a documented
/// decision, which is why it is refused: the argument is the whole difference
/// between an exception and silence.
#[test]
fn an_exception_with_no_reason_is_refused() {
    let findings = checking(&COLLECTED, &[("alo-agentd", "it is Linux")])
        .expect_err("an exception with a shrug beside it was accepted");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::AnExceptionWithNoReason { crate_name, said }
                if crate_name == "alo-agentd" && said == "it is Linux"
        )),
        "a crate was excused from saying anything to anybody, for no stated \
         reason: {findings:?}"
    );
}

/// An exception about a crate with no words, which is an exception nobody needs —
/// and which would still be standing on the day a crate of that name has words.
#[test]
fn an_exception_nobody_needs_is_a_finding() {
    let findings = checking(&COLLECTED, &[APART[0], ("alo-collected", APART[0].1)])
        .expect_err("an exception about a crate with no words was accepted");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::AnExceptionNobodyNeeds { crate_name } if crate_name == "alo-collected"
        )),
        "a standing permission to say nothing outlived the crate it was about: \
         {findings:?}"
    );
}

/// A crate on both lists, and a crate on one list twice. Either way the two
/// counts add up while the workspace does not.
#[test]
fn a_crate_on_both_lists_or_twice_on_one_is_refused() {
    let both = checking(&["alo-telling", "alo-agentd"], &APART)
        .expect_err("a crate both collected and named apart was accepted");
    assert!(
        both.iter().any(|finding| matches!(
            finding,
            Finding::ACrateBothCollectedAndApart { crate_name } if crate_name == "alo-agentd"
        )),
        "whether a crate's words reach a person depended on which list somebody \
         read: {both:?}"
    );

    let twice = checking(&["alo-telling", "alo-telling"], &APART)
        .expect_err("a crate named twice on one list was accepted");
    assert!(
        twice.iter().any(|finding| matches!(
            finding,
            Finding::ACrateNamedTwice { crate_name, times, .. }
                if crate_name == "alo-telling" && *times == 2
        )),
        "a list counted one crate twice, so its count would have covered for a \
         crate it dropped: {twice:?}"
    );

    let apart_twice = checking(&COLLECTED, &[APART[0], APART[0]])
        .expect_err("a crate named twice among the exceptions was accepted");
    assert!(
        apart_twice
            .iter()
            .any(|finding| matches!(finding, Finding::ACrateNamedTwice { .. })),
        "{apart_twice:?}"
    );
}

/// And the one that would otherwise be silent: a workspace this check cannot
/// walk. An empty list of crates that declare words is indistinguishable from a
/// workspace where nothing says anything, and one of those is a check and the
/// other is a green light.
#[test]
fn a_workspace_that_cannot_be_read_is_checking_nothing() {
    let nothing = |_: &str| None;
    assert!(
        held(&COLLECTED, &APART, "", &nothing)
            .expect_err("a workspace that could not be read was accepted")
            .iter()
            .any(|finding| matches!(finding, Finding::NoWorkspaceToWalk { .. })),
        "a manifest that named nothing was taken for a workspace in which no \
         crate says anything"
    );

    let moved = |named: &str| match named {
        "crates/alo-telling/src/words.rs" | "crates/alo-agentd/src/words.rs" => None,
        other => a_workspace(other),
    };
    assert!(
        held(
            &COLLECTED,
            &APART,
            &a_workspace("Cargo.toml").expect("the fixture has a manifest"),
            &moved,
        )
        .expect_err("a convention that had moved was accepted")
        .iter()
        .any(|finding| matches!(finding, Finding::NoWorkspaceToWalk { .. })),
        "every crate's words moved out of src/words.rs and the check went on \
         passing over a workspace it no longer understood"
    );
}
