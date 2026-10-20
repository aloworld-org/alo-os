//! Where the alo Bar's open panel goes, and where everything inside it sits.
//!
//! **Arithmetic from the display's own size, not the design's.** Every number
//! here is derived from `docs/design/the-alo-bar-when-it-is-open.md`, which
//! read them off `alo Bar / Open` (`81:890`) at the design's reference size of
//! 1440 × 960 — and that is a size no machine has. `CLAUDE.md`: *a figure taken
//! from a frame becomes a proportion or a named rule, never a constant that
//! holds only at the size somebody drew.*
//!
//! # What is a proportion and what is a measure
//!
//! The **panel** is a proportion: 47.2% of the display's width, its top 21.7%
//! of the way down. A panel in logical pixels would be a third of a small
//! screen and a stamp on a large one.
//!
//! The **field and each row** are a measure — `A_TARGET_IS_TALL` — because
//! they are things a person points at, and a target does not shrink with the
//! screen it is on. The design draws a row as **exactly the box the field is**,
//! which is it saying that a row is a thing you act on, drawn at the size of
//! the thing you type into.
//!
//! # The list runs past the bottom, and that is the design
//!
//! Four rows are drawn into a panel 390 tall and the fourth begins at 379, so
//! it is cut. The design is showing a list that **scrolls**, which is why
//! nothing here caps the rows to what fits: `crate::the_bar_finds` returns
//! every match, and this says which of them are on the screen.

use smithay::utils::{Physical, Rectangle, Size};

/// How tall a thing a person points at is, in this surface's own pixels.
///
/// The field, and every row. Named rather than repeated because the design
/// draws them as one box at one size, and two numbers would let them drift.
pub(crate) const A_TARGET_IS_TALL: i32 = 56;

/// The gap between a row and the next, and between the field and the line
/// under it. 12 at the reference size, and it is a measure rather than a
/// proportion for the reason the target is.
pub(crate) const BETWEEN_THINGS: i32 = 12;

/// How much air there is inside the panel, all the way round.
///
/// 25 at the reference, which is the panel's own width over 27.2 — so it grows
/// with the panel rather than staying a hairline on a large screen.
pub(crate) const INSIDE_THE_PANEL: f32 = 1.0 / 27.2;

/// How wide the panel is as a share of the display.
const THE_PANEL_IS_WIDE: f32 = 680.0 / 1440.0;

/// How far down the display the panel's top sits.
const THE_PANEL_BEGINS: f32 = 208.0 / 960.0;

/// How tall the panel is as a share of the display.
const THE_PANEL_IS_TALL: f32 = 390.0 / 960.0;

/// Where the Bar and everything in it is, on a display of this size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheBarsPlaces {
    /// The panel itself.
    pub panel: Rectangle<i32, Physical>,
    /// The field somebody types into.
    pub field: Rectangle<i32, Physical>,
    /// The line that says where the results came from.
    pub origin: Rectangle<i32, Physical>,
    /// Where the first row begins, and how wide every row is.
    pub first_row: Rectangle<i32, Physical>,
    /// How far down from one row to the next.
    pub row_to_row: i32,
    /// How many rows are wholly on the panel.
    ///
    /// **Whole ones only.** The design draws a fourth row cut off by the
    /// panel's edge, which says the list scrolls; a row half drawn is a row a
    /// person cannot read, so this counts what can be read and the surface
    /// scrolls for the rest.
    pub rows_that_fit: usize,
}

/// Work out where the Bar goes on a display this size.
///
/// [`None`] for a display too small to hold the panel at all — which is a real
/// answer rather than a clamp: a Bar squeezed onto a screen it does not fit is
/// a surface a person cannot use, and the caller can say so instead of drawing
/// one.
pub(crate) fn where_the_bar_opens(display: Size<i32, Physical>) -> Option<TheBarsPlaces> {
    let across = (display.w as f32 * THE_PANEL_IS_WIDE).round() as i32;
    let down = (display.h as f32 * THE_PANEL_IS_TALL).round() as i32;
    let inset = (across as f32 * INSIDE_THE_PANEL).round().max(1.0) as i32;

    // The panel has to hold its own air twice over, the field, and at least one
    // whole row — anything less is a surface with nothing on it.
    let least = inset * 2 + A_TARGET_IS_TALL * 2 + BETWEEN_THINGS * 3;
    if across <= inset * 2 || down < least {
        return None;
    }

    let left = (display.w - across) / 2;
    let top = (display.h as f32 * THE_PANEL_BEGINS).round() as i32;
    let panel = Rectangle::new((left, top).into(), (across, down).into());
    // **The content is a share of the panel, not the panel less its air twice
    // over**, and the difference is two pixels at the reference size which
    // says something real: the frame insets the content by 25 on the left and
    // leaves 23 on the right, because the panel's own width carries a hairline
    // the content does not sit inside. Deriving the width by subtraction would
    // be inventing a symmetry the design does not have.
    let inner = (across as f32 * (632.0 / 680.0)).round() as i32;

    // Down the panel: the heading, the field, the line, then the rows. The
    // heading's own height is the design's 40 at the reference, taken as the
    // target's height less the gap so that one measure moves them all.
    let heading = A_TARGET_IS_TALL - BETWEEN_THINGS - 4;
    let field_top = top + inset + heading + BETWEEN_THINGS;
    let field = Rectangle::new(
        (left + inset, field_top).into(),
        (inner, A_TARGET_IS_TALL).into(),
    );
    let origin_top = field_top + A_TARGET_IS_TALL + BETWEEN_THINGS;
    let origin = Rectangle::new((left + inset, origin_top).into(), (inner, 18).into());
    let rows_top = origin_top + 18 + BETWEEN_THINGS;
    let first_row = Rectangle::new(
        (left + inset, rows_top).into(),
        (inner, A_TARGET_IS_TALL).into(),
    );

    let row_to_row = A_TARGET_IS_TALL + BETWEEN_THINGS;
    // **Measured to the panel's own edge and not to the air inside it.** The
    // frame draws its third row ending 23 short of the bottom and its fourth
    // cut off, which is the list saying it scrolls — insetting the bottom as
    // well would count two where the design shows three.
    let room = (top + down) - rows_top;
    let rows_that_fit = usize::try_from(((room + BETWEEN_THINGS) / row_to_row).max(0)).unwrap_or(0);

    Some(TheBarsPlaces {
        panel,
        field,
        origin,
        first_row,
        row_to_row,
        rows_that_fit,
    })
}

#[cfg(test)]
#[path = "where_the_bar_opens_tests.rs"]
mod tests;
