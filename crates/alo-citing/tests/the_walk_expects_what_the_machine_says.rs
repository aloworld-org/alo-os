//! **A walk tells somebody what to look for on a screen, so it has to quote
//! what the screen says.**
//!
//! `docs/autonomy/the-mac-lanes-order-while-the-owner-is-away.md` carries a
//! thirteen-step walk for a booted machine, each step with the words that count
//! as a pass. A step quoting words the machine does not say sends the walker to
//! report a fault that is not one — and that report is the most expensive thing
//! this repository can produce, because the walk exists to settle the one
//! condition a green gate says nothing about: *somebody has seen it on a
//! machine*.
//!
//! # Two steps were wrong, and both the same way
//!
//! Measured 2026-10-17:
//!
//! - step 8 said *nobody can tell*, which is what the **code** calls the fifth
//!   waiting answer. The screen says `waiting for something alo cannot
//!   identify`;
//! - step 10 expected `Tests · 23 tests passed · 1 test failed`, and the design
//!   draws `Tests · 23 passed · 1 failed`.
//!
//! Both came from writing a step against an identifier or an earlier draft
//! instead of against the string a person reads. Nothing checked either,
//! because a document and a vocabulary are different files and nobody had put
//! them beside each other.
//!
//! # Why this is a named list and not a sweep
//!
//! Most of what the walk quotes is **not** a vocabulary sentence: *a desktop,
//! and the sign-in screen gone* is prose, `bash · UTF-8` is composed from a
//! shell's own name, and `Ctrl`+`Alt`+`W` is a key. A sweep over every
//! backticked phrase would report all of those and be ignored within a week.
//!
//! So the steps that quote a sentence this machine can say are named here with
//! the key they must match, and the rest are left alone. A step added later that
//! quotes a sentence and is not listed is not caught — which is a real limit,
//! said plainly rather than papered over, and the reason
//! [`every_named_step_is_really_in_the_walk`] exists is to stop the list
//! rotting the other way.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic naming the step and the sentence is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

/// The document holding the walk.
const THE_ORDER: &str = "docs/autonomy/the-mac-lanes-order-while-the-owner-is-away.md";

/// The vocabulary snapshot: every key this machine can say, and its English.
const THE_VOCABULARY: &str = "crates/alo-saying/the-vocabulary.txt";

/// A step of the walk that quotes a sentence this machine can say.
///
/// The number is the step, the key is what it must quote. **Only the steps that
/// quote a vocabulary sentence** — see this file's header for why this is a
/// named list.
const WHAT_EACH_STEP_QUOTES: [(u32, &str); 4] = [
    // Step 3 reads the gutter beside a program's output.
    (3, "terminal.who.program"),
    // Step 6 asks why a command is waiting and reads the answer.
    (6, "terminal.waiting.keyboard"),
    // Step 8 does the same for a clock, where no answer settles it. **This is
    // the one that was wrong**: it quoted the code's name for the answer.
    (8, "terminal.waiting.not-known"),
    // Step 10 reads the provenance under a recognised test result.
    (10, "terminal.result.not-verified"),
];

/// The repository's root, from this crate's own place in it.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("this crate sits two directories under the repository")
}

/// One file of the repository, read.
fn read(named: &str) -> String {
    let at = the_repository().join(named);
    fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// The English the vocabulary gives this key.
///
/// The snapshot is `key<tab>english`, one line each, and it is generated and
/// checked in — so reading it here asks what the machine says rather than what
/// a crate's source declares, which is the same question one step further from
/// the answer.
fn what_the_machine_says(key: &str) -> String {
    let snapshot = read(THE_VOCABULARY);
    for line in snapshot.lines() {
        if let Some((named, english)) = line.split_once('\t')
            && named == key
        {
            return english.to_owned();
        }
    }
    panic!("{key} is not in {THE_VOCABULARY}, so nothing on a machine says it");
}

/// The walk's row for one step, as the document writes it.
fn the_step(number: u32) -> String {
    let order = read(THE_ORDER);
    let start = format!("| {number} | ");
    for line in order.lines() {
        if line.starts_with(&start) {
            return line.to_owned();
        }
    }
    panic!("the walk has no step {number} in {THE_ORDER}");
}

/// **Every named step quotes the sentence the machine really says.**
///
/// The sentence is read from the generated vocabulary snapshot rather than from
/// a crate's source, because the snapshot is what a machine is built from.
#[test]
fn every_named_step_quotes_what_the_machine_says() {
    for (number, key) in WHAT_EACH_STEP_QUOTES {
        let said = what_the_machine_says(key);
        // A templated sentence — `Source: {what} · Not independently
        // verified` — is quoted in the walk with its gap filled, so the parts
        // either side of a gap are what must appear.
        for part in said.split(['{', '}']).step_by(2) {
            let part = part.trim();
            if part.len() < 4 {
                continue;
            }
            assert!(
                the_step(number).contains(part),
                "walk step {number} does not quote {part:?}, which is what {key} says on a \
                 machine. A walker told to look for anything else reports a fault that is not \
                 one."
            );
        }
    }
}

/// **Every step this file names is really in the walk.**
///
/// The other half: a list naming step 14 of a thirteen-step walk would pass the
/// test above by checking nothing, and a renumbered walk would quietly stop
/// being checked. [`the_step`] panics for a step that is not there, so this
/// asserts each one is found.
#[test]
fn every_named_step_is_really_in_the_walk() {
    for (number, _) in WHAT_EACH_STEP_QUOTES {
        let row = the_step(number);
        assert!(
            row.len() > 40,
            "walk step {number} is {row:?}, which is too short to be a step with a pass"
        );
    }
}

/// **The vocabulary lookup finds something, and says so when it does not.**
///
/// The control. Without it, a `what_the_machine_says` that returned an empty
/// string for everything would make the test above pass about nothing — the
/// shape where an empty result and a negative result look identical.
#[test]
fn the_lookup_finds_a_sentence_and_refuses_a_key_that_is_not_there() {
    assert_eq!(what_the_machine_says("terminal.who.program"), "program");
    assert!(
        what_the_machine_says("terminal.waiting.not-known").contains("cannot identify"),
        "the fifth waiting answer is not what this test was written against"
    );

    let refused =
        std::panic::catch_unwind(|| what_the_machine_says("terminal.who.nobody-said-this"));
    assert!(
        refused.is_err(),
        "a key that is not in the snapshot was answered rather than refused"
    );
}
