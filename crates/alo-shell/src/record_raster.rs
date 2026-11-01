//! The record window laid out and rasterised for one output, as the owner
//! approved it on 2026-11-01 (`560:142`, `560:316`, `560:490`).
//!
//! # What is drawn
//!
//! While the window is closed, **nothing**. While it is open, one window in one
//! raster (`crate::record_frame_drawn`): its title strip, *What alo did* and
//! the sentence under it, everything the record says about itself, today's
//! counts (`crate::record_header`), and under a rule **the list** — newest
//! first, each day under its own heading in a card of its own, each entry as
//! `crate::record_entry_drawn` draws it — and the footer. While a refusal is
//! open, the heading and the refuser's sentence, and no list.
//!
//! # Every entry is drawn whole or not at all
//!
//! An entry that does not fit above the footer waits for the view to move; it
//! is never cut, where the design's frame shows its last card running under
//! the footer — the window's choice since 2026-10-17, kept because a view that
//! moves by whole entries cannot hide half of one. When entries are out of
//! view, a rail on the trailing edge says where the view is. A window that
//! cannot hold its header and the entry at the top of the view **refuses the
//! frame** ([`RenderError::RecordScene`]).
//!
//! # What it reads
//!
//! `crate::RecordShows`, the person's vocabulary, the person's formats and
//! `crate::record_palette`'s colours. Every clause, remark and refusal is the
//! record's own or `alo-recounting`'s; the window's own words are
//! `crate::the_records_words`.

use alo_strings::{Counting, Direction, Filling, Strings};
use cosmic_text::FontSystem;
use smithay::utils::Rectangle;

use crate::display_scale::DisplayScale;
use crate::painted::Washed;
use crate::record_clock::RecordClock;
use crate::record_entry_drawn::{TheColumns, TheWindowsWords, shaped, the_time};
use crate::record_header::{ACount, TheHeaderSays, placed};
use crate::the_records_icons::ARecordIcon;
use crate::record_kinds::{EntryKind, Today};
use crate::record_lines::{EntryWords, newest_first};
use crate::record_measures as m;
use crate::record_palette::RecordPalette;
use crate::record_room::Room;
use crate::settings_canvas::{Area, Canvas};
use crate::settings_type::{TheType, written};
use crate::the_records_words as w;
use crate::{RecordLook, RecordShows, RenderError, WindowControlLabels};

/// One piece of text as drawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextDrawn {
    /// The text inked, exactly as it arrived.
    pub(crate) text: String,
    /// Its box, where it is drawn.
    pub(crate) area: Area,
}

/// One entry as drawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EntryDrawn {
    /// The words it was drawn from.
    pub(crate) words: EntryWords,
    /// Which of the design's kinds it was drawn as.
    pub(crate) kind: EntryKind,
    /// Its row in its card.
    pub(crate) area: Area,
    /// Its icon's tile.
    pub(crate) icon: Area,
    /// The day above it, when it is the first of its day in view.
    pub(crate) day: Option<TextDrawn>,
    /// The date beside that day.
    pub(crate) date: Option<TextDrawn>,
    /// The time of day, at its head.
    pub(crate) time: Option<TextDrawn>,
    /// Whose authority it was under, when the record names anybody.
    pub(crate) agent: Option<TextDrawn>,
    /// The clause at its head.
    pub(crate) clause: TextDrawn,
    /// Everything after, in order.
    pub(crate) after: Vec<TextDrawn>,
    /// That the person said yes.
    pub(crate) approved: Option<TextDrawn>,
    /// *Refused*.
    pub(crate) refused: Option<TextDrawn>,
    /// *Already sent*.
    pub(crate) already_sent: Option<TextDrawn>,
}

/// The window for one output size, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecordPicture {
    /// The output size it was laid out for.
    pub(crate) size: (i32, i32),
    /// The window, when anything is drawn.
    pub(crate) panel: Option<Area>,
    /// Close's target, when the window is drawn.
    pub(crate) close: Option<Area>,
    /// The window and everything in it, as one raster.
    pub(crate) washed: Option<Washed>,
    /// The refusal, when one is drawn instead of an account.
    pub(crate) refusal: Option<TextDrawn>,
    /// What the record says about itself, above the list.
    pub(crate) remarks: Vec<TextDrawn>,
    /// Today's counts.
    pub(crate) counts: Vec<TextDrawn>,
    /// The entries in view, most recent first.
    pub(crate) entries: Vec<EntryDrawn>,
    /// Each day's card in view, in order.
    pub(crate) cards: Vec<Area>,
    /// Whether more recent entries are out of view above.
    pub(crate) more_recent_out_of_view: bool,
    /// Whether older entries are out of view below.
    pub(crate) older_out_of_view: bool,
    /// The rail, when anything is out of view: its track and where the view is.
    pub(crate) rail: Option<(Area, Area)>,
}

impl RecordPicture {
    /// Nothing drawn, for an output of `size`.
    fn empty(size: (i32, i32)) -> Self {
        Self {
            size,
            panel: None,
            close: None,
            washed: None,
            refusal: None,
            remarks: Vec::new(),
            counts: Vec::new(),
            entries: Vec::new(),
            cards: Vec::new(),
            more_recent_out_of_view: false,
            older_out_of_view: false,
            rail: None,
        }
    }

    /// Whether nothing at all is drawn.
    pub(crate) const fn is_empty(&self) -> bool {
        self.washed.is_none()
    }
}

/// One of the window's own words, in the person's language.
fn said(strings: &Strings, word: alo_strings::Word) -> String {
    strings.say(&word.key(), &Filling::nothing()).into_text()
}

/// Today's counts, in the order the design draws them.
fn todays_counts(strings: &Strings, today: Today, palette: RecordPalette) -> Vec<ACount> {
    let count = |counted: w::Counted, how_many: u64| {
        strings
            .count(&counted.key(), &Counting::of(how_many), &Filling::nothing())
            .into_text()
    };
    vec![
        ACount {
            icon: ARecordIcon::Allowed,
            colour: palette.allowed,
            said: count(w::CHANGES_YOU_ALLOWED, today.allowed),
        },
        ACount {
            icon: ARecordIcon::Left,
            colour: palette.secondary,
            said: count(w::REQUESTS_THAT_LEFT, today.left),
        },
        ACount {
            icon: ARecordIcon::RefusedSmall,
            colour: palette.refused,
            said: count(w::REFUSED_TODAY, today.refused),
        },
    ]
}

/// Lay the window out for an output of `size` drawn at `display`, and
/// rasterise it.
///
/// # Errors
/// [`RenderError::RecordScene`] when the window is open and the output is
/// larger than any this window is laid out for, too small for it, or cannot
/// hold its header and the entry at the top of the view whole.
pub(crate) fn picture(
    shows: RecordShows<'_>,
    strings: &Strings,
    labels: &mut WindowControlLabels,
    size: (i32, i32),
    look: RecordLook,
    clock: RecordClock<'_>,
    display: DisplayScale,
) -> Result<RecordPicture, RenderError> {
    let mut drawn = RecordPicture::empty(size);
    let palette = RecordPalette::of(look.scheme, look.contrast);
    let (says, account) = match shows {
        RecordShows::Nothing => return Ok(drawn),
        RecordShows::Refusal(why) => (
            TheHeaderSays {
                heading: said(strings, w::WHAT_ALO_DID),
                under_it: why.said(strings).into_text(),
                remarks: Vec::new(),
                today: None,
            },
            None,
        ),
        RecordShows::Account {
            account,
            moved_past,
        } => (
                TheHeaderSays {
                    heading: said(strings, w::WHAT_ALO_DID),
                    under_it: said(strings, w::WHAT_IS_IN_IT),
                    remarks: account
                        .said(strings)
                        .into_iter()
                        .map(alo_strings::Said::into_text)
                        .collect(),
                    today: Some((
                        strings
                            .say(&alo_formats::words::TODAY.key(), &Filling::nothing())
                            .into_text(),
                        todays_counts(strings, Today::in_the(account, clock), palette),
                    )),
                },
                Some((account, moved_past)),
            ),
    };
    let room = Room::on(size, look, display)?;
    let mut canvas = Canvas::over(room.canvas(size));
    if room.reading == Direction::RightToLeft {
        canvas.turned_round(room.window);
    }
    let fonts = &mut labels.fonts;
    let frame = crate::record_frame_drawn::drawn(
        &mut canvas,
        fonts,
        room,
        palette,
        (&says.heading, &said(strings, w::KEPT_ON_THIS_COMPUTER)),
    );
    let header =
        crate::record_header::drawn(&mut canvas, fonts, room, palette, &says, frame.footer_top)
            .ok_or(RenderError::RecordScene)?;
    drawn.panel = Some(room.window);
    drawn.close = Some(frame.close);
    drawn.remarks = header.remarks;
    drawn.counts = header.counts;
    match account {
        None => drawn.refusal = Some(header.under_it),
        Some((account, moved_past)) => {
            let (top_pad, foot_pad) = m::THE_LIST_PADS;
            let list = (header.bottom + room.px(top_pad), frame.footer_top - room.px(foot_pad));
            let (refused, already_sent) = (said(strings, w::REFUSED), said(strings, w::ALREADY_SENT));
            let words = TheWindowsWords {
                refused: &refused,
                already_sent: &already_sent,
            };
            the_list(
                &mut canvas,
                fonts,
                (room, palette),
                newest_first(account, moved_past, strings, clock),
                words,
                list,
                &mut drawn,
            )?;
            let total = account.how_many();
            drawn.more_recent_out_of_view = moved_past > 0 && total > 0;
            drawn.older_out_of_view = moved_past + drawn.entries.len() < total;
            if drawn.more_recent_out_of_view || drawn.older_out_of_view {
                drawn.rail = Some(the_rail(
                    &mut canvas,
                    (room, palette),
                    list,
                    (moved_past, drawn.entries.len(), total),
                ));
            }
        }
    }
    drawn.washed = Some(canvas.washed());
    Ok(drawn)
}

/// The list between `list.0` and `list.1`: each day's heading and card, and
/// as many entries as fit whole.
fn the_list(
    canvas: &mut Canvas,
    fonts: &mut FontSystem,
    (room, palette): (Room, RecordPalette),
    entries: impl Iterator<Item = EntryWords>,
    words: TheWindowsWords<'_>,
    (top, bottom): (i32, i32),
    drawn: &mut RecordPicture,
) -> Result<(), RenderError> {
    let px = |at_one: i32| room.px(at_one);
    let most = usize::try_from((bottom - top) / px(48)).unwrap_or(0) + 1;
    let in_view: Vec<EntryWords> = entries.take(most).collect();
    // **One time column for every entry in view**, measured before any is
    // placed, so every icon and clause starts at one edge whatever the
    // minutes are — the design's 44 at the least.
    let widest = in_view
        .iter()
        .filter_map(|words| words.time.as_deref())
        .map(|time| the_time(fonts, room, palette, time).width_of_box)
        .max()
        .unwrap_or(0);
    let columns = TheColumns {
        left: room.body.loc.x + px(m::PAD),
        wide: room.body.size.w - 2 * px(m::PAD),
        time: px(m::THE_TIME_COLUMN).max(widest),
    };
    let (day_tall, day_in, day_gap) = m::A_DAY;
    let mut y = top;
    let mut card_top: Option<i32> = None;
    for entry_words in in_view {
        let day = entry_words.day.clone();
        let date = entry_words.date.clone();
        let starts_a_card = day.is_some() || card_top.is_none();
        let heading = if day.is_some() {
            px(day_tall) + px(m::DAY_TO_CARD)
        } else {
            0
        };
        let before = match (starts_a_card, card_top) {
            (true, Some(_)) => px(m::BETWEEN_DAYS),
            (false, _) => room.hairline,
            (true, None) => 0,
        };
        let entry = shaped(fonts, room, palette, columns, (entry_words, words));
        if y + before + heading + entry.height > bottom {
            if drawn.entries.is_empty() {
                return Err(RenderError::RecordScene);
            }
            break;
        }
        if starts_a_card {
            if let Some(from) = card_top.take() {
                a_card(canvas, (room, palette), columns, (from, y), drawn);
            }
            y += before;
        }
        let mut day_drawn = None;
        let mut date_drawn = None;
        if let Some(day) = day {
            let x = columns.left + px(day_in);
            let shaped_day = written(
                fonts,
                &day,
                TheType::Label,
                columns.wide,
                room.measure,
                (palette.surface, palette.ink),
            )
            .cropped_to_its_ink();
            let after_day = x + shaped_day.width_of_box + px(day_gap);
            day_drawn = Some(placed(canvas, room, &shaped_day, &day, x, y));
            if let Some(date) = date {
                let shaped_date = written(
                    fonts,
                    &date,
                    TheType::Label,
                    columns.wide,
                    room.measure,
                    (palette.surface, palette.muted),
                )
                .cropped_to_its_ink();
                date_drawn = Some(placed(canvas, room, &shaped_date, &date, after_day, y));
            }
            y += heading;
        }
        if starts_a_card {
            card_top = Some(y);
        } else {
            canvas.fill(
                Rectangle::new(
                    (columns.left, y).into(),
                    (columns.wide, room.hairline).into(),
                ),
                0.0,
                palette.edge,
                1.0,
            );
            y += room.hairline;
        }
        let height = entry.height;
        let mut entry_drawn = entry.drawn(canvas, room, palette, columns, y);
        entry_drawn.day = day_drawn;
        entry_drawn.date = date_drawn;
        drawn.entries.push(entry_drawn);
        y += height;
    }
    if let Some(from) = card_top {
        a_card(canvas, (room, palette), columns, (from, y), drawn);
    }
    Ok(())
}

/// A day's card round `from` to `to`: its edge, which is all of it the ground
/// does not already draw.
fn a_card(
    canvas: &mut Canvas,
    (room, palette): (Room, RecordPalette),
    columns: TheColumns,
    (from, to): (i32, i32),
    drawn: &mut RecordPicture,
) {
    let card = Rectangle::new(
        (columns.left, from).into(),
        (columns.wide, to - from).into(),
    );
    canvas.border(
        card,
        room.px(m::A_CARDS_CORNER) as f32,
        room.hairline as f32,
        palette.edge,
    );
    drawn.cards.push(room.as_drawn(card));
}

/// The rail on the trailing edge of the list, and where the view is on it.
fn the_rail(
    canvas: &mut Canvas,
    (room, palette): (Room, RecordPalette),
    (top, bottom): (i32, i32),
    (moved_past, in_view, total): (usize, usize, usize),
) -> (Area, Area) {
    let (wide, from_the_edge) = m::THE_RAIL;
    let x = room.body.loc.x + room.body.size.w - room.px(from_the_edge) - room.px(wide);
    let tall = (bottom - top).max(room.px(wide));
    let track = Rectangle::new((x, top).into(), (room.px(wide), tall).into());
    let at = |entries: usize| {
        i32::try_from(usize::try_from(tall).unwrap_or(0) * entries / total.max(1)).unwrap_or(0)
    };
    let thumb = Rectangle::new(
        (x, top + at(moved_past)).into(),
        (room.px(wide), at(in_view).max(room.px(8))).into(),
    );
    let round = room.px(wide) as f32 / 2.0;
    canvas.fill(track, round, palette.edge, 1.0);
    canvas.fill(thumb, round, palette.muted, 1.0);
    (room.as_drawn(track), room.as_drawn(thumb))
}

#[cfg(test)]
#[path = "record_raster_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "record_times_tests.rs"]
mod times_tests;
