//! The record window laid out and rasterised for one output.
//!
//! # What is drawn
//!
//! While the window is closed, **nothing**. While an account is open, one tall
//! panel: first every sentence the record says about itself
//! (`alo_recounting::Account::said` — whether anything answered, whether this is
//! all of it, whether the record goes all the way back, whether it is what it
//! says it is, what could not be read), and under a rule the entries, most
//! recent first, each with its clause at its head and the machine's own words
//! after it (`crate::record_lines`). While a refusal is open, the panel holds
//! that sentence and nothing else.
//!
//! The record's sentences about itself are drawn above the entries and never
//! move with them, so no view of the account hides that it is incomplete.
//!
//! # A refusal is drawn as plainly as what ran
//!
//! Every entry is laid out by the same code, in the same type, the same two
//! colours and the same spacing, whatever became of it. There is no dimming, no
//! collapsing and no second style for what was stopped, because a record that
//! showed its successes more plainly than its refusals could not answer what a
//! security review asks.
//!
//! # Every entry is drawn whole or not at all
//!
//! An entry that does not fit below the last one waits for the view to move;
//! it is never cut. When entries are out of view, a rail on the trailing edge
//! says where the view is — a shape, so no sentence is needed and none is
//! invented. A window that cannot hold the record's own sentences and the entry
//! at the top of the view **refuses the frame** ([`RenderError::RecordScene`]).
//!
//! # What it reads
//!
//! `crate::RecordShows`, the person's vocabulary, and `alo-appearance`'s tokens.
//! Terracotta is not among the colours: it means the agent acting, and this
//! window is a person reading.

use alo_strings::{Direction, Strings};
use cosmic_text::FontSystem;
use smithay::utils::{Physical, Rectangle};

use crate::painted::{Inked, Solid};
use crate::painted_text::Shaped;
use crate::record_lines::{EntryWords, newest_first};
use crate::record_room::{Room, framed, rule};
use crate::{RecordLook, RecordShows, RenderError, WindowControlLabels};

/// One piece of text as drawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextDrawn {
    /// The text inked, exactly as it arrived.
    pub(crate) text: String,
    /// Its box.
    pub(crate) area: Rectangle<i32, Physical>,
}

/// One entry as drawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EntryDrawn {
    /// The words it was drawn from.
    pub(crate) words: EntryWords,
    /// The clause at its head.
    pub(crate) clause: TextDrawn,
    /// Whose authority it was under, when the record names anybody.
    pub(crate) agent: Option<TextDrawn>,
    /// Everything after, in order.
    pub(crate) after: Vec<TextDrawn>,
}

/// The window for one output size, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecordPicture {
    /// The output size it was laid out for.
    pub(crate) size: (i32, i32),
    /// The panel, when anything is drawn.
    pub(crate) panel: Option<Rectangle<i32, Physical>>,
    /// The refusal, when one is drawn instead of an account.
    pub(crate) refusal: Option<TextDrawn>,
    /// What the record says about itself, above the entries.
    pub(crate) remarks: Vec<TextDrawn>,
    /// The entries in view, most recent first.
    pub(crate) entries: Vec<EntryDrawn>,
    /// Whether more recent entries are out of view above.
    pub(crate) more_recent_out_of_view: bool,
    /// Whether older entries are out of view below.
    pub(crate) older_out_of_view: bool,
    /// The rail, when anything is out of view: its track and where the view is.
    pub(crate) rail: Option<(Rectangle<i32, Physical>, Rectangle<i32, Physical>)>,
    /// Flat shapes, painted first.
    pub(crate) solids: Vec<Solid>,
    /// Text, painted after the shapes.
    pub(crate) inked: Vec<Inked>,
}

impl RecordPicture {
    /// Nothing drawn, for an output of `size`.
    fn empty(size: (i32, i32)) -> Self {
        Self {
            size,
            panel: None,
            refusal: None,
            remarks: Vec::new(),
            entries: Vec::new(),
            more_recent_out_of_view: false,
            older_out_of_view: false,
            rail: None,
            solids: Vec::new(),
            inked: Vec::new(),
        }
    }

    /// Whether nothing at all is drawn.
    pub(crate) fn is_empty(&self) -> bool {
        self.panel.is_none() && self.solids.is_empty() && self.inked.is_empty()
    }
}

/// Lay the window out for an output of `size` and rasterise it.
///
/// # Errors
/// [`RenderError::RecordScene`] when the window is open and the output is
/// larger than any this window is laid out for, too small for a panel, or
/// cannot hold the record's own sentences and the entry at the top of the view
/// whole.
pub(crate) fn picture(
    shows: RecordShows<'_>,
    strings: &Strings,
    labels: &mut WindowControlLabels,
    size: (i32, i32),
    look: RecordLook,
) -> Result<RecordPicture, RenderError> {
    let mut drawn = RecordPicture::empty(size);
    match shows {
        RecordShows::Nothing => Ok(drawn),
        RecordShows::Refusal(why) => {
            let room = Room::on(size, look)?;
            let said = why.said(strings).into_text();
            let shaped = room.shaped(&mut labels.fonts, &said, room.inner);
            let panel_height = 2 * room.pad + shaped.height;
            if panel_height > size.1 - 2 * room.margin {
                return Err(RenderError::RecordScene);
            }
            let top = (size.1 - panel_height) / 2;
            let panel = Rectangle::new((room.left, top).into(), (room.width, panel_height).into());
            framed(&mut drawn.solids, panel, room.measure.px(2), room.palette);
            drawn.panel = Some(panel);
            drawn.refusal = Some(placed(
                &mut drawn.inked,
                shaped,
                said,
                room.text_x,
                top + room.pad,
            ));
            Ok(drawn)
        }
        RecordShows::Account {
            account,
            moved_past,
        } => {
            let room = Room::on(size, look)?;
            let top = room.margin;
            let bottom = size.1 - room.margin - room.pad;
            let panel = Rectangle::new(
                (room.left, top).into(),
                (room.width, size.1 - 2 * room.margin).into(),
            );
            framed(&mut drawn.solids, panel, room.measure.px(2), room.palette);
            drawn.panel = Some(panel);

            let fonts = &mut labels.fonts;
            let mut y = top + room.pad;
            for said in account.said(strings) {
                let text = said.into_text();
                let shaped = room.shaped(fonts, &text, room.inner);
                if y + shaped.height > bottom {
                    return Err(RenderError::RecordScene);
                }
                let height = shaped.height;
                drawn
                    .remarks
                    .push(placed(&mut drawn.inked, shaped, text, room.text_x, y));
                y += height + room.gap;
            }
            y += room.gap;
            rule(&mut drawn.solids, room, y);
            y += 1 + 2 * room.gap;
            let entries_top = y;

            for words in newest_first(account, moved_past, strings) {
                let Some(entry) = entry_at(fonts, words, room, look.reading, y, bottom) else {
                    if drawn.entries.is_empty() {
                        return Err(RenderError::RecordScene);
                    }
                    break;
                };
                let (entry, height, inked) = entry;
                drawn.inked.extend(inked);
                drawn.entries.push(entry);
                y += height + room.gap;
                rule(&mut drawn.solids, room, y);
                y += 1 + room.gap;
            }

            let total = account.how_many();
            drawn.more_recent_out_of_view = moved_past > 0 && total > 0;
            drawn.older_out_of_view = moved_past + drawn.entries.len() < total;
            if drawn.more_recent_out_of_view || drawn.older_out_of_view {
                let track = Rectangle::new(
                    (room.rail_x, entries_top).into(),
                    (room.rail, (bottom - entries_top).max(room.rail)).into(),
                );
                let height = track.size.h;
                let at = |entries: usize| (height as usize * entries / total.max(1)) as i32;
                let thumb = Rectangle::new(
                    (room.rail_x, entries_top + at(moved_past)).into(),
                    (room.rail, at(drawn.entries.len()).max(room.measure.px(8))).into(),
                );
                framed(&mut drawn.solids, track, room.measure.px(1), room.palette);
                drawn.solids.push(Solid {
                    area: thumb,
                    colour: room.palette.ink,
                });
                drawn.rail = Some((track, thumb));
            }
            Ok(drawn)
        }
    }
}

/// One entry laid out from `y`, or [`None`] when it does not fit whole above
/// `bottom`.
fn entry_at(
    fonts: &mut FontSystem,
    words: EntryWords,
    room: Room,
    reading: Direction,
    y: i32,
    bottom: i32,
) -> Option<(EntryDrawn, i32, Vec<Inked>)> {
    let (after_x, after_width) = room.after_box(reading);
    let clause = room.shaped(fonts, &words.clause, room.inner);
    let agent = words
        .agent
        .as_deref()
        .map(|name| room.shaped(fonts, name, after_width));
    let after: Vec<Shaped> = words
        .after
        .iter()
        .map(|text| room.shaped(fonts, text, after_width))
        .collect();
    let height = clause.height
        + agent.as_ref().map_or(0, |shaped| shaped.height)
        + after.iter().map(|shaped| shaped.height).sum::<i32>();
    if y + height > bottom {
        return None;
    }
    let mut inked = Vec::new();
    let mut at = y;
    let clause_height = clause.height;
    let clause = placed(&mut inked, clause, words.clause.clone(), room.text_x, at);
    at += clause_height;
    let agent = match (agent, &words.agent) {
        (Some(shaped), Some(name)) => {
            let rows = shaped.height;
            let drawn = placed(&mut inked, shaped, name.clone(), after_x, at);
            at += rows;
            Some(drawn)
        }
        _ => None,
    };
    let mut after_drawn = Vec::new();
    for (shaped, text) in after.into_iter().zip(words.after.iter()) {
        let rows = shaped.height;
        after_drawn.push(placed(&mut inked, shaped, text.clone(), after_x, at));
        at += rows;
    }
    Some((
        EntryDrawn {
            words,
            clause,
            agent,
            after: after_drawn,
        },
        height,
        inked,
    ))
}

/// Text placed whole at (`x`, `y`): it was measured to fit, so nothing is cut.
fn placed(inked: &mut Vec<Inked>, shaped: Shaped, text: String, x: i32, y: i32) -> TextDrawn {
    let area = Rectangle::new((x, y).into(), (shaped.width_of_box, shaped.height).into());
    let bottom = y + shaped.height;
    inked.extend(shaped.placed(x, y, bottom));
    TextDrawn { text, area }
}

#[cfg(test)]
#[path = "record_raster_tests.rs"]
mod tests;
