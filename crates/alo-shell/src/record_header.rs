//! The top of the record window's body: the heading, the sentence under it,
//! every sentence the record says about itself, today's counts, and the rule
//! the list starts under (`560:193` to `560:210`).
//!
//! **The record's sentences about itself are drawn here and never scroll**
//! (`alo_recounting::Account::said`: whether anything answered, whether this
//! is all of it, whether it goes all the way back, whether it is what it says
//! it is). They sit between the sentence and the counts, in the ink, so no
//! view of the list hides that the record is incomplete.

use cosmic_text::FontSystem;
use smithay::utils::Rectangle;

use crate::the_records_icons::ARecordIcon;
use crate::record_measures as m;
use crate::record_palette::RecordPalette;
use crate::record_raster::TextDrawn;
use crate::record_room::Room;
use crate::settings_canvas::Canvas;
use crate::settings_type::{TheType, written};

/// One of today's counts: its icon, its colour, and what it says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ACount {
    /// Its icon.
    pub(crate) icon: ARecordIcon,
    /// The colour its icon and words are drawn in.
    pub(crate) colour: [u8; 3],
    /// What it says, the number in it.
    pub(crate) said: String,
}

/// What the header says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TheHeaderSays {
    /// The heading.
    pub(crate) heading: String,
    /// The sentence under it — or, where the record could not be shown, why.
    pub(crate) under_it: String,
    /// What the record says about itself.
    pub(crate) remarks: Vec<String>,
    /// *Today*, and today's counts, where there is an account to count.
    pub(crate) today: Option<(String, Vec<ACount>)>,
}

/// What the header drew.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TheHeaderDrawn {
    /// The sentence under the heading.
    pub(crate) under_it: TextDrawn,
    /// What the record says about itself, in order.
    pub(crate) remarks: Vec<TextDrawn>,
    /// Today's counts, in order.
    pub(crate) counts: Vec<TextDrawn>,
    /// Where the list starts: under the rule.
    pub(crate) bottom: i32,
}

/// Draw the header into the body from its top; [`None`] when it does not fit
/// above `stop`.
pub(crate) fn drawn(
    canvas: &mut Canvas,
    fonts: &mut FontSystem,
    room: Room,
    palette: RecordPalette,
    says: &TheHeaderSays,
    stop: i32,
) -> Option<TheHeaderDrawn> {
    let px = |at_one: i32| room.px(at_one);
    let x = room.body.loc.x + px(m::PAD);
    let inner = room.body.size.w - 2 * px(m::PAD);
    let mut y = room.body.loc.y + px(m::HEADING_AT);
    let heading = written(
        fonts,
        &says.heading,
        TheType::PageHeading,
        inner,
        room.measure,
        (palette.surface, palette.ink),
    );
    canvas.words(x, y, &heading);
    y += heading.height + px(m::AFTER_THE_HEADING);
    let under_it = written(
        fonts,
        &says.under_it,
        TheType::Body,
        inner.min(px(m::THE_INTRODUCTION_IS)),
        room.measure,
        (palette.surface, palette.secondary),
    );
    let under_it_drawn = placed(canvas, room, &under_it, &says.under_it, x, y);
    y += under_it.height;
    let mut remarks = Vec::new();
    for remark in &says.remarks {
        y += px(m::BETWEEN_LINES);
        let shaped = written(
            fonts,
            remark,
            TheType::Body,
            inner,
            room.measure,
            (palette.surface, palette.ink),
        );
        remarks.push(placed(canvas, room, &shaped, remark, x, y));
        y += shaped.height;
    }
    let mut counts = Vec::new();
    if let Some((today, each)) = &says.today {
        y += px(m::BEFORE_THE_COUNTS);
        y = the_counts(canvas, fonts, room, palette, (today, each), (x, y), &mut counts);
    }
    y += px(m::BEFORE_THE_RULE);
    canvas.fill(
        Rectangle::new(
            (room.body.loc.x, y).into(),
            (room.body.size.w, room.hairline).into(),
        ),
        0.0,
        palette.edge,
        1.0,
    );
    let bottom = y + room.hairline;
    (bottom < stop).then_some(TheHeaderDrawn {
        under_it: under_it_drawn,
        remarks,
        counts,
        bottom,
    })
}

/// *Today* and its counts, each a pill: a row of them that goes on to a second
/// row where the first is full. Answers the foot of the last row.
fn the_counts(
    canvas: &mut Canvas,
    fonts: &mut FontSystem,
    room: Room,
    palette: RecordPalette,
    (today, each): (&str, &[ACount]),
    (x, y): (i32, i32),
    drawn: &mut Vec<TextDrawn>,
) -> i32 {
    let px = |at_one: i32| room.px(at_one);
    let end = room.body.loc.x + room.body.size.w - px(m::PAD);
    let tall = px(m::A_COUNT_IS);
    let label = written(
        fonts,
        today,
        TheType::Label,
        end - x,
        room.measure,
        (palette.surface, palette.muted),
    )
    .cropped_to_its_ink();
    canvas.words(x, y + (tall - label.height) / 2, &label);
    let (icon_at, words_at, after) = m::IN_A_COUNT;
    let icon = px(14);
    let mut at = x + label.width_of_box + px(m::BETWEEN_COUNTS);
    let mut row = y;
    for count in each {
        let shaped = written(
            fonts,
            &count.said,
            TheType::SmallLabel,
            end - x,
            room.measure,
            (palette.cool, count.colour),
        )
        .cropped_to_its_ink();
        let wide = px(words_at) + shaped.width_of_box + px(after);
        if at + wide > end && at > x {
            row += tall + px(m::BETWEEN_COUNTS);
            at = x;
        }
        let pill = Rectangle::new((at, row).into(), (wide, tall).into());
        canvas.fill(pill, tall as f32 / 2.0, palette.cool, 1.0);
        count.icon.drawn(
            canvas,
            Rectangle::new(
                (at + px(icon_at), row + (tall - icon) / 2).into(),
                (icon, icon).into(),
            ),
            count.colour,
        );
        drawn.push(placed(
            canvas,
            room,
            &shaped,
            &count.said,
            at + px(words_at),
            row + (tall - shaped.height) / 2,
        ));
        at += wide + px(m::BETWEEN_COUNTS);
    }
    row + tall
}

/// Text drawn at (`x`, `y`), and where it is for the person's direction.
pub(crate) fn placed(
    canvas: &mut Canvas,
    room: Room,
    shaped: &crate::painted_text::Shaped,
    text: &str,
    x: i32,
    y: i32,
) -> TextDrawn {
    canvas.words(x, y, shaped);
    TextDrawn {
        text: text.to_owned(),
        area: room.as_drawn(Rectangle::new(
            (x, y).into(),
            (shaped.width_of_box, shaped.height).into(),
        )),
    }
}
