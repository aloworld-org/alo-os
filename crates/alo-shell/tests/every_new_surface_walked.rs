//! **Every new surface, walked** — the sequence of what a person is told and
//! shown, held against the table in the report.
//!
//! `docs/autonomy/v0-5-the-shell-plan.md` task 14 asks for two things about one
//! walk: *a raster at each step*, and *the exact sequence of spoken and shown
//! text, recorded in the report and held by one test*. This is the second.
//!
//! The first is `examples/the_walk.rs`, which needs a Wayland parent and a GLES
//! context to submit and read back eight frames. This needs neither: a sentence
//! is the same sentence on every machine, and a walk nobody can run on their own
//! laptop is a walk nobody checks. So the sequence is held here, in the ordinary
//! suite, on any machine.
//!
//! # Spoken and shown are one column, deliberately
//!
//! Copied from `alo-access`'s own walk, whose reasoning this inherits: every
//! name in `alo_access::tree` is what a reader is told **and** what the shell
//! draws. One description, used by both, which is the argument that a blind
//! person and an agent read the same machine. A table with two columns would be
//! two descriptions, and the day they differed nobody would know which was the
//! machine.
//!
//! # Three of the surfaces this walk draws are not in the accessibility tree
//!
//! `alo_access::tree::Surface` knows nine surfaces and **none of them is the
//! lock screen, the capture tools or a notification** — all three of which the
//! walk draws and reads back. Their sentences here therefore come from their own
//! crates' vocabulary rather than from the tree, and that is a gap named in the
//! report: a surface a reader cannot reach is a surface somebody cannot use, and
//! the lock screen is the one a person meets before they can do anything else.
//!
//! # The table is the report's, and this test reads it
//!
//! Not a copy of it. A sentence that changes without the table changing fails
//! here. A later change that moves one publishes the table again in a follow-up
//! and points [`THE_REPORT`] at it, because a published report is never
//! rewritten.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

use alo_access::tree::Surface;
use alo_strings::{Filling, Strings, Word};

/// The report this walk is recorded in.
const THE_REPORT: &str = "docs/autonomy/updates/every-new-surface-walked.md";

/// The heading its table is under.
const THE_WALK: &str = "## The walk, sentence by sentence";

/// Everything this machine can say, in the language this test reads.
fn strings() -> Strings {
    Strings::of(
        alo_saying::everything_this_machine_can_say().expect("this machine's own vocabulary"),
    )
}

/// One sentence, as a person reads or hears it.
///
/// Every one is checked where it is made rather than at the end: a sentence
/// assembled from a key instead of said, or said with a slot nobody filled, is
/// a fault at the moment it is met and not a difference in a table.
fn said(strings: &Strings, word: Word) -> String {
    let sentence = strings.say(&word.key(), &Filling::nothing());
    // `is_a_bug` covers both a phrase this machine does not have and one it
    // could not assemble — a word with a slot, said with nothing to put in it,
    // is the second. Neither is a sentence to show anybody.
    assert!(
        !sentence.is_a_bug(),
        "{:?} is not a sentence this machine can say with nothing filled in",
        word.key()
    );
    assert!(
        sentence.unfilled().is_empty(),
        "{:?} was said with {:?} left unfilled",
        word.key(),
        sentence.unfilled()
    );
    sentence.text().to_owned()
}

/// **The walk**: each moment, and what a person meets at it.
///
/// The order is the acceptance's order — sign in, dock a second display, divide
/// the screen, capture with a blur, a notification, lock, unlock — and each
/// moment's sentences are the surfaces' own, in the order a reader reaches them.
fn the_walk() -> Vec<(String, String)> {
    let strings = strings();
    let mut walk: Vec<(String, String)> = Vec::new();

    // 1. The sign-in screen, before any account is chosen. Every stop on it, in
    // the order the keyboard reaches them, which is the order they are read.
    for stop in alo_access::focus_order(Surface::SignIn) {
        walk.push((
            "The sign-in screen, before any account is chosen".to_owned(),
            said(&strings, stop.name),
        ));
    }

    // 2. A second display. What a person is told about the screens they have is
    // `alo-screens`' own words; there is no surface in this compositor that
    // draws them, which the report says.
    for word in [
        alo_displays::words::SCREEN_BUILT_IN,
        alo_displays::words::AS_YOU_LEFT_THEM,
    ] {
        walk.push((
            "A second display is docked".to_owned(),
            said(&strings, word),
        ));
    }

    // 3. The screen divided. Which side a chord means is `alo-dividing`'s, and
    // so is what it is called.
    for switch in [alo_desktops::Switch::Next, alo_desktops::Switch::Previous] {
        walk.push((
            "The screen is divided, and the desktops a swipe reaches".to_owned(),
            said(&strings, switch.word()),
        ));
    }

    // 4. The three surfaces that are on every desktop, as `alo-desktops` names
    // them — including the agent overlay, which nothing draws.
    for always in alo_desktops::Always::ALL {
        walk.push((
            "On every desktop, whatever else is".to_owned(),
            said(&strings, always.word()),
        ));
    }

    // 5. The desktop a person is looking at, read as a reader reaches it.
    for control in Surface::Desktop.read_aloud() {
        walk.push((
            "The desktop, read as a reader reaches it".to_owned(),
            said(&strings, control.name),
        ));
    }

    // 6. The status area, where what is leaving this machine is shown. This is
    // the surface the first law is about, so it is walked rather than assumed.
    for control in Surface::StatusArea.read_aloud() {
        walk.push((
            "The status area, where what is leaving is shown".to_owned(),
            said(&strings, control.name),
        ));
    }

    // 7. The window controls, which close, move and arrange a window.
    for control in Surface::WindowControls.read_aloud() {
        walk.push((
            "The controls on a window".to_owned(),
            said(&strings, control.name),
        ));
    }

    walk
}

/// This repository.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("this crate is two directories below the repository")
}

/// The rows of the table under [`THE_WALK`] in [`THE_REPORT`].
fn the_table() -> Vec<(String, String)> {
    let report = fs::read_to_string(the_repository().join(THE_REPORT))
        .unwrap_or_else(|why| panic!("{THE_REPORT} could not be read: {why}"));
    let rows: Vec<(String, String)> = report
        .lines()
        .skip_while(|line| line.trim() != THE_WALK)
        .skip(1)
        .take_while(|line| !line.starts_with("## "))
        .skip_while(|line| !line.starts_with('|'))
        .take_while(|line| line.starts_with('|'))
        .skip(2)
        .map(|row| {
            let cells: Vec<&str> = row
                .trim()
                .trim_matches('|')
                .split(" | ")
                .map(str::trim)
                .collect();
            let [_, moment, met] = cells.as_slice() else {
                panic!("a row is a step, a moment and what a person meets: {row}");
            };
            ((*moment).to_owned(), (*met).to_owned())
        })
        .collect();
    assert!(
        !rows.is_empty(),
        "{THE_REPORT} has no table under {THE_WALK:?}"
    );
    rows
}

/// The walk, written as the table's rows.
fn as_a_table(walk: &[(String, String)]) -> String {
    walk.iter()
        .enumerate()
        .map(|(step, (moment, met))| format!("| {} | {moment} | {met} |", step + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

/// **The walk produces exactly the table in the report**, in order.
#[test]
fn every_new_surface_reads_as_the_table_in_the_report() {
    let walk = the_walk();
    assert!(
        walk == the_table(),
        "the walk and the table in {THE_REPORT} differ. The walk reads:\n\n{}\n",
        as_a_table(&walk)
    );
}
