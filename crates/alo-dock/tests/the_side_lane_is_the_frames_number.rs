//! **The side Dock's lane, held to the number the design file's frames give.**
//!
//! `CLAUDE.md`: *a screenshot is not a specification — take the numbers off the
//! frames and record them, because a layout matched by eye is a layout nobody can
//! check.* This is the second half of that sentence. The number is recorded in
//! `docs/design/the-alo-dock.md` and used in `crate::measures`, and nothing held
//! the two together until this file.
//!
//! # Why this one needs holding more than most
//!
//! **The design file disagrees with itself about it.** Its frames say 70, its
//! section caption says x24–92 — which is 68 — and its own build contract says
//! x24–94, which is 70 again. Two of three agree and the odd one is prose.
//!
//! A reader who meets the caption first and "corrects" the constant to 68 would be
//! making the code match the design file, which is what this repository asks for,
//! and would be wrong. So the test names the frame as the source and the caption as
//! the thing it is not, rather than only pinning a number.
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "an unreadable design file is the failure this test reports, and a formatted panic \
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

/// **The lane this crate reserves is the one the frames measured.**
#[test]
fn the_side_lane_is_what_the_design_note_recorded() {
    let note = the_note();
    let row = note
        .lines()
        .find(|line| line.starts_with('|') && line.contains("Side Dock lane, left"))
        .unwrap_or_else(|| panic!("{THE_DOCK} has no row for the left side lane"));

    // The cell reads `` `x=24`, **70** wide → x24–94 ``, so the lane is the figure
    // between the asterisks — taken that way rather than as *the first number in
    // the row*, which would answer 24 and would be the origin.
    let measured: u32 = row
        .split("**")
        .nth(1)
        .expect("the row emphasises its measurement")
        .trim()
        .parse()
        .unwrap_or_else(|why| panic!("the left lane's measurement is not a number: {why}"));

    assert_eq!(
        measured,
        alo_dock::measures::A_SIDE_DOCKS_LANE,
        "the lane this crate reserves and {THE_DOCK}'s frame reading disagree"
    );
    assert_ne!(
        measured, 24,
        "the extractor took the origin instead of the extent — the row carries both"
    );
}

/// **Two measured numbers, one per orientation, and neither is derived from the
/// other.**
///
/// This asserted that the side lane (70) was **wider** than a dock of icons,
/// and that the gap was 6. It was, while the across dock's thickness was the
/// proposed `MARGIN + ICON + MARGIN` = 64. The owner replaced that with the
/// frames' own measurement on 2026-10-10 — **76** — so the relationship has
/// inverted: the side lane is now the *narrower* of the two.
///
/// **That is not a fault and nothing needs reconciling.** The design has a
/// 70-wide side dock and a 76-tall horizontal one; both come off the frames,
/// and a crate that forced them equal would be fitting one edge's measurement
/// to the other's. What the original test was protecting — that nobody quietly
/// moves a measure to make two edges agree — is what is held below, in the
/// direction that is now true.
#[test]
fn each_orientation_keeps_its_own_measured_thickness() {
    let lane = alo_dock::Room::a_side_docks_lane().as_pixels();
    let across = alo_dock::Room::a_dock_of_icons().as_pixels();
    assert_eq!(lane, 70, "the frames' side dock");
    assert_eq!(across, 76, "the frames' `Dock + alo Bar`");
    assert_ne!(
        lane, across,
        "the two orientations have been made the same number. The design gives 70 down a side \
         and 76 across, so if they now agree either a measure moved to fit one edge or the \
         design changed, and both want saying out loud."
    );
    // **Still six apart, and that is a coincidence worth naming** so nobody
    // reads it as a rule: 64 was six below the lane and 76 is six above it.
    // Neither number was chosen with the other in view.
    assert_eq!(
        across - lane,
        6,
        "the six pixels between the two orientations' measurements have changed size. They are \
         two readings of {THE_DOCK}'s frames rather than one derived from the other, so a \
         change here means a frame moved."
    );
}
