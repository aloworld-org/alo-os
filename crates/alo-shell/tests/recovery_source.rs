//! The recovery screen's promises that are about shape, read from the source
//! this crate ships.
//!
//! The plan's constraint is that **nothing is decided here; a rollback is
//! `alo-keeping-up`'s act carried out through the broker**, and its acceptance
//! adds that the screen is **reachable before sign-in** and that it **touches
//! nothing a person owns**. A test that drew the screen would only show the
//! frame it was handed. These read the files instead, so a compositor that
//! opens a person's folder, starts a rollback itself, asks who is signed in, or
//! writes a sentence of its own is a failing build.
//!
//! Comments and string literals' contents are taken out first, and unit tests
//! are not read: every `#[cfg(test)]` module in these files is the last thing
//! in its file, and `*_tests.rs` and `*_testing.rs` files are tests.
#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// This crate's source directory.
fn src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// The files that make up the recovery screen: their names and their code.
fn the_recovery_files() -> Vec<(String, Vec<(usize, String)>)> {
    let mut read = Vec::new();
    for entry in std::fs::read_dir(src()).unwrap() {
        let at = entry.unwrap().path();
        let named = at.file_name().unwrap().to_string_lossy().into_owned();
        let is_one = named.starts_with("recovery_") || named == "nested_recovery.rs";
        let is_a_test = named.ends_with("_tests.rs") || named.ends_with("_testing.rs");
        if is_one && !is_a_test {
            read.push((named, the_code_of(&std::fs::read_to_string(&at).unwrap())));
        }
    }
    read.sort();
    let names: Vec<&str> = read.iter().map(|(named, _)| named.as_str()).collect();
    assert_eq!(
        names,
        [
            "nested_recovery.rs",
            "recovery_keys.rs",
            "recovery_paint.rs",
            "recovery_raster.rs",
            "recovery_reached.rs",
            "recovery_screen.rs",
            "recovery_seat.rs",
        ],
        "a recovery file was added or lost, and this test has to be told"
    );
    read
}

/// The code of a file: no unit tests, no comments, no string literals.
fn the_code_of(written: &str) -> Vec<(usize, String)> {
    let mut code = Vec::new();
    let mut inside = false;
    let mut escaped = false;
    for (which, line) in written.lines().enumerate() {
        if line.trim_start().starts_with("#[cfg(test)]") {
            break;
        }
        if !inside && line.trim_start().starts_with("//") {
            continue;
        }
        let outside = outside_strings(line, &mut inside, &mut escaped);
        let code_part = match outside.split_once("//") {
            Some((before, _)) => before.to_owned(),
            None => outside,
        };
        code.push((which + 1, code_part));
    }
    code
}

/// One line with the contents of its string literals taken out.
fn outside_strings(line: &str, inside: &mut bool, escaped: &mut bool) -> String {
    let mut outside = String::new();
    let mut previous = None;
    for letter in line.chars() {
        if *inside {
            if *escaped {
                *escaped = false;
            } else if letter == '\\' {
                *escaped = true;
            } else if letter == '"' {
                *inside = false;
                outside.push('"');
            }
            previous = Some(letter);
            continue;
        }
        if letter == '"' && previous != Some('\'') {
            *inside = true;
        }
        outside.push(letter);
        previous = Some(letter);
    }
    *escaped = false;
    outside
}

/// **It is reachable before anybody has signed in.** Nothing in these files
/// names an account, a session, a greeting or the sign-in screen — so the
/// screen a person reaches when their machine will not start does not first
/// need the part of the machine that may be the thing that will not start.
#[test]
fn it_is_reachable_before_anybody_has_signed_in() {
    for (named, code) in the_recovery_files() {
        for (which, line) in code {
            for asking in [
                "alo_accounts",
                "alo_sessiond",
                "alo_greeting",
                "alo_setting_up",
                "Session",
                "SignIn",
                "Greeting",
            ] {
                assert!(
                    !line.contains(asking),
                    "{named}:{which} needs somebody signed in (`{asking}`): {line}"
                );
            }
        }
    }
}

/// **It touches nothing a person owns.** No file is opened, no path is named
/// and no folder of anybody's is reached: the screen is handed what the base
/// reported and the person's vocabulary, and it reads nothing else.
#[test]
fn it_touches_nothing_a_person_owns() {
    for (named, code) in the_recovery_files() {
        for (which, line) in code {
            for reaching in [
                "std::fs",
                "fs::",
                "File",
                "PathBuf",
                "read_to_string",
                "read_dir",
                "alo_files",
                "alo_keeping ",
                "alo_record",
                "alo_remembering",
                "home",
            ] {
                assert!(
                    !line.contains(reaching),
                    "{named}:{which} reaches for what a person owns (`{reaching}`): {line}"
                );
            }
        }
    }
}

/// **It carries no rollback out.** Going back is `alo-keeping-up`'s act,
/// carried out through the broker; nothing here stages, returns, runs a command
/// or starts a process, so a compositor cannot replace the operating system on
/// its own.
#[test]
fn it_carries_no_rollback_out() {
    for (named, code) in the_recovery_files() {
        for (which, line) in code {
            for doing in [
                "Returning",
                "Staging",
                "alo_updating",
                "alo_broker",
                "alo_brokerd",
                "Command",
                "process::",
                "bootc",
                "systemctl",
            ] {
                assert!(
                    !line.contains(doing),
                    "{named}:{which} carries a rollback out (`{doing}`): {line}"
                );
            }
        }
    }
}

/// **It words nothing.** Every sentence on the screen arrives through a
/// `said(` from the crate that decided it; nothing here builds a sentence,
/// declares a word or names a colour.
#[test]
fn it_words_nothing_and_names_no_colour() {
    for (named, code) in the_recovery_files() {
        for (which, line) in code {
            for writing in [
                "Word::saying",
                "declare_into",
                "format!",
                "push_str",
                "to_uppercase",
                "Token::",
                "Colour::of",
            ] {
                assert!(
                    !line.contains(writing),
                    "{named}:{which} words or colours something itself (`{writing}`): {line}"
                );
            }
        }
    }
}
