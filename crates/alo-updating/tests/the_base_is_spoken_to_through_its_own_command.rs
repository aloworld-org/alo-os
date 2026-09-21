//! ADR 0011 as it now stands, held to this crate.
//!
//! The decision said *the base is rented and unmodified, and it is spoken to
//! through its own command*, without exception, until 2026-09-21. It now holds
//! without exception for every act that **changes** the machine and is narrowed
//! for one question the base refuses to answer to the person at all — *which
//! build is this machine running*.
//!
//! A narrowing that lives only in a document is a narrowing that widens the
//! next time somebody is in a hurry, so this file is the three halves of it
//! that can be checked:
//!
//! 1. **the decision carries it**, in its own text, with the refusal that
//!    forced it quoted;
//! 2. **the road the narrowing permits runs no program at all** — it reads two
//!    files the base has already written, and if it ever starts a process this
//!    fails;
//! 3. **the base's own command is still the only program this crate starts**,
//!    with each argument arriving as one argument and no shell between them,
//!    which is law 2 at the place it is kept.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_updating::{Base, THE_PROGRAM, THE_STATUS, TheBase};

/// The decision this crate is held to.
const THE_DECISION: &str =
    "../../docs/decisions/0011-the-base-is-rented-and-the-image-is-a-container.md";

/// The road that reads what the base has already written down, whole.
const THE_ROAD: [&str; 3] = ["booted.rs", "origin.rs", "written_down.rs"];

/// The one file allowed to start a program.
const THE_ONE_THAT_STARTS_IT: &str = "the_base.rs";

/// A file of this crate's, as text.
fn source(named: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join(named),
    )
    .expect("a source file of this crate's")
}

/// A file's text with everything written for a reader taken out: doc comments,
/// ordinary comments, and the strings a test's fixture spells.
fn without_what_is_written_for_a_reader(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<&str>>()
        .join("\n")
}

/// Every file of this crate's source, by name and text.
fn every_source_file() -> Vec<(String, String)> {
    let mut read = Vec::new();
    for entry in std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("src"))
        .expect("this crate's source")
    {
        let path: PathBuf = entry.expect("a source file").path();
        let named = path
            .file_name()
            .expect("a file name")
            .to_string_lossy()
            .into_owned();
        read.push((
            named,
            std::fs::read_to_string(&path).expect("a source file"),
        ));
    }
    assert!(
        read.len() >= 10,
        "only {} source files were read",
        read.len()
    );
    read
}

/// **The decision carries the narrowing, and the refusal that forced it.**
///
/// Quoted rather than summarised: a reader a year from now meets the sentence
/// the base actually said, which is the only part of this that cannot be
/// argued with.
#[test]
fn the_decision_carries_the_narrowing_and_the_refusal_that_forced_it() {
    let decision =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(THE_DECISION))
            .expect("ADR 0011");

    assert!(
        decision.contains("This command must be executed as the root user"),
        "the refusal that forced the narrowing is not quoted in the decision"
    );
    for saying in [
        "acts that change the machine",
        "which build is this machine running",
        "world-readable",
        "ADR 0001 §2 does not fire",
    ] {
        assert!(
            decision.contains(saying),
            "the decision does not say `{saying}`"
        );
    }
    assert!(
        decision.contains("What this does not permit"),
        "a narrowing with no limit written down is a narrowing that widens"
    );
}

/// **The road the narrowing permits runs no program at all.**
///
/// This is what keeps the amendment as small as it is: alo OS does not speak to
/// a *second* command of the base's, it reads two files the base has already
/// written. A process started anywhere on this road would be a different
/// decision to the one that was made.
#[test]
fn the_road_that_reads_what_was_written_down_runs_no_program() {
    for named in THE_ROAD {
        let text = source(named);
        // `std::process::id()` is deliberately not on this list: what is being
        // refused is **starting** a program, not knowing which process this is.
        for starting in ["Command", "Stdio", "spawn(", ".output()", "exec("] {
            assert!(
                !text.contains(starting),
                "{named} names {starting}, and this road runs no program"
            );
        }
        // And it reads what the base wrote, never the answer the base refuses
        // to give the person. **Read past the comments**, which say `bootc
        // status` and `imageDigest` on purpose: the measurement that decided
        // this road belongs where somebody changing it will meet it, and a
        // guard that forbade naming the thing being refused would push the
        // reason out of the file.
        let code = without_what_is_written_for_a_reader(&text);
        for asking in ["imageDigest", "THE_STATUS", "Base", "bootc"] {
            assert!(
                !code.contains(asking),
                "{named} names {asking} in its code, and this road does not ask the base anything"
            );
        }
    }
}

/// **The base's own command is still the only program this crate starts.**
#[test]
fn the_bases_own_command_is_the_only_program_this_crate_starts() {
    for (named, text) in every_source_file() {
        if named == THE_ONE_THAT_STARTS_IT {
            continue;
        }
        assert!(
            !text.contains("Command::new"),
            "{named} starts a program, and only {THE_ONE_THAT_STARTS_IT} may"
        );
    }
    assert_eq!(THE_PROGRAM, "/usr/bin/bootc");
    assert_eq!(
        THE_STATUS,
        ["status", "--format", "json", "--format-version", "1"],
        "the arguments the base is asked with are this repository's, decided once"
    );
}

/// **Each argument arrives as one argument, and no shell is between them** —
/// law 2 where it is kept, measured through the program rather than argued.
///
/// The same promise `crate::the_base`'s own test makes from the inside, made
/// here from outside the crate, because the acceptance of this change is that
/// the road that **changes** the machine did not move while the road that reads
/// it did.
#[cfg(unix)]
#[test]
fn each_argument_the_base_is_given_arrives_as_one_argument() {
    let marker = std::env::temp_dir().join("alo-updating-adr-0011-was-run");
    let _ = std::fs::remove_file(&marker);

    let answered = TheBase::at(Path::new("/bin/echo"))
        .asked(&[
            "switch".to_owned(),
            format!("a; touch {}", marker.display()),
            "$(b)".to_owned(),
        ])
        .expect("the program answered");

    let said = String::from_utf8(answered).expect("what it printed");
    assert!(said.contains("$(b)"), "{said}");
    assert!(
        !marker.exists(),
        "a shell read what was handed to the base's own command"
    );
}
