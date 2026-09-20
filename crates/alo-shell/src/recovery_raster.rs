//! The recovery screen laid out and rasterised for one output: a panel with
//! the sentence `alo-keeping-up` decided, and under it the two moments going
//! back can happen at — or that sentence alone, when it cannot.
//!
//! # It looks like the panel a person already knows
//!
//! The same two colours, the same measures and the same framed panel as the
//! approval surface (`crate::approval_raster`), deliberately. This is the
//! screen a person reaches on the day their machine would not start, and
//! meeting a surface that looks like nothing else on the machine is the moment
//! they stop believing it is their machine at all.
//!
//! # The sentence is drawn whole or not at all
//!
//! It is wrapped at word boundaries to the panel's width and never shortened,
//! never given an ellipsis and never cut at the bottom of the output. A panel
//! that cannot hold the whole of it **refuses the frame**
//! ([`RenderError::RecoveryScene`]) — half of a sentence about replacing the
//! operating system is a different claim from the whole of it.
//!
//! # The two moments stand one above the other
//!
//! Always, rather than side by side when they happen to fit. Both are whole
//! sentences in most languages — *go back at the next restart*, *restart and go
//! back now* — and two long sentences side by side is where one of them gets
//! shortened. One column also means the order a person Tabs through is the
//! order they read, with no reading direction to mirror.
//!
//! # A selected moment is never told apart by colour alone
//!
//! Neither moment is selected when the screen goes up, and the two are then
//! drawn identically. The selected one gets a thicker edge **and** a bar under
//! its words — two shapes, in the same two colours as everything else on the
//! panel — so a person who cannot tell one hue from another can still tell
//! what Enter would do (EN 301 549; `docs/features.md`, ★).

use alo_appearance::{Scheme, TextScale};
use alo_keeping_up::WhenItApplies;
use alo_strings::Said;
use smithay::utils::{Physical, Rectangle};

use crate::approval_raster::{Measure, Palette, framed, placed};
use crate::painted::{Inked, Solid};
use crate::painted_text::sentence;
use crate::recovery_screen::RecoveryShows;
use crate::{Contrast, RenderError, WindowControlLabels};

/// The largest output side, in pixels, the screen is laid out for.
const LARGEST_SIDE: i32 = 16_384;

/// No moment at all, which is what a screen that cannot offer going back has
/// under its sentence.
const NO_MOMENTS: &[(WhenItApplies, Said)] = &[];

/// How the recovery screen looks, as the person's appearance decides it.
///
/// There is no reading direction here: the two moments stand one above the
/// other, so there is no row to mirror.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryLook {
    /// Light or dark, as `alo-appearance` decides it.
    pub scheme: Scheme,
    /// The person's text scale, applied to every measure.
    pub scale: TextScale,
    /// The design's palette, or the one high contrast decides
    /// (`crate::access_contrast`).
    pub contrast: Contrast,
}

/// One moment as drawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MomentDrawn {
    /// Which moment it is.
    pub(crate) when: WhenItApplies,
    /// Its whole box, edge included.
    pub(crate) area: Rectangle<i32, Physical>,
    /// The words inked in it, exactly as the vocabulary answered them.
    pub(crate) words: String,
    /// Whether it is the moment Enter would choose.
    pub(crate) selected: bool,
}

/// The screen for one output size, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecoveryPicture {
    /// The output size it was laid out for.
    pub(crate) size: (i32, i32),
    /// The panel.
    pub(crate) panel: Rectangle<i32, Physical>,
    /// The sentence as drawn, and where: the offer's, or the refusal's.
    pub(crate) sentence: (String, Rectangle<i32, Physical>),
    /// The moments, top to bottom. Empty when going back is not offered.
    pub(crate) moments: Vec<MomentDrawn>,
    /// Flat shapes, painted first.
    pub(crate) solids: Vec<Solid>,
    /// Text, painted after the shapes.
    pub(crate) inked: Vec<Inked>,
}

/// Lay the screen out for an output of `size` and rasterise it.
///
/// # Errors
/// [`RenderError::RecoveryScene`] for an output larger than any this screen is
/// laid out for, too narrow for a panel, or too short to hold the whole
/// sentence and both moments.
pub(crate) fn picture(
    shows: RecoveryShows<'_>,
    labels: &mut WindowControlLabels,
    size: (i32, i32),
    look: RecoveryLook,
) -> Result<RecoveryPicture, RenderError> {
    let (width, height) = size;
    if width > LARGEST_SIDE || height > LARGEST_SIDE {
        return Err(RenderError::RecoveryScene);
    }
    let measure = Measure::of(look.scale);
    let palette = Palette::of(look.scheme, look.contrast);
    let margin = measure.px(16);
    let pad = measure.px(20);
    let panel_width = (width - 2 * margin).min(measure.px(560));
    let inner = panel_width - 2 * pad;
    if inner < measure.px(160) {
        return Err(RenderError::RecoveryScene);
    }

    let (said, moments, selected) = match shows {
        RecoveryShows::Offered {
            offer,
            moments,
            selected,
        } => (offer, moments, selected),
        RecoveryShows::CannotGoBack(said) => (said, NO_MOMENTS, None),
    };

    let fonts = &mut labels.fonts;
    let line = |fonts: &mut cosmic_text::FontSystem, text: &str| {
        sentence(
            fonts,
            text,
            inner,
            measure.metrics(),
            palette.ground,
            palette.ink,
        )
    };

    let sentence_text = said.text().to_owned();
    let sentence_shaped = line(fonts, &sentence_text);
    let inset = measure.px(16);
    let gap = measure.px(12);
    let shaped: Vec<_> = moments
        .iter()
        .map(|(when, said)| {
            let text = said.text().to_owned();
            let shaped = line(fonts, &text);
            (*when, text, shaped)
        })
        .collect();
    let tallest = shaped
        .iter()
        .map(|(_, _, shaped)| shaped.height)
        .max()
        .unwrap_or(0);
    let box_height = tallest + 2 * measure.px(12);
    let count = i32::try_from(shaped.len()).unwrap_or(0);
    let moments_height = if count == 0 {
        0
    } else {
        measure.px(20) + count * box_height + (count - 1) * gap
    };

    let panel_height = pad + sentence_shaped.height + moments_height + pad;
    if panel_height > height - 2 * margin {
        return Err(RenderError::RecoveryScene);
    }
    let left = (width - panel_width) / 2;
    let top = (height - panel_height) / 2;
    let panel = Rectangle::new((left, top).into(), (panel_width, panel_height).into());

    let mut solids = Vec::new();
    let mut inked = Vec::new();
    framed(&mut solids, panel, measure.px(2), palette);

    let mut y = top + pad;
    let sentence_area = Rectangle::new(
        (left + pad, y).into(),
        (sentence_shaped.width_of_box, sentence_shaped.height).into(),
    );
    inked.extend(placed(sentence_shaped, left + pad, y));
    y += sentence_area.size.h;

    let mut drawn_moments = Vec::new();
    if count > 0 {
        y += measure.px(20);
        for (index, (when, words, shaped)) in shaped.into_iter().enumerate() {
            let index = i32::try_from(index).unwrap_or(0);
            let box_top = y + index * (box_height + gap);
            let area = Rectangle::new((left + pad, box_top).into(), (inner, box_height).into());
            let is_selected = selected == Some(when);
            let edge = if is_selected {
                measure.px(3)
            } else {
                measure.px(1)
            };
            framed(&mut solids, area, edge, palette);
            let text_top = box_top + (box_height - shaped.height) / 2;
            if is_selected {
                solids.push(Solid {
                    area: Rectangle::new(
                        (left + pad + inset, text_top + shaped.height).into(),
                        (inner - 2 * inset, measure.px(3)).into(),
                    ),
                    colour: palette.ink,
                });
            }
            inked.extend(placed(shaped, left + pad + inset, text_top));
            drawn_moments.push(MomentDrawn {
                when,
                area,
                words,
                selected: is_selected,
            });
        }
    }

    Ok(RecoveryPicture {
        size,
        panel,
        sentence: (sentence_text, sentence_area),
        moments: drawn_moments,
        solids,
        inked,
    })
}

#[cfg(test)]
#[path = "recovery_raster_tests.rs"]
mod tests;
