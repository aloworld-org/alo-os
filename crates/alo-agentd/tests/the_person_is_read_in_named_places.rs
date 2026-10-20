//! **Every place this crate asks the environment who the person is, is on a
//! list that says why — and a new one fails this test in any spelling.**
//!
//! **`alo-agentd` runs as the person** — `alo-agentd.service` sets `User=alo`, and
//! `image/usr/lib/tmpfiles.d/alo.conf` says the same in words: *alo-agentd —
//! which runs as the person, holding nothing*. So `$HOME` and
//! `$XDG_CONFIG_HOME` as this process has them **are** theirs, and reading them
//! is the daemon reading its own settings.
//!
//! **That is a condition, not an assumption.** The day something serves two
//! people from one process, every one of these reads becomes one person reading
//! another's directories, and it becomes so silently.
//!
//! *`settings.rs` attributed this to "ADR 0019 §3" and that citation is wrong:
//! ADR 0019 is `a-runtime-is-found-not-configured`, which `alo-models` cites
//! correctly. The claim is discussed in ADR 0088, which quotes the `tmpfiles`
//! comment above. Corrected here rather than carried forward, and the unit file
//! is cited instead because it is the thing that makes it true.*
//!
//! The protection that condition asks for is therefore not *read it once* — some of
//! these reads are for different jobs and cannot be merged. It is that the
//! list of places is **short, named, and known**, so that day is a day of
//! editing a list rather than searching a crate.
//!
//! # Why a test and not a paragraph
//!
//! `settings.rs` carried the paragraph. It said *the environment is read here
//! and nowhere else in this crate*, and on 2026-10-20 that was wrong **three
//! times over** — and the three wrong readers each used a different spelling,
//! which is why no single search had ever found them:
//!
//! | | spelling |
//! |---|---|
//! | `questions.rs` (a plain duplicate of the settings road, now removed) | `var_os(CONFIG_HOME)` — the bare constant |
//! | `starting.rs` (the desktop-entry specification's own answer) | `var_os("HOME")` — a string literal |
//! | `the_bracket_this_turn_takes.rs` (the bracket road) | `var_os(alo_choosing::HOME)` — a qualified path |
//!
//! A grep for the bare constant misses the literal; a grep for the literal
//! misses the qualified path. **So this test does not grep for a spelling. It
//! finds every `env::var`/`env::var_os` call and reads its argument**, which
//! catches all three and anything else anybody invents.
//!
//! # If this test has stopped you
//!
//! Adding your file to [`WHERE_THE_PERSON_IS_READ`] is a legitimate fix *if you
//! can write the reason*. If the reason is *I needed the person's home here*,
//! the fix is almost certainly to take it as an argument instead, the way
//! `settings::of_a_session` and `WhereBracketsGo::of_a_session` both do — that
//! is what makes them testable against a directory that is not yours.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// Every variable that names a directory belonging to the person.
///
/// `DBUS_SESSION_BUS_ADDRESS` and the like are deliberately absent: they name a
/// socket this process is already talking to, not a place somebody's documents
/// are, so a second person in one process does not turn them into the wrong
/// person's files.
const WHAT_NAMES_THE_PERSONS_PLACE: [&str; 4] =
    ["HOME", "XDG_CONFIG_HOME", "XDG_DATA_HOME", "XDG_DATA_DIRS"];

/// The files allowed to read one, and why each may.
///
/// **Adding a row is a decision, not a formality.** Each is a place that becomes
/// wrong on the day one process serves two people.
const WHERE_THE_PERSON_IS_READ: [(&str, &str); 3] = [
    (
        "settings.rs",
        "the settings road: whose configuration file this service reads, as the person it runs as",
    ),
    (
        "starting.rs",
        "where applications are installed, which the desktop-entry specification \
         answers with the environment itself",
    ),
    (
        "the_bracket_this_turn_takes.rs",
        "the bracket road: where a turn's snapshot is kept",
    ),
];

/// **Nothing reads the person's place except the files that say why.**
#[test]
fn the_person_is_read_only_where_a_reason_is_written_down() {
    let found = every_read_of_the_persons_place();

    // A control first. An empty list here would pass the assertion below for
    // the worst possible reason — a walk that read no files, or a matcher that
    // matches nothing — and this file's whole subject is a search that missed
    // three things.
    assert!(
        found.len() >= 3,
        "this test found {} reads of the person's place, and there are at least \
         three in this crate. The walk or the matcher is broken, not the code: {found:#?}",
        found.len()
    );

    let unnamed: Vec<&(PathBuf, String)> = found
        .iter()
        .filter(|(file, _)| {
            let name = file
                .file_name()
                .expect("a file has a name")
                .to_string_lossy();
            !WHERE_THE_PERSON_IS_READ
                .iter()
                .any(|(allowed, _)| *allowed == name)
        })
        .collect();

    assert!(
        unnamed.is_empty(),
        "these read a variable naming the person's own directories, and no reason \
         is written down for them.\n\nEither take the value as an argument — as \
         settings::of_a_session and WhereBracketsGo::of_a_session do — or add the \
         file to WHERE_THE_PERSON_IS_READ with the reason it must read the \
         environment itself.\n\n{unnamed:#?}"
    );
}

/// **Every file on the list is still there and still reads one.**
///
/// The other direction, and the one that rots silently: a list that keeps
/// granting permission to a file that has been deleted, renamed, or fixed is a
/// list nobody can trust the length of. Without this, the test above would keep
/// passing while the list grew into a historical record.
#[test]
fn the_list_has_no_rows_that_nothing_needs_any_more() {
    let found = every_read_of_the_persons_place();
    let stale: Vec<&str> = WHERE_THE_PERSON_IS_READ
        .iter()
        .map(|(file, _)| *file)
        .filter(|allowed| {
            !found.iter().any(|(file, _)| {
                file.file_name()
                    .expect("a file has a name")
                    .to_string_lossy()
                    == *allowed
            })
        })
        .collect();

    assert!(
        stale.is_empty(),
        "these are allowed to read the person's place and no longer do. Remove \
         them from WHERE_THE_PERSON_IS_READ, so its length keeps meaning \
         something: {stale:?}"
    );
}

/// **And every reason is a reason**, rather than an empty string somebody added
/// to get past the test above.
#[test]
fn every_row_says_why() {
    for (file, why) in WHERE_THE_PERSON_IS_READ {
        assert!(
            why.split_whitespace().count() >= 4,
            "{file} is allowed to read the person's place with no reason worth \
             reading: {why:?}"
        );
    }
}

/// Every `env::var`/`env::var_os` call in this crate's own source whose argument
/// names the person's place, by any spelling.
///
/// The argument is read rather than the call being matched against a pattern,
/// so a bare constant, a string literal and a fully-qualified path are all
/// caught — which the three readers in the header are, one of each.
fn every_read_of_the_persons_place() -> Vec<(PathBuf, String)> {
    let mut found = Vec::new();
    for file in every_file(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src")) {
        let text = std::fs::read_to_string(&file).expect("this crate's own source");
        for line in text.lines() {
            let code = only_the_code(line);
            let Some(at) = code.find("env::var") else {
                continue;
            };
            let argument = &code[at..];
            if WHAT_NAMES_THE_PERSONS_PLACE
                .iter()
                .any(|named| argument.contains(named))
            {
                found.push((file.clone(), line.trim().to_owned()));
            }
        }
    }
    found
}

/// Every `.rs` file under a directory, including the nested ones.
fn every_file(at: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let Ok(entries) = std::fs::read_dir(at) else {
        return files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(every_file(&path));
        } else if path.extension().is_some_and(|it| it == "rs") {
            files.push(path);
        }
    }
    files
}

/// A line with any comment taken off it, so prose naming a variable is not read
/// as code reading one.
///
/// This file's own header names all four variables in a table, and several of
/// the crate's doc comments do too — without this, the test would report the
/// documentation that explains it.
///
/// **Measured 2026-10-20: removing this changes no verdict, and that is said
/// here rather than left for somebody to discover.** Every line in this crate
/// where `env::var` and one of the four names appear together — comments
/// included — is already in a file on the list. So this mutation is not caught,
/// and the reason it is kept anyway is that what it prevents is a **false
/// failure**: a doc comment somewhere else explaining `$HOME` beside an
/// unrelated `env::var` would fail a test about something it does not do. A
/// nuisance rather than a hole, and the only mutation of six this test does not
/// catch.
fn only_the_code(line: &str) -> &str {
    match line.find("//") {
        Some(at) => &line[..at],
        None => line,
    }
}
