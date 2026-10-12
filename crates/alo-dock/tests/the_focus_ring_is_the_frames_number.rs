//! **The focus ring's two constants and the design note cannot drift apart.**
//!
//! `docs/design/the-alo-dock.md` now records *two* readings of the same thing:
//! the first-start card the popovers carried their ring from, and the Dock's
//! own bar slots, which the design draws at 3 rather than 2. Two numbers for
//! one measure is exactly the state in which somebody corrects the constant,
//! leaves the prose, and nothing notices — so the note's row and the constant
//! are held together here.
//!
//! This is the same shape as `the_side_lane_is_the_frames_number`, and for the
//! same reason: a figure written in a document and a figure compiled into a
//! crate are two places, and a repository with no check between them has a
//! measurement in one place and a rumour in the other.
#![expect(
    clippy::panic,
    reason = "an unreadable design note is the failure this test reports, and a formatted panic \
              names the row it was reading"
)]

use std::path::Path;

/// The design note this crate's dock figures are recorded in.
const THE_DOCK: &str = "docs/design/the-alo-dock.md";

/// That file, read from the repository rather than from a copy.
fn the_note() -> String {
    let at = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(THE_DOCK);
    std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{THE_DOCK} is the design note and must be readable: {why}"))
}

/// The emphasised figure in the note's row whose first cell is `what`.
///
/// Taken from between the asterisks rather than as *the first number in the
/// row*, which would answer `384` from the node cell on one row and `3` from
/// nothing at all on another.
fn the_row_measuring(note: &str, what: &str) -> u32 {
    let row = note
        .lines()
        .find(|line| line.starts_with('|') && line.contains(what))
        .unwrap_or_else(|| panic!("{THE_DOCK} has no row for {what}"));
    row.split("**")
        .nth(1)
        .unwrap_or_else(|| panic!("the row for {what} emphasises no measurement: {row}"))
        .trim()
        .parse()
        .unwrap_or_else(|why| panic!("the measurement for {what} is not a number: {why}"))
}

/// **The ring the popovers draw is the thickness the note recorded.**
#[test]
fn the_popover_rings_thickness_is_what_the_design_note_recorded() {
    let measured = the_row_measuring(&the_note(), "Popover focus ring, thick");

    assert_eq!(
        measured,
        alo_dock::measures::THE_FOCUS_RING_IS_THICK,
        "the thickness this crate draws and {THE_DOCK}'s reading disagree"
    );
    assert_ne!(
        measured, 3,
        "the extractor took the Dock bar's reading, which is the other row and is not \
         what any constant implements"
    );
}

/// **And the gap it leaves is the stand-off the note recorded.**
#[test]
fn the_popover_rings_stand_off_is_what_the_design_note_recorded() {
    let measured = the_row_measuring(&the_note(), "Popover focus ring, stand-off");

    assert_eq!(
        measured,
        alo_dock::measures::THE_FOCUS_RING_STANDS_OFF,
        "the stand-off this crate draws and {THE_DOCK}'s reading disagree"
    );
}

/// **The Dock's own focus nodes stay named, because a reading nobody can find
/// again is not evidence.**
///
/// The two bar rings are the only reason this file exists rather than one
/// assertion: they are what makes the popover's 2 a *carried* number and not
/// simply *the* number. Deleting the row that records them would leave the two
/// assertions above passing and the note misleading, which is the failure this
/// catches and neither of them does.
#[test]
fn the_notes_bar_reading_names_the_node_it_came_from() {
    let note = the_note();

    for node in ["322:19211", "328:20616", "324:19531"] {
        assert!(
            note.contains(node),
            "{THE_DOCK} no longer names {node}, so its bar focus reading cannot be checked again"
        );
    }
    assert_eq!(
        the_row_measuring(&note, "Bar slot focus ring, thick"),
        3,
        "the note's bar reading is no longer the 3 the design draws"
    );
}
