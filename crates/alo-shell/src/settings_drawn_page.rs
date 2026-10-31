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
use crate::settings_page::{ABlock, APage, ASection, AnAction};
use crate::settings_palette::SettingsPalette;
use crate::settings_type::{TheType, written};
use crate::settings_window::SettingsRow;
use crate::the_settings_icons::ASettingsIcon;

/// One thing to draw, placed from the top of the page.
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
    /// One of the window's icons, in its frame, in a colour.
    Icon(ASettingsIcon, Area, [u8; 3]),
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
    pub(crate) fn px(&self, logical: u32) -> i32 {
        self.measure.px(i32::try_from(logical).unwrap_or(i32::MAX))
    }

    /// Words shaped in `style`, `width` pixels at most, on `ground` in `ink`.
    pub(crate) fn words(
        &mut self,
        words: &str,
        style: TheType,
        width: i32,
        colours: ([u8; 3], [u8; 3]),
    ) -> Shaped {
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
            ABlock::AnAction(action) => an_action(&mut out, laying, y, action),
        };
    }
    out.tall = y + laying.above;
    out
}

/// Place shaped words at (`x`, `y`) and remember them; answer the bottom.
pub(crate) fn text(out: &mut ThePageDrawn, x: i32, y: i32, shaped: Shaped, words: &str) -> i32 {
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
    out.drawn
        .push(Drawn::Fill(area, m::A_CARDS_CORNER, palette.cool));
    text(out, laying.left + across, y + down, shaped, said);
    y + tall
}

/// The page's own action: `550:853`, a Primary button — `text/primary`
/// ground, `bg/surface` label, 48 tall at radius 12 — and the line beside it,
/// 16 away, in `text/muted`.
fn an_action(out: &mut ThePageDrawn, laying: &mut Laying<'_>, y: i32, action: &AnAction) -> i32 {
    let palette = laying.palette;
    let shaped = laying
        .words(
            &action.label,
            TheType::Label,
            laying.wide / 2,
            (palette.ink, palette.surface),
        )
        .cropped_to_its_ink();
    let (wide, tall) = (
        shaped.width_of_box + 2 * laying.px(m::BESIDE_A_BUTTONS_LABEL),
        laying.px(m::A_TARGET),
    );
    let button = Rectangle::new((laying.left, y).into(), (wide, tall).into());
    out.drawn
        .push(Drawn::Fill(button, m::A_CARDS_CORNER, palette.ink));
    let label = shaped.height;
    let label_wide = shaped.width_of_box;
    text(
        out,
        laying.left + (wide - label_wide) / 2,
        y + (tall - label) / 2,
        shaped,
        &action.label,
    );
    let gap = laying.px(m::INSIDE_A_ROW);
    let beside = laying.words(
        &action.beside,
        TheType::Body,
        laying.wide - wide - gap,
        (palette.surface, palette.muted),
    );
    let beside_tall = beside.height;
    let bottom = text(
        out,
        laying.left + wide + gap,
        y + (tall - beside_tall).max(0) / 2,
        beside,
        &action.beside,
    );
    out.targets.push(ATargetDrawn {
        row: action.row.clone(),
        area: button,
        section_starts: y,
    });
    bottom.max(y + tall)
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
        y = crate::settings_drawn_row::a_row(out, laying, y, row, top);
    }
    y += line;
    let card = Rectangle::new((left, card_top).into(), (wide, y - card_top).into());
    out.drawn.insert(
        fill_at,
        Drawn::Fill(card, m::A_CARDS_CORNER, palette.surface),
    );
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

/// Everything drawn and written since `from`, moved down by `by`.
pub(crate) fn moved_down(out: &mut ThePageDrawn, (drawn, texts): (usize, usize), by: i32) {
    let down = |area: &mut Area| area.loc.y += by;
    for one in out.drawn.iter_mut().skip(drawn) {
        match one {
            Drawn::Fill(area, ..)
            | Drawn::Border(area, ..)
            | Drawn::Disc(area, _)
            | Drawn::Ring(area, ..) => {
                down(area);
            }
            Drawn::Words(_, y, _) => *y += by,
            Drawn::Icon(_, area, _) => down(area),
        }
    }
    for text in out.texts.iter_mut().skip(texts) {
        down(&mut text.area);
    }
}

/// `area` taken in by `by` on every side.
pub(crate) fn shrunk(area: Area, by: i32) -> Area {
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
            Drawn::Icon(icon, area, colour) => icon.drawn(canvas, moved(area), *colour),
        }
    }
}
