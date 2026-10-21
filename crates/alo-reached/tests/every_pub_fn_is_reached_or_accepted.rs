//! **Every public function of this workspace has a caller or a test, or is
//! named in a list that may only shrink.**
//!
//! This is the gate `docs/autonomy/what-nothing-calls.md` proposed and declined
//! to write. The fault it catches has been found **seven times** in this
//! repository: a mechanism built, tested, gated green, and reachable by nobody.
//! The sweep that found those ran once, by hand, on 2026-10-20 — and the one
//! after it arrived five hours later, which no hand sweep could have caught.
//!
//! # It runs on the real tree, and says so
//!
//! Unlike the unit tests beside the code, this one reads the repository, because
//! that is its subject. What keeps it honest is the refusal: if the
//! most-mentioned public function in the whole workspace is mentioned nowhere,
//! the walk read no code and the test **fails** rather than reporting that
//! everything is reached.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic naming what was measured is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_reached::{Accepted, Finding, THE_LIST, the_line_for, what_nothing_reaches};

/// The repository, from this crate's own manifest directory.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the repository two directories above this crate")
        .to_path_buf()
}

/// The environment variable that rewrites the list, as the vocabulary
/// snapshot's does.
const REWRITE: &str = "UPDATE_REACHED_SNAPSHOT";

/// **Every public function has a caller, a test, or a line in the list.**
///
/// Set `UPDATE_REACHED_SNAPSHOT=1` to rewrite the list instead of failing. That
/// is the only way it changes, and the change is a reviewable diff: a name
/// leaving needs no explanation, a name arriving needs a sentence.
#[test]
fn every_pub_fn_is_reached_or_accepted() {
    let root = the_repository();
    let found = what_nothing_reaches(&root);
    let at = root.join(THE_LIST);
    let listed = Accepted::read(&std::fs::read_to_string(&at).unwrap_or_default());

    // The refusal that tells a sweep which found nothing from a sweep which
    // could not run. `what_nothing_reaches` answers an empty list for both, so
    // the question has to be asked separately.
    let read_any_code = root.join("crates/alo-reached/src/lib.rs").exists();

    let lines: Vec<String> = found.iter().map(the_line_for).collect();
    if std::env::var_os(REWRITE).is_some() {
        std::fs::write(&at, Accepted::as_written(&lines)).expect("writing the list");
        return;
    }

    let findings = Finding::every(&found, listed.lines(), read_any_code);
    assert!(
        findings.is_empty(),
        "{}\n\n{} finding(s). {} public function(s) are reached by nothing, and the list \
         carries {}.",
        findings
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n\n"),
        findings.len(),
        found.len(),
        listed.lines().len()
    );
}

/// **The sweep really walked the workspace**, which is the premise every other
/// assertion here rests on.
///
/// Without this, a walk that read nothing would report an empty list of
/// orphans, the gate above would pass, and the whole crate would be a check
/// that cannot fail. Asserted against figures nobody has to keep in step: more
/// crates than a handful, and more public functions than a thousand.
#[test]
fn the_sweep_walked_the_workspace_rather_than_nothing() {
    let root = the_repository();
    let manifest =
        std::fs::read_to_string(root.join("Cargo.toml")).expect("the workspace manifest");
    let members: Vec<String> = alo_by_hand::declaring::members_of(&manifest)
        .into_iter()
        .filter(|named| named.starts_with("crates/"))
        .collect();
    assert!(
        members.len() > 100,
        "this workspace has over a hundred crates; the walk found {}",
        members.len()
    );

    let mut pub_fns = 0_usize;
    for member in &members {
        let files = alo_reached::a_crates_files(&root.join(member));
        pub_fns += files
            .product
            .iter()
            .map(|(_, code)| {
                code.lines()
                    .filter(|line| {
                        let it = line.trim_start();
                        it.starts_with("pub fn ")
                            || it.starts_with("pub const fn ")
                            || it.starts_with("pub async fn ")
                    })
                    .count()
            })
            .sum::<usize>();
    }
    assert!(
        pub_fns > 1_000,
        "this workspace declares thousands of public functions; the walk counted {pub_fns}"
    );
}

/// **The list only names functions that are really there.**
///
/// A list that outlived its files would quietly accept a name nothing declares,
/// and the gate above cannot see that: it compares what is unreached against
/// the list, and a line for a deleted file matches nothing either way.
#[test]
fn the_list_names_no_file_that_is_gone() {
    let root = the_repository();
    let listed = Accepted::read(&std::fs::read_to_string(root.join(THE_LIST)).unwrap_or_default());
    let gone: Vec<&String> = listed
        .lines()
        .iter()
        .filter(|line| {
            line.split('\t')
                .next()
                .is_some_and(|at| !root.join(at).exists())
        })
        .collect();
    assert!(
        gone.is_empty(),
        "the list names {} file(s) that are not there any more, so those lines accept nothing \
         and hide nothing. Regenerate with `{REWRITE}=1 cargo test -p alo-reached`:\n  {}",
        gone.len(),
        gone.iter()
            .map(|line| (*line).clone())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
