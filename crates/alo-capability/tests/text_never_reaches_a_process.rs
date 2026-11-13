//! **No source file that reads a text argument starts a process.**
//!
//! ADR 0001, amended 2026-11-13: text is the one kind of argument that is free
//! text, so what keeps law 2 true for it is that nothing on this machine hands
//! it to an interpreter or a shell. This test reads every Rust source file in
//! the workspace, finds each one that names `Value::Text` — the only way to get
//! a text argument's contents out of a call — and refuses the build if that
//! file also starts a process.
//!
//! **A tripwire, and honest about it.** A file could take the text out and pass
//! it to another file that starts a process, and this would not see it; the
//! rule that stops that is review, as ADR 0001 §1 says of every verb. What this
//! catches is the obvious version, at the moment somebody writes it, on every
//! build. It proves it ran: it must find the files that are known to name the
//! value, or it fails rather than passing over nothing.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

/// What starting a process looks like in Rust source.
const STARTS_A_PROCESS: [&str; 5] = [
    "std::process",
    "process::Command",
    "Command::new",
    "tokio::process",
    "posix_spawn",
];

/// Every `.rs` file under a directory.
fn rust_files(directory: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == "target") {
                continue;
            }
            rust_files(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
}

/// Every file naming the text value is free of a process start, and the scan
/// found the files it must have found.
#[test]
fn no_file_that_reads_a_text_argument_starts_a_process() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the crates directory")
        .to_path_buf();
    let mut files = Vec::new();
    rust_files(&crates, &mut files);
    // This file names both halves in its own words, so it is not a reader.
    let this_file = Path::new(file!())
        .file_name()
        .expect("this test's own name");

    let mut readers = Vec::new();
    for file in &files {
        if file.file_name() == Some(this_file) {
            continue;
        }
        let Ok(source) = fs::read_to_string(file) else {
            continue;
        };
        if !source.contains("Value::Text") {
            continue;
        }
        readers.push(file.clone());
        for starts in STARTS_A_PROCESS {
            if source.contains(starts) {
                panic!(
                    "{} reads a text argument and also names `{starts}`: a text argument is \
                     never handed to a process (ADR 0001, amended 2026-11-13)",
                    file.display()
                );
            }
        }
    }

    for known in [
        ["alo-capability", "src", "arg.rs"],
        ["alo-record", "src", "written.rs"],
    ] {
        let known: PathBuf = known.iter().collect();
        assert!(
            readers.iter().any(|reader| reader.ends_with(&known)),
            "the scan did not find {} naming the text value, so it did not read what it claims \
             to; it read {} files",
            known.display(),
            files.len()
        );
    }
}
