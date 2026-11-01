//! One entry of the record, as `560:216` draws it: the time, the kind's icon
//! on its tile, what happened — the agent's name and the clause, the
//! machine's own sentences, and the marks under them — and, at its end,
//! *Already sent* where something left this computer.
//!
//! **Shaped first and drawn after**, so an entry is measured whole before
//! anything of it reaches the canvas: an entry that does not fit waits for
//! the view to move, and is never cut.
//!
//! # What is drawn the same for every entry
//!
//! The clause is drawn in one style and one ink whatever became of the entry,
//! and so is every line after it: **nothing refused is dimmer, smaller or
//! folded away.** A refusal *adds* — its icon, and the word *Refused* in the
//! danger colour under its lines — so it is easier to find, never harder.

use cosmic_text::FontSystem;
use smithay::utils::Rectangle;

use crate::painted_text::Shaped;
use crate::record_header::placed;
use crate::the_records_icons::ARecordIcon;
use crate::record_kinds::EntryKind;
use crate::record_lines::EntryWords;
use crate::record_measures as m;
use crate::record_palette::RecordPalette;
use crate::record_raster::{EntryDrawn, TextDrawn};
use crate::record_room::Room;
use crate::settings_canvas::Canvas;
use crate::settings_type::{TheType, written};

/// The words the window adds to an entry, in the person's language.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TheWindowsWords<'a> {
    /// *Refused*, under a refusal's lines.
    pub(crate) refused: &'a str,
    /// *Already sent*, at the end of what left.
    pub(crate) already_sent: &'a str,
}

/// Where an entry's columns are, across a card.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TheColumns {
    /// The card's left edge.
    pub(crate) left: i32,
    /// The card's width.
    pub(crate) wide: i32,
    /// The time column's width, one for every entry in view.
    pub(crate) time: i32,
}

/// Words shaped, and the words they were shaped from.
type Said = (Shaped, String);

/// One entry, shaped and measured, not yet drawn.
pub(crate) struct AnEntryShaped {
    /// The words it was shaped from.
    words: EntryWords,
    /// Which of the design's kinds it is.
    kind: EntryKind,
    /// The time.
    time: Option<Shaped>,
    /// The agent's name.
    agent: Option<Shaped>,
    /// The clause.
    clause: Shaped,
    /// The lines after it.
    after: Vec<Shaped>,
    /// That the person said yes.
    approved: Option<Shaped>,
    /// *Refused*.
    refused: Option<Said>,
    /// *Already sent*.
    already_sent: Option<Said>,
    /// How tall it is, inside its card.
    pub(crate) height: i32,
}

/// The time, shaped, for measuring the column every entry shares.
pub(crate) fn the_time(
    fonts: &mut FontSystem,
    room: Room,
    palette: RecordPalette,
    time: &str,
) -> Shaped {
    written(
        fonts,
        time,
        TheType::Label,
        room.px(200),
        room.measure,
        (palette.surface, palette.secondary),
    )
    .cropped_to_its_ink()
}

/// A mark's words — *Refused*, *Already sent*, that the person said yes — in
/// the small label, at most `wide`.
fn a_mark(
    fonts: &mut FontSystem,
    room: Room,
    (text, colour, ground): (&str, [u8; 3], [u8; 3]),
    wide: i32,
) -> Shaped {
    written(
        fonts,
        text,
        TheType::SmallLabel,
        wide.max(1),
        room.measure,
        (ground, colour),
    )
    .cropped_to_its_ink()
}

/// Shape `words` for a card laid out as `columns`.
pub(crate) fn shaped(
    fonts: &mut FontSystem,
    room: Room,
    palette: RecordPalette,
    columns: TheColumns,
    (words, said): (EntryWords, TheWindowsWords<'_>),
) -> AnEntryShaped {
    let px = |at_one: i32| room.px(at_one);
    let (pad, side, between) = m::IN_AN_ENTRY;
    let (tile, _, _) = m::THE_TILE;
    let (head, mark, after_mark) = m::THE_HEAD;
    let (above, mark_tall, between_marks, mark_icon, before_words) = m::THE_MARKS;
    let kind = EntryKind::of(words.outcome);
    let ground = palette.surface;
    let already_sent = (kind == EntryKind::Left).then(|| {
        let shaped = a_mark(
            fonts,
            room,
            (said.already_sent, palette.muted, ground),
            px(200),
        );
        (shaped, said.already_sent.to_owned())
    });
    let content = (columns.wide
        - 2 * px(side)
        - columns.time
        - 2 * px(between)
        - px(tile)
        - already_sent
            .as_ref()
            .map_or(0, |(shaped, _)| shaped.width_of_box + px(between)))
    .max(px(80));
    let time = words
        .time
        .as_deref()
        .map(|time| the_time(fonts, room, palette, time));
    let agent = words.agent.as_deref().map(|name| {
        written(
            fonts,
            name,
            TheType::Label,
            content / 2,
            room.measure,
            (ground, palette.agent),
        )
        .cropped_to_its_ink()
    });
    let clause_wide = content
        - agent.as_ref().map_or(0, |name| {
            px(mark) + 2 * px(after_mark) + name.width_of_box
        });
    let clause = written(
        fonts,
        &words.clause,
        TheType::Name,
        clause_wide.max(px(80)),
        room.measure,
        (ground, palette.ink),
    );
    let after: Vec<Shaped> = words
        .after
        .iter()
        .map(|line| {
            written(
                fonts,
                line,
                TheType::Body,
                content,
                room.measure,
                (ground, palette.secondary),
            )
        })
        .collect();
    let mark_wide = content / 2 - px(mark_icon) - px(before_words) - px(between_marks);
    let approved = words
        .approved
        .as_deref()
        .map(|text| a_mark(fonts, room, (text, palette.allowed, ground), mark_wide));
    let refused = (kind == EntryKind::Refused).then(|| {
        let shaped = a_mark(
            fonts,
            room,
            (said.refused, palette.refused, ground),
            mark_wide,
        );
        (shaped, said.refused.to_owned())
    });
    let marks_tall = approved
        .iter()
        .chain(refused.iter().map(|(shaped, _)| shaped))
        .map(|shaped| shaped.height.max(px(mark_tall)))
        .max();
    let lines: i32 = after
        .iter()
        .map(|line| px(m::BETWEEN_LINES) + line.height)
        .sum();
    let what_happened = clause.height.max(px(head))
        + lines
        + marks_tall.map_or(0, |tall| px(m::BETWEEN_LINES) + px(above) + tall);
    let tallest = what_happened
        .max(px(tile))
        .max(time.as_ref().map_or(0, |time| time.height));
    AnEntryShaped {
        words,
        kind,
        time,
        agent,
        clause,
        after,
        approved,
        refused,
        already_sent,
        height: tallest + 2 * px(pad),
    }
}

/// Where the next mark under an entry's lines goes.
struct TheMarksRow {
    /// Across.
    x: i32,
    /// Its top.
    y: i32,
}

impl TheMarksRow {
    /// Draw one mark — its icon and its words — and move along.
    fn mark(
        &mut self,
        canvas: &mut Canvas,
        room: Room,
        (shaped, text): (&Shaped, &str),
        (icon, colour): (ARecordIcon, [u8; 3]),
    ) -> TextDrawn {
        let px = |at_one: i32| room.px(at_one);
        let (_, mark_tall, between_marks, mark_icon, before_words) = m::THE_MARKS;
        let tall = shaped.height.max(px(mark_tall));
        icon.drawn(
            canvas,
            Rectangle::new(
                (self.x, self.y + (tall - px(mark_icon)) / 2).into(),
                (px(mark_icon), px(mark_icon)).into(),
            ),
            colour,
        );
        let words_x = self.x + px(mark_icon) + px(before_words);
        let drawn = placed(
            canvas,
            room,
            shaped,
            text,
            words_x,
            self.y + (tall - shaped.height) / 2,
        );
        self.x = words_x + shaped.width_of_box + px(between_marks);
        drawn
    }
}

impl AnEntryShaped {
    /// Draw it with its top at `y` in a card laid out as `columns`.
    pub(crate) fn drawn(
        self,
        canvas: &mut Canvas,
        room: Room,
        palette: RecordPalette,
        columns: TheColumns,
        y: i32,
    ) -> EntryDrawn {
        let px = |at_one: i32| room.px(at_one);
        let (pad, side, between) = m::IN_AN_ENTRY;
        let (tile, tile_corner, icon) = m::THE_TILE;
        let (head, mark, after_mark) = m::THE_HEAD;
        let top = y + px(pad);
        let time_x = columns.left + px(side);
        let time = self
            .time
            .as_ref()
            .zip(self.words.time.as_deref())
            .map(|(shaped, text)| placed(canvas, room, shaped, text, time_x, top));
        let tile_x = time_x + columns.time + px(between);
        let tile_area = Rectangle::new((tile_x, top).into(), (px(tile), px(tile)).into());
        canvas.fill(tile_area, px(tile_corner) as f32, palette.cool, 1.0);
        let inset = (px(tile) - px(icon)) / 2;
        let kinds_icon = match self.kind {
            EntryKind::Changed | EntryKind::Else => ARecordIcon::Changed,
            EntryKind::Left => ARecordIcon::Sent,
            EntryKind::Refused => ARecordIcon::Refused,
        };
        kinds_icon.drawn(
            canvas,
            Rectangle::new(
                (tile_x + inset, top + inset).into(),
                (px(icon), px(icon)).into(),
            ),
            palette.ink,
        );
        let x = tile_x + px(tile) + px(between);
        let mut at = x;
        let agent = self
            .agent
            .as_ref()
            .zip(self.words.agent.as_deref())
            .map(|(shaped, name)| {
                ARecordIcon::Agent.drawn(
                    canvas,
                    Rectangle::new(
                        (at, top + (px(head) - px(mark)) / 2).into(),
                        (px(mark), px(mark)).into(),
                    ),
                    palette.agent,
                );
                at += px(mark) + px(after_mark);
                let drawn = placed(
                    canvas,
                    room,
                    shaped,
                    name,
                    at,
                    top + (px(head) - shaped.height) / 2,
                );
                at += shaped.width_of_box + px(after_mark);
                drawn
            });
        let clause = placed(canvas, room, &self.clause, &self.words.clause, at, top);
        let mut line_y = top + self.clause.height.max(px(head));
        let mut after = Vec::new();
        for (shaped, text) in self.after.iter().zip(self.words.after.iter()) {
            line_y += px(m::BETWEEN_LINES);
            after.push(placed(canvas, room, shaped, text, x, line_y));
            line_y += shaped.height;
        }
        let (above, _, _, _, _) = m::THE_MARKS;
        let mut row = TheMarksRow {
            x,
            y: line_y + px(m::BETWEEN_LINES) + px(above),
        };
        let approved = self
            .approved
            .as_ref()
            .zip(self.words.approved.as_deref())
            .map(|said| {
                row.mark(
                    canvas,
                    room,
                    said,
                    (ARecordIcon::Allowed, palette.allowed),
                )
            });
        let refused = self.refused.as_ref().map(|(shaped, text)| {
            row.mark(
                canvas,
                room,
                (shaped, text),
                (ARecordIcon::RefusedSmall, palette.refused),
            )
        });
        let already_sent = self.already_sent.as_ref().map(|(shaped, text)| {
            let end = columns.left + columns.wide - px(side);
            placed(canvas, room, shaped, text, end - shaped.width_of_box, top)
        });
        EntryDrawn {
            area: room.as_drawn(Rectangle::new(
                (columns.left, y).into(),
                (columns.wide, self.height).into(),
            )),
            icon: room.as_drawn(tile_area),
            kind: self.kind,
            words: self.words,
            day: None,
            date: None,
            time,
            agent,
            clause,
            after,
            approved,
            refused,
            already_sent,
        }
    }
}
