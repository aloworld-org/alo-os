//! Contents are never sent anywhere, nothing here is a conversation, the
//! index is not the record, and the walk is `alo-files`'.
//!
//! The plan's constraint: *contents are indexed, and contents are never sent
//! anywhere — nothing in this crate opens a socket, and a test reads the
//! crate's shipped source to say so. Nothing here is a conversation: no model
//! is asked what a file is about. The walk is `alo-files`' walker. The index
//! is not the record and never holds anything the record does.*
//!
//! | The promise | The test |
//! |---|---|
//! | nothing in the shipped source opens a socket, runs anything, or asks a model | [`nothing_in_the_shipped_source_opens_a_socket_or_asks_anybody`] |
//! | the walk is `alo-files`', the format is JSON, and nothing else is rented | [`the_walk_is_alo_files_and_nothing_else_is_rented`] |
//! | the index takes no account of who asked, and holds nothing the record does | [`the_index_takes_no_account_of_who_asked`] |
//!
//! # What is read, and what is deliberately not
//!
//! Comments and string literals are taken out first, so a file may explain
//! at length that it opens no socket. What is left is code, and in code none
//! of the identifiers below may appear. Unit tests at the foot of a file are
//! not read: every `#[cfg(test)]` module is the last thing in its file.

#![expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on a file that could not be read is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_finding::{Entry, Index, NotIndexed, Query};

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

/// **Nothing in the shipped source opens a socket, runs anything, asks a
/// model, or reads the record.**
///
/// Each name is one road from an index to somewhere it must not go: the
/// network, a process, a model, the record, a grant, the environment.
#[test]
fn nothing_in_the_shipped_source_opens_a_socket_or_asks_anybody() {
    const FORBIDDEN: &[&str] = &[
        // the network
        "net",
        "TcpStream",
        "TcpListener",
        "UdpSocket",
        "UnixStream",
        "UnixListener",
        "connect",
        "bind",
        "ureq",
        "http",
        "https",
        // a conversation
        "alo_asking",
        "alo_answering",
        "alo_models",
        "alo_turn",
        "alo_protocol",
        "Question",
        "Model",
        // running anything
        "Command",
        "spawn",
        "libc",
        "rustix",
        // the record, a grant, and who is asking
        "alo_record",
        "alo_capability",
        "alo_granted",
        "Record",
        "Grant",
        "Caller",
        "Agent",
        // a setting from the environment
        "env",
        "var",
        "var_os",
        "args",
        // a walk of its own — the walk is alo-files'
        "read_dir",
    ];
    for at in shipped_source() {
        let code = code_of(&at);
        for (number, line) in code.lines().enumerate() {
            for word in identifiers(line) {
                assert!(
                    !FORBIDDEN.contains(&word),
                    "{}:{}: `{word}` — this crate indexes, and sends nothing anywhere",
                    at.display(),
                    number + 1
                );
            }
        }
    }
}

/// **The walk is `alo-files`', the file is JSON, and nothing else is
/// rented.** The manifest names exactly five dependencies, none of which
/// reaches the network, and every absolute path in the shipped source is a
/// path in a doc comment's example rather than one the code opens.
#[test]
fn the_walk_is_alo_files_and_nothing_else_is_rented() {
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
    assert_eq!(
        dependencies,
        [
            "alo-files",
            "alo-strings",
            "serde",
            "serde_json",
            "thiserror"
        ]
    );
    assert!(
        !manifest.contains("[dev-dependencies]"),
        "a test indexes the disk with nothing rented either"
    );

    let walks = shipped_source()
        .iter()
        .filter(|at| code_of(at).contains("Walking"))
        .count();
    assert_eq!(walks, 1, "one file walks, and it walks with alo-files");

    for at in shipped_source() {
        for (number, line) in code_of(&at).lines().enumerate() {
            assert!(
                !line.contains("/proc") && !line.contains("/etc") && !line.contains("/var"),
                "{}:{}: a path of the machine's, in code",
                at.display(),
                number + 1
            );
        }
    }
}

/// **The index takes no account of who asked, and holds nothing the record
/// does.**
///
/// [`Index::of`] takes a folder and nothing else: no caller, no grant, no
/// name. [`Index::find`] takes the index and a query. There is no argument
/// through which an agent and a person could be told apart, so there is no
/// road by which they could be given different answers — and an [`Entry`]
/// has five fields, none of which is an agent, an approval or a moment
/// anybody asked. The assignments are the test; they do not compile against
/// a signature that asks.
#[test]
fn the_index_takes_no_account_of_who_asked() {
    let of: fn(&Path) -> Result<Index, NotIndexed> = Index::of;
    let again: fn(&Index) -> Result<Index, NotIndexed> = Index::again;
    let find: for<'a> fn(&'a Index, &Query) -> Vec<&'a Entry> = Index::find;
    // Nothing is indexed by the assignment, and nothing here needs it to
    // be: the shape is the fact.
    let _ = (of, again, find);

    let text = std::fs::read_to_string(here().join("src").join("entry.rs")).unwrap();
    let fields: Vec<&str> = text
        .split("pub struct Entry {")
        .nth(1)
        .unwrap()
        .split('}')
        .next()
        .unwrap()
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("pub "))
        .collect();
    assert_eq!(
        fields,
        [
            "pub below: String,",
            "pub kind: Kind,",
            "pub bytes: u64,",
            "pub modified: Moment,",
            "pub contents: Contents,",
        ]
    );
}
