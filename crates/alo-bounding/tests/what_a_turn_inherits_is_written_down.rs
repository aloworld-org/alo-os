//! The account of what a turn inherits, held to the programme that does the
//! deciding.
//!
//! `what_a_turn_inherits.rs` measures each inherited descriptor against a
//! running kernel. This file is about the other half of the same job — that
//! somebody auditing the boundary can find the account, and that the account is
//! still true of the machine.
//!
//! # Why a closed gap needs a test at all
//!
//! `the_unwatched_mutations_are_written_down.rs` makes the argument for a gap
//! that is open: documentation of a security limit rots in the direction of
//! understating the protection, which is the direction nobody checks. A gap
//! that has been **closed** rots the other way, and it is worse. The entry says
//! the kernel decides about every read and write; if the hook that makes that
//! true ever leaves the programme — a merge, a rename, a loader built from an
//! older source — the entry goes on saying it, and what is left is a document
//! telling the next person that contents cannot leave a grant when they can.
//!
//! And the entry names the one thing the closure leaves open, a mapping, with
//! the hook that would close it. If that hook arrives and the entry does not
//! move, the document understates the boundary again.
//!
//! So the table in `docs/quirks.md` is parsed rather than admired, and held to
//! six things:
//!
//! - **The hook that decides about a descriptor is on the programme.** Read
//!   out of `crates/alo-bounding-kernel/src/kernel.rs`, where hooks are
//!   declared, and the entry names it.
//! - **The hook that would close what is left is not on the programme**, and
//!   the entry names it too, because *what would close it* is the half of a
//!   documented limit that turns it into work rather than a shrug.
//! - **Every row says what the boundary refuses the inherited thing**, at
//!   length. A row that only said what a turn can still do would be a list
//!   with no boundary in it.
//! - **Every row says what it still permits**, because a hook that refused
//!   every descriptor would pass every refusal in the other file and break
//!   the daemon, and the permitted half is what says it does not.
//! - **Every row names a release `docs/features.md` knows.**
//! - **Every row is reproduced**, in the file the row itself names, against a
//!   running kernel. A row nothing reproduces is a claim about a machine
//!   nobody asked.
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
const THE_ENTRY: &str = "### A descriptor opened before a turn began is decided about on every use";

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

/// The hook that decides about a descriptor rather than about an open, which
/// is the one that closed this gap and has to stay on the programme for the
/// entry to be true.
///
/// `file_permission` fires on every read and every write. Written out rather
/// than counted, for the reason `the_boundary_decides_and_forgets` names its
/// two maps: this hook leaving is a change to what the boundary *is*, and it
/// should leave with somebody looking at it.
const DECIDES_IT: &str = "file_permission";

/// The hook that would close what the closure leaves open: a mapping of a
/// file, which is read by the processor rather than by a syscall.
///
/// Not on the programme, and the entry has to say why. If it arrives, the
/// entry has to move with it.
const STILL_OPEN: &str = "mmap_file";

/// What a turn inherits on this machine, as an exact list.
///
/// Written out for the same reason and read the same way: a row that a stray
/// character in the table made the parser drop would otherwise leave this test
/// green over an account it had stopped checking, and a row somebody removed is
/// a claim about the daemon that should arrive with a person looking at it.
///
/// `a socket already connected` left this list on 2026-09-12 when
/// `socket_sendmsg` decided about it, and came back the same day when
/// `file_permission` closed the rest: the table is now what the boundary says
/// about each inherited thing rather than what it does not, and a socket is
/// the one it says something different about.
const EVERY_ROW: &[&str] = &[
    "a file open for reading",
    "a file open for writing",
    "a file open for appending",
    "a directory descriptor",
    "a socket already connected",
    "a pipe",
    "the way out of a turn",
];

/// One row of the table: something a turn inherits.
#[derive(Debug)]
struct Inherited {
    /// What it is.
    what: String,

    /// What the boundary refuses it now.
    refuses: String,

    /// What it still permits, which is the half that keeps the daemon working.
    permits: String,

    /// The test file that reproduces both against a running kernel.
    reproduced_in: String,

    /// The release that owns it.
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
        let [what, refuses, permits, reproduced_in, release] = cells.as_slice() else {
            continue;
        };
        let Some(what) = what.strip_prefix('`').and_then(|it| it.strip_suffix('`')) else {
            continue;
        };
        rows.push(Inherited {
            what: what.to_owned(),
            refuses: (*refuses).to_owned(),
            permits: (*permits).to_owned(),
            reproduced_in: reproduced_in.trim_matches('`').to_owned(),
            release: (*release).to_owned(),
        });
    }
    rows
}

/// How much of an answer *what the boundary refuses it* and *what it still
/// permits* each have to be.
///
/// A row saying `everything` would pass every other check here. This is not a
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

    if !watched.contains(DECIDES_IT) {
        return Err(format!(
            "the programme no longer has a hook `{DECIDES_IT}`, and {THE_ACCOUNT} still says a \
             descriptor opened before a turn began is decided about on every use. That hook is \
             what stops contents leaving a grant through a descriptor that already existed; if \
             it went on purpose, come to that entry and say what decides now"
        ));
    }
    if !entry.contains(DECIDES_IT) {
        return Err(format!(
            "the entry does not name `{DECIDES_IT}`, which is the hook that decides about a \
             descriptor after it is opened. An account of a closed gap has to say what closed it"
        ));
    }
    if watched.contains(STILL_OPEN) {
        return Err(format!(
            "the programme now has a hook `{STILL_OPEN}`, and {THE_ACCOUNT} still says a mapping \
             of a file is not decided about. Come to that entry: say what the hook decides, and \
             reproduce it — by hand if the suite still cannot"
        ));
    }
    if !entry.contains(STILL_OPEN) {
        return Err(format!(
            "the entry does not name `{STILL_OPEN}`, which is what this closure leaves open. What \
             would close a documented limit is the half that turns it into work rather than a \
             shrug"
        ));
    }

    for row in rows {
        if row.refuses.len() < AN_ANSWER {
            return Err(format!(
                "`{}` is listed without saying what the boundary refuses it. That sentence is \
                 the boundary, and a list of inherited things with no refusal beside them is not \
                 an account of one",
                row.what
            ));
        }
        if row.permits.len() < AN_ANSWER {
            return Err(format!(
                "`{}` is listed without saying what it still permits. A hook that refused every \
                 descriptor would pass every refusal and break the daemon; the permitted half is \
                 what says it does not",
                row.what
            ));
        }
        if !releases.contains(&format!("[{}]", row.release)) {
            return Err(format!(
                "`{}` names the release `{}`, and {THE_RELEASES} has no such tier. Which release \
                 owns a boundary is what makes it a promise rather than a change",
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
/// Everything this repository says about what a turn inherits, read out of
/// the files that say it and compared against the programme that decides.
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
        let text = reading(named);
        assert!(
            text.contains(THE_SUBJECT),
            "{named} does not mention what a turn inherits, and it is one of the two files \
             somebody auditing this boundary's code reads. An account that lives only in \
             {THE_ACCOUNT} is an account the person reading the crate never sees"
        );
        assert!(
            text.contains(DECIDES_IT) && text.contains(STILL_OPEN),
            "{named} does not name both `{DECIDES_IT}` and `{STILL_OPEN}`, and somebody reading \
             the code is owed what decides about a descriptor and what still does not"
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
    let watched: BTreeSet<String> = ["file_open", "file_permission"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    let sound = |what: &str| Inherited {
        what: what.to_owned(),
        refuses: "the first byte, with EACCES, and nothing of the file reaches the folder the \
                  turn was granted"
            .to_owned(),
        permits: "the descriptor stays valid, and the same descriptor to a file inside the grant \
                  is read through as it always was"
            .to_owned(),
        reproduced_in: "what_a_turn_inherits.rs".to_owned(),
        release: "v0.5".to_owned(),
    };
    let entry = "file_permission decides on every read and write; mmap_file is not hooked";
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

    // The one this test exists for: the hook that decides about a descriptor
    // is gone, and the entry still says it decides.
    let unhooked: BTreeSet<String> = ["file_open".to_owned()].into_iter().collect();
    assert!(
        whether_it_is_written_down(
            &unhooked,
            &[sound("a file open for reading")],
            entry,
            releases,
            &reproducing
        )
        .is_err_and(|why| why.contains("no longer has")),
        "a programme with no hook on reads and writes was accepted while the account said the \
         kernel decides about every one"
    );

    // The other direction: the hook that would close what is left arrived,
    // and the entry still calls a mapping unwatched.
    let mapped: BTreeSet<String> = ["file_open", "file_permission", "mmap_file"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    assert!(
        whether_it_is_written_down(
            &mapped,
            &[sound("a file open for reading")],
            entry,
            releases,
            &reproducing
        )
        .is_err_and(|why| why.contains("now has a hook")),
        "a programme that decides about a mapping was accepted while the account said none does"
    );

    // An entry that no longer says what decides, or what is left.
    assert!(
        whether_it_is_written_down(
            &watched,
            &[sound("a file open for reading")],
            "mmap_file is not hooked, and nothing is said about what is",
            releases,
            &reproducing
        )
        .is_err_and(|why| why.contains("does not name `file_permission`")),
        "an entry that does not say what closed the gap was accepted"
    );
    assert!(
        whether_it_is_written_down(
            &watched,
            &[sound("a file open for reading")],
            "file_permission decides, and nothing is said about what it leaves",
            releases,
            &reproducing
        )
        .is_err_and(|why| why.contains("does not name `mmap_file`")),
        "an entry that does not say what the closure leaves open was accepted"
    );

    // A row with no boundary in it, and one with no daemon left.
    let toothless = Inherited {
        refuses: "nothing much".to_owned(),
        ..sound("a file open for reading")
    };
    assert!(
        whether_it_is_written_down(&watched, &[toothless], entry, releases, &reproducing)
            .is_err_and(|why| why.contains("refuses it")),
        "a row that does not say what the boundary refuses was accepted"
    );
    let total = Inherited {
        permits: "nothing".to_owned(),
        ..sound("a file open for reading")
    };
    assert!(
        whether_it_is_written_down(&watched, &[total], entry, releases, &reproducing)
            .is_err_and(|why| why.contains("still permits")),
        "a row that does not say what the inherited descriptor can still do was accepted"
    );

    // A release nobody has heard of, which is a boundary nobody owns.
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
| What a turn inherits | What the boundary refuses it now | What it still permits | Reproduced in | Release |
|---|---|---|---|---|
| `a file open for reading` | the first byte | a descriptor inside the grant | `what_a_turn_inherits.rs` | v0.5 |
| `a directory descriptor` | its listing | the handle stays valid | `what_a_turn_inherits.rs` | v0.5 |

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
        rows.first().map(|row| row.refuses.as_str()),
        Some("the first byte"),
        "the refusal cell is not where the parser thinks it is"
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
/// prose names the hook it asserts the absence of.
#[test]
fn the_hooks_are_read_from_where_they_are_declared() {
    let source = "\
/// The hook is `mmap_file` in a sentence, and this is not a declaration.
#[lsm(hook = \"file_open\")]
fn file_open(file: *const c_void) -> i32 {
#[lsm(hook = \"file_permission\")]
";
    assert_eq!(
        hooks_declared_in(source),
        ["file_open".to_owned(), "file_permission".to_owned()]
            .into_iter()
            .collect::<BTreeSet<String>>(),
        "a hook named in prose was taken for one the programme has, which would make this test \
         fail on the entry's own explanation of what is left open"
    );
}
