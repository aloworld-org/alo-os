//! **The regions' measurements, held to the design file's own table.**
//!
//! `docs/design/the-regions-a-pointer-can-be-in.md` gives the four regions a pointer
//! can be in, with a figure for each, and says of itself that *a prototype is evidence
//! of intent, not of behaviour*. This holds the one of those figures this crate now
//! carries to the document, so the copy cannot drift from it silently.
//!
//! # Its own file rather than a test beside the frame's numbers
//!
//! `the_frame_in_numbers.rs` does this for `the-canvas-in-numbers.md`, and the reason
//! it exists is worth repeating because it is the same reason here: a name band was
//! **derived** as 32 while the document had been giving 48 since #191, and nothing held
//! the two together, so being wrong cost nothing at the time and would have cost a
//! frame's whole feel later. *A number a document already gives is not a number to
//! derive.*
//!
//! A different document is a different responsibility, so this is a different file —
//! and the two extractors are not shared, because the documents do not have the same
//! shape. The canvas file gives a row per measurement; this one gives a table of
//! regions with an origin and an extent in one cell, so *the number after the ×* is the
//! height and the parser has to know that. Sharing one extractor would mean one of them
//! parsing a shape it was not written for.
#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "an unexpected None or Err here is the failure this test reports, and a \
              formatted panic names which row it was reading"
)]

use std::path::Path;

/// The design file this crate's region figures come from.
const THE_REGIONS: &str = "docs/design/the-regions-a-pointer-can-be-in.md";

/// That file, read from the repository rather than from a copy.
fn the_document() -> String {
    // Two directories up from this crate: `crates/alo-shell` to the root.
    let at = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(THE_REGIONS);
    std::fs::read_to_string(&at).unwrap_or_else(|why| {
        panic!("{THE_REGIONS} is the design file and must be readable: {why}")
    })
}

/// The height the document's table gives for one region.
///
/// The row reads `| Top controls | (0, 0) | 1440 × 84 |`, so the extent is the third
/// cell and the height is what follows the `×`. Taken from the row found **by the
/// region's name**, so a table that gains a row above this one does not move the
/// answer — a positional read would be a maintained count.
fn height_of(region: &str) -> f64 {
    let row = the_document()
        .lines()
        .find(|line| line.starts_with('|') && line.contains(region))
        .unwrap_or_else(|| panic!("{THE_REGIONS} has no table row for {region}"))
        .to_owned();
    let extent = row
        .split('|')
        .find(|cell| cell.contains('×'))
        .unwrap_or_else(|| panic!("{region}'s row gives no extent with a ×: {row}"));
    let after = extent
        .split_once('×')
        .map(|(_, rest)| rest)
        .expect("the cell contains the × it was found by");
    let digits: String = after
        .chars()
        .skip_while(|it| !it.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect();
    assert!(
        !digits.is_empty(),
        "{region}'s extent gives no height after its ×: {row}"
    );
    digits
        .parse()
        .unwrap_or_else(|why| panic!("{region}'s height is not a number: {why}"))
}

/// **The top controls are as tall as the design file says.**
///
/// `top_controls_region`'s constant against the document's own row. The region is
/// reserved before anything paints in it — canvas task 6a's contents are not laid out —
/// so this figure is the only thing deciding how much room the canvas does not get, and
/// nothing else would notice it drifting.
#[test]
fn the_top_controls_are_the_height_the_design_file_gives() {
    assert_eq!(
        alo_shell::the_top_controls(),
        height_of("Top controls"),
        "the band this crate reserves and {THE_REGIONS}'s table disagree about how tall \
         the top controls are"
    );
}

/// **The extractor reads the height rather than the first number it meets.**
///
/// The row holds four numbers — an origin of `(0, 0)` and an extent of `1440 × 84` — and
/// the first of them is zero. A reader taking *the first digits in the row* would answer
/// 0, a band of no height would reserve nothing, and the test above would pass while the
/// canvas quietly got the whole screen back. So the parser is checked against the one
/// mistake it could make and still look right.
#[test]
fn the_width_is_not_mistaken_for_the_height() {
    let height = height_of("Top controls");
    assert!(
        height > 0.0,
        "a height of {height} means the extractor found an origin rather than an extent"
    );
    assert!(
        (height - 1440.0).abs() > f64::EPSILON,
        "the extractor answered 1440, which is the width in the same cell"
    );
}
