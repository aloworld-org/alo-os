//! One page of Settings laid out in a column: every shape and every piece of
//! text, placed from the top of the page, ready to be scrolled and drawn.
//!
//! The figures are `crate::settings_measures`, read off the redesign's frames;
//! the shapes are what `docs/design/the-settings-window.md` records each row
//! kind as — a ring for a choice, a pill for a switch, a box for what a choice
//! risks. **Laid out before anything is drawn**, because the page scrolls: which
//! part of it is in view depends on where the keyboard is, and that is known
//! only once every row's height is.

use cosmic_text::FontSystem;
use smithay::utils::Rectangle;

use crate::desktop_look::Measure;
use crate::painted_text::Shaped;
use crate::settings_canvas::{Area, Canvas};
use crate::settings_measures as m;
use crate::settings_page::{ABlock, APage, ARisk, ARow, ASection, SwitchIs, TheKind};
use crate::settings_palette::SettingsPalette;
use crate::settings_type::{TheType, written};
use crate::settings_window::SettingsRow;

/// One thing to draw, placed from the top of the page.
#[derive(Debug, Clone)]
pub(crate) enum Drawn {
    /// A rounded rectangle, filled.
    Fill(Area, u32, [u8; 3]),
    /// A rounded rectangle's border, one hairline.
    Border(Area, u32, [u8; 3]),
    /// A circle, filled: its centre's corner box and its colour.
    Disc(Area, [u8; 3]),
    /// A ring, by its box, its stroke in design units times ten, and colour.
    Ring(Area, u32, [u8; 3]),
    /// Words already inked on their ground, at a point.
    Words(i32, i32, Shaped),
}

/// Everything a person can act on, where it is on the page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ATargetDrawn {
    /// What it acts on.
    pub(crate) row: SettingsRow,
    /// Where it is, from the top of the page.
    pub(crate) area: Area,
    /// Where the section it is in begins, so a scroll can keep its heading in
    /// view with it.
    pub(crate) section_starts: i32,
}

/// One piece of text, as drawn: for a test to read back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SettingsText {
    /// The words.
    pub(crate) text: String,
    /// Where they are, from the top of the page.
    pub(crate) area: Area,
}

/// A page laid out.
#[derive(Debug, Clone)]
pub(crate) struct ThePageDrawn {
    /// Every shape and every piece of text, in the order painted.
    pub(crate) drawn: Vec<Drawn>,
    /// Every target, in the keyboard's order.
    pub(crate) targets: Vec<ATargetDrawn>,
    /// Every piece of text.
    pub(crate) texts: Vec<SettingsText>,
    /// How tall the page is, its padding below included.
    pub(crate) tall: i32,
}

/// What laying out needs at hand.
pub(crate) struct Laying<'a> {
    /// The fonts.
    pub(crate) fonts: &'a mut FontSystem,
    /// The person's text size on this display.
    pub(crate) measure: Measure,
    /// The colours.
    pub(crate) palette: SettingsPalette,
    /// Where the column begins, across, in pixels.
    pub(crate) left: i32,
    /// How wide it is, in pixels.
    pub(crate) wide: i32,
    /// The padding above the page, in pixels; the same is left below it.
    pub(crate) above: i32,
}

impl Laying<'_> {
    /// A logical measure in pixels.
    fn px(&self, logical: u32) -> i32 {
        self.measure.px(i32::try_from(logical).unwrap_or(i32::MAX))
    }

    /// Words shaped in `style`, `width` pixels at most, on `ground` in `ink`.
    fn words(&mut self, words: &str, style: TheType, width: i32, colours: ([u8; 3], [u8; 3])) -> Shaped {
        written(self.fonts, words, style, width, self.measure, colours)
    }
}

/// Lay `page` out.
pub(crate) fn laid_out(page: &APage, laying: &mut Laying<'_>) -> ThePageDrawn {
    let mut out = ThePageDrawn {
        drawn: Vec::new(),
        targets: Vec::new(),
        texts: Vec::new(),
        tall: 0,
    };
    let palette = laying.palette;
    let (left, wide) = (laying.left, laying.wide);
    let mut y = laying.above;

    let heading = laying.words(
        &page.heading,
        TheType::PageHeading,
        wide,
        (palette.surface, palette.ink),
    );
    y = text(&mut out, left, y, heading, &page.heading);
    y += laying.px(m::UNDER_THE_PAGE_HEADING);
    let introduction = laying.words(
        &page.introduction,
        TheType::Introduction,
        wide,
        (palette.surface, palette.secondary),
    );
    y = text(&mut out, left, y, introduction, &page.introduction);
    let between = laying.px(m::between_blocks_on(page.category));

    for block in &page.blocks {
        y += between;
        y = match block {
            ABlock::Said(said) => a_box_that_says(&mut out, laying, y, said),
            ABlock::Section(section) => a_section(&mut out, laying, y, section),
        };
    }
    out.tall = y + laying.above;
    out
}

/// Place shaped words at (`x`, `y`) and remember them; answer the bottom.
fn text(out: &mut ThePageDrawn, x: i32, y: i32, shaped: Shaped, words: &str) -> i32 {
    let tall = shaped.height;
    out.texts.push(SettingsText {
        text: words.to_owned(),
        area: Rectangle::new((x, y).into(), (shaped.width, tall).into()),
    });
    out.drawn.push(Drawn::Words(x, y, shaped));
    y + tall
}

/// A sentence the page has to say, in a box of its own: `550:807`, `bg/cool`,
/// 16 across and 14 down inside, at `radius/md`.
fn a_box_that_says(out: &mut ThePageDrawn, laying: &mut Laying<'_>, y: i32, said: &str) -> i32 {
    let palette = laying.palette;
    let (across, down) = (laying.px(16), laying.px(14));
    let shaped = laying.words(
        said,
        TheType::Body,
        laying.wide - 2 * across,
        (palette.cool, palette.ink),
    );
    let tall = shaped.height + 2 * down;
    let area = Rectangle::new((laying.left, y).into(), (laying.wide, tall).into());
    out.drawn.push(Drawn::Fill(area, m::A_CARDS_CORNER, palette.cool));
    text(out, laying.left + across, y + down, shaped, said);
    y + tall
}

/// A section: its heading, its card of rows, and its footnote.
fn a_section(out: &mut ThePageDrawn, laying: &mut Laying<'_>, top: i32, section: &ASection) -> i32 {
    let palette = laying.palette;
    let (left, wide) = (laying.left, laying.wide);
    let beside = laying.px(m::BESIDE_A_SECTIONS_WORDS);
    let gap = laying.px(m::INSIDE_A_SECTION);
    let mut y = top;
    if let Some(heading) = &section.heading {
        let shaped = laying.words(
            heading,
            TheType::Label,
            wide - 2 * beside,
            (palette.surface, palette.secondary),
        );
        y = text(out, left + beside, y, shaped, heading) + gap;
    }
    let line = laying.px(m::A_HAIRLINE);
    let card_top = y;
    // **The card is drawn after its rows are laid out**, because how tall it
    // is depends on them; its fill goes in first so the rows paint over it.
    let fill_at = out.drawn.len();
    y += line;
    for (which, row) in section.rows.iter().enumerate() {
        if which > 0 {
            out.drawn.push(Drawn::Fill(
                Rectangle::new((left + line, y).into(), (wide - 2 * line, line).into()),
                0,
                palette.edge,
            ));
            y += line;
        }
        y = a_row(out, laying, y, row, top);
    }
    y += line;
    let card = Rectangle::new((left, card_top).into(), (wide, y - card_top).into());
    out.drawn
        .insert(fill_at, Drawn::Fill(card, m::A_CARDS_CORNER, palette.surface));
    out.drawn
        .push(Drawn::Border(card, m::A_CARDS_CORNER, palette.edge));
    if let Some(footnote) = &section.footnote {
        let shaped = laying.words(
            footnote,
            TheType::Body,
            wide - 2 * beside,
            (palette.surface, palette.muted),
        );
        y = text(out, left + beside, y + gap, shaped, footnote);
    }
    y
}

/// One row of a card, from its top; answer its bottom.
fn a_row(
    out: &mut ThePageDrawn,
    laying: &mut Laying<'_>,
    top: i32,
    row: &ARow,
    section_starts: i32,
) -> i32 {
    let palette = laying.palette;
    let line = laying.px(m::A_HAIRLINE);
    let nested = matches!(
        row.kind,
        TheKind::ANestedChoice { .. } | TheKind::AnAddingRow
    );
    let (before, beside, above, gap) = if nested {
        (
            laying.px(m::A_NESTED_ROW_IN),
            laying.px(m::BESIDE_A_ROW),
            laying.px(m::ABOVE_A_NESTED_ROW),
            laying.px(m::INSIDE_A_NESTED_ROW),
        )
    } else {
        (
            laying.px(m::BESIDE_A_ROW),
            laying.px(m::BESIDE_A_ROW),
            laying.px(m::ABOVE_A_ROW),
            laying.px(m::INSIDE_A_ROW),
        )
    };
    let inside_left = laying.left + line;
    let inside_wide = laying.wide - 2 * line;
    let mut x = inside_left + before;
    let mut right = inside_left + inside_wide - beside;

    // What is at the row's start.
    let mark = match row.kind {
        TheKind::AChoice { .. } => laying.px(m::A_CHOICE),
        TheKind::ANestedChoice { .. } | TheKind::AnAddingRow => laying.px(m::A_NESTED_CHOICE),
        TheKind::AnOpeningRow | TheKind::ASwitch(_) | TheKind::Plain => 0,
    };
    let mark_at = x;
    if mark > 0 {
        x += mark + gap;
    }

    // What is at its end, measured first so the words know their room.
    let mut end: Vec<(i32, Shaped, String)> = Vec::new();
    let mut end_wide = 0;
    let switch = laying.px(m::A_SWITCH_IS_WIDE);
    if let TheKind::ASwitch(_) = row.kind {
        end_wide += switch;
    }
    if let Some(words) = &row.at_its_end {
        let shaped = laying
            .words(words, TheType::Label, inside_wide / 3, (palette.surface, palette.muted))
            .cropped_to_its_ink();
        end_wide += shaped.width_of_box
            + if end_wide > 0 {
                laying.px(m::BEFORE_A_SWITCH)
            } else {
                0
            };
        end.push((0, shaped, words.clone()));
    }
    let chevron = laying.px(m::A_CHEVRON);
    if matches!(row.kind, TheKind::AnOpeningRow | TheKind::AnAddingRow) {
        end_wide += chevron + if end_wide > 0 { gap } else { 0 };
    }
    let mut button: Option<(Shaped, String, i32)> = None;
    if let Some((label, _)) = &row.button {
        let shaped = laying
            .words(label, TheType::Label, inside_wide / 3, (palette.surface, palette.ink))
            .cropped_to_its_ink();
        let wide = shaped.width_of_box + 2 * laying.px(m::BESIDE_A_BUTTONS_LABEL);
        end_wide += wide + if end_wide > 0 { gap } else { 0 };
        button = Some((shaped, label.clone(), wide));
    }
    if end_wide > 0 {
        right -= end_wide + gap;
    }
    let words_wide = (right - x).max(laying.px(48));

    // The words.
    let mut y = top + above;
    let title = laying.words(&row.title, TheType::Name, words_wide, (palette.surface, palette.ink));
    let title_tall = title.height;
    y = text(out, x, y, title, &row.title);
    for line_words in &row.lines {
        let shaped = laying.words(line_words, TheType::Body, words_wide, (palette.surface, palette.muted));
        y = text(out, x, y + laying.px(m::UNDER_A_ROWS_TITLE), shaped, line_words);
    }
    if let Some(note) = &row.note {
        let icon = laying.px(m::A_NOTES_ICON) + laying.px(m::AFTER_A_NOTES_ICON);
        let ink = if note.warns { palette.warning } else { palette.muted };
        let shaped = laying.words(&note.words, TheType::SmallLabel, words_wide - icon, (palette.surface, ink));
        y = text(out, x + icon, y + laying.px(m::UNDER_A_ROWS_TITLE), shaped, &note.words);
    }
    if let Some(risk) = &row.risk {
        y = a_risk(
            out,
            laying,
            (x, y + laying.px(m::UNDER_A_ROWS_TITLE), words_wide),
            risk,
        );
    }
    let least = laying.px(if matches!(row.kind, TheKind::AnAddingRow) {
        m::AN_OPENING_ROW_AT_LEAST
    } else {
        m::A_ROW_AT_LEAST
    });
    let bottom = (y + above).max(top + least);

    // The mark, level with the title's line.
    let centre_of_title = top + above + title_tall / 2;
    match row.kind {
        TheKind::AChoice { chosen } | TheKind::ANestedChoice { chosen } => {
            a_choice(out, laying, mark_at, centre_of_title, mark, chosen);
        }
        TheKind::AnAddingRow => {
            // The plus is artwork, drawn with the icons; its room is kept.
        }
        TheKind::AnOpeningRow | TheKind::ASwitch(_) | TheKind::Plain => {}
    }

    // The end, right to left: chevron, switch with its word, or the button.
    let mut at = inside_left + inside_wide - beside;
    if matches!(row.kind, TheKind::AnOpeningRow | TheKind::AnAddingRow) {
        // The chevron is artwork too; its room is kept, centred on the row.
        at -= chevron;
    }
    if let TheKind::ASwitch(is) = row.kind {
        at -= switch;
        a_switch(out, laying, at, centre_of_title, is);
        at -= laying.px(m::BEFORE_A_SWITCH);
    }
    for (_, shaped, words) in end {
        let tall = shaped.height;
        at -= shaped.width_of_box;
        text(out, at, centre_of_title - tall / 2, shaped, &words);
    }
    let mut button_area = None;
    if let (Some((shaped, label, wide)), Some((_, target))) = (button, &row.button) {
        at -= wide;
        let tall = laying.px(m::A_TARGET);
        let area = Rectangle::new((at, top + above).into(), (wide, tall).into());
        out.drawn.push(Drawn::Fill(area, m::A_CARDS_CORNER, palette.surface));
        out.drawn.push(Drawn::Border(area, m::A_CARDS_CORNER, palette.edge));
        let label_tall = shaped.height;
        let label_wide = shaped.width_of_box;
        text(
            out,
            at + (wide - label_wide) / 2,
            top + above + (tall - label_tall) / 2,
            shaped,
            &label,
        );
        button_area = Some((area, target.clone()));
    }
    let bottom = button_area
        .as_ref()
        .map_or(bottom, |(area, _)| bottom.max(area.loc.y + area.size.h + above));

    if let Some(target) = &row.row {
        out.targets.push(ATargetDrawn {
            row: target.clone(),
            area: Rectangle::new((inside_left, top).into(), (inside_wide, bottom - top).into()),
            section_starts,
        });
    }
    if let Some((area, target)) = button_area {
        out.targets.push(ATargetDrawn {
            row: target,
            area,
            section_starts,
        });
    }
    bottom
}

/// A choice's ring, and its dot when it is chosen: `550:170` and `550:184`.
///
/// Not chosen: a ring 1.5 thick at radius 9.25 of 22, in `text/muted`, on
/// `bg/surface`. Chosen: a ring 2 thick at radius 9 in the accent, and a dot
/// of radius 5 in the accent. The 18 under another is the same at 7.25, 7
/// and 4.
fn a_choice(out: &mut ThePageDrawn, laying: &Laying<'_>, x: i32, centre: i32, side: i32, chosen: bool) {
    let palette = laying.palette;
    let area = Rectangle::new((x, centre - side / 2).into(), (side, side).into());
    out.drawn.push(Drawn::Disc(area, palette.surface));
    if chosen {
        out.drawn.push(Drawn::Ring(shrunk(area, laying.px(1)), 20, palette.accent));
        let dot = if side > laying.px(m::A_NESTED_CHOICE) { 6 } else { 5 };
        out.drawn
            .push(Drawn::Disc(shrunk(area, laying.px(dot)), palette.accent));
    } else {
        out.drawn
            .push(Drawn::Ring(shrunk(area, laying.px(1)), 15, palette.muted));
    }
}

/// A switch: `550:256` off and `551:196` on.
///
/// Off: a pill 44 × 26 in `bg/cool` with a 1.5 edge in `text/muted` and a knob
/// of radius 10 at the start, in `text/muted`. On: the pill in the accent and
/// the knob in `bg/surface` at the end. *Not set* is drawn as off is, and its
/// word says which it is.
fn a_switch(out: &mut ThePageDrawn, laying: &Laying<'_>, x: i32, centre: i32, is: SwitchIs) {
    let palette = laying.palette;
    let (wide, tall) = (laying.px(m::A_SWITCH_IS_WIDE), laying.px(m::A_SWITCH_IS_TALL));
    let track = Rectangle::new((x, centre - tall / 2).into(), (wide, tall).into());
    let knob = laying.px(2 * m::A_KNOB);
    let inset = (tall - knob) / 2;
    match is {
        SwitchIs::On => {
            out.drawn.push(Drawn::Fill(track, m::A_SWITCH_IS_TALL / 2, palette.accent));
            let at = Rectangle::new(
                (track.loc.x + wide - inset - knob, track.loc.y + inset).into(),
                (knob, knob).into(),
            );
            out.drawn.push(Drawn::Disc(at, palette.surface));
        }
        SwitchIs::Off | SwitchIs::NotSet => {
            out.drawn.push(Drawn::Fill(track, m::A_SWITCH_IS_TALL / 2, palette.cool));
            out.drawn
                .push(Drawn::Ring(shrunk(track, 0), 15, palette.muted));
            let at = Rectangle::new(
                (track.loc.x + inset, track.loc.y + inset).into(),
                (knob, knob).into(),
            );
            out.drawn.push(Drawn::Disc(at, palette.muted));
        }
    }
}

/// What a choice risks: `550:246`, `bg/canvas`, 12 across and 10 down inside,
/// at radius 10, the heading in `status/warning` and the sentence in
/// `text/primary`.
fn a_risk(
    out: &mut ThePageDrawn,
    laying: &mut Laying<'_>,
    (x, y, wide): (i32, i32, i32),
    risk: &ARisk,
) -> i32 {
    let palette = laying.palette;
    let (across, down) = (laying.px(m::BESIDE_A_RISK), laying.px(m::ABOVE_A_RISK));
    let icon = laying.px(m::A_RISKS_ICON) + laying.px(m::AFTER_A_RISKS_ICON);
    let inside = wide - 2 * across - icon;
    let heading = laying.words(
        &risk.heading,
        TheType::Label,
        inside,
        (palette.canvas, palette.warning),
    );
    let sentence = laying.words(&risk.words, TheType::Body, inside, (palette.canvas, palette.ink));
    let between = laying.px(m::UNDER_A_RISKS_HEADING);
    let tall = heading.height + between + sentence.height + 2 * down;
    let area = Rectangle::new((x, y).into(), (wide, tall).into());
    out.drawn.push(Drawn::Fill(area, m::A_SMALL_CORNER, palette.canvas));
    let below = text(out, x + across + icon, y + down, heading, &risk.heading);
    text(out, x + across + icon, below + between, sentence, &risk.words);
    y + tall
}

/// `area` taken in by `by` on every side.
fn shrunk(area: Area, by: i32) -> Area {
    Rectangle::new(
        (area.loc.x + by, area.loc.y + by).into(),
        ((area.size.w - 2 * by).max(0), (area.size.h - 2 * by).max(0)).into(),
    )
}

/// Paint what was laid out onto `canvas`, moved up by `scrolled` and down by
/// `top`.
pub(crate) fn painted(drawn: &[Drawn], canvas: &mut Canvas, top: i32, scale: f32) {
    let moved = |area: &Area| Rectangle::new((area.loc.x, area.loc.y + top).into(), area.size);
    for one in drawn {
        match one {
            Drawn::Fill(area, corner, colour) => {
                canvas.fill(moved(area), *corner as f32 * scale, *colour, 1.0);
            }
            Drawn::Border(area, corner, colour) => {
                canvas.border(moved(area), *corner as f32 * scale, scale.max(1.0), *colour);
            }
            Drawn::Disc(area, colour) => {
                let area = moved(area);
                let radius = area.size.w as f32 / 2.0;
                canvas.disc(
                    area.loc.x as f32 + radius,
                    area.loc.y as f32 + radius,
                    radius,
                    *colour,
                );
            }
            Drawn::Ring(area, tenths, colour) => {
                let area = moved(area);
                let width = *tenths as f32 / 10.0 * scale;
                let (hw, hh) = (area.size.w as f32 / 2.0, area.size.h as f32 / 2.0);
                if area.size.w == area.size.h {
                    canvas.ring(
                        area.loc.x as f32 + hw,
                        area.loc.y as f32 + hh,
                        hw - width / 2.0,
                        width,
                        *colour,
                    );
                } else {
                    canvas.border(area, hh, width, *colour);
                }
            }
            Drawn::Words(x, y, shaped) => canvas.words(*x, *y + top, shaped),
        }
    }
}
