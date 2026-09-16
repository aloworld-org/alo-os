//! Settings laid out and rasterised for one output.
//!
//! # What is drawn
//!
//! While Settings is closed, **nothing**. While it is open, one tall panel
//! holding the sections in order (`crate::settings_lines`), a rule between
//! each two: first what a section has to say — its file did not read, a change
//! was refused, a revocation came back — and then its rows.
//!
//! # Shapes carry state, words carry meaning
//!
//! Every row is drawn in the same type and the same two colours. What is
//! chosen now is a filled square at the start of the row; the row with the
//! focus has an edge around it, and a doubled edge while Settings waits for a
//! chord. **Nothing is dimmed**, because nothing drawn here is disabled: a row
//! exists only for something a person can do, and a state that is only a hue
//! fails anybody who cannot tell the hue apart (EN 301 549). Terracotta is not
//! among the colours: it means the agent acting, and Settings is a person
//! choosing.
//!
//! # Every row is drawn whole or not at all
//!
//! The view moves to keep the focused row in it, and keeps the sentences its
//! section says in view above it where they fit. Nothing is cut: a line that
//! does not fit below the last one waits for the view to move, a rail on the
//! trailing edge says where the view is, and a window that cannot hold the
//! focused row whole **refuses the frame** ([`RenderError::SettingsScene`]).

use std::time::SystemTime;

use alo_appearance::{Scheme, TextScale};
use alo_strings::{Direction, Strings};
use smithay::utils::{Physical, Rectangle};

use crate::painted::{Inked, Solid};
use crate::painted_text::Shaped;
use crate::record_room::{Room, framed, rule};
use crate::settings_lines::{LineWords, sections};
use crate::{RecordLook, RenderError, SettingsWindow, WindowControlLabels};

/// How Settings looks, as the person's appearance and language decide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsLook {
    /// Light or dark, as `alo-appearance` decides it.
    pub scheme: Scheme,
    /// The person's text scale, applied to every measure.
    pub scale: TextScale,
    /// Which way the person reads, which decides where the mark and the rail
    /// are.
    pub reading: Direction,
}

/// One piece of text as drawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SettingsText {
    /// The text inked, exactly as it arrived.
    pub(crate) text: String,
    /// Its box.
    pub(crate) area: Rectangle<i32, Physical>,
}

/// One line as drawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SettingsLineDrawn {
    /// The words it was drawn from.
    pub(crate) words: LineWords,
    /// Each piece of text, in order.
    pub(crate) parts: Vec<SettingsText>,
    /// The whole line's box.
    pub(crate) area: Rectangle<i32, Physical>,
    /// Whether it has the focus.
    pub(crate) focused: bool,
    /// The mark saying it is chosen, when it is.
    pub(crate) mark: Option<Rectangle<i32, Physical>>,
}

/// Settings for one output size, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SettingsPicture {
    /// The output size it was laid out for.
    pub(crate) size: (i32, i32),
    /// The panel, while Settings is open.
    pub(crate) panel: Option<Rectangle<i32, Physical>>,
    /// Every sentence a section says, in view, in order.
    pub(crate) said: Vec<SettingsText>,
    /// Every line in view, in order.
    pub(crate) lines: Vec<SettingsLineDrawn>,
    /// Whether anything is out of view above or below.
    pub(crate) out_of_view: bool,
    /// Flat shapes, painted first.
    pub(crate) solids: Vec<Solid>,
    /// Text, painted after the shapes.
    pub(crate) inked: Vec<Inked>,
}

impl SettingsPicture {
    /// Nothing drawn, for an output of `size`.
    fn empty(size: (i32, i32)) -> Self {
        Self {
            size,
            panel: None,
            said: Vec::new(),
            lines: Vec::new(),
            out_of_view: false,
            solids: Vec::new(),
            inked: Vec::new(),
        }
    }

    /// Whether nothing at all is drawn.
    pub(crate) fn is_empty(&self) -> bool {
        self.panel.is_none() && self.solids.is_empty() && self.inked.is_empty()
    }
}

/// One thing laid out in the panel, before the view is chosen.
enum Block {
    /// A sentence a section says.
    Said(String, Shaped),
    /// A line, each piece shaped.
    Line(LineWords, Vec<Shaped>),
    /// The rule between two sections.
    Rule,
}

impl Block {
    /// How tall it is.
    fn height(&self, room: Room) -> i32 {
        match self {
            Self::Said(_, shaped) => shaped.height + room.gap,
            Self::Line(_, parts) => {
                parts.iter().map(|shaped| shaped.height).sum::<i32>() + 2 * room.gap
            }
            Self::Rule => 1 + 2 * room.gap,
        }
    }
}

/// Lay Settings out for an output of `size` and rasterise it.
///
/// # Errors
/// [`RenderError::SettingsScene`] when Settings is open and the output is too
/// small for a panel, larger than any it is laid out for, or cannot hold the
/// focused row whole.
pub(crate) fn picture(
    window: &SettingsWindow,
    strings: &Strings,
    labels: &mut WindowControlLabels,
    size: (i32, i32),
    look: SettingsLook,
    now: SystemTime,
) -> Result<SettingsPicture, RenderError> {
    let mut drawn = SettingsPicture::empty(size);
    let Some(open) = window.open() else {
        return Ok(drawn);
    };
    let room = Room::on(
        size,
        RecordLook {
            scheme: look.scheme,
            scale: look.scale,
            reading: look.reading,
        },
    )
    .map_err(|_| RenderError::SettingsScene)?;
    let top = room.margin;
    let bottom = size.1 - room.margin - room.pad;
    let panel = Rectangle::new(
        (room.left, top).into(),
        (room.width, size.1 - 2 * room.margin).into(),
    );
    framed(&mut drawn.solids, panel, room.measure.px(2), room.palette);
    drawn.panel = Some(panel);

    let fonts = &mut labels.fonts;
    let mark = room.measure.px(12);
    let (after_x, after_width) = room.after_box(look.reading);
    let focused = window.focused(now);
    let mut blocks = Vec::new();
    let mut focus_at = None;
    let mut section_at = 0;
    for (which, section) in sections(open, strings, now).into_iter().enumerate() {
        if which > 0 {
            blocks.push(Block::Rule);
        }
        let starts = blocks.len();
        for text in section.said {
            let shaped = room.shaped(fonts, &text, room.inner);
            blocks.push(Block::Said(text, shaped));
        }
        for line in section.lines {
            let parts = line
                .parts
                .iter()
                .enumerate()
                .map(|(at, text)| {
                    let width = if at == 0 {
                        room.inner - room.indent
                    } else {
                        after_width - room.indent
                    };
                    room.shaped(fonts, text, width)
                })
                .collect();
            if line.row.is_some() && line.row == focused {
                focus_at = Some(blocks.len());
                section_at = starts;
            }
            blocks.push(Block::Line(line, parts));
        }
    }

    let room_for = bottom - (top + room.pad);
    let heights: Vec<i32> = blocks.iter().map(|block| block.height(room)).collect();
    let from = match focus_at {
        Some(focus) => {
            let fits = |from: usize| {
                heights
                    .get(from..=focus)
                    .map_or(0, |seen| seen.iter().sum::<i32>())
                    <= room_for
            };
            let mut from = 0;
            while from < focus && !fits(from) {
                from += 1;
            }
            if !fits(from) {
                return Err(RenderError::SettingsScene);
            }
            if from > section_at && fits(section_at) {
                section_at
            } else {
                from
            }
        }
        None => 0,
    };

    let edge = room.measure.px(2);
    let mut y = top + room.pad;
    let mut shown = 0;
    for (block, height) in blocks.into_iter().zip(heights.iter().copied()).skip(from) {
        if y + height > bottom {
            break;
        }
        match block {
            Block::Said(text, shaped) => {
                let rows = shaped.height;
                drawn
                    .said
                    .push(placed(&mut drawn.inked, shaped, text, room.text_x, y));
                y += rows + room.gap;
            }
            Block::Rule => {
                y += room.gap;
                rule(&mut drawn.solids, room, y);
                y += 1 + room.gap;
            }
            Block::Line(words, parts) => {
                let area = Rectangle::new(
                    (room.text_x - edge * 2, y).into(),
                    (room.inner + edge * 4, height).into(),
                );
                let is_focused = words.row.is_some() && words.row == focused;
                if is_focused {
                    framed(&mut drawn.solids, area, edge, room.palette);
                    if window.is_waiting_for_a_chord() {
                        let inner = Rectangle::new(
                            (area.loc.x + 2 * edge, area.loc.y + 2 * edge).into(),
                            (area.size.w - 4 * edge, area.size.h - 4 * edge).into(),
                        );
                        framed(&mut drawn.solids, inner, edge, room.palette);
                    }
                }
                let mut at = y + room.gap;
                let (first_x, mark_x) = match look.reading {
                    Direction::RightToLeft => (room.text_x, room.text_x + room.inner - mark),
                    Direction::LeftToRight => (room.text_x + room.indent, room.text_x),
                };
                let marked = words.chosen.then(|| {
                    Rectangle::new(
                        (mark_x, at + room.measure.px(6)).into(),
                        (mark, mark).into(),
                    )
                });
                if let Some(square) = marked {
                    drawn.solids.push(Solid {
                        area: square,
                        colour: room.palette.ink,
                    });
                }
                let mut texts = Vec::new();
                for (which, (shaped, text)) in parts.into_iter().zip(words.parts.iter()).enumerate()
                {
                    let rows = shaped.height;
                    let x = if which == 0 {
                        first_x
                    } else {
                        after_x + room.indent
                    };
                    texts.push(placed(&mut drawn.inked, shaped, text.clone(), x, at));
                    at += rows;
                }
                drawn.lines.push(SettingsLineDrawn {
                    words,
                    parts: texts,
                    area,
                    focused: is_focused,
                    mark: marked,
                });
                y += height;
            }
        }
        shown += 1;
    }

    if shown == 0 && !heights.is_empty() {
        return Err(RenderError::SettingsScene);
    }
    drawn.out_of_view = from > 0 || from + shown < heights.len();
    if drawn.out_of_view {
        let track = Rectangle::new(
            (room.rail_x, top + room.pad).into(),
            (room.rail, room_for.max(room.rail)).into(),
        );
        let total = heights.len().max(1);
        let at = |blocks: usize| (track.size.h as usize * blocks / total) as i32;
        let thumb = Rectangle::new(
            (room.rail_x, track.loc.y + at(from)).into(),
            (room.rail, at(shown).max(room.measure.px(8))).into(),
        );
        framed(&mut drawn.solids, track, room.measure.px(1), room.palette);
        drawn.solids.push(Solid {
            area: thumb,
            colour: room.palette.ink,
        });
    }
    Ok(drawn)
}

/// Text placed whole at (`x`, `y`): it was measured to fit, so nothing is cut.
fn placed(inked: &mut Vec<Inked>, shaped: Shaped, text: String, x: i32, y: i32) -> SettingsText {
    let area = Rectangle::new((x, y).into(), (shaped.width_of_box, shaped.height).into());
    let bottom = y + shaped.height;
    inked.extend(shaped.placed(x, y, bottom));
    SettingsText { text, area }
}

#[cfg(test)]
#[path = "settings_raster_tests.rs"]
mod tests;
