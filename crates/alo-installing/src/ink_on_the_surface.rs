//! Putting a line's coverage onto the framebuffer.
//!
//! The one file here that touches the machine, kept to one job for the reason
//! `a_surface_to_draw_on` gives for its own `fill`: everything it needs was
//! worked out by tested code above it, so the part that cannot be tested
//! without a screen is as small as it can be made.
//!
//! # What is tested here and what is not
//!
//! | | |
//! |---|---|
//! | [`a_row_of_ink`] | the bytes one row becomes — **tested, on any machine** |
//! | [`draw`] | handing those bytes to `/dev/fb0` — **not tested here**, and the boot is the measurement it is owed |
//!
//! That split is the whole design of this file. The arithmetic of blending ink
//! into a background, clipping a line to the screen, and laying bytes out at
//! the right stride is all in the first row, and it is where a fault would
//! actually be.
//!
//! # Why the ink is a grey
//!
//! Because [`Grey`] is the colour that cannot be wrong about channel order,
//! and the order this framebuffer wants is **unmeasured** — `a_surface_to_draw_on`
//! says so in as many words, which is why the first thing this environment
//! ever drew was a grey fill. Drawing text in a grey keeps that property: a
//! sentence a person can read is worth more than a sentence in the right
//! colour that might come out blue, and the moment somebody measures the
//! channel order on a real machine this file gains a colour and loses nothing.

use crate::a_line_as_coverage::ACoverage;
use crate::a_surface_to_draw_on::{Grey, TheSurface};
#[cfg(unix)]
use std::fs::File;
#[cfg(unix)]
use std::io;
#[cfg(unix)]
use std::os::unix::fs::FileExt as _;

/// One row of the framebuffer, with a line's ink blended into it.
///
/// `beneath` is what the row already holds — the screen's background — and
/// `ink` is what the letters are drawn in. Coverage of nought leaves the
/// background exactly as it was, full coverage gives the ink, and between the
/// two they are mixed in proportion, which is what makes the edge of a letter
/// smooth rather than jagged.
///
/// The row returned is **the stride long**, not the width long, for the same
/// reason [`TheSurface::a_row_of`] is: the padding at the end of a row is
/// written too, so nothing is left holding whatever the firmware put there.
///
/// Where the line would reach past the right-hand edge it is **clipped**
/// rather than wrapped, because a letter continued on the next row down reads
/// as a corrupted screen.
#[must_use]
pub fn a_row_of_ink(
    surface: TheSurface,
    beneath: Grey,
    ink: Grey,
    coverage: &ACoverage,
    at_across: u32,
    row_of_the_line: u32,
) -> Vec<u8> {
    let mut row = surface.a_row_of(beneath);
    let how_much = coverage.row(row_of_the_line);
    let each_pixel = surface.each_pixel() as usize;

    for (along, &covered) in how_much.iter().enumerate() {
        if covered == 0 {
            continue;
        }
        let Some(across) = at_across.checked_add(along as u32) else {
            break;
        };
        if across >= surface.across() {
            break;
        }
        let mixed = mixed(beneath, ink, covered);
        let from = across as usize * each_pixel;
        for channel in 0..each_pixel {
            if let Some(byte) = row.get_mut(from + channel) {
                *byte = mixed;
            }
        }
    }
    row
}

/// This much ink over that background.
///
/// Rounded to nearest rather than truncated: truncating biases every edge
/// pixel towards the background, which thins the letters — visible as text
/// that looks lighter than the weight the design asks for, and this
/// repository has already shipped a weight it did not mean to once.
#[must_use]
const fn mixed(beneath: Grey, ink: Grey, covered: u8) -> u8 {
    let beneath = beneath.as_a_value() as u32;
    let ink = ink.as_a_value() as u32;
    let covered = covered as u32;
    let mixed = (ink * covered + beneath * (255 - covered) + 127) / 255;
    mixed as u8
}

/// Draw a line's coverage onto the screen at this place.
///
/// Rows of the line that fall outside the screen are skipped, so a line placed
/// near the bottom edge draws the part that is on the screen rather than
/// failing or writing past the end of the framebuffer.
///
/// # Errors
///
/// Whatever the write failed with. A framebuffer that is not there, or is not
/// writable by this program, arrives here rather than as a wrong picture.
///
/// # Why this is gated for Unix
///
/// `FileExt::write_all_at` is Unix's, and this crate **must cross-compile for
/// Windows** because `alo-installer`'s tests pull it in. Everything above is
/// arithmetic and is portable; only the write is not, so only the write is
/// gated — exactly as `a_surface_to_draw_on::fill` is, and for the fault that
/// was caught there.
#[cfg(unix)]
pub fn draw(
    at: &File,
    surface: TheSurface,
    beneath: Grey,
    ink: Grey,
    coverage: &ACoverage,
    at_across: u32,
    at_down: u32,
) -> io::Result<()> {
    for row_of_the_line in 0..coverage.down() {
        let Some(down) = at_down.checked_add(row_of_the_line) else {
            break;
        };
        if down >= surface.down() {
            break;
        }
        let row = a_row_of_ink(surface, beneath, ink, coverage, at_across, row_of_the_line);
        at.write_all_at(&row, surface.where_row_begins(down))?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "ink_on_the_surface_tests.rs"]
mod tests;

/// Put this much of this ink onto a row that already has something on it.
///
/// # Why this exists beside [`a_row_of_ink`]
///
/// `a_row_of_ink` **builds** a row: it starts from the background and returns
/// the whole thing. That is right for one line of text on an empty screen and
/// wrong for everything a screen actually has on it, because a second call
/// throws away the first — so *Continue* would erase *Back* beside it, a
/// card's border would erase its own fill either side of itself, and the words
/// on a card would erase the card.
///
/// This one **adds** to a row that is already there, which is what lets a
/// screen be composed — canvas, then boxes, then words — and written once.
///
/// `at_across` may put part of the coverage off either edge; what falls off is
/// dropped rather than wrapping onto the next row.
pub fn onto(row: &mut [u8], surface: TheSurface, ink: Grey, how_much: &[f64], at_across: u32) {
    let as_bytes: Vec<u8> = how_much
        .iter()
        .map(|covered| (covered.clamp(0.0, 1.0) * 255.0).round() as u8)
        .collect();
    onto_coverage(row, surface, ink, &as_bytes, at_across);
}

/// The same, for coverage that is already the 0-255 a glyph is rasterised to.
///
/// Text arrives from [] in bytes and boxes are
/// computed in fractions; converting one to the other at the boundary keeps
/// one compositor rather than two that could disagree about what a
/// half-covered pixel means.
pub fn onto_coverage(
    row: &mut [u8],
    surface: TheSurface,
    ink: Grey,
    how_much: &[u8],
    at_across: u32,
) {
    let each_pixel = surface.each_pixel() as usize;
    for (along, &covered) in how_much.iter().enumerate() {
        if covered == 0 {
            continue;
        }
        let Some(across) = at_across.checked_add(along as u32) else {
            break;
        };
        if across >= surface.across() {
            break;
        }
        let from = across as usize * each_pixel;
        // The row already holds whatever is under this, so the blend is
        // against that rather than against the canvas — which is what makes a
        // border antialias against its own card.
        let Some(&under) = row.get(from) else { break };
        let value = mixed(Grey::of(under), ink, covered);
        for channel in 0..each_pixel {
            if let Some(byte) = row.get_mut(from + channel) {
                *byte = value;
            }
        }
    }
}
