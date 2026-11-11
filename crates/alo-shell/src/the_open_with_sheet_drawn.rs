//! **The *Open with* sheet, laid out for one display** — frame `25:52`
//! (`docs/design/default-apps-and-open-with.md`): its title, the line naming
//! who asked and the address, a card of the applications offered — the row
//! Settings → Default apps draws, ring, picture, name and line — and *Cancel*
//! and *Open*.
//!
//! # Drawn with the Settings window's own parts
//!
//! The frame is drawn with Settings' measures, so the sheet is laid out by the
//! same code: a row is `crate::settings_drawn_row::a_row`, painted by
//! `crate::settings_drawn_page::painted` onto a `crate::settings_canvas::Canvas`,
//! in the person's palette and accent. Its figures — the 480 sheet, the 25
//! inside it, the 48 buttons — are logical measures taken through the display's
//! scale (`CLAUDE.md`, *nothing is built to one screen size*), and the sheet is
//! never wider than the display allows.
//!
//! # Where the keyboard is, is drawn
//!
//! The row the keyboard is on is ringed (its choice ring filled) and outlined in
//! `text/primary`, as a row of Settings is; Enter opens that one.

use alo_strings::{Filling, Strings};
use smithay::utils::Rectangle;

use crate::desktop_look::Measure;
use crate::display_scale::DisplayScale;
use crate::painted::Washed;
use crate::settings_canvas::{AShadow, Area, Canvas};
use crate::settings_drawn_page::{Laying, ThePageDrawn, painted};
use crate::settings_measures as m;
use crate::settings_page::{ARow, AnApplicationsMark, TheKind};
use crate::settings_palette::SettingsPalette;
use crate::settings_type::{TheType, written};
use crate::the_open_with_sheet::SheetShows;
use crate::the_open_with_sheets_words as words;
use crate::{RenderError, SettingsLook, WindowControlLabels};

/// The sheet's width, `25:57`.
const THE_SHEET_IS_WIDE: u32 = 480;
/// Inside the sheet, to the title and the card (`25:58` at 25).
const INSIDE_THE_SHEET: u32 = 25;
/// From the title's top to the line's (`25:59` at 67, the title at 23).
const UNDER_THE_TITLE: u32 = 14;
/// From the line to the card (`25:60` at 103).
const ABOVE_THE_CARD: u32 = 11;
/// From the card to the buttons (`25:74` at 263, after a card ending 249).
const ABOVE_THE_BUTTONS: u32 = 14;
/// Between *Cancel* and *Open*.
const BETWEEN_THE_BUTTONS: u32 = 8;
/// Beside a button's label.
const BESIDE_A_LABEL: u32 = 18;
/// The least the display leaves beside the sheet.
const BESIDE_THE_SHEET: u32 = 16;

/// The sheet drawn, for one display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OpenWithPicture {
    /// The display size it was laid out for.
    pub(crate) size: (i32, i32),
    /// The sheet and everything on it, as one raster.
    pub(crate) washed: Washed,
    /// Each offered application's row, in order, where it is drawn.
    pub(crate) rows: Vec<Area>,
    /// *Cancel*, where it is drawn.
    pub(crate) cancel: Area,
    /// *Open*, where it is drawn.
    pub(crate) open: Area,
    /// Every piece of text drawn, for a test to read back.
    pub(crate) texts: Vec<String>,
}

/// **The sheet for a display of `size` at `display`.**
///
/// # Errors
/// [`RenderError::ApprovalScene`] when the display is too small to hold the
/// sheet whole, or the person's accent is not one they may choose — and the
/// sheet is then answered *cannot show* rather than hold every key with
/// nothing on the screen.
pub(crate) fn drawn(
    shows: SheetShows<'_>,
    strings: &Strings,
    labels: &mut WindowControlLabels,
    size: (i32, i32),
    look: SettingsLook,
    display: DisplayScale,
    names: &dyn Fn(&str) -> (String, Option<alo_dock::artwork::APicture>),
) -> Result<OpenWithPicture, RenderError> {
    let palette = SettingsPalette::of(look).map_err(|_| RenderError::ApprovalScene)?;
    let measure = Measure::of(look.scale).on(display);
    let px = |logical: u32| measure.px(i32::try_from(logical).unwrap_or(i32::MAX));
    let wide = px(THE_SHEET_IS_WIDE).min(size.0 - 2 * px(BESIDE_THE_SHEET));
    let inside = px(INSIDE_THE_SHEET);
    let column = wide - 2 * inside;
    if column < px(240) {
        return Err(RenderError::ApprovalScene);
    }
    let say = |word: alo_strings::Word, filling: &Filling| -> String {
        strings.say(&word.key(), filling).into_text()
    };
    let (asking, _) = names(shows.asking.identifier());
    let title = say(words::OPEN_WITH, &Filling::nothing());
    let line = say(
        words::ASKED_TO_OPEN,
        &Filling::of("application", asking).and("address", shows.link.as_str()),
    );
    let the_default = say(
        match shows.link.kind() {
            alo_applications::LinkKind::Web => words::YOUR_DEFAULT_FOR_WEB_LINKS,
            alo_applications::LinkKind::Mail => words::YOUR_DEFAULT_FOR_EMAIL_ADDRESSES,
        },
        &Filling::nothing(),
    );
    let installed = say(words::INSTALLED, &Filling::nothing());
    let rows: Vec<ARow> = shows
        .offered
        .iter()
        .enumerate()
        .map(|(at, application)| {
            let (name, picture) = names(application.identifier());
            let letter = name.chars().next().map(String::from).unwrap_or_default();
            let mut row = ARow::named(
                None,
                TheKind::AChoice {
                    chosen: at == shows.focus,
                },
                name,
            );
            row.lines.push(if at == 0 {
                the_default.clone()
            } else {
                installed.clone()
            });
            row.application = Some(AnApplicationsMark { picture, letter });
            row
        })
        .collect();

    // **Laid out first, from the sheet's top**, so its height is known.
    let mut out = ThePageDrawn {
        drawn: Vec::new(),
        targets: Vec::new(),
        texts: Vec::new(),
        tall: 0,
    };
    let mut laying = Laying {
        fonts: &mut labels.fonts,
        measure,
        palette,
        left: inside,
        wide: column,
        above: 0,
    };
    let mut y = inside;
    let title_shaped = laying.words(&title, TheType::Title, column, (palette.surface, palette.ink));
    y = crate::settings_drawn_page::text(&mut out, inside, y, title_shaped, &title);
    y += px(UNDER_THE_TITLE);
    let line_shaped = laying.words(&line, TheType::Body, column, (palette.surface, palette.secondary));
    y = crate::settings_drawn_page::text(&mut out, inside, y, line_shaped, &line);
    y += px(ABOVE_THE_CARD);
    let hairline = px(m::A_HAIRLINE);
    let card_top = y;
    let card_fill_at = out.drawn.len();
    y += hairline;
    let mut row_areas = Vec::with_capacity(rows.len());
    for (which, row) in rows.iter().enumerate() {
        if which > 0 {
            out.drawn.push(crate::settings_drawn_page::Drawn::Fill(
                Rectangle::new((inside + hairline, y).into(), (column - 2 * hairline, hairline).into()),
                0,
                palette.edge,
            ));
            y += hairline;
        }
        let bottom = crate::settings_drawn_row::a_row(&mut out, &mut laying, y, row, card_top);
        row_areas.push(Rectangle::new((inside, y).into(), (column, bottom - y).into()));
        y = bottom;
    }
    y += hairline;
    let card = Rectangle::new((inside, card_top).into(), (column, y - card_top).into());
    out.drawn.insert(
        card_fill_at,
        crate::settings_drawn_page::Drawn::Fill(card, m::A_CARDS_CORNER, palette.surface),
    );
    out.drawn.push(crate::settings_drawn_page::Drawn::Border(
        card,
        m::A_CARDS_CORNER,
        palette.edge,
    ));
    // **Where the keyboard is**, outlined as a Settings row is.
    if let Some(focused) = row_areas.get(shows.focus) {
        out.drawn.push(crate::settings_drawn_page::Drawn::Border(
            *focused,
            m::A_CARDS_CORNER,
            palette.ink,
        ));
    }
    y += px(ABOVE_THE_BUTTONS);
    let buttons_top = y;
    let button_tall = px(m::A_TARGET);
    let tall = buttons_top + button_tall + inside;
    if tall > size.1 - 2 * px(BESIDE_THE_SHEET) {
        return Err(RenderError::ApprovalScene);
    }

    // **Then placed**, centred on the display, and painted.
    let left = (size.0 - wide) / 2;
    let top = (size.1 - tall) / 2;
    let sheet = Rectangle::new((left, top).into(), (wide, tall).into());
    let raised = crate::how_a_panel_is_raised::HowItIsRaised::FLOATING.at(f64::from(measure.factor()));
    let reach = raised.reach();
    let screen: Area = Rectangle::new((0, 0).into(), size.into());
    let canvas_area = Rectangle::new(
        (sheet.loc.x - reach, sheet.loc.y - reach).into(),
        (sheet.size.w + 2 * reach, sheet.size.h + 2 * reach).into(),
    )
    .intersection(screen)
    .ok_or(RenderError::ApprovalScene)?;
    let mut canvas = Canvas::over(canvas_area);
    let corner = px(m::THE_WINDOWS_CORNER) as f32;
    canvas.shadow(
        sheet,
        corner,
        AShadow {
            falls: raised.falls(),
            blurs: raised.blurs(),
            spreads: raised.spreads(),
            darkness: raised.darkness(),
        },
    );
    canvas.fill(sheet, corner, palette.surface, 1.0);
    // Everything laid out from the sheet's corner, moved there.
    let moved: Vec<crate::settings_drawn_page::Drawn> = out
        .drawn
        .into_iter()
        .map(|one| shifted(one, left))
        .collect();
    painted(&moved, &mut canvas, top, measure.factor());
    let at = |area: Area| Rectangle::new((area.loc.x + left, area.loc.y + top).into(), area.size);

    // *Open*, navy, at the right; *Cancel*, quiet, before it.
    let label = |words: &str, ground: [u8; 3], ink: [u8; 3], fonts: &mut _| {
        written(fonts, words, TheType::Label, column / 2, measure, (ground, ink)).cropped_to_its_ink()
    };
    let open_words = say(words::OPEN, &Filling::nothing());
    let cancel_words = say(words::CANCEL, &Filling::nothing());
    let open_label = label(&open_words, palette.ink, palette.surface, &mut labels.fonts);
    let cancel_label = label(&cancel_words, palette.surface, palette.ink, &mut labels.fonts);
    let beside = px(BESIDE_A_LABEL);
    let open_wide = open_label.width_of_box + 2 * beside;
    let cancel_wide = cancel_label.width_of_box + 2 * beside;
    let open = Rectangle::new(
        (left + wide - inside - open_wide, top + buttons_top).into(),
        (open_wide, button_tall).into(),
    );
    let cancel = Rectangle::new(
        (open.loc.x - px(BETWEEN_THE_BUTTONS) - cancel_wide, top + buttons_top).into(),
        (cancel_wide, button_tall).into(),
    );
    let button_corner = px(m::A_CARDS_CORNER) as f32;
    canvas.fill(open, button_corner, palette.ink, 1.0);
    canvas.words(
        open.loc.x + beside,
        open.loc.y + (button_tall - open_label.height) / 2,
        &open_label,
    );
    canvas.words(
        cancel.loc.x + beside,
        cancel.loc.y + (button_tall - cancel_label.height) / 2,
        &cancel_label,
    );
    let mut texts: Vec<String> = out.texts.into_iter().map(|text| text.text).collect();
    texts.extend([cancel_words, open_words]);
    Ok(OpenWithPicture {
        size,
        washed: canvas.washed(),
        rows: row_areas.into_iter().map(at).collect(),
        cancel,
        open,
        texts,
    })
}

/// One thing laid out from the sheet's left edge, moved `by` across.
fn shifted(one: crate::settings_drawn_page::Drawn, by: i32) -> crate::settings_drawn_page::Drawn {
    use crate::settings_drawn_page::Drawn;
    let across = |area: Area| Rectangle::new((area.loc.x + by, area.loc.y).into(), area.size);
    match one {
        Drawn::Fill(area, corner, colour) => Drawn::Fill(across(area), corner, colour),
        Drawn::Border(area, corner, colour) => Drawn::Border(across(area), corner, colour),
        Drawn::Disc(area, colour) => Drawn::Disc(across(area), colour),
        Drawn::Ring(area, tenths, colour) => Drawn::Ring(across(area), tenths, colour),
        Drawn::Words(x, y, shaped) => Drawn::Words(x + by, y, shaped),
        Drawn::Icon(icon, area, colour) => Drawn::Icon(icon, across(area), colour),
        Drawn::Picture(area, picture) => Drawn::Picture(across(area), picture),
    }
}

#[cfg(test)]
#[path = "the_open_with_sheet_drawn_tests.rs"]
mod tests;
