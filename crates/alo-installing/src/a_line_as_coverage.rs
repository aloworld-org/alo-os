//! A line of text turned into coverage — how much of each pixel the letters
//! cover — before anything touches the screen.
//!
//! # Why coverage and not pixels
//!
//! Because coverage is testable and pixels are not. A glyph rasterised to
//! coverage is a grid of *how much ink is here*, nought to full, and every
//! question worth asking can be asked of it on a machine with no screen: is
//! the line as wide as the face said it would be, is the text inside the area
//! it was given, did a letter come out blank. Turning coverage into the bytes
//! a framebuffer wants is [`crate::ink_on_the_surface`]'s one job, and it is
//! the only part that cannot be tested without hardware.
//!
//! That split is this file's reason for existing, and it follows what
//! `a_surface_to_draw_on` already does: everything is arithmetic until the one
//! function that writes.
//!
//! # The baseline, which is the thing to get wrong
//!
//! A layout gives a line a **top**. A font positions a glyph from its
//! **baseline**, and swash hands back each glyph's box as an offset from that
//! baseline — `top` counted upwards, so a positive `top` is above the baseline
//! and most descenders have a negative one. Adding those two without the
//! ascent between them draws every line about four fifths of its own height
//! too high, which on the environment's screens means the eyebrow drawn off
//! the top of the display.
//!
//! So the baseline is asked of the face ([`crate::AlosOwnFace::the_baseline_at`])
//! and the line's own height is the em — not the glyph's box, which differs
//! per letter and would make *acne* and *Agile* sit at different heights.

use crate::alos_own_face::AlosOwnFace;
use swash::scale::{Render, ScaleContext, Source, StrikeWith};
use swash::zeno::Format;

/// How much the letters cover each pixel of a line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ACoverage {
    /// Pixels across.
    across: u32,
    /// Pixels down.
    down: u32,
    /// One byte a pixel: nought for none of it, 255 for all of it.
    how_much: Vec<u8>,
}

impl ACoverage {
    /// Pixels across.
    #[must_use]
    pub const fn across(&self) -> u32 {
        self.across
    }

    /// Pixels down.
    #[must_use]
    pub const fn down(&self) -> u32 {
        self.down
    }

    /// How much the letters cover this pixel, and nought outside the line.
    #[must_use]
    pub fn at(&self, across: u32, down: u32) -> u8 {
        if across >= self.across || down >= self.down {
            return 0;
        }
        let at = down as usize * self.across as usize + across as usize;
        self.how_much.get(at).copied().unwrap_or(0)
    }

    /// Every pixel's coverage, a row at a time from the top.
    #[must_use]
    pub fn row(&self, down: u32) -> &[u8] {
        if down >= self.down {
            return &[];
        }
        let from = down as usize * self.across as usize;
        let to = from + self.across as usize;
        self.how_much.get(from..to).unwrap_or_default()
    }

    /// Whether any letter covers any pixel at all.
    ///
    /// What a caller checks before writing: coverage with nothing in it means
    /// the text was not drawn, and a screen that wrote nothing and said
    /// nothing is the failure this whole area keeps producing.
    #[must_use]
    pub fn anything_at_all(&self) -> bool {
        self.how_much.iter().any(|&how_much| how_much > 0)
    }
}

/// Draw this text into coverage, at this many pixels to the em.
///
/// The coverage is as wide as [`AlosOwnFace::how_wide`] says the text is and
/// as tall as the em, so a caller that laid the line out by that measure and
/// draws this at that place cannot disagree with itself.
///
/// A letter the face has no glyph for is skipped and advances nothing, which
/// is what [`AlosOwnFace::how_wide`] counted.
///
/// # Panics
///
/// Never. A glyph the scaler will not render is left blank rather than
/// stopping the screen, because a missing letter is legible and a stopped
/// installer is not.
#[must_use]
pub fn a_line_as_coverage(
    face: AlosOwnFace<'_>,
    text: &str,
    em: u32,
    line_is_tall: u32,
) -> ACoverage {
    let across = face.how_wide(text, em);
    let down = line_is_tall;
    let mut how_much = vec![0_u8; across as usize * down as usize];
    if across == 0 || down == 0 {
        return ACoverage {
            across,
            down,
            how_much,
        };
    }

    let baseline = face.the_baseline_in(em, line_is_tall);
    let metrics = face.as_swash_reads_it().glyph_metrics(&[]).scale(em as f32);
    let mut context = ScaleContext::new();
    let mut scaler = context
        .builder(face.as_swash_reads_it())
        .size(em as f32)
        .hint(true)
        .build();

    let mut pen = 0.0_f32;
    for character in text.chars() {
        let Some(glyph) = face.the_glyph_for(character) else {
            continue;
        };
        let drawn = Render::new(&[
            Source::ColorOutline(0),
            Source::ColorBitmap(StrikeWith::BestFit),
            Source::Outline,
        ])
        .format(Format::Alpha)
        .render(&mut scaler, glyph);

        if let Some(image) = drawn {
            // `left` is from the pen, `top` is **upwards from the baseline**,
            // so the box's top edge is the baseline minus it.
            let left = pen + image.placement.left as f32;
            let top = i64::from(baseline) - i64::from(image.placement.top);
            put(
                &mut how_much,
                across,
                down,
                &image.data,
                image.placement.width,
                image.placement.height,
                left.round() as i64,
                top,
            );
        }
        pen += metrics.advance_width(glyph);
    }

    ACoverage {
        across,
        down,
        how_much,
    }
}

/// Put one glyph's coverage into the line's, clipped to it.
///
/// **Clipped rather than trusted.** A glyph's box can reach outside the line —
/// an accent above a capital on a tight em, a descender on the bottom line —
/// and the honest answer is to draw the part that is inside. The alternative
/// is an index past the end of the line, in the one program that runs as root
/// against a disk it is about to overwrite.
#[expect(
    clippy::too_many_arguments,
    reason = "a glyph's box, the line's box, and where one goes in the other"
)]
fn put(
    into: &mut [u8],
    across: u32,
    down: u32,
    glyph: &[u8],
    glyph_across: u32,
    glyph_down: u32,
    at_across: i64,
    at_down: i64,
) {
    for glyph_row in 0..glyph_down {
        let row = at_down + i64::from(glyph_row);
        if row < 0 || row >= i64::from(down) {
            continue;
        }
        for glyph_column in 0..glyph_across {
            let column = at_across + i64::from(glyph_column);
            if column < 0 || column >= i64::from(across) {
                continue;
            }
            let from = glyph_row as usize * glyph_across as usize + glyph_column as usize;
            let Some(&how_much) = glyph.get(from) else {
                continue;
            };
            let to = row as usize * across as usize + column as usize;
            if let Some(there) = into.get_mut(to) {
                // Letters in one line do not overlap in any of the 24
                // languages this draws, but a glyph's box can, so the darker
                // of the two wins rather than the later one. That keeps the
                // result the same whichever order the letters were drawn in.
                *there = (*there).max(how_much);
            }
        }
    }
}

#[cfg(test)]
#[path = "a_line_as_coverage_tests.rs"]
mod tests;
