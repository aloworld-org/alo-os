//! The path and both modes this crate makes are read **out of the published
//! contract**, not agreed with another crate's constant.
//!
//! `docs/contracts/kept-undo-folder.md` is what a third party builds against and
//! what `docs/decisions/0045-what-undoing-rewinds-to.md` terms 8 and 10 settle.
//! Three constants in this crate repeat what it says, and `alo-letting-go`
//! declares the same path again for its own reasons — four copies of one fact.
//!
//! # Why this reads the document and not the other crate
//!
//! Because two constants compared against each other **pass while both are
//! wrong together**, and that is the whole failure this guards. A test asserting
//! `alo_giving_a_folder::THE_FOLDER == alo_letting_go::THE_FOLDER` would stay
//! green through a change that moved both and left every document behind — and
//! the document is the thing a person reading this repository, or a third party
//! writing against it, actually has.
//!
//! It also keeps this crate from depending on `alo-letting-go`, which
//! [ADR 0119](../../../docs/decisions/0119-the-capability-to-give-a-directory-away-goes-to-the-smallest-thing-that-can-hold-it.md)
//! refuses: that crate carries the machine's one privileged remover, and a giver
//! that could reach a remover would be two powers in the place chosen to hold
//! one.
//!
//! # The modes are read from the contract's own diagram
//!
//! Not from a sentence about them. The contract draws the shape in a fenced
//! block and that block is what a reader believes, so that block is what this
//! parses — which is also why a mode written only in prose would fail here and
//! should.

#![cfg(target_os = "linux")]
#![expect(
    clippy::panic,
    reason = "in a test, a panic on a document that no longer says what this crate does is the               failure being reported"
)]

use std::path::Path;

use alo_giving_a_folder::{A_PERSONS_MODE, THE_FOLDER, THE_FOLDERS_MODE, what_to_make};

/// The contract, as the repository holds it.
fn the_contract() -> String {
    let at = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/contracts/kept-undo-folder.md");
    std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// Every line inside a fenced block in the contract, in order.
///
/// The fences are counted so an unclosed block is a failure rather than a silent
/// half-read — a document that lost a fence would otherwise hand this test the
/// rest of itself as though it were a diagram.
fn fenced_lines() -> Vec<String> {
    let mut lines = Vec::new();
    let mut inside = false;
    for line in the_contract().lines() {
        if line.trim_start().starts_with("```") {
            inside = !inside;
        } else if inside {
            lines.push(line.to_owned());
        }
    }
    assert!(!inside, "a fenced block in the contract is never closed");
    lines
}

/// **The folder is where the contract says it is.**
#[test]
fn the_folder_is_where_the_contract_puts_it() {
    let said = fenced_lines();
    assert!(
        said.iter().any(|line| line.trim() == THE_FOLDER),
        "no fenced block in the contract says the folder is {THE_FOLDER}; it says {said:#?}"
    );
}

/// **And the shape diagram agrees about the folder's own mode.**
///
/// The line is `/var/lib/alo-undo/   0711 root:root`, so the mode is read from
/// the diagram rather than from the prose around it.
#[test]
fn the_folders_mode_is_the_one_the_contract_draws() {
    let drawn = the_mode_drawn_for(&format!("{THE_FOLDER}/"));
    assert_eq!(
        drawn, THE_FOLDERS_MODE,
        "the contract draws the folder as {drawn:o} and this crate makes it {THE_FOLDERS_MODE:o}"
    );
}

/// **And about a person's directory inside it.**
///
/// `docs/decisions/0045-what-undoing-rewinds-to.md` term 10 is why this reads
/// one word and not two: the owner is set and the group is left alone, so the
/// contract draws `0700 ada` rather than `0700 ada:ada`. A diagram that still
/// promised a group would fail here, which is the point — the amendment and the
/// code have to move together or this test says so.
#[test]
fn a_persons_mode_is_the_one_the_contract_draws() {
    let drawn = the_mode_drawn_for("ada/");
    assert_eq!(
        drawn, A_PERSONS_MODE,
        "the contract draws a person's directory as {drawn:o} and this crate makes it \
         {A_PERSONS_MODE:o}"
    );
}

/// **The contract no longer promises a group**, because this crate does not set
/// one.
///
/// Term 10 of ADR 0045, and the one assertion here that is about an absence. It
/// is written as its own test rather than folded into the two above, because a
/// mode comparison would pass unchanged if somebody put `:ada` back — the mode
/// is the first word either way.
#[test]
fn the_contract_promises_no_group_for_a_persons_directory() {
    let promising: Vec<String> = fenced_lines()
        .into_iter()
        .filter(|line| line.contains(&format!("{A_PERSONS_MODE:04o} ada:")))
        .collect();
    assert!(
        promising.is_empty(),
        "the contract promises a group this crate does not set: {promising:#?}"
    );
}

/// **What this crate would make on a machine is what the contract draws**, which
/// is the two above joined into the thing a reader of either actually cares
/// about.
///
/// The control for the parser: a test that only read the document could pass
/// against a crate that made nothing at all.
#[test]
fn what_would_be_made_on_a_machine_matches_the_drawing() {
    let under = Path::new(THE_FOLDER);
    let [folder, theirs] = what_to_make(under, 1000);

    assert_eq!(folder.at, under);
    assert_eq!(folder.mode, the_mode_drawn_for(&format!("{THE_FOLDER}/")));
    assert_eq!(folder.owner, None, "the folder is root's own");

    assert_eq!(theirs.at, under.join("1000"));
    assert_eq!(theirs.mode, the_mode_drawn_for("ada/"));
    assert_eq!(theirs.owner, Some(1000), "a person's directory is theirs");
}

/// The octal mode the contract's diagram draws against a path.
///
/// Panics when the diagram does not mention it, which is the honest answer: a
/// contract that stopped drawing one of these is a contract this crate can no
/// longer be held to. A test that skipped instead would make a missing line
/// read exactly like an agreeing one.
fn the_mode_drawn_for(what: &str) -> u32 {
    let lines = fenced_lines();
    let drawn: Vec<&String> = lines
        .iter()
        .filter(|line| line.trim_start().starts_with(what))
        .collect();
    assert_eq!(
        drawn.len(),
        1,
        "the contract's diagrams draw {what} {} times, and this reads one: {drawn:#?}",
        drawn.len()
    );

    let line = drawn
        .first()
        .unwrap_or_else(|| panic!("the contract draws {what} nowhere"));
    let mode = line
        .split_whitespace()
        .nth(1)
        .unwrap_or_else(|| panic!("the contract draws {what} with no mode after it: {line:?}"));
    u32::from_str_radix(mode, 8)
        .unwrap_or_else(|why| panic!("the contract draws {what} as {mode:?}, not a mode: {why}"))
}
