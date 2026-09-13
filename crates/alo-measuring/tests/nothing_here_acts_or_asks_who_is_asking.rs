//! This crate reads, and does nothing else; and the list is the same
//! whoever asked for it.
//!
//! The plan's constraint: *read-only, and no process is signalled, stopped or
//! reniced here — what is using the machine is a measurement, and acting on
//! it is a verb that would need a grant. No `sysinfo`-style dependency: the
//! numbers this promise rests on are files in `/proc`, and renting a crate to
//! read them would be a second author's opinion about what memory means.*
//! And the acceptance's last clause: *the list is the same whether an agent
//! or a person asked, because it is a list of facts.*
//!
//! | The promise | The test |
//! |---|---|
//! | nothing in the shipped source signals, stops, renices, writes or opens a socket | [`nothing_in_the_shipped_source_signals_stops_renices_or_writes`] |
//! | the numbers come from `/proc`, through no rented reader | [`the_numbers_come_from_proc_and_from_no_rented_crate`] |
//! | the list takes no account of who asked | [`the_list_takes_no_account_of_who_asked`] |
//! | only the verbs' declaration and their door name the capability model | [`only_the_verbs_and_their_door_name_the_capability_model`] |
//!
//! # What is read, and what is deliberately not
//!
//! Comments and string literals are taken out first, so a file may explain
//! at length that it does not signal a process. What is left is code, and in
//! code none of the identifiers below may appear. Unit tests at the foot of a
//! file are not read: every `#[cfg(test)]` module is the last thing in its
//! file, and a test spawns a process to have something to measure.
//!
//! # Two files may name the capability model, and only those two
//!
//! `what_is_running` and `what_is_filling` are verbs, and a verb is declared
//! in `alo-capability`'s shape and carried out under an
//! `alo_capability::Authorised`. So `verbs.rs` and `measured.rs` name that
//! crate, and **nothing else here does**: the reading, the counting and the
//! tree never see a grant, an agent or an authority, which is what keeps the
//! numbers the same whoever asked.
//! [`only_the_verbs_and_their_door_name_the_capability_model`] holds both
//! halves — those two may, and no third file may.

#![expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on a file that could not be read is the failure being reported"
)]

use std::path::{Path, PathBuf};
use std::time::Duration;

use alo_measuring::{Holding, NotMeasured, Reading, Running};

/// This crate's own directory.
fn here() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every file of shipped source: `src/*.rs`.
fn shipped_source() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(here().join("src"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "the crate has source");
    files
}

/// The code of a file: no unit tests, no comments, no string literals.
fn code_of(at: &Path) -> String {
    let text = std::fs::read_to_string(at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()));
    let shipped = text.split("#[cfg(test)]").next().unwrap();
    // One pass over the whole file rather than a line at a time, because a
    // string literal in this crate's `words.rs` runs over several lines with
    // a `\` at the end of each, and the second line of one is still a string.
    let mut code = String::new();
    let mut in_string = false;
    let mut in_comment = false;
    let mut chars = shipped.chars().peekable();
    while let Some(c) = chars.next() {
        if in_comment {
            if c == '\n' {
                in_comment = false;
                code.push('\n');
            }
            continue;
        }
        if in_string {
            if c == '\\' {
                chars.next();
            } else if c == '"' {
                in_string = false;
            } else if c == '\n' {
                code.push('\n');
            }
            continue;
        }
        if c == '"' {
            in_string = true;
            continue;
        }
        if c == '/' && chars.peek() == Some(&'/') {
            in_comment = true;
            continue;
        }
        code.push(c);
    }
    code
}

/// Every identifier in a piece of code.
fn identifiers(code: &str) -> Vec<&str> {
    code.split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|word| !word.is_empty())
        .collect()
}

/// The two files that declare the verbs and carry them out, which are the
/// only two allowed to name the capability model.
const THE_VERBS_AND_THEIR_DOOR: [&str; 2] = ["verbs.rs", "measured.rs"];

/// The name of the capability model, as code names it.
const THE_CAPABILITY_MODEL: &str = "alo_capability";

/// Whether this file is one of the two.
fn is_a_verb_or_its_door(at: &Path) -> bool {
    at.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| THE_VERBS_AND_THEIR_DOOR.contains(&name))
}

/// The lines of a file's code that name this identifier.
fn lines_naming(at: &Path, named: &str) -> Vec<usize> {
    code_of(at)
        .lines()
        .enumerate()
        .filter(|(_, line)| identifiers(line).contains(&named))
        .map(|(number, _)| number + 1)
        .collect()
}

/// The names in one section of the manifest.
fn section_of<'a>(manifest: &'a str, section: &str) -> Vec<&'a str> {
    manifest
        .split(section)
        .nth(1)
        .unwrap()
        .split("\n[")
        .next()
        .unwrap()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.split('=').next().unwrap().trim())
        .collect()
}

/// **Nothing in the shipped source acts on a process, writes anything, or
/// opens a socket.**
///
/// Each name is one road from a measurement to an action: spawning or
/// signalling a process, changing its priority, writing or removing a file,
/// reaching the network, reading the environment for a setting.
#[test]
fn nothing_in_the_shipped_source_signals_stops_renices_or_writes() {
    const FORBIDDEN: &[&str] = &[
        // acting on a process
        "Command",
        "kill",
        "signal",
        "nice",
        "renice",
        "setpriority",
        "libc",
        "rustix",
        // writing
        "write",
        "write_all",
        "File",
        "OpenOptions",
        "create",
        "create_dir",
        "create_dir_all",
        "remove_file",
        "remove_dir",
        "remove_dir_all",
        "rename",
        "copy",
        // the network
        "net",
        "TcpStream",
        "TcpListener",
        "UdpSocket",
        "UnixStream",
        "connect",
        "bind",
        // a setting, or who is asking — the capability model itself is held
        // apart below, to exactly two files
        "env",
        "var",
        "var_os",
        "args",
        "Caller",
        "Grant",
        "alo_granted",
    ];
    for at in shipped_source() {
        let code = code_of(&at);
        for (number, line) in code.lines().enumerate() {
            for word in identifiers(line) {
                assert!(
                    !FORBIDDEN.contains(&word),
                    "{}:{}: `{word}` — this crate measures, and does nothing else",
                    at.display(),
                    number + 1
                );
            }
        }
    }
}

/// **The numbers come from `/proc`, read by this crate's own code.**
///
/// The manifest names exactly four dependencies, none of which reads a
/// process — `alo-files` is the walk under a folder, borrowed so that this
/// repository has one opinion about what a link is, and `alo-capability` is
/// the shape of a verb and nothing more — and every absolute path in the
/// shipped source is under `/proc`. The one dependency a test has is the
/// record, so a test can show a measurement being written down; nothing
/// shipped reads or writes one.
#[test]
fn the_numbers_come_from_proc_and_from_no_rented_crate() {
    let manifest = std::fs::read_to_string(here().join("Cargo.toml")).unwrap();
    assert_eq!(
        section_of(&manifest, "[dependencies]"),
        ["alo-capability", "alo-files", "alo-strings", "thiserror"]
    );
    assert_eq!(
        section_of(&manifest, "[dev-dependencies]"),
        ["alo-record"],
        "a test measures the kernel with nothing rented but the record it writes into"
    );

    for at in shipped_source() {
        let text = std::fs::read_to_string(&at).unwrap();
        let shipped = text.split("#[cfg(test)]").next().unwrap();
        for (number, line) in shipped.lines().enumerate() {
            // The odd-numbered pieces between quotation marks are the
            // string literals on the line.
            for literal in line.split('"').skip(1).step_by(2) {
                if literal.starts_with('/') {
                    assert!(
                        literal.starts_with("/proc"),
                        "{}:{}: `{literal}` is a path outside /proc",
                        at.display(),
                        number + 1
                    );
                }
            }
        }
    }
}

/// **The list takes no account of who asked, and neither does the tree.**
///
/// [`Reading::now`] takes nothing: no caller, no grant, no name. There is no
/// argument through which an agent and a person could be told apart, so
/// there is no road by which they could be given different lists.
/// [`Holding::of`] takes a folder and nothing else, for the same reason. The
/// assignments are the test; they do not compile against a signature that
/// asks.
#[test]
fn the_list_takes_no_account_of_who_asked() {
    let now: fn() -> Result<Reading, NotMeasured> = Reading::now;
    let since: fn(&Reading, &Reading, Duration) -> Result<Running, NotMeasured> = Reading::since;
    let of: fn(&Path) -> Result<Holding, NotMeasured> = Holding::of;
    // Nothing is measured by the assignment, and nothing here needs it to
    // be: the shape is the fact.
    let _ = (now, since, of);
}

/// **Only the verbs' declaration and their door name the capability model.**
///
/// Both halves, because each is a way of quietly becoming a different crate.
/// A third file naming it would be the reading or the counting starting to
/// care who asked; the two files not naming it would be a verb declared out
/// of nothing, which cannot compile — so the second half is here to keep the
/// first from passing on an empty list.
#[test]
fn only_the_verbs_and_their_door_name_the_capability_model() {
    let mut naming_it = Vec::new();
    for at in shipped_source() {
        let lines = lines_naming(&at, THE_CAPABILITY_MODEL);
        if is_a_verb_or_its_door(&at) {
            assert!(
                !lines.is_empty(),
                "{} declares or carries out a verb and never names {THE_CAPABILITY_MODEL}",
                at.display()
            );
            naming_it.push(at);
            continue;
        }
        assert!(
            lines.is_empty(),
            "{}:{}: `{THE_CAPABILITY_MODEL}` — the list takes no account of who asked, and \
             only the verbs and their door may name the capability model",
            at.display(),
            lines.first().copied().unwrap_or(0)
        );
    }
    assert_eq!(naming_it.len(), THE_VERBS_AND_THEIR_DOOR.len());
}
