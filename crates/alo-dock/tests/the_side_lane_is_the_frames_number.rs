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

/// **The lane is not the icon sum, and the gap is deliberate.**
///
/// `a_dock_of_icons` is 64 and the frames say 70. If a later change ever makes
/// those equal it will be because somebody moved `MARGIN` to fit one edge, which is
/// fitting the measures to a frame and moves the bottom dock with it. The two are
/// held apart here so that reconciling them is a decision rather than a tidy-up.
#[test]
fn the_side_lane_is_not_the_icon_sum_and_says_by_how_much() {
    let lane = alo_dock::Room::a_side_docks_lane().as_pixels();
    let icons = alo_dock::Room::a_dock_of_icons().as_pixels();
    assert!(
        lane > icons,
        "the side lane ({lane}) is no longer wider than a dock of icons ({icons}). The design \
         file's frames gave 70 against this crate's 64 — if they now agree, either a measure \
         moved to fit one edge or the design changed, and both want saying out loud."
    );
    assert_eq!(
        lane - icons,
        6,
        "the six pixels between the frames' lane and this crate's arithmetic have changed size. \
         They are recorded in {THE_DOCK} as unexplained rather than derived, so a change here \
         means somebody learned what they are for — which belongs in that note."
    );
}
