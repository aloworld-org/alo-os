//! The egress indicator laid out and rasterised for one output: a row for each
//! line `alo-egress` wrote, at the far end of the dock, and nothing at all
//! while nothing is leaving.
//!
//! # What is drawn
//!
//! While the machine's indicator is quiet — every question answered here, no
//! errand under way — **nothing**: no row, no mark, no terracotta, not one
//! pixel of this surface. While something is leaving, one row per line, each a
//! mark (`crate::egress_status_mark`) and the line itself, in the order the
//! indicator holds them, stacked from the status area's corner
//! (`crate::egress_status_place`).
//!
//! # What it reads
//!
//! `alo_indicator::Drawn` and nothing else about what is leaving. Each row's
//! words are `alo_indicator::Drawn::lines_said` — `alo-egress`'s sentence,
//! answered through the person's own vocabulary — and are drawn as they came:
//! never shortened, re-worded, or assembled here. An agent's egress and alo
//! OS's own errand are one list and are drawn the same way, because *no
//! telemetry* is a promise only somebody who can see the machine's unasked
//! traffic can check.
//!
//! # When the lines do not all fit
//!
//! A line is never dropped silently. When the output cannot hold every row, as
//! many as fit are drawn and the last row that fits carries the indicator's
//! own count instead — `alo_indicator::Drawn::lamp_said`, *3 things are
//! leaving this machine right now* — with its mark, so the screen still says
//! how much is leaving even where it cannot say all of what.

use alo_appearance::{Scheme, TextScale, Token};
use alo_dock::{Dock, Screen};
use alo_indicator::Drawn;
use alo_strings::{Direction, Strings};
use cosmic_text::{FontSystem, Metrics};
use smithay::utils::{Physical, Rectangle};

use crate::egress_status_mark::{mark, rgb};
use crate::egress_status_place::{Across, Place, Stacked};
use crate::painted::{Inked, Solid};
use crate::painted_text::sentence;
use crate::{RenderError, WindowControlLabels};

/// How the indicator looks, as the person's appearance and language decide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EgressStatusLook {
    /// Light or dark, as `alo-appearance` decides it.
    pub scheme: Scheme,
    /// The person's text scale, applied to every measure.
    pub scale: TextScale,
    /// Which way the person reads, which decides the far end of a row and
    /// which side of its words the mark is on.
    pub reading: Direction,
}

/// The largest output side, in pixels, the indicator is laid out for.
const LARGEST_SIDE: i32 = 16_384;

/// One row as drawn: where it is, and the sentence in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Row {
    /// The whole row, in output pixels.
    pub(crate) area: Rectangle<i32, Physical>,
    /// The mark's square.
    pub(crate) mark: Rectangle<i32, Physical>,
    /// Where the words are inked.
    pub(crate) words: Rectangle<i32, Physical>,
    /// The sentence inked there, exactly as it was handed over.
    pub(crate) sentence: String,
}

/// The indicator for one output size, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EgressStatusPicture {
    /// The output size it was laid out for.
    pub(crate) size: (i32, i32),
    /// Every row, nearest the dock first.
    pub(crate) rows: Vec<Row>,
    /// Flat shapes, painted first.
    pub(crate) solids: Vec<Solid>,
    /// Words, painted after the shapes.
    pub(crate) inked: Vec<Inked>,
}

impl EgressStatusPicture {
    /// Whether nothing at all is drawn.
    pub(crate) fn is_empty(&self) -> bool {
        self.rows.is_empty() && self.solids.is_empty() && self.inked.is_empty()
    }
}

/// Measures scaled by the person's text scale, never below one pixel.
#[derive(Clone, Copy)]
struct Measure {
    /// The scale, in percent.
    percent: i32,
}

impl Measure {
    /// `at_one` pixels at the ordinary scale, scaled.
    fn px(self, at_one: i32) -> i32 {
        (at_one * self.percent / 100).max(1)
    }

    /// The text metrics every row uses.
    fn metrics(self) -> Metrics {
        let factor = self.percent as f32 / 100.0;
        Metrics::new(16.0 * factor, 24.0 * factor)
    }
}

/// A row's colours for one scheme. Terracotta is the mark's and not here.
#[derive(Clone, Copy)]
struct Palette {
    /// The row's ground.
    ground: [u8; 3],
    /// Its words and its edge.
    ink: [u8; 3],
}

impl Palette {
    /// `alo-appearance`'s tokens for this scheme.
    fn of(scheme: Scheme) -> Self {
        match scheme {
            Scheme::Light => Self {
                ground: rgb(Token::Cream.colour()),
                ink: rgb(Token::Navy.colour()),
            },
            Scheme::Dark => Self {
                ground: rgb(Token::Charcoal.colour()),
                ink: rgb(Token::Cream.colour()),
            },
        }
    }
}

/// Lay the indicator out for an output of `size` and rasterise it.
///
/// A quiet indicator is an empty picture on any output. A lit one is laid out
/// at the far end of `dock` as `alo-dock` places it on this output.
///
/// # Errors
/// [`RenderError::EgressStatusScene`] when something is leaving and the output
/// is too small for `alo-dock` to lay a dock out on, or larger than any output
/// this surface is laid out for. A frame that cannot carry the indicator is
/// refused rather than drawn without it.
pub(crate) fn picture(
    drawn: &Drawn,
    strings: &Strings,
    dock: &Dock,
    labels: &mut WindowControlLabels,
    size: (i32, i32),
    look: EgressStatusLook,
) -> Result<EgressStatusPicture, RenderError> {
    let mut picture = EgressStatusPicture {
        size,
        rows: Vec::new(),
        solids: Vec::new(),
        inked: Vec::new(),
    };
    if drawn.lamp().is_dark() {
        return Ok(picture);
    }
    let (width, height) = size;
    if width > LARGEST_SIDE || height > LARGEST_SIDE {
        return Err(RenderError::EgressStatusScene);
    }
    let screen = Screen::of(
        u32::try_from(width).map_err(|_| RenderError::EgressStatusScene)?,
        u32::try_from(height).map_err(|_| RenderError::EgressStatusScene)?,
    )
    .map_err(|_| RenderError::EgressStatusScene)?;
    let measure = Measure {
        percent: i32::from(look.scale.as_percent()),
    };
    let place = Place::of(
        dock.layout_on(screen, look.scale, look.reading),
        size,
        measure.px(8),
    );
    let fonts = &mut labels.fonts;
    let palette = Palette::of(look.scheme);

    let sentences: Vec<String> = drawn
        .lines_said(strings)
        .into_iter()
        .map(|said| said.into_text())
        .collect();
    let gap = measure.px(4);
    let mut shaped: Vec<Laid> = sentences
        .iter()
        .map(|text| Laid::of(fonts, text, place, measure, palette))
        .collect();
    let whole = |rows: &[Laid]| -> i32 {
        rows.iter().map(|row| row.height).sum::<i32>() + gap * (rows.len() as i32 - 1).max(0)
    };
    if whole(&shaped) > place.room_down {
        let count = Laid::of(
            fonts,
            drawn.lamp_said(strings).text(),
            place,
            measure,
            palette,
        );
        while !shaped.is_empty() && whole(&shaped) + gap + count.height > place.room_down {
            shaped.pop();
        }
        shaped.push(count);
    }

    let mut at_y = place.stacked;
    for laid in shaped {
        let left = match place.across {
            Across::FromLeft(x) => x,
            Across::FromRight(x) => x - laid.width,
        };
        let top = match at_y {
            Stacked::Downwards(y) => {
                at_y = Stacked::Downwards(y + laid.height + gap);
                y
            }
            Stacked::Upwards(y) => {
                at_y = Stacked::Upwards(y - laid.height - gap);
                y - laid.height
            }
        };
        laid.place(
            &mut picture,
            (left.max(0), top.clamp(0, (height - 1).max(0))),
            measure,
            palette,
            look,
        );
    }
    Ok(picture)
}

/// One row shaped and measured, not yet placed.
struct Laid {
    /// The sentence.
    sentence: String,
    /// The words, inked.
    words: crate::painted_text::Shaped,
    /// The whole row's width.
    width: i32,
    /// The whole row's height.
    height: i32,
}

impl Laid {
    /// Shape `text` as a row that fits `place`.
    fn of(
        fonts: &mut FontSystem,
        text: &str,
        place: Place,
        measure: Measure,
        palette: Palette,
    ) -> Self {
        let chrome = 2 * measure.px(8) + measure.px(18) + measure.px(8);
        let most = (place.room_across - chrome).min(measure.px(480)).max(1);
        let first = sentence(
            fonts,
            text,
            most,
            measure.metrics(),
            palette.ground,
            palette.ink,
        );
        // Shaped again at the width its ink needs, so the row is as wide as
        // its words rather than as wide as the most a row may be.
        let words = if first.width > 0 && first.width + 1 < most {
            sentence(
                fonts,
                text,
                first.width + 1,
                measure.metrics(),
                palette.ground,
                palette.ink,
            )
        } else {
            first
        };
        let inside = words.height.max(measure.px(24));
        Self {
            sentence: text.to_owned(),
            width: chrome + words.width_of_box,
            height: inside + 2 * measure.px(6),
            words,
        }
    }

    /// Put the row's edge, ground, mark and words into `picture` with its
    /// top-left corner at `corner`, cut at the output's edges.
    fn place(
        self,
        picture: &mut EgressStatusPicture,
        corner: (i32, i32),
        measure: Measure,
        palette: Palette,
        look: EgressStatusLook,
    ) {
        let (left, top) = corner;
        let output = Rectangle::<i32, Physical>::from_size(picture.size.into());
        let area = Rectangle::new((left, top).into(), (self.width, self.height).into());
        let edge = measure.px(1);
        let side = measure.px(18);
        let pad = measure.px(8);
        let (mark_x, words_x) = match look.reading {
            Direction::LeftToRight => (left + pad, left + pad + side + pad),
            Direction::RightToLeft => (left + pad + self.words.width_of_box + pad, left + pad),
        };
        let words_y = top + measure.px(6);
        let mark_y = words_y + (measure.px(24) - side) / 2;
        let mut solids = vec![
            Solid {
                area,
                colour: palette.ink,
            },
            Solid {
                area: Rectangle::new(
                    (left + edge, top + edge).into(),
                    (self.width - 2 * edge, area.size.h - 2 * edge).into(),
                ),
                colour: palette.ground,
            },
        ];
        solids.extend(mark(mark_x, mark_y, side, look.scheme));
        for solid in solids {
            if let Some(kept) = solid.area.intersection(output) {
                picture.solids.push(Solid {
                    area: kept,
                    colour: solid.colour,
                });
            }
        }
        let words_area = Rectangle::new(
            (words_x, words_y).into(),
            (self.words.width_of_box, self.words.height).into(),
        );
        if let Some(inked) = self.words.placed(words_x, words_y, picture.size.1) {
            picture.inked.extend(cut_across(inked, picture.size.0));
        }
        picture.rows.push(Row {
            area: area.intersection(output).unwrap_or_default(),
            mark: Rectangle::new((mark_x, mark_y).into(), (side, side).into()),
            words: words_area,
            sentence: self.sentence,
        });
    }
}

/// Inked words with every column outside an output `width` wide cut away.
fn cut_across(inked: Inked, width: i32) -> Option<Inked> {
    let from = inked.area.loc.x.max(0);
    let to = (inked.area.loc.x + inked.area.size.w).min(width);
    if to <= from {
        return None;
    }
    if from == inked.area.loc.x && to == inked.area.loc.x + inked.area.size.w {
        return Some(inked);
    }
    let columns = inked.area.size.w.max(1) as usize;
    let skip = (from - inked.area.loc.x) as usize;
    let keep = (to - from) as usize;
    let pixels = inked
        .pixels
        .chunks(columns)
        .flat_map(|row| row.iter().skip(skip).take(keep).copied())
        .collect();
    Some(Inked {
        area: Rectangle::new(
            (from, inked.area.loc.y).into(),
            (to - from, inked.area.size.h).into(),
        ),
        pixels,
    })
}

#[cfg(test)]
#[path = "egress_status_raster_tests.rs"]
mod tests;
