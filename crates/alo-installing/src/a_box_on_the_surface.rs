//! The boxes a screen is made of: a card, the ring on the one a person is on,
//! and the two buttons.
//!
//! # What was missing, and what a person could not see
//!
//! `a_screen_is_drawn` placed text and had **no rectangle in it at all**, so
//! the six screens were the design's words in the design's places with none of
//! its containers. Three things followed, and only the first is cosmetic:
//!
//! - a card was a line of text with another under it, where the design has a
//!   white card on the canvas with a quiet border;
//! - **the ring was nowhere.** The frame shows the chosen card outlined in
//!   navy, and that outline is the only thing on the screen that says *this is
//!   the one you are on*. Without it a person pressing the down arrow had no
//!   feedback at all — the screen did not change;
//! - *Continue* and *Back* were not drawn, so the way forward was invisible
//!   even though the walk acts on Enter and Escape.
//!
//! Read off `530:30877 S3 · Choose a network` on 2026-10-21, the frame the
//! owner adopted.
//!
//! # Everything here is a proportion, never a pixel
//!
//! The design is 1440 × 960 and no machine is obliged to be. A radius of 12 is
//! a *logical* measure: it is scaled by the same factor the rest of the layout
//! is, so a dense screen draws the same corner sharper rather than smaller.
//!
//! # What a corner costs, said plainly
//!
//! A rounded corner is antialiased against what is behind it, and what is
//! behind the **edge** of a card is the card, while what is behind the card is
//! the canvas. So each box is drawn in two passes, each told what it sits on:
//! the fill against the canvas, then the border against the fill. At the four
//! corners the border's outer quarter blends with the fill rather than the
//! canvas, which at a 12-unit radius in grey is below what a photograph of a
//! screen can show. It is written down rather than discovered.

use crate::a_surface_to_draw_on::Grey;
use crate::where_a_screen_goes::{AnArea, THE_DESIGN_WAS_DRAWN_AT};

/// A card's corner, in the design's own units.
///
/// `radius/md` on the frame, which is the only radius the installer's screens
/// use.
pub const THE_CORNER: u32 = 12;

/// How thick the quiet border around an unchosen card is.
pub const AN_EDGE: u32 = 1;

/// How thick the ring around the chosen card is.
///
/// Twice the quiet border, because it is the one mark on the screen that
/// answers *where am I* and a person has to find it without looking for it.
pub const THE_RING: u32 = 2;

/// What a box is, once a screen has decided where it goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ABoxToDraw {
    /// Where it sits and how large it is.
    pub area: AnArea,
    /// What fills it.
    pub fill: Grey,
    /// What outlines it.
    pub edge: Grey,
    /// How thick that outline is, in pixels on this display. Nought for none.
    pub edge_is: u32,
    /// The corner, in pixels on this display.
    pub corner: u32,
}

/// How much of each pixel of one row this box covers, as the fill.
///
/// Row is counted from the top of the box. The values are coverage in
/// `0.0..=1.0`, the same shape [`crate::a_line_as_coverage`] produces, so the
/// same compositing draws both and a box and a letter cannot disagree about
/// what a half-covered pixel means.
#[must_use]
pub fn the_fill_of(a_box: ABoxToDraw, row: u32) -> Vec<f64> {
    coverage(a_box, row, 0)
}

/// How much of each pixel of one row this box's border covers.
///
/// The border is the fill minus the fill inset by its own thickness, so a
/// corner's inner curve is the outer curve moved in rather than a second
/// drawing of it — which is what keeps the two concentric at every radius.
#[must_use]
pub fn the_edge_of(a_box: ABoxToDraw, row: u32) -> Vec<f64> {
    if a_box.edge_is == 0 {
        return vec![0.0; a_box.area.wide as usize];
    }
    let outer = coverage(a_box, row, 0);
    let inner = coverage(a_box, row, a_box.edge_is);
    outer
        .into_iter()
        .zip(inner)
        .map(|(out, within)| (out - within).max(0.0))
        .collect()
}

/// The coverage of a rounded rectangle inset by this much on every side.
fn coverage(a_box: ABoxToDraw, row: u32, inset: u32) -> Vec<f64> {
    let wide = a_box.area.wide as usize;
    let mut how_much = vec![0.0; wide];

    let inset = f64::from(inset);
    let left = inset;
    let right = f64::from(a_box.area.wide) - inset;
    let top = inset;
    let bottom = f64::from(a_box.area.tall) - inset;
    if right <= left || bottom <= top {
        return how_much;
    }
    // A corner shrinks with the inset, so the inner curve stays concentric
    // with the outer one rather than becoming a different shape.
    let corner = (f64::from(a_box.corner) - inset).max(0.0);

    // The centre of this row, which is what a corner is measured from.
    let y = f64::from(row) + 0.5;
    if y < top || y > bottom {
        return how_much;
    }

    // How far in the shape starts on this row. Straight down the middle it is
    // the edge; within a corner's height it is the circle.
    let pull = if y < top + corner {
        corner
            - (corner * corner - (top + corner - y).powi(2))
                .max(0.0)
                .sqrt()
    } else if y > bottom - corner {
        corner
            - (corner * corner - (y - (bottom - corner)).powi(2))
                .max(0.0)
                .sqrt()
    } else {
        0.0
    };
    let from = left + pull;
    let to = right - pull;
    if to <= from {
        return how_much;
    }

    for (at, value) in how_much.iter_mut().enumerate() {
        // The pixel runs from `at` to `at + 1`; how much of it is inside.
        let begins = at as f64;
        let ends = begins + 1.0;
        let covered = ends.min(to) - begins.max(from);
        *value = covered.clamp(0.0, 1.0);
    }
    how_much
}

/// This measure on a display of this size.
///
/// The same proportion the rest of the layout is scaled by, so a corner is a
/// corner at every size and never a pixel count that held at one.
#[must_use]
pub fn as_this_display_draws_it(measure: u32, down: u32) -> u32 {
    let (_, drawn_at) = THE_DESIGN_WAS_DRAWN_AT;
    let scaled = f64::from(measure) * f64::from(down) / f64::from(drawn_at);
    // A border that rounds to nothing is a border a person cannot see, so the
    // thinnest anything here draws is one pixel.
    (scaled.round() as u32).max(1)
}

#[cfg(test)]
#[path = "a_box_on_the_surface_tests.rs"]
mod tests;
