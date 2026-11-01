//! *Put back…* at the end of a change, and the confirmation that opens under
//! it, as `560:242` and `560:763` draw them — shaped first, so the entry they
//! are in is measured whole, and drawn after.
//!
//! The confirmation asks the question, names every folder and where what is
//! in it now will be kept, says that putting back is written in the list
//! too, and offers *Put back* and *Cancel* — or, where nothing can be put
//! back, says why and offers *Cancel* alone. Where the keyboard is, a ring
//! two off the button says so.

use cosmic_text::FontSystem;
use smithay::utils::Rectangle;

use crate::painted_text::Shaped;
use crate::putting_back_from_the_record::OnTheConfirmation;
use crate::record_header::placed;
use crate::record_lines::TheConfirmationWords;
use crate::record_measures as m;
use crate::record_palette::RecordPalette;
use crate::record_raster::TextDrawn;
use crate::record_room::Room;
use crate::settings_canvas::{Area, Canvas};
use crate::settings_type::{TheType, written};
use crate::the_records_icons::ARecordIcon;

/// *Put back…*, shaped.
pub(crate) struct AButtonShaped {
    /// Its words.
    label: Shaped,
    /// Whether the keyboard is on it.
    chosen: bool,
    /// How wide it is.
    pub(crate) wide: i32,
}

/// Shape *Put back…*.
pub(crate) fn the_button(
    fonts: &mut FontSystem,
    room: Room,
    palette: RecordPalette,
    (text, chosen): (&str, bool),
) -> AButtonShaped {
    let px = |at_one: i32| room.px(at_one);
    let (_, _, before, icon, gap, after) = m::THE_BUTTON;
    let label = written(
        fonts,
        text,
        TheType::Label,
        px(240),
        room.measure,
        (palette.surface, palette.ink),
    )
    .cropped_to_its_ink();
    let wide = px(before) + px(icon) + px(gap) + label.width_of_box + px(after);
    AButtonShaped {
        label,
        chosen,
        wide,
    }
}

impl AButtonShaped {
    /// Draw it with its right edge at `right` and its top at `top`; where it
    /// is drawn.
    pub(crate) fn drawn(
        &self,
        canvas: &mut Canvas,
        room: Room,
        palette: RecordPalette,
        (right, top): (i32, i32),
    ) -> Area {
        let px = |at_one: i32| room.px(at_one);
        let (tall, corner, before, icon, gap, _) = m::THE_BUTTON;
        let area = Rectangle::new(
            (right - self.wide, top).into(),
            (self.wide, px(tall)).into(),
        );
        a_button(
            canvas,
            room,
            (area, px(corner)),
            (palette.surface, palette.edge),
        );
        ARecordIcon::PutBack.drawn(
            canvas,
            Rectangle::new(
                (area.loc.x + px(before), top + (px(tall) - px(icon)) / 2).into(),
                (px(icon), px(icon)).into(),
            ),
            palette.ink,
        );
        canvas.words(
            area.loc.x + px(before) + px(icon) + px(gap),
            top + (px(tall) - self.label.height) / 2,
            &self.label,
        );
        if self.chosen {
            the_ring(canvas, room, (area, px(corner)), palette.ink);
        }
        room.as_drawn(area)
    }
}

/// The confirmation, shaped.
pub(crate) struct TheConfirmationShaped {
    /// The words it was shaped from.
    words: TheConfirmationWords,
    /// The question.
    question: Option<Shaped>,
    /// A line per folder, and that it is written down.
    lines: Vec<Shaped>,
    /// Why nothing will be, or was, put back.
    refusal: Option<Shaped>,
    /// *Put back*, where it can be pressed.
    put_back: Option<Shaped>,
    /// *Cancel*.
    cancel: Shaped,
    /// *Putting back…*.
    waiting: Option<Shaped>,
    /// How tall it is.
    pub(crate) height: i32,
}

/// What the confirmation drew that a press needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TheConfirmationDrawn {
    /// The whole box.
    pub(crate) area: Area,
    /// The question.
    pub(crate) question: Option<TextDrawn>,
    /// Every line under it.
    pub(crate) lines: Vec<TextDrawn>,
    /// Why nothing will be, or was, put back.
    pub(crate) refusal: Option<TextDrawn>,
    /// *Put back*, where it is drawn.
    pub(crate) put_back: Option<Area>,
    /// *Cancel*, where it is drawn — none while it is being done.
    pub(crate) cancel: Option<Area>,
}

/// Shape the confirmation `wide` across.
pub(crate) fn the_confirmation(
    fonts: &mut FontSystem,
    room: Room,
    palette: RecordPalette,
    words: TheConfirmationWords,
    wide: i32,
) -> TheConfirmationShaped {
    let px = |at_one: i32| room.px(at_one);
    let (_, above, side, below, between) = m::THE_CONFIRMATION;
    let (button_tall, _, _, _, buttons_above) = m::ITS_BUTTONS;
    let inside = (wide - 2 * px(side)).max(px(80));
    let ground = palette.canvas;
    let mut write = |text: &str, style: TheType, colour: [u8; 3]| {
        written(fonts, text, style, inside, room.measure, (ground, colour))
    };
    let question = words
        .question
        .as_deref()
        .map(|text| write(text, TheType::Name, palette.ink));
    let lines: Vec<Shaped> = words
        .lines
        .iter()
        .map(|text| write(text, TheType::Body, palette.secondary))
        .collect();
    let refusal = words
        .refusal
        .as_deref()
        .map(|text| write(text, TheType::Body, palette.refused));
    let waiting = words
        .waiting
        .as_deref()
        .map(|text| write(text, TheType::Label, palette.muted).cropped_to_its_ink());
    let put_back = words.put_back.as_deref().map(|text| {
        written(
            fonts,
            text,
            TheType::Label,
            px(240),
            room.measure,
            (palette.ink, palette.surface),
        )
        .cropped_to_its_ink()
    });
    let cancel = written(
        fonts,
        &words.cancel,
        TheType::Label,
        px(240),
        room.measure,
        (palette.surface, palette.ink),
    )
    .cropped_to_its_ink();
    let texts: Vec<i32> = question
        .iter()
        .chain(lines.iter())
        .chain(refusal.iter())
        .map(|shaped| shaped.height)
        .collect();
    let between_them = px(between) * i32::try_from(texts.len().saturating_sub(1)).unwrap_or(0);
    let row = waiting
        .as_ref()
        .map_or(px(button_tall), |shaped| shaped.height.max(px(button_tall)));
    let height = px(above)
        + texts.iter().sum::<i32>()
        + between_them
        + px(between)
        + px(buttons_above)
        + row
        + px(below);
    TheConfirmationShaped {
        words,
        question,
        lines,
        refusal,
        put_back,
        cancel,
        waiting,
        height,
    }
}

impl TheConfirmationShaped {
    /// Draw it at (`x`, `y`), `wide` across.
    pub(crate) fn drawn(
        &self,
        canvas: &mut Canvas,
        room: Room,
        palette: RecordPalette,
        (x, y, wide): (i32, i32, i32),
    ) -> TheConfirmationDrawn {
        let px = |at_one: i32| room.px(at_one);
        let (corner, above, side, _, between) = m::THE_CONFIRMATION;
        let (button_tall, button_pad, button_corner, buttons_between, buttons_above) =
            m::ITS_BUTTONS;
        let area = Rectangle::new((x, y).into(), (wide, self.height).into());
        a_button(
            canvas,
            room,
            (area, px(corner)),
            (palette.canvas, palette.edge),
        );
        let left = x + px(side);
        let mut at = y + px(above);
        let mut texts = Vec::new();
        let question = self
            .question
            .as_ref()
            .zip(self.words.question.as_deref())
            .map(|(shaped, text)| {
                let drawn = placed(canvas, room, shaped, text, left, at);
                at += shaped.height + px(between);
                drawn
            });
        for (shaped, text) in self.lines.iter().zip(&self.words.lines) {
            texts.push(placed(canvas, room, shaped, text, left, at));
            at += shaped.height + px(between);
        }
        let refusal = self
            .refusal
            .as_ref()
            .zip(self.words.refusal.as_deref())
            .map(|(shaped, text)| {
                let drawn = placed(canvas, room, shaped, text, left, at);
                at += shaped.height + px(between);
                drawn
            });
        let row = at + px(buttons_above);
        if let Some(waiting) = &self.waiting {
            canvas.words(left, row + (px(button_tall) - waiting.height) / 2, waiting);
            return TheConfirmationDrawn {
                area: room.as_drawn(area),
                question,
                lines: texts,
                refusal,
                put_back: None,
                cancel: None,
            };
        }
        let mut button_x = left;
        let mut a_labelled =
            |canvas: &mut Canvas, label: &Shaped, (fill, edge): ([u8; 3], [u8; 3]), on: bool| {
                let wide = label.width_of_box + 2 * px(button_pad);
                let it = Rectangle::new((button_x, row).into(), (wide, px(button_tall)).into());
                a_button(canvas, room, (it, px(button_corner)), (fill, edge));
                canvas.words(
                    button_x + px(button_pad),
                    row + (px(button_tall) - label.height) / 2,
                    label,
                );
                if on {
                    the_ring(canvas, room, (it, px(button_corner)), palette.ink);
                }
                button_x += wide + px(buttons_between);
                room.as_drawn(it)
            };
        let put_back = self.put_back.as_ref().map(|label| {
            a_labelled(
                canvas,
                label,
                (palette.ink, palette.ink),
                self.words.on == OnTheConfirmation::PutBack,
            )
        });
        let cancel = a_labelled(
            canvas,
            &self.cancel,
            (palette.surface, palette.edge),
            self.words.on == OnTheConfirmation::Cancel,
        );
        TheConfirmationDrawn {
            area: room.as_drawn(area),
            question,
            lines: texts,
            refusal,
            put_back,
            cancel: Some(cancel),
        }
    }
}

/// A rounded box in `fill` with a hairline of `edge`.
fn a_button(
    canvas: &mut Canvas,
    room: Room,
    (area, corner): (Area, i32),
    (fill, edge): ([u8; 3], [u8; 3]),
) {
    canvas.fill(area, corner as f32, fill, 1.0);
    canvas.border(area, corner as f32, room.hairline as f32, edge);
}

/// Where the keyboard is: a ring round `area`, two off it.
fn the_ring(canvas: &mut Canvas, room: Room, (area, corner): (Area, i32), ink: [u8; 3]) {
    let (thick, off) = m::THE_KEYBOARDS_RING;
    let reach = room.px(thick) + room.px(off);
    let ring = Rectangle::new(
        (area.loc.x - reach, area.loc.y - reach).into(),
        (area.size.w + 2 * reach, area.size.h + 2 * reach).into(),
    );
    canvas.border(ring, (corner + reach) as f32, room.px(thick) as f32, ink);
}

#[cfg(test)]
#[path = "record_put_back_drawn_tests.rs"]
mod tests;
