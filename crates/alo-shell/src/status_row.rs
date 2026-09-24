//! One row of a status-area indicator: a mark, a sentence, and the box round
//! them.
//!
//! **Two indicators sit in the status area and they have to look alike.** What
//! is leaving this machine (`crate::egress_status_raster`) and what is in use
//! (`crate::in_use_raster`) are different lists decided by different crates, and
//! a person reads them as one surface — so the height of a row, the padding
//! round its words, the side its mark sits on and the way its words are cut at
//! the screen's edge are decided **here**, once, rather than twice in two files
//! that would drift.
//!
//! What each indicator keeps for itself is its own: which lines there are, what
//! each says, and the shape of its mark.

use alo_appearance::TextScale;
use alo_strings::Direction;
use cosmic_text::{FontSystem, Metrics};
use smithay::utils::{Physical, Rectangle};

use crate::egress_status_place::Place;
use crate::painted::{Inked, Solid};
use crate::painted_text::{Shaped, sentence};

/// Measures scaled by the person's text scale, never below one pixel.
#[derive(Clone, Copy)]
pub(crate) struct Measure {
    /// The scale, in percent.
    pub(crate) percent: i32,
}

impl Measure {
    /// The measures for this person's text scale.
    pub(crate) fn of(scale: TextScale) -> Self {
        Self {
            percent: i32::from(scale.as_percent()),
        }
    }

    /// `at_one` pixels at the ordinary scale, scaled.
    pub(crate) fn px(self, at_one: i32) -> i32 {
        (at_one * self.percent / 100).max(1)
    }

    /// The text metrics every row uses.
    pub(crate) fn metrics(self) -> Metrics {
        let factor = self.percent as f32 / 100.0;
        Metrics::new(16.0 * factor, 24.0 * factor)
    }
}

/// A row's ground and its ink. A mark's own colours are the mark's.
#[derive(Clone, Copy)]
pub(crate) struct Palette {
    /// The row's ground.
    pub(crate) ground: [u8; 3],
    /// Its words and its edge.
    pub(crate) ink: [u8; 3],
}

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

/// A placed row, in the pieces an indicator's picture is made of.
pub(crate) struct Drawn {
    /// Where it ended up and what it says.
    pub(crate) row: Row,
    /// Its edge, its ground and its mark, in painting order.
    pub(crate) solids: Vec<Solid>,
    /// Its words, when any of them are on the output at all.
    pub(crate) inked: Option<Inked>,
}

/// A row shaped for a place, before it is put anywhere.
pub(crate) struct Laid {
    /// The sentence.
    sentence: String,
    /// The words, inked.
    words: Shaped,
    /// The whole row's width.
    pub(crate) width: i32,
    /// The whole row's height.
    pub(crate) height: i32,
}

impl Laid {
    /// Shape `text` as a row that fits `place`.
    pub(crate) fn of(
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

    /// Put the row's edge, ground, mark and words at `corner`, cut at the
    /// output's edges.
    ///
    /// `mark` is given the square its shape goes in and hands back the shapes
    /// that make it — each indicator's own, and the only part of a row that
    /// differs between them.
    pub(crate) fn place(
        self,
        corner: (i32, i32),
        output: (i32, i32),
        measure: Measure,
        palette: Palette,
        reading: Direction,
        mark: impl FnOnce(Rectangle<i32, Physical>) -> Vec<Solid>,
    ) -> Drawn {
        let (left, top) = corner;
        let whole = Rectangle::<i32, Physical>::from_size(output.into());
        let area = Rectangle::new((left, top).into(), (self.width, self.height).into());
        let edge = measure.px(1);
        let side = measure.px(18);
        let pad = measure.px(8);
        let (mark_x, words_x) = match reading {
            Direction::LeftToRight => (left + pad, left + pad + side + pad),
            Direction::RightToLeft => (left + pad + self.words.width_of_box + pad, left + pad),
        };
        let words_y = top + measure.px(6);
        let mark_y = words_y + (measure.px(24) - side) / 2;
        let mark_area = Rectangle::new((mark_x, mark_y).into(), (side, side).into());
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
        solids.extend(mark(mark_area));
        let kept = solids
            .into_iter()
            .filter_map(|solid| {
                solid.area.intersection(whole).map(|area| Solid {
                    area,
                    colour: solid.colour,
                })
            })
            .collect();
        let words_area = Rectangle::new(
            (words_x, words_y).into(),
            (self.words.width_of_box, self.words.height).into(),
        );
        let inked = self
            .words
            .placed(words_x, words_y, output.1)
            .and_then(|inked| cut_across(inked, output.0));
        Drawn {
            row: Row {
                area: area.intersection(whole).unwrap_or_default(),
                mark: mark_area,
                words: words_area,
                sentence: self.sentence,
            },
            solids: kept,
            inked,
        }
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
