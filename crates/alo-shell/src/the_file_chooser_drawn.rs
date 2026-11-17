//! **The file chooser, laid out for one display** (`docs/design/the-file-chooser.md`):
//! its title, the line naming who asked, where the person is, a card of the
//! folder's rows, a sentence about what just happened, the name when saving,
//! and *Cancel* and *Open* or *Save*.
//!
//! # Drawn as *Open with* is drawn
//!
//! The design draws no file chooser, so this follows the nearest drawn sheet,
//! *Open with* (frame `25:52`, `crate::the_open_with_sheet_drawn`), with its
//! figures and Settings' parts: the 480 sheet, the 25 inside it, the 48
//! buttons, a Settings row for each folder and file. Every figure is a logical
//! measure taken through the display's scale.
//!
//! # As many rows as the display holds
//!
//! From [`THE_MOST_ROWS`] down to [`THE_FEWEST_ROWS`], the most that fit with
//! the rest of the sheet on this display — **derived from the display, never a
//! fixed count** — a page at a time, so the row the keyboard is on is always
//! shown. A display that cannot hold the fewest is answered *cannot show*.

use alo_file_chooser::{How, Kind, Where};
use alo_strings::{Filling, Strings};
use smithay::utils::Rectangle;

use crate::desktop_look::Measure;
use crate::display_scale::DisplayScale;
use crate::painted::Washed;
use crate::settings_canvas::{AShadow, Area, Canvas};
use crate::settings_drawn_page::{Drawn, Laying, ThePageDrawn, painted};
use crate::settings_measures as m;
use crate::settings_page::{ARow, TheKind};
use crate::settings_palette::SettingsPalette;
use crate::settings_type::{TheType, written};
use crate::the_file_chooser::FileChooserShows;
use crate::the_file_choosers_words as words;
use crate::the_settings_icons::ASettingsIcon;
use crate::{RenderError, SettingsLook, WindowControlLabels};

/// The sheet's width, *Open with*'s `25:57`.
const THE_SHEET_IS_WIDE: u32 = 480;
/// Inside the sheet.
const INSIDE_THE_SHEET: u32 = 25;
/// From the title to the line under it.
const UNDER_THE_TITLE: u32 = 14;
/// Between two lines of words.
const BETWEEN_LINES: u32 = 6;
/// From the lines to the card.
const ABOVE_THE_CARD: u32 = 11;
/// From the card to what is under it.
const UNDER_THE_CARD: u32 = 14;
/// Inside the name's field, across.
const INSIDE_THE_FIELD: u32 = 12;
/// The caret's width.
const THE_CARET: u32 = 2;
/// Between *Cancel* and the other button.
const BETWEEN_THE_BUTTONS: u32 = 8;
/// Beside a button's label.
const BESIDE_A_LABEL: u32 = 18;
/// The least the display leaves beside the sheet.
const BESIDE_THE_SHEET: u32 = 16;
/// The most rows shown at once.
pub(crate) const THE_MOST_ROWS: usize = 8;
/// The fewest rows a sheet is shown with.
pub(crate) const THE_FEWEST_ROWS: usize = 3;

/// The sheet drawn, for one display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FileChooserPicture {
    /// The sheet and everything on it, as one raster.
    pub(crate) washed: Washed,
    /// Each row shown, with its place in the whole list, where it is drawn.
    pub(crate) rows: Vec<(usize, Area)>,
    /// The name's field, when saving.
    pub(crate) name: Option<Area>,
    /// *Cancel*.
    pub(crate) cancel: Area,
    /// *Open* or *Save*.
    pub(crate) button: Area,
    /// Every piece of text drawn, for a test to read back.
    pub(crate) texts: Vec<String>,
}

/// The sheet laid out from its own corner, before it is placed.
struct LaidOut {
    /// What is drawn.
    out: ThePageDrawn,
    /// Each row shown, with its place in the list.
    rows: Vec<(usize, Area)>,
    /// The name's field.
    name: Option<Area>,
    /// Where the buttons' top is.
    buttons_top: i32,
    /// How tall the sheet is.
    tall: i32,
}

/// **The sheet for a display of `size` at `display`.**
///
/// # Errors
/// [`RenderError::ApprovalScene`] when the display cannot hold the sheet with
/// its fewest rows, or the person's accent is not one they may choose — and
/// the sheet is then answered *cannot show*.
pub(crate) fn drawn(
    shows: FileChooserShows<'_>,
    strings: &Strings,
    labels: &mut WindowControlLabels,
    size: (i32, i32),
    look: SettingsLook,
    display: DisplayScale,
    called: &dyn Fn(&str) -> String,
) -> Result<FileChooserPicture, RenderError> {
    let palette = SettingsPalette::of(look).map_err(|_| RenderError::ApprovalScene)?;
    let measure = Measure::of(look.scale).on(display);
    let px = |logical: u32| measure.px(i32::try_from(logical).unwrap_or(i32::MAX));
    let wide = px(THE_SHEET_IS_WIDE).min(size.0 - 2 * px(BESIDE_THE_SHEET));
    let inside = px(INSIDE_THE_SHEET);
    let column = wide - 2 * inside;
    if column < px(240) {
        return Err(RenderError::ApprovalScene);
    }
    let room = size.1 - 2 * px(BESIDE_THE_SHEET);
    let mut laid = None;
    for how_many in (THE_FEWEST_ROWS..=THE_MOST_ROWS).rev() {
        let mut laying = Laying {
            fonts: &mut labels.fonts,
            measure,
            palette,
            left: inside,
            wide: column,
            above: 0,
        };
        let one = laid_out(shows, strings, &mut laying, how_many, called);
        if one.tall <= room {
            laid = Some(one);
            break;
        }
    }
    let laid = laid.ok_or(RenderError::ApprovalScene)?;

    // **Placed**, centred on the display, and painted.
    let left = (size.0 - wide) / 2;
    let top = (size.1 - laid.tall) / 2;
    let sheet = Rectangle::new((left, top).into(), (wide, laid.tall).into());
    let raised =
        crate::how_a_panel_is_raised::HowItIsRaised::FLOATING.at(f64::from(measure.factor()));
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
    let moved: Vec<Drawn> = laid
        .out
        .drawn
        .into_iter()
        .map(|one| crate::the_open_with_sheet_drawn::shifted(one, left))
        .collect();
    painted(&moved, &mut canvas, top, measure.factor());
    let at = |area: Area| Rectangle::new((area.loc.x + left, area.loc.y + top).into(), area.size);

    // *Open* or *Save*, navy, at the right; *Cancel*, quiet, before it.
    let say = |word: alo_strings::Word| strings.say(&word.key(), &Filling::nothing()).into_text();
    let button_words = say(match shows.chooser.how() {
        How::Open => words::OPEN,
        How::Save => words::SAVE,
    });
    let cancel_words = say(words::CANCEL);
    let label = |text: &str, ground: [u8; 3], ink: [u8; 3], fonts: &mut _| {
        written(
            fonts,
            text,
            TheType::Label,
            column / 2,
            measure,
            (ground, ink),
        )
        .cropped_to_its_ink()
    };
    let button_label = label(
        &button_words,
        palette.ink,
        palette.surface,
        &mut labels.fonts,
    );
    let cancel_label = label(
        &cancel_words,
        palette.surface,
        palette.ink,
        &mut labels.fonts,
    );
    let beside = px(BESIDE_A_LABEL);
    let button_tall = px(m::A_TARGET);
    let button_wide = button_label.width_of_box + 2 * beside;
    let cancel_wide = cancel_label.width_of_box + 2 * beside;
    let button = Rectangle::new(
        (left + wide - inside - button_wide, top + laid.buttons_top).into(),
        (button_wide, button_tall).into(),
    );
    let cancel = Rectangle::new(
        (
            button.loc.x - px(BETWEEN_THE_BUTTONS) - cancel_wide,
            top + laid.buttons_top,
        )
            .into(),
        (cancel_wide, button_tall).into(),
    );
    let button_corner = px(m::A_CARDS_CORNER) as f32;
    canvas.fill(button, button_corner, palette.ink, 1.0);
    canvas.words(
        button.loc.x + beside,
        button.loc.y + (button_tall - button_label.height) / 2,
        &button_label,
    );
    canvas.words(
        cancel.loc.x + beside,
        cancel.loc.y + (button_tall - cancel_label.height) / 2,
        &cancel_label,
    );
    let mut texts: Vec<String> = laid.out.texts.into_iter().map(|text| text.text).collect();
    texts.extend([cancel_words, button_words]);
    Ok(FileChooserPicture {
        washed: canvas.washed().premultiplied(),
        rows: laid
            .rows
            .into_iter()
            .map(|(row, area)| (row, at(area)))
            .collect(),
        name: laid.name.map(at),
        cancel,
        button,
        texts,
    })
}

/// Everything but the buttons, from the sheet's own top, with `how_many` rows.
fn laid_out(
    shows: FileChooserShows<'_>,
    strings: &Strings,
    laying: &mut Laying<'_>,
    how_many: usize,
    called: &dyn Fn(&str) -> String,
) -> LaidOut {
    let palette = laying.palette;
    let measure = laying.measure;
    let px = |logical: u32| measure.px(i32::try_from(logical).unwrap_or(i32::MAX));
    let inside = laying.left;
    let column = laying.wide;
    let chooser = shows.chooser;
    let say = |word: alo_strings::Word, filling: &Filling| -> String {
        strings.say(&word.key(), filling).into_text()
    };
    let mut out = ThePageDrawn {
        drawn: Vec::new(),
        targets: Vec::new(),
        texts: Vec::new(),
        tall: 0,
    };
    let (title, asked) = match chooser.how() {
        How::Open => (words::OPEN_A_FILE, words::ASKED_TO_OPEN),
        How::Save => (words::SAVE_A_FILE, words::ASKED_TO_SAVE),
    };
    let title = say(title, &Filling::nothing());
    let asking = called(shows.asking.identifier());
    let line = say(asked, &Filling::of("application", asking));
    let folder = chooser.at().to_string_lossy().into_owned();
    let where_ = say(words::IN_THE_FOLDER, &Filling::of("folder", folder));

    let mut y = inside;
    let shaped = laying.words(
        &title,
        TheType::Title,
        column,
        (palette.surface, palette.ink),
    );
    y = crate::settings_drawn_page::text(&mut out, inside, y, shaped, &title);
    y += px(UNDER_THE_TITLE);
    for (said, colour) in [(&line, palette.secondary), (&where_, palette.secondary)] {
        let shaped = laying.words(said, TheType::Body, column, (palette.surface, colour));
        y = crate::settings_drawn_page::text(&mut out, inside, y, shaped, said);
        y += px(BETWEEN_LINES);
    }
    y += px(ABOVE_THE_CARD) - px(BETWEEN_LINES);

    // **The card**, the page the keyboard is on.
    let hairline = px(m::A_HAIRLINE);
    let card_top = y;
    let card_fill_at = out.drawn.len();
    y += hairline;
    let all = chooser.rows();
    let shown = chooser.shown(how_many);
    let mut rows = Vec::with_capacity(shown.len());
    for (which, at) in shown.enumerate() {
        let Some(one) = all.get(at) else {
            continue;
        };
        if which > 0 {
            out.drawn.push(Drawn::Fill(
                Rectangle::new(
                    (inside + hairline, y).into(),
                    (column - 2 * hairline, hairline).into(),
                ),
                0,
                palette.edge,
            ));
            y += hairline;
        }
        let (kind, icon, under) = match one.kind {
            Kind::Above => (
                TheKind::Plain,
                ASettingsIcon::Folder,
                words::THE_FOLDER_ABOVE,
            ),
            Kind::Folder => (TheKind::AnOpeningRow, ASettingsIcon::Folder, words::FOLDER),
            Kind::File => (TheKind::Plain, ASettingsIcon::File, words::FILE),
        };
        let mut row = ARow::named(None, kind, one.name.to_owned());
        row.lines.push(say(under, &Filling::nothing()));
        row.lead = Some(icon);
        let bottom = crate::settings_drawn_row::a_row(&mut out, laying, y, &row, card_top);
        rows.push((
            at,
            Rectangle::new((inside, y).into(), (column, bottom - y).into()),
        ));
        y = bottom;
    }
    y += hairline;
    let card = Rectangle::new((inside, card_top).into(), (column, y - card_top).into());
    out.drawn.insert(
        card_fill_at,
        Drawn::Fill(card, m::A_CARDS_CORNER, palette.surface),
    );
    out.drawn
        .push(Drawn::Border(card, m::A_CARDS_CORNER, palette.edge));
    // **Where the keyboard is**, outlined as a Settings row is.
    if chooser.on() == Where::TheRows
        && let Some((_, focused)) = rows.iter().find(|(at, _)| *at == chooser.focus())
    {
        out.drawn
            .push(Drawn::Border(*focused, m::A_CARDS_CORNER, palette.ink));
    }
    y += px(UNDER_THE_CARD);

    // **What just happened**, when anything did.
    if let Some(note) = chooser.note() {
        let said = words::said(strings, note);
        let shaped = laying.words(
            &said,
            TheType::Body,
            column,
            (palette.surface, palette.secondary),
        );
        y = crate::settings_drawn_page::text(&mut out, inside, y, shaped, &said);
        y += px(UNDER_THE_CARD);
    }

    // **The name**, when saving.
    let mut name_area = None;
    if chooser.how() == How::Save {
        let label = say(words::NAME, &Filling::nothing());
        let shaped = laying.words(
            &label,
            TheType::Label,
            column,
            (palette.surface, palette.ink),
        );
        y = crate::settings_drawn_page::text(&mut out, inside, y, shaped, &label);
        y += px(BETWEEN_LINES);
        let field_tall = px(m::A_TARGET);
        let field = Rectangle::new((inside, y).into(), (column, field_tall).into());
        let in_it = chooser.on() == Where::TheName;
        out.drawn.push(Drawn::Border(
            field,
            m::A_CARDS_CORNER,
            if in_it { palette.ink } else { palette.edge },
        ));
        let across = px(INSIDE_THE_FIELD);
        let shaped = laying.words(
            chooser.name(),
            TheType::Body,
            column - 2 * across,
            (palette.surface, palette.ink),
        );
        let (text_wide, text_tall) = (shaped.width, shaped.height);
        let text_top = y + (field_tall - text_tall) / 2;
        let _ = crate::settings_drawn_page::text(
            &mut out,
            inside + across,
            text_top,
            shaped,
            chooser.name(),
        );
        if in_it {
            out.drawn.push(Drawn::Fill(
                Rectangle::new(
                    (inside + across + text_wide, text_top).into(),
                    (px(THE_CARET).max(1), text_tall.max(px(16))).into(),
                ),
                0,
                palette.ink,
            ));
        }
        name_area = Some(field);
        y += field_tall + px(UNDER_THE_CARD);
    }
    let buttons_top = y;
    let tall = buttons_top + px(m::A_TARGET) + inside;
    LaidOut {
        out,
        rows,
        name: name_area,
        buttons_top,
        tall,
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_file_chooser_drawn_tests.rs"]
mod tests;
