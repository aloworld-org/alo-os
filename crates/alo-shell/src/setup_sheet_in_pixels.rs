//! A first-start sheet's layout, converted to the pixels it is drawn in.
//!
//! `crate::setup_sheet_layout` places everything in logical pixels, because
//! the design's numbers are logical. A sheet drawn at 200 per cent is laid out
//! at half its pixels, and every rectangle it letters into is converted here,
//! once, through the same smithay rounding `crate::setup_sheet_surface` uses
//! for the cards — so a line of type and the card it sits in cannot land a
//! pixel apart. `crate::sign_in_in_pixels` is the same answer for the sign-in
//! screen.
//!
//! # The text is measured in pixels and laid out in logical ones
//!
//! A sheet's title and introduction are shaped first, in the type the picture
//! is drawn in, and their heights decide where everything below them goes. So
//! those heights arrive here in pixels and are taken back to logical ones,
//! **rounded up**: a block a fraction of a logical pixel taller than its box
//! would otherwise touch whatever comes after it.
//!
//! At a scale of one every conversion is the identity, so a sheet at 100 per
//! cent is the picture it was.

use smithay::utils::{Logical, Physical, Rectangle, Scale, Size};

use crate::display_scale::DisplayScale;
use crate::setup_sheet_layout::{
    SetupSheetLayout, THE_COLUMN_IS_WIDE, TheTextIsTall, setup_sheet_laid_out,
};

/// The rectangles a sheet letters into, in its own pixels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SetupSheetInPixels {
    /// The stage line at the top right.
    pub(crate) stage: Rectangle<i32, Physical>,
    /// The small line above the title.
    pub(crate) eyebrow: Rectangle<i32, Physical>,
    /// The heading.
    pub(crate) title: Rectangle<i32, Physical>,
    /// The sentence under it.
    pub(crate) introduction: Rectangle<i32, Physical>,
    /// Each choice row, in order.
    pub(crate) choices: Vec<Rectangle<i32, Physical>>,
    /// What this sheet's answer means.
    pub(crate) consequence: Rectangle<i32, Physical>,
    /// Each action, in order.
    pub(crate) actions: Vec<Rectangle<i32, Physical>>,
    /// *Accessibility*, at the bottom left.
    pub(crate) accessibility: Rectangle<i32, Physical>,
    /// The footer line at the bottom right.
    pub(crate) footer: Rectangle<i32, Physical>,
}

/// A sheet laid out twice over: logically, for its cards, and in pixels, for
/// its words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TheSheetIsLaidOut {
    /// Where everything goes, in logical pixels, which is what
    /// `crate::setup_sheet_surface` rasterises through the scale.
    pub(crate) logical: SetupSheetLayout,
    /// The same, in the picture's pixels.
    pub(crate) pixels: SetupSheetInPixels,
}

/// How wide a sheet's column is in pixels on an output `width` pixels across,
/// for shaping its words before they are placed.
pub(crate) fn the_column_in_pixels(width: i32, display: DisplayScale) -> i32 {
    display.px(THE_COLUMN_IS_WIDE.unsigned_abs()).min(width)
}

/// Lay a sheet out on an output of `pixels` drawn at `display`, with its text
/// heights measured in pixels.
///
/// [`None`] where `crate::setup_sheet_layout` refuses: a screen too small at
/// this scale to hold the sheet.
pub(crate) fn laid_out(
    pixels: (i32, i32),
    tall: TheTextIsTall,
    display: DisplayScale,
) -> Option<TheSheetIsLaidOut> {
    let logical = |pixels: i32| i32::try_from(display.logical(pixels)).unwrap_or(i32::MAX);
    let up = |pixels: i32| -> i32 {
        let per_cent = i64::from(display.per_cent());
        let scaled = (i64::from(pixels.max(0)) * 100 + per_cent - 1) / per_cent;
        i32::try_from(scaled).unwrap_or(i32::MAX)
    };
    let at = setup_sheet_laid_out(
        Size::<i32, Logical>::from((logical(pixels.0), logical(pixels.1))),
        TheTextIsTall {
            title: up(tall.title),
            introduction: up(tall.introduction),
            consequence: up(tall.consequence),
            choices: tall.choices,
            actions: tall.actions,
        },
    )?;
    let scale = Scale::from(display.factor());
    let px = |logical: Rectangle<i32, Logical>| -> Rectangle<i32, Physical> {
        logical.to_physical_precise_round(scale)
    };
    let in_pixels = SetupSheetInPixels {
        stage: px(at.stage),
        eyebrow: px(at.eyebrow),
        title: px(at.title),
        introduction: px(at.introduction),
        choices: at.choices.iter().map(|it| px(*it)).collect(),
        consequence: px(at.consequence),
        actions: at.actions.iter().map(|it| px(*it)).collect(),
        accessibility: px(at.accessibility),
        footer: px(at.footer),
    };
    Some(TheSheetIsLaidOut {
        logical: at,
        pixels: in_pixels,
    })
}

#[cfg(test)]
#[path = "setup_sheet_in_pixels_tests.rs"]
mod tests;
