//! The account of what a turn inherits, held to the programme that does the
//! deciding.
//!
//! `what_a_turn_inherits.rs` measures each inherited descriptor against a
//! running kernel. This file is about the other half of the same job — that
//! somebody auditing the boundary can find the account, and that the account is
//! still true of the machine.
//!
//! # Why a documented limit needs a test at all
//!
//! `the_unwatched_mutations_are_written_down.rs` makes the argument and it is
//! the same one here: a gap that is written down stops being a gap when somebody
//! closes it, and nothing about closing it makes the paragraph change.
//! Documentation of a security limit rots in the direction of understating the
//! protection, which is the direction nobody checks.
//!
//! This one rots in a second direction as well, and it is worse. The entry says
//! **what would close this and why neither half is a patch** — a hook on every
//! read, or a turn that is a process of its own. If either arrives and the entry
//! does not move, what is left is a document telling the next person that a
//! decision is still open when it has been taken.
//!
//! So the table in `docs/quirks.md` is parsed rather than admired, and held to
//! five things:
//!
//! - **Neither hook that would close this is on the programme.** They are read
//!   out of `crates/alo-bounding-kernel/src/kernel.rs`, where hooks are declared.
//! - **The entry names both of them**, because *what would close it* is the half
//!   of a documented limit that turns it into work rather than a shrug.
//! - **Every row says what the inherited thing does not permit.** A row that only
//!   said what a turn can do would be a list of holes with no floor under it, and
//!   the floor is what makes this a gap rather than an absence of a boundary.
//! - **Every row names a release `docs/features.md` knows.**
//! - **Every row is reproduced**, in the file the row itself names, against a
//!   running kernel. A row nothing reproduces is a claim about a machine nobody
//!   asked.
//!
//! # It needs no kernel
//!
//! Nothing here loads a programme or makes a control group: it reads this
//! repository's own files. That is deliberate — the account has to be checkable
//! on a machine that cannot run the boundary at all, which is most of them.

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

/// The heading of the entry in `docs/quirks.md` that carries the account.
const THE_ENTRY: &str = "### A descriptor opened before a turn began is inside no boundary";

/// The phrase the two files somebody auditing this boundary's code reads must
/// carry, so the account cannot live only in a document they are not reading.
const THE_SUBJECT: &str = "opened before a turn began";

/// Where the hooks are declared, relative to the repository.
const THE_PROGRAMME: &str = "crates/alo-bounding-kernel/src/kernel.rs";

/// Where the account is, relative to the repository.
const THE_ACCOUNT: &str = "docs/quirks.md";

/// The only list of what gets built, which is where a release has to be from.
const THE_RELEASES: &str = "docs/features.md";

/// Where the tests each row names live.
const WHERE_THE_TESTS_ARE: &str = "crates/alo-bounding/tests";

/// The two places somebody auditing this boundary's code reads.
const WHERE_IT_ALSO_BELONGS: &[&str] = &[
    "crates/alo-bounding-kernel/src/deciding.rs",
    "crates/alo-bounding/src/lib.rs",
];

/// The hooks that decide about a descriptor rather than about an open, which
/// are the two that would close this gap in the kernel.
///
/// `file_permission` fires on every read and every write; `file_receive` fires
/// when a descriptor arrives from somewhere else. Written out rather than
/// counted, for the reason `the_boundary_decides_and_forgets` names its two
/// maps: either of these arriving is a change to what this boundary *is*, and it
/// should arrive with somebody looking at it.
const WOULD_CLOSE_IT: &[&str] = &["file_permission", "file_receive"];

/// What a turn inherits on this machine, as an exact list.
///
/// Written out for the same reason and read the same way: a row that a stray
/// character in the table made the parser drop would otherwise leave this test
/// green over an account it had stopped checking, and a row somebody removed is
/// a claim about the daemon that should arrive with a person looking at it.
///
/// `a socket already connected` was the fifth row until 2026-09-12, when
/// `socket_sendmsg` closed it: a socket inherited into a turn is decided about
/// on every message, so it is no longer something a turn inherits the use of.
/// It left this list with a person looking at it, which is what the list is
/// for.
const EVERY_ROW: &[&str] = &[
    "a file open for reading",
    "a file open for appending",
    "a directory descriptor",
    "the way out of a turn",
];

/// One row of the table: something a turn inherits.
#[derive(Debug)]
struct Inherited {
    /// What it is.
    what: String,

    /// What it permits inside the boundary.
    permits: String,

    /// What it still does not permit, which is the floor under the gap.
    does_not: String,

    /// The test file that reproduces it against a running kernel.
    reproduced_in: String,

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

/// The section under a heading, up to the next one.
fn the_section_under<'a>(document: &'a str, heading: &str) -> &'a str {
    let Some(from) = document.find(heading) else {
        return "";
    };
    let rest = document.get(from..).unwrap_or_default();
    let after = rest.get(heading.len()..).unwrap_or_default();
    match after.find("\n### ").or_else(|| after.find("\n## ")) {
        Some(to) => rest.get(..heading.len() + to).unwrap_or_default(),
        None => rest,
    }
}

/// The table in a section, as rows.
///
/// A row is a line beginning with a pipe whose first cell is a backticked name;
/// the header and the rule beneath it are neither, so they fall away without
/// being special-cased.
fn the_table_in(section: &str) -> Vec<Inherited> {
    let mut rows = Vec::new();
    for line in section.lines() {
        if !line.trim_start().starts_with('|') {
            continue;
        }
        let cells: Vec<&str> = line
            .trim()
            .trim_start_matches('|')
            .trim_end_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        let [what, permits, does_not, reproduced_in, release] = cells.as_slice() else {
            continue;
        };
        let Some(what) = what.strip_prefix('`').and_then(|it| it.strip_suffix('`')) else {
            continue;
        };
        rows.push(Inherited {
            what: what.to_owned(),
            permits: (*permits).to_owned(),
            does_not: (*does_not).to_owned(),
            reproduced_in: reproduced_in.trim_matches('`').to_owned(),
            release: (*release).to_owned(),
        });
    }
    rows
}

/// How much of an answer *what this still does not permit* has to be.
///
/// A row saying `nothing much` would pass every other check here. This is not a
/// judge of the sentence — nothing mechanical can be — it is the floor beneath
/// one, and the reader of the table is the real check.
const AN_ANSWER: usize = 60;

/// Whether the account is still true of the programme, and complete.
///
/// Separate from the files it reads so that each refusal below can be shown
/// happening: a documentation test that has never been seen to fail is a
/// documentation test nobody should believe.
///
/// `reproducing` answers with the text of a test file the table names, or
/// [`None`] when there is no such file.
///
/// # Errors
/// A sentence naming the row and what is wrong with it.
fn whether_it_is_written_down(
    watched: &BTreeSet<String>,
    rows: &[Inherited],
    entry: &str,
    releases: &str,
    reproducing: &dyn Fn(&str) -> Option<String>,
) -> Result<(), String> {
    if rows.is_empty() {
        return Err(format!(
            "there is no table of what a turn inherits under `{THE_ENTRY}` in {THE_ACCOUNT}, so \
             this test is checking nothing. It is the account somebody auditing this boundary \
             reads; if it moved, this test moves with it"
        ));
    }

    for hook in WOULD_CLOSE_IT {
        if watched.contains(*hook) {
            return Err(format!(
                "the programme now has a hook `{hook}`, and {THE_ACCOUNT} still says a descriptor \
                 opened before a turn began is inside no boundary. Come to that entry: say what \
                 the hook decides, move the rows it closes out of the table, and change the \
                 reproductions that assert today's behaviour"
            ));
        }
        if !entry.contains(*hook) {
            return Err(format!(
                "the entry does not name `{hook}`, which is one of the two hooks that would close \
                 this. What would close a documented limit is the half that turns it into work \
                 rather than a shrug"
            ));
        }
    }

    for row in rows {
        if row.permits.is_empty() || row.does_not.len() < AN_ANSWER {
            return Err(format!(
                "`{}` is listed without saying what it still does not permit. That sentence is \
                 the floor under the gap, and a list of holes with no floor under it is not an \
                 account of a boundary",
                row.what
            ));
        }
        if !releases.contains(&format!("[{}]", row.release)) {
            return Err(format!(
                "`{}` names the release `{}`, and {THE_RELEASES} has no such tier. Which release \
                 owns closing a limit is what makes it work rather than a shrug",
                row.what, row.release
            ));
        }
        let Some(reproduction) = reproducing(&row.reproduced_in) else {
            return Err(format!(
                "`{}` says it is reproduced in `{}`, and there is no such test in \
                 {WHERE_THE_TESTS_ARE}",
                row.what, row.reproduced_in
            ));
        };
        if !reproduction.contains(&row.what) {
            return Err(format!(
                "`{}` says it is reproduced in `{}`, and that file does not mention it. Every row \
                 here is measured against a running kernel; a row nothing reproduces is a claim \
                 about a machine nobody asked",
                row.what, row.reproduced_in
            ));
        }
    }
    Ok(())
}

/// **The account is still true, complete, and somewhere an auditor will find
/// it.**
///
/// Everything this repository says about what a turn inherits, read out of the
/// files that say it and compared against the programme that decides.
#[test]
fn what_a_turn_inherits_is_written_down_where_an_auditor_will_find_it() {
    let watched = hooks_declared_in(&reading(THE_PROGRAMME));
    let entry = reading(THE_ACCOUNT);
    let section = the_section_under(&entry, THE_ENTRY);
    let rows = the_table_in(section);
    let reproducing = |named: &str| {
        fs::read_to_string(the_repository().join(WHERE_THE_TESTS_ARE).join(named)).ok()
    };

    if let Err(why) = whether_it_is_written_down(
        &watched,
        &rows,
        section,
        &reading(THE_RELEASES),
        &reproducing,
    ) {
        panic!("{why}");
    }

    assert_eq!(
        rows.iter().map(|row| row.what.as_str()).collect::<Vec<_>>(),
        EVERY_ROW,
        "what a turn inherits is not what it inherited. That is the change worth looking at \
         rather than the test that stopped matching: say what arrived or went away in \
         crates/alo-bounding/src/lib.rs, in crates/alo-bounding-kernel/src/deciding.rs and in the \
         entry `{THE_ENTRY}` in {THE_ACCOUNT}, reproduce it, and name it here"
    );

    for named in WHERE_IT_ALSO_BELONGS {
        assert!(
            reading(named).contains(THE_SUBJECT),
            "{named} does not mention what a turn inherits, and it is one of the two files \
             somebody auditing this boundary's code reads. An account that lives only in \
             {THE_ACCOUNT} is an account the person reading the crate never sees"
        );
    }
}

/// **And the check would catch each way the account can go wrong**, which is the
/// half a green run cannot tell anybody.
///
/// A documentation test that has never refused anything passes on the day it
/// stops looking, in exactly the same colour. So every failure it exists for is
/// put in front of it here.
#[test]
fn the_check_catches_an_account_that_has_stopped_being_true() {
    let watched: BTreeSet<String> = ["file_open".to_owned()].into_iter().collect();
    let sound = |what: &str| Inherited {
        what: what.to_owned(),
        permits: "every byte of it".to_owned(),
        does_not: "it cannot be reopened by name, and the name the kernel gives it does not turn \
                   it back into an open"
            .to_owned(),
        reproduced_in: "what_a_turn_inherits.rs".to_owned(),
        release: "v0.5".to_owned(),
    };
    let entry = "file_permission fires on every read and file_receive on a descriptor arriving";
    let releases = "**[v0.01]** = it boots · **[v0.5]** = a person can work on it";
    let reproducing = |named: &str| {
        (named == "what_a_turn_inherits.rs")
            .then(|| "a file open for reading is measured here".to_owned())
    };

    assert_eq!(
        whether_it_is_written_down(
            &watched,
            &[sound("a file open for reading")],
            entry,
            releases,
            &reproducing
        ),
        Ok(()),
        "a sound account was refused, so the refusals below say nothing"
    );

    // The one this test exists for: a hook that decides about a descriptor
    // landed, and the entry still says none has.
    let closed: BTreeSet<String> = ["file_open", "file_permission"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    assert!(
        whether_it_is_written_down(
            &closed,
            &[sound("a file open for reading")],
            entry,
            releases,
            &reproducing
        )
        .is_err_and(|why| why.contains("still says")),
        "a programme that watches a descriptor after it is opened was accepted while the account \
         said nothing does"
    );

    // An entry that no longer says what would close it, which is how a limit
    // stops being work and becomes a shrug.
    assert!(
        whether_it_is_written_down(
            &watched,
            &[sound("a file open for reading")],
            "there is no hook named here at all",
            releases,
            &reproducing
        )
        .is_err_and(|why| why.contains("would close this")),
        "an entry that names neither hook that would close this was accepted"
    );

    // A row with no floor under it.
    let vague = Inherited {
        does_not: "nothing much".to_owned(),
        ..sound("a file open for reading")
    };
    assert!(
        whether_it_is_written_down(&watched, &[vague], entry, releases, &reproducing)
            .is_err_and(|why| why.contains("does not permit")),
        "a row that does not say what the inherited descriptor still cannot do was accepted"
    );

    // A release nobody has heard of, which is a limit nobody owns.
    let someday = Inherited {
        release: "later".to_owned(),
        ..sound("a file open for reading")
    };
    assert!(
        whether_it_is_written_down(&watched, &[someday], entry, releases, &reproducing)
            .is_err_and(|why| why.contains("no such tier")),
        "a row naming a release that is not in the only list of what gets built was accepted"
    );

    // A row pointing at a test that is not there, and one pointing at a test
    // that is there and says nothing about it.
    let nowhere = Inherited {
        reproduced_in: "a_test_nobody_wrote.rs".to_owned(),
        ..sound("a file open for reading")
    };
    assert!(
        whether_it_is_written_down(&watched, &[nowhere], entry, releases, &reproducing)
            .is_err_and(|why| why.contains("no such test")),
        "a row naming a test file that does not exist was accepted"
    );
    assert!(
        whether_it_is_written_down(
            &watched,
            &[sound("a socket already connected")],
            entry,
            releases,
            &reproducing
        )
        .is_err_and(|why| why.contains("does not mention it")),
        "a row whose named test says nothing about it was accepted"
    );

    // And the one that means this test is looking at nothing at all.
    assert!(
        whether_it_is_written_down(&watched, &[], entry, releases, &reproducing)
            .is_err_and(|why| why.contains("checking nothing")),
        "an empty table was accepted, so a heading that moved would pass in silence"
    );
}

/// **The table is read the way a person reads it**, header and rule and all, and
/// the section ends where the next heading begins.
///
/// The parser is what stands between this test and a green run over a table it
/// never found, so it is shown finding one — and shown not carrying on into the
/// entry underneath.
#[test]
fn the_table_is_the_one_under_that_heading_and_no_other() {
    let document = format!(
        "\
## Filesystems and paths

{THE_ENTRY}
| What a turn inherits | What it permits | What it does not permit | Reproduced in | Release |
|---|---|---|---|---|
| `a file open for reading` | every byte | it cannot be reopened | `what_a_turn_inherits.rs` | v0.5 |
| `a directory descriptor` | the handle stays valid | openat is still an open | `what_a_turn_inherits.rs` | v0.5 |

### Something else entirely
| `a file open for appending` | watched since tomorrow | never | `nowhere.rs` | v9 |
"
    );
    let section = the_section_under(&document, THE_ENTRY);
    let rows = the_table_in(section);
    assert_eq!(
        rows.iter().map(|row| row.what.as_str()).collect::<Vec<_>>(),
        ["a file open for reading", "a directory descriptor"],
        "the parser read a table that is not the account, or stopped reading part way through one"
    );
    assert_eq!(
        rows.first().map(|row| row.reproduced_in.as_str()),
        Some("what_a_turn_inherits.rs"),
        "the reproduction cell is not where the parser thinks it is"
    );
    assert_eq!(
        rows.first().map(|row| row.release.as_str()),
        Some("v0.5"),
        "the release cell is not where the parser thinks it is"
    );
    assert!(
        the_table_in(the_section_under(
            &document,
            "### A heading nothing is under"
        ))
        .is_empty(),
        "the parser found rows under a heading that is not in the document"
    );
}

/// And the hooks are read from the declaration rather than from a sentence
/// about one — which matters more here than anywhere, because this entry's own
/// prose names both hooks it is asserting the absence of.
#[test]
fn the_hooks_are_read_from_where_they_are_declared() {
    let source = "\
/// The hook is `file_permission` in a sentence, and this is not a declaration.
#[lsm(hook = \"file_open\")]
fn file_open(file: *const c_void) -> i32 {
#[lsm(hook = \"socket_connect\")]
";
    assert_eq!(
        hooks_declared_in(source),
        ["file_open".to_owned(), "socket_connect".to_owned()]
            .into_iter()
            .collect::<BTreeSet<String>>(),
        "a hook named in prose was taken for one the programme has, which would make this test \
         fail on the entry's own explanation of what would close it"
    );
}
