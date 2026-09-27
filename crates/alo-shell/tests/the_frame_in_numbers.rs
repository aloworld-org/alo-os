//! **The frame's measurements, held to the design file's own reading.**
//!
//! `docs/design/the-canvas-in-numbers.md` is a reading of page *07 — alo OS ·
//! Living canvas*, and it says so itself: *"This is a reading, not a source…
//! Nothing yet holds **these** to anything, which is the gap named at the end."*
//! This closes that gap for the numbers this crate carries, the same way
//! `crates/alo-appearance/tests/a_palette_with_one_source.rs` closes it for the
//! six colours.
//!
//! # Why this exists, plainly
//!
//! I invented a name band of 32 pixels, reasoning from the size of a window
//! control, and wrote a paragraph justifying it. The number was already read off
//! the design file and sitting in this repository — 48 — and had been since #191,
//! before I wrote the file that got it wrong. A number a document already gives is
//! not a number to derive, and nothing was holding the two together, so being
//! wrong cost nothing at the time and would have cost a frame's whole feel later.
//!
//! So the copies stay and the drift goes. A compositor cannot parse a document at
//! the moment it places a band, so the value is a Rust constant; the document stays
//! the reading a person checks against the design file. This fails when they part.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#![expect(
    clippy::panic,
    reason = "a missing document or a missing row names itself in the failure, which an \
              unwrap could not"
)]

use std::path::Path;

/// The reading this test holds the crate to.
const THE_NUMBERS: &str = "docs/design/the-canvas-in-numbers.md";

/// That document, as text.
fn the_numbers() -> String {
    let at = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(THE_NUMBERS);
    std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{THE_NUMBERS} is where it says it is: {why}"))
}

/// The pixels a row of the frame's table gives for `named`.
///
/// Read out of the row rather than from a line number, because a table gains rows
/// and a line number silently starts naming a different one. The row is
/// `| **Name band** | **48 px tall**, … |`, so the first run of digits after the
/// name is the measurement.
fn pixels_for(named: &str) -> f64 {
    let written = the_numbers();
    let row = written
        .lines()
        .find(|line| line.starts_with('|') && line.contains(named))
        .unwrap_or_else(|| panic!("{THE_NUMBERS} has no table row naming {named}"));
    let after = row
        .split_once(named)
        .map(|(_, rest)| rest)
        .expect("the row contains the name it was found by");
    let digits: String = after
        .chars()
        .skip_while(|it| !it.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect();
    assert!(
        !digits.is_empty(),
        "{THE_NUMBERS}'s row for {named} gives no number: {row}"
    );
    digits
        .parse()
        .unwrap_or_else(|why| panic!("{named} is measured in something other than a number: {why}"))
}

/// **The name band is as tall as the design file says.**
///
/// `crate::frame_handle::THE_NAMES_BAND` against the document's own row. These
/// disagreed on the day the band was written — 32 against 48 — and nothing said so.
#[test]
fn the_names_band_is_the_height_the_design_file_gives() {
    assert_eq!(
        alo_shell::the_names_band(),
        pixels_for("Name band"),
        "the band this crate places and {THE_NUMBERS}'s reading of the design file \
         disagree about how tall a frame's name is"
    );
}
