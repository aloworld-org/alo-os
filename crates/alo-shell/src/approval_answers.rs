//! The two answers on the approval panel: shaped, sized alike, and placed in
//! reading order.
//!
//! Both boxes are as wide and as tall as the larger of the two, so neither
//! answer looks like the default by being bigger. They stand side by side at
//! the far end of the reading line when they fit across the panel, and one
//! above the other when they do not — a long translation of *no* is given its
//! lines, never shortened. The selected answer, when the person has selected
//! one, gets a thicker edge and a bar under its words: two shapes, never a
//! colour of its own.

use alo_strings::Direction;
use cosmic_text::FontSystem;
use smithay::utils::Rectangle;

use crate::ApprovalAnswer;
use crate::approval_raster::{AnswerDrawn, ApprovalPicture, Measure, Palette, framed, placed};
use crate::painted::Solid;
use crate::painted_text::{Shaped, sentence};

/// Both answers shaped, and how they fit across the panel.
pub(crate) struct Answers {
    /// Each answer and its words, shaped to the width of its box's inside.
    shaped: Vec<(ApprovalAnswer, String, Shaped)>,
    /// One box's width: both are as wide as the wider of the two.
    box_width: i32,
    /// One box's height: both are as tall as the taller of the two.
    box_height: i32,
    /// Whether the two stand side by side rather than one above the other.
    side_by_side: bool,
    /// The height the two take together.
    pub(crate) height: i32,
}

impl Answers {
    /// Shape both answers, side by side when they fit across `inner` and one
    /// above the other when they do not — never shortened.
    pub(crate) fn laid_out(
        fonts: &mut FontSystem,
        words: Vec<(ApprovalAnswer, String)>,
        inner: i32,
        measure: Measure,
        palette: Palette,
    ) -> Self {
        let inset = measure.px(16);
        let gap = measure.px(12);
        let widest = inner - 2 * inset;
        let natural = words
            .iter()
            .map(|(_, text)| {
                sentence(
                    fonts,
                    text,
                    widest,
                    measure.metrics(),
                    palette.ground,
                    palette.ink,
                )
                .width
            })
            .max()
            .unwrap_or(0);
        let side_by_side = 2 * (natural + 2 * inset) + gap <= inner;
        let inside = if side_by_side {
            (natural + 1).min(widest)
        } else {
            widest
        };
        let shaped: Vec<_> = words
            .into_iter()
            .map(|(answer, text)| {
                let shaped = sentence(
                    fonts,
                    &text,
                    inside,
                    measure.metrics(),
                    palette.ground,
                    palette.ink,
                );
                (answer, text, shaped)
            })
            .collect();
        let tallest = shaped
            .iter()
            .map(|(_, _, shaped)| shaped.height)
            .max()
            .unwrap_or(0);
        let box_height = tallest + 2 * measure.px(12);
        let box_width = inside + 2 * inset;
        let count = shaped.len() as i32;
        let height = if side_by_side {
            box_height
        } else {
            count * box_height + (count - 1).max(0) * gap
        };
        Self {
            shaped,
            box_width,
            box_height,
            side_by_side,
            height,
        }
    }

    /// Place both boxes from `origin`, at the far end of the reading line when
    /// side by side, in reading order.
    #[expect(
        clippy::too_many_arguments,
        reason = "one layout step: where, how wide, what is selected, which way it reads, and the two measures"
    )]
    pub(crate) fn placed(
        self,
        drawn: &mut ApprovalPicture,
        origin: (i32, i32),
        inner: i32,
        selected: Option<ApprovalAnswer>,
        reading: Direction,
        measure: Measure,
        palette: Palette,
    ) {
        let gap = measure.px(12);
        let inset = measure.px(16);
        let (x0, y0) = origin;
        let count = self.shaped.len() as i32;
        let row_width = count * self.box_width + (count - 1).max(0) * gap;
        let right_to_left = reading == Direction::RightToLeft;
        for (index, (answer, words, shaped)) in self.shaped.into_iter().enumerate() {
            let index = index as i32;
            let (x, y) = if self.side_by_side {
                let from_start = index * (self.box_width + gap);
                if right_to_left {
                    (x0 + row_width - from_start - self.box_width, y0)
                } else {
                    (x0 + inner - row_width + from_start, y0)
                }
            } else {
                (x0, y0 + index * (self.box_height + gap))
            };
            let area = Rectangle::new((x, y).into(), (self.box_width, self.box_height).into());
            let is_selected = selected == Some(answer);
            let edge = if is_selected {
                measure.px(3)
            } else {
                measure.px(1)
            };
            framed(&mut drawn.solids, area, edge, palette);
            let text_top = y + (self.box_height - shaped.height) / 2;
            if is_selected {
                drawn.solids.push(Solid {
                    area: Rectangle::new(
                        (x + inset, text_top + shaped.height).into(),
                        (self.box_width - 2 * inset, measure.px(3)).into(),
                    ),
                    colour: palette.ink,
                });
            }
            drawn.inked.extend(placed(shaped, x + inset, text_top));
            drawn.answers.push(AnswerDrawn {
                answer,
                area,
                words,
                selected: is_selected,
            });
        }
    }
}
