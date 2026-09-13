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
//!
//! # What is read, and what is deliberately not
//!
//! Comments and string literals are taken out first, so a file may explain
//! at length that it does not signal a process. What is left is code, and in
//! code none of the identifiers below may appear. Unit tests at the foot of a
//! file are not read: every `#[cfg(test)]` module is the last thing in its
//! file, and a test spawns a process to have something to measure.

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
        // a setting, or who is asking
        "env",
        "var",
        "var_os",
        "args",
        "Caller",
        "Grant",
        "alo_capability",
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
/// The manifest names exactly three dependencies, none of which reads a
/// process — `alo-files` is the walk under a folder, borrowed so that this
/// repository has one opinion about what a link is — and every absolute path
/// in the shipped source is under `/proc`.
#[test]
fn the_numbers_come_from_proc_and_from_no_rented_crate() {
    let manifest = std::fs::read_to_string(here().join("Cargo.toml")).unwrap();
    let dependencies: Vec<&str> = manifest
        .split("[dependencies]")
        .nth(1)
        .unwrap()
        .split("\n[")
        .next()
        .unwrap()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.split('=').next().unwrap().trim())
        .collect();
    assert_eq!(dependencies, ["alo-files", "alo-strings", "thiserror"]);
    assert!(
        !manifest.contains("[dev-dependencies]"),
        "a test measures the kernel with nothing rented either"
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
