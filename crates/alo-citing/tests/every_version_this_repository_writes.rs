//! **Every version this repository writes, against the rule `ROADMAP.md` gives
//! for writing one.**
//!
//! `src/versions.rs` shows each refusal happening against fixtures. This is
//! where the rule meets the real documents.
//!
//! # Why this exists
//!
//! `ROADMAP.md`'s *Two numbers that look alike* records that on 2026-09-26 the
//! owner and the loop spent an evening each meaning a different thing by one
//! character. It then wrote the rules down — and **nothing read them.** On
//! 2026-10-05 twelve sentences across three plans broke one or the other, and a
//! person found them by reading, which is exactly the thing this repository
//! keeps discovering does not scale.
//!
//! A rule stated in prose and enforced by attention is a rule that holds until
//! somebody is tired. This is the other half.
//!
//! # It counts what it read, and refuses to pass on nothing
//!
//! Borrowed from `every_source_this_repository_points_at_exists.rs`, whose own
//! words are better than any restatement: *a walk that quietly finds nothing
//! reports nothing, and a green bar from a check that stopped looking is
//! indistinguishable from one that looked.* The count is asserted first.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

use alo_citing::versions::{Amiss, held, paragraphs};

/// Fewer paragraphs than this means the walk broke, not that the repository
/// stopped writing prose. Measured on 2026-10-05.
const TOO_FEW_TO_BELIEVE: usize = 2_000;

/// This repository, from the crate this test is in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the repository is where this crate says it is")
}

/// What is not this repository's own prose.
fn is_not_ours(name: &str) -> bool {
    matches!(name, ".git" | "target" | "node_modules" | "engines") || name.starts_with("target-")
}

/// Documents whose version numbers are not this rule's business, each with the
/// reason, because a list without reasons rots into permission nobody needs.
///
/// Checked in both directions by the test below: an entry naming a file that is
/// gone is refused as well.
const NOT_THIS_RULE_S: [(&str, &str); 4] = [
    (
        "ROADMAP.md",
        "it states the rule, and a rule has to be able to quote the form it forbids",
    ),
    (
        "docs/decisions/0097-one-version-a-person-says-and-one-build-identifier.md",
        "it is the rule for the dated scheme rather than a user of it: five hundred lines \
         whose subject is these forms, quoting every one it forbids, including the build \
         identifier its own first draft proposed and rejected. `states_the_rule` matches a \
         paragraph by its words, which is right for a short rule and brittle for a document \
         where nearly every section quotes a form",
    ),
    (
        "image/pinned.toml",
        "it is the source: a record of all five images, by version and digest, and \
         ROADMAP.md says nothing published is renamed",
    ),
    (
        "image/release-notes.md",
        "it is the published notes of image 0.0.5, carrying the registry's own tag \
         and digest as they were published",
    ),
];

/// Every markdown file in the repository, with its path relative to the root.
fn every_document(at: &Path, root: &Path, into: &mut Vec<(String, String)>) {
    let Ok(entries) = fs::read_dir(at) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if is_not_ours(&name) {
            continue;
        }
        if path.is_dir() {
            every_document(&path, root, into);
        } else if path.extension().is_some_and(|it| it == "md")
            && let Ok(text) = fs::read_to_string(&path)
        {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            into.push((relative, text));
        }
    }
}

/// **Every version in this repository's documents is written the way its kind
/// requires.**
#[test]
fn every_version_this_repository_writes_says_which_kind_it_is() {
    let root = the_repository();
    let mut documents = Vec::new();
    every_document(&root, &root, &mut documents);

    let read: usize = documents
        .iter()
        .map(|(_, text)| paragraphs(text).len())
        .sum();
    assert!(
        read >= TOO_FEW_TO_BELIEVE,
        "this walk read {read} paragraphs across {} documents, and fewer than \
         {TOO_FEW_TO_BELIEVE} means it stopped looking rather than that the \
         repository stopped writing",
        documents.len()
    );

    let mut amiss: Vec<Amiss> = Vec::new();
    for (file, text) in &documents {
        if NOT_THIS_RULE_S.iter().any(|(named, _)| named == file) {
            continue;
        }
        amiss.extend(held(file, text));
    }

    assert!(
        amiss.is_empty(),
        "{} version(s) are written in a form ROADMAP.md's *Two numbers that look \
         alike* refuses, out of {read} paragraphs read:\n\n- {}",
        amiss.len(),
        amiss
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n\n- ")
    );
}

/// **Every exemption still names a file that is here**, so the list cannot rot
/// into permission for something that moved.
#[test]
fn every_exemption_names_a_document_that_exists() {
    let root = the_repository();
    for (named, why) in NOT_THIS_RULE_S {
        assert!(
            root.join(named).exists(),
            "the exemption list says {named} is not this rule's business because \
             {why}, and there is no such file. An exemption for something that \
             moved is permission nobody asked for"
        );
        assert!(
            !why.trim().is_empty(),
            "{named} is exempted without a reason, and a list without reasons is a \
             list nobody can review"
        );
    }
}
