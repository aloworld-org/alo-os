//! The list of what this boundary does **not** watch, held to the programme
//! that does the watching.
//!
//! `what_a_bound_turn_can_still_change.rs` reproduces each unwatched mutation
//! against a running kernel. This file is about the other half of the same
//! job — that somebody auditing the boundary can find the list, and that the
//! list is still true.
//!
//! # Why a documented limit needs a test at all
//!
//! A gap that is written down stops being a gap when somebody closes it, and
//! nothing about closing it makes the paragraph change. The failure that follows
//! is worse than the original gap: a document that says *a turn can still make a
//! symbolic link anywhere* on a machine where it cannot is read by the next
//! person as the current state of the boundary, and every plan made from it is
//! made from a false one. Documentation of a security limit rots in the
//! direction of understating the protection, which is the direction nobody
//! checks.
//!
//! So the table in `docs/quirks.md` is parsed rather than admired, and held to
//! four things:
//!
//! - **Nothing listed as unwatched is watched.** The hooks are read out of
//!   `crates/alo-bounding-kernel/src/kernel.rs`, where they are declared. The day
//!   one of these lands, this test fails and names the row to change.
//! - **Every hook the programme has is documented too**, in the two files
//!   somebody auditing the code reads, so a sixth hook cannot arrive silently.
//! - **Every row names a release `docs/features.md` knows**, because *which
//!   release owns closing it* is the half of a documented limit that turns it
//!   into work rather than a shrug.
//! - **Every row is reproduced.** A row whose hook is not named in the test file
//!   that reproduces these is a claim about a kernel nobody asked.
//!
//! # It needs no kernel
//!
//! Nothing here loads a programme or makes a control group: it reads this
//! repository's own files. That is deliberate — the list has to be checkable on
//! a machine that cannot run the boundary at all, which is most of them.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

/// The heading of the entry in `docs/quirks.md` that carries the list.
const THE_ENTRY: &str = "### Four hooks are not a filesystem";

/// Where the hooks are declared, relative to the repository.
const THE_PROGRAMME: &str = "crates/alo-bounding-kernel/src/kernel.rs";

/// Where the list is, relative to the repository.
const THE_LIST: &str = "docs/quirks.md";

/// The only list of what gets built, which is where a release has to be from.
const THE_RELEASES: &str = "docs/features.md";

/// The two places somebody auditing this boundary's code reads, and the file
/// that reproduces every row.
const WHERE_IT_ALSO_BELONGS: &[&str] = &[
    "crates/alo-bounding-kernel/src/deciding.rs",
    "crates/alo-bounding/src/lib.rs",
    "crates/alo-bounding/tests/what_a_bound_turn_can_still_change.rs",
];

/// The hooks this programme has, as an exact list.
///
/// Written out rather than counted, for the reason
/// `the_boundary_decides_and_forgets` names its two maps: a thirteenth hook is
/// a change to what this boundary is, and it should arrive with somebody
/// looking at it rather than as a test that still passes.
const EVERY_HOOK: &[&str] = &[
    "file_open",
    "file_permission",
    "inode_link",
    "inode_remove_acl",
    "inode_removexattr",
    "inode_rename",
    "inode_set_acl",
    "inode_setattr",
    "inode_setxattr",
    "inode_unlink",
    "socket_connect",
    "socket_sendmsg",
];

/// One row of the table: a mutation nothing watches.
#[derive(Debug)]
struct Unwatched {
    /// The kernel hook that would watch it.
    hook: String,

    /// What a bound turn can still do with it.
    doing: String,

    /// Why it is not a way to move a file's contents past a grant.
    why: String,

    /// The release that owns closing it.
    release: String,
}

/// This repository, from the crate this test is in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the repository is where this crate says it is")
}

/// A file of this repository, read.
fn reading(named: &str) -> String {
    let at = the_repository().join(named);
    fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// Every hook a source declares, from the attribute that attaches one.
///
/// `#[lsm(hook = "file_open")]` is how the programme says which hook a function
/// is, so this is the declaration itself rather than a list kept beside it.
fn hooks_declared_in(source: &str) -> BTreeSet<String> {
    source
        .lines()
        .filter_map(|line| line.split_once("lsm(hook = \""))
        .filter_map(|(_, rest)| rest.split_once('"'))
        .map(|(hook, _)| hook.to_owned())
        .collect()
}

/// The table under a heading, as rows.
///
/// A row is a line beginning with a pipe whose first cell is a backticked name;
/// the header and the rule beneath it are neither, so they fall away without
/// being special-cased. The section ends at the next heading, so a table
/// somewhere else in the same document is not this one.
fn the_table_under(document: &str, heading: &str) -> Vec<Unwatched> {
    let mut rows = Vec::new();
    let mut inside = false;
    for line in document.lines() {
        if line.starts_with(heading) {
            inside = true;
            continue;
        }
        if inside && (line.starts_with("## ") || line.starts_with("### ")) {
            break;
        }
        if !inside || !line.trim_start().starts_with('|') {
            continue;
        }
        let cells: Vec<&str> = line
            .trim()
            .trim_start_matches('|')
            .trim_end_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        let [hook, doing, why, release] = cells.as_slice() else {
            continue;
        };
        let Some(hook) = hook.strip_prefix('`').and_then(|it| it.strip_suffix('`')) else {
            continue;
        };
        rows.push(Unwatched {
            hook: hook.to_owned(),
            doing: (*doing).to_owned(),
            why: (*why).to_owned(),
            release: (*release).to_owned(),
        });
    }
    rows
}

/// How much of an answer *why this is not a way out* has to be.
///
/// A row saying `it is fine` would pass every other check here. This is not a
/// judge of the sentence — nothing mechanical can be — it is the floor beneath
/// one, and the reader of the table is the real check.
const AN_ANSWER: usize = 60;

/// Whether the list is still true of the programme, and complete.
///
/// Separate from the files it reads so that each refusal below can be shown
/// happening: a documentation test that has never been seen to fail is a
/// documentation test nobody should believe.
///
/// # Errors
/// A sentence naming the row and what is wrong with it.
fn whether_it_is_written_down(
    watched: &BTreeSet<String>,
    rows: &[Unwatched],
    also: &[(&str, &str)],
    releases: &str,
) -> Result<(), String> {
    if rows.is_empty() {
        return Err(format!(
            "there is no table of unwatched mutations under `{THE_ENTRY}` in {THE_LIST}, so \
             this test is checking nothing. It is the list somebody auditing this boundary \
             reads; if it moved, this test moves with it"
        ));
    }
    if watched.is_empty() {
        return Err(format!(
            "no hooks were found in {THE_PROGRAMME}, so nothing below is being compared against \
             the programme. The declaration is `#[lsm(hook = \"...\")]`"
        ));
    }

    for hook in watched {
        for (named, text) in also {
            if !text.contains(hook.as_str()) {
                return Err(format!(
                    "the programme has a hook `{hook}` that {named} does not mention. A hook is \
                     what this boundary is, and it arrives with a document saying so or it \
                     arrives silently"
                ));
            }
        }
    }

    for row in rows {
        if watched.contains(&row.hook) {
            return Err(format!(
                "`{}` is listed in {THE_LIST} as a mutation nothing watches, and the programme \
                 now has a hook on it. Come to that row: say what it decides, move it out of the \
                 table, and change the reproduction that asserts today's behaviour",
                row.hook
            ));
        }
        if row.doing.is_empty() || row.why.len() < AN_ANSWER {
            return Err(format!(
                "`{}` is listed without saying why it is not a way to move a file's contents \
                 past a grant. That sentence is the whole reason the four hooks that exist were \
                 the four chosen",
                row.hook
            ));
        }
        if !releases.contains(&format!("[{}]", row.release)) {
            return Err(format!(
                "`{}` names the release `{}`, and {THE_RELEASES} has no such tier. Which release \
                 owns closing a limit is what makes it work rather than a shrug",
                row.hook, row.release
            ));
        }
        for (named, text) in also {
            if !text.contains(&row.hook) {
                return Err(format!(
                    "`{}` is listed in {THE_LIST} and {named} does not mention it. Every row here \
                     is reproduced against a running kernel and argued beside the code that \
                     decides; a row in neither is a claim about a kernel nobody asked",
                    row.hook
                ));
            }
        }
    }
    Ok(())
}

/// **The list is still true, complete, and somewhere an auditor will find it.**
///
/// Everything this repository says about what the boundary does not watch, read
/// out of the files that say it and compared against the programme that decides.
#[test]
fn every_mutation_this_boundary_does_not_watch_is_written_down() {
    let programme = reading(THE_PROGRAMME);
    let watched = hooks_declared_in(&programme);
    let read: Vec<String> = WHERE_IT_ALSO_BELONGS.iter().map(|it| reading(it)).collect();
    let also: Vec<(&str, &str)> = WHERE_IT_ALSO_BELONGS
        .iter()
        .zip(read.iter())
        .map(|(named, text)| (*named, text.as_str()))
        .collect();

    let rows = the_table_under(&reading(THE_LIST), THE_ENTRY);
    if let Err(why) = whether_it_is_written_down(&watched, &rows, &also, &reading(THE_RELEASES)) {
        panic!("{why}");
    }

    assert_eq!(
        watched.iter().map(String::as_str).collect::<Vec<&str>>(),
        EVERY_HOOK,
        "the hooks this boundary has are not the ones it had. That is the change worth looking \
         at rather than the test that stopped matching: say what the new one decides in \
         crates/alo-bounding-kernel/src/deciding.rs, in crates/alo-bounding/src/lib.rs and in \
         the entry `{THE_ENTRY}` in {THE_LIST}, and name it here"
    );
}

/// **And the check would catch each way the list can go wrong**, which is the
/// half a green run cannot tell anybody.
///
/// A documentation test that has never refused anything passes on the day it
/// stops looking, in exactly the same colour. So every failure it exists for is
/// put in front of it here.
#[test]
fn the_check_catches_a_list_that_has_stopped_being_true() {
    let watched: BTreeSet<String> = ["file_open".to_owned()].into_iter().collect();
    let sound = |hook: &str| Unwatched {
        hook: hook.to_owned(),
        doing: "make one anywhere".to_owned(),
        why: "a name is not contents, and reading through it is an open, which is watched and \
              refused"
            .to_owned(),
        release: "v0.5".to_owned(),
    };
    let releases = "**[v0.01]** = it boots · **[v0.5]** = a person can work on it";
    let named_everywhere = "file_open and inode_symlink are both words in this file";
    let also = [("a source", named_everywhere)];

    assert_eq!(
        whether_it_is_written_down(&watched, &[sound("inode_symlink")], &also, releases),
        Ok(()),
        "a sound list was refused, so the refusals below say nothing"
    );

    // The one this test exists for: the gap was closed and the paragraph was
    // not, so the document now understates the boundary.
    let closed = whether_it_is_written_down(&watched, &[sound("file_open")], &also, releases);
    assert!(
        closed.is_err_and(|why| why.contains("now has a hook on it")),
        "a mutation listed as unwatched that the programme watches was accepted"
    );

    // A row with no answer to the question the whole list is about.
    let vague = Unwatched {
        why: "it is fine".to_owned(),
        ..sound("inode_symlink")
    };
    assert!(
        whether_it_is_written_down(&watched, &[vague], &also, releases)
            .is_err_and(|why| why.contains("not a way to move")),
        "a row that does not say why contents stay inside the grant was accepted"
    );

    // A release nobody has heard of, which is a limit nobody owns.
    let someday = Unwatched {
        release: "later".to_owned(),
        ..sound("inode_symlink")
    };
    assert!(
        whether_it_is_written_down(&watched, &[someday], &also, releases)
            .is_err_and(|why| why.contains("no such tier")),
        "a row naming a release that is not in the only list of what gets built was accepted"
    );

    // A row nothing reproduces and nothing argues.
    let unmentioned =
        whether_it_is_written_down(&watched, &[sound("inode_rmdir")], &also, releases);
    assert!(
        unmentioned.is_err_and(|why| why.contains("does not mention it")),
        "a row no test reproduces was accepted"
    );

    // A hook that arrived without the documents moving.
    let arrived: BTreeSet<String> = ["file_open", "inode_mkdir"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    assert!(
        whether_it_is_written_down(&arrived, &[sound("inode_symlink")], &also, releases)
            .is_err_and(|why| why.contains("does not mention")),
        "a hook the programme has that no document names was accepted"
    );

    // And the two that mean this test is looking at nothing at all.
    assert!(
        whether_it_is_written_down(&watched, &[], &also, releases)
            .is_err_and(|why| why.contains("checking nothing")),
        "an empty table was accepted, so a heading that moved would pass in silence"
    );
    assert!(
        whether_it_is_written_down(&BTreeSet::new(), &[sound("inode_symlink")], &also, releases)
            .is_err_and(|why| why.contains("nothing below is being compared")),
        "a programme with no hooks found in it was accepted"
    );
}

/// **The table is read the way a person reads it**, header and rule and all.
///
/// The parser is what stands between this test and a green run over a table it
/// never found, so it is shown finding one — and shown not finding a table under
/// a different heading in the same document.
#[test]
fn the_table_is_the_one_under_that_heading_and_no_other() {
    let document = "\
## Filesystems and paths

### Four hooks are not a filesystem: what a bound turn can still change
| Hook | What a bound turn can still do | Why no contents leave a grant | Release |
|---|---|---|---|
| `inode_symlink` | make one anywhere | a name is not contents | v0.5 |
| `inode_mkdir` | make a folder | a directory holds no bytes | v0.5 |

### Something else entirely
| | |
|---|---|
| `inode_rename` | watched since 2026-09-07 |
";
    let rows = the_table_under(document, THE_ENTRY);
    assert_eq!(
        rows.iter().map(|row| row.hook.as_str()).collect::<Vec<_>>(),
        ["inode_symlink", "inode_mkdir"],
        "the parser read a table that is not the list"
    );
    assert_eq!(
        rows.first().map(|row| row.release.as_str()),
        Some("v0.5"),
        "the release cell is not where the parser thinks it is"
    );
    assert!(
        the_table_under(document, "### A heading nothing is under").is_empty(),
        "the parser found rows under a heading that is not in the document"
    );
}

/// And the hooks are read from the declaration rather than from a sentence
/// about one.
#[test]
fn the_hooks_are_read_from_where_they_are_declared() {
    let source = "\
/// The hook is `inode_symlink` in a sentence, and this is not a declaration.
#[lsm(hook = \"file_open\")]
fn file_open(file: *const c_void) -> i32 {
#[lsm(hook = \"socket_connect\")]
";
    assert_eq!(
        hooks_declared_in(source),
        ["file_open".to_owned(), "socket_connect".to_owned()]
            .into_iter()
            .collect::<BTreeSet<String>>(),
        "a hook named in prose was taken for one the programme has, or a declared one was missed"
    );
}
