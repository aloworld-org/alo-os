//! **The sheet alo asks for an application on, laid out for one display** —
//! frame `501:35539` *Settings detail / window*
//! (`docs/design/when-alo-asks-for-an-application.md`): its heading, the line
//! under it, the three lengths each in its own card (`Terminal / Settings
//! choice`), the note naming Escape, then *Not now* and the navy *Let alo use
//! {application}*, from the left (`501:35574`).
//!
//! # Drawn with Settings' own parts, as *Open with* is
//!
//! A length is `crate::settings_drawn_row::a_row`, painted by
//! `crate::settings_drawn_page::painted` onto a `crate::settings_canvas::Canvas`
//! in the person's palette, raised with the FLOATING shadow and centred on the
//! display, with nothing behind it dimmed (the owner's ruling for *Open with*
//! and the GitHub sheet). Every figure below is the frame's, a logical measure
//! taken through the display's scale and the person's text size; the sheet is
//! never wider than the display allows, and a display that cannot hold it whole
//! is refused rather than drawn in part.
//!
//! # Where the keyboard is, is drawn
//!
//! The length the keyboard is on has its ring filled and its card outlined in
//! `text/primary`, as a row of Settings is; Enter grants that one.

use alo_strings::{Filling, Strings, Word};
use smithay::utils::Rectangle;

use crate::desktop_look::Measure;
use crate::display_scale::DisplayScale;
use crate::painted::Washed;
use crate::settings_canvas::{AShadow, Area, Canvas};
use crate::settings_drawn_page::{Drawn, Laying, ThePageDrawn, painted, text};
use crate::settings_measures as m;
use crate::settings_page::{ARow, TheKind};
use crate::settings_palette::SettingsPalette;
use crate::settings_type::TheType;
use crate::the_application_sheet::{
    ApplicationSheetShows, HowLongForAnApplication, OnTheApplicationSheet,
};
use crate::the_application_sheets_words as words;
use crate::{RenderError, SettingsLook, WindowControlLabels};

/// The sheet's width (`501:35539`, 680).
const THE_SHEET_IS_WIDE: u32 = 680;
/// Inside the sheet, to everything on it (the heading at 25, 25).
const INSIDE_THE_SHEET: u32 = 25;
/// Between one thing and the next: heading to line (55 → 71), line to the
/// first length (93 → 109), length to length (181 → 197), the last length to
/// the note (267 → 283), and the note to the buttons (305 → 321).
const BETWEEN: u32 = 16;
/// *Not now*'s least width (`501:35575`, 160).
const NOT_NOW_IS_AT_LEAST: u32 = 160;
/// The navy button's least width (`501:35577`, 280).
const THE_NAVY_ONE_IS_AT_LEAST: u32 = 280;
/// Between the two buttons (`501:35577` at 172, after 160).
const BETWEEN_THE_BUTTONS: u32 = 12;
/// Beside a button's label, the *Open with* sheet's.
const BESIDE_A_LABEL: u32 = 18;
/// The keyboard's ring, outside a button's edge, the GitHub sheet's.
const THE_RING_IS_OUTSIDE: u32 = 3;
/// The least the display leaves beside the sheet.
const BESIDE_THE_SHEET: u32 = 16;
/// The narrowest column the sheet's words are laid out in.
const THE_NARROWEST_COLUMN: u32 = 240;

/// The sheet drawn, for one display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ApplicationSheetPicture {
    /// The display size it was laid out for.
    pub(crate) size: (i32, i32),
    /// The sheet and everything on it, as one raster.
    pub(crate) washed: Washed,
    /// The sheet itself, on the display.
    pub(crate) sheet: Area,
    /// Every length and button, where it is drawn on the display.
    pub(crate) targets: Vec<(OnTheApplicationSheet, Area)>,
    /// Every piece of text drawn, for a test to read back.
    pub(crate) texts: Vec<String>,
}

/// The words a length is drawn with: its name, and the line under it.
pub(crate) const fn the_words_of(how_long: HowLongForAnApplication) -> (Word, Word) {
    match how_long {
        HowLongForAnApplication::ThisTask => (words::THIS_TASK, words::UNTIL_THIS_TASK_ENDS),
        HowLongForAnApplication::ThisSession => (words::THIS_SESSION, words::UNTIL_YOU_SIGN_OUT),
        HowLongForAnApplication::Remember => (words::REMEMBER, words::UNTIL_YOU_TAKE_IT_BACK),
    }
}

/// **The sheet for a display of `size` at `display`.**
///
/// # Errors
/// [`RenderError::ApprovalScene`] when the display is too small to hold the
/// sheet whole, or the person's accent is not one they may choose — and the
/// sheet is then answered *not now* rather than hold every key with nothing on
/// the screen.
pub(crate) fn drawn(
    shows: ApplicationSheetShows<'_>,
    strings: &Strings,
    labels: &mut WindowControlLabels,
    size: (i32, i32),
    look: SettingsLook,
    display: DisplayScale,
) -> Result<ApplicationSheetPicture, RenderError> {
    let palette = SettingsPalette::of(look).map_err(|_| RenderError::ApprovalScene)?;
    let measure = Measure::of(look.scale).on(display);
    let px = |logical: u32| measure.px(i32::try_from(logical).unwrap_or(i32::MAX));
    let wide = px(THE_SHEET_IS_WIDE).min(size.0 - 2 * px(BESIDE_THE_SHEET));
    let inside = px(INSIDE_THE_SHEET);
    let column = wide - 2 * inside;
    if column < px(THE_NARROWEST_COLUMN) {
        return Err(RenderError::ApprovalScene);
    }
    let named = Filling::of(words::THE_APPLICATION, shows.wanted.called().to_owned());
    let say = |word: Word, filling: &Filling| strings.say(&word.key(), filling).into_text();
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
    let mut targets: Vec<(OnTheApplicationSheet, Area)> = Vec::new();

    // **The heading and the line under it.**
    let heading = say(words::LET_ALO_USE, &named);
    let shaped = laying.words(
        &heading,
        TheType::Title,
        column,
        (palette.surface, palette.ink),
    );
    let mut y = text(&mut out, inside, inside, shaped, &heading);
    y += px(BETWEEN);
    let line = say(words::ALO_ASKED, &named);
    let shaped = laying.words(
        &line,
        TheType::Body,
        column,
        (palette.surface, palette.secondary),
    );
    y = text(&mut out, inside, y, shaped, &line);

    // **The three lengths, each in its own card.**
    for how_long in HowLongForAnApplication::ALL {
        y += px(BETWEEN);
        let (name, under) = the_words_of(how_long);
        let mut row = ARow::named(
            None,
            TheKind::AChoice {
                chosen: how_long == shows.focus,
            },
            say(name, &Filling::nothing()),
        );
        row.lines.push(say(under, &Filling::nothing()));
        let card_fill_at = out.drawn.len();
        let bottom = crate::settings_drawn_row::a_row(&mut out, &mut laying, y, &row, y);
        let card = Rectangle::new((inside, y).into(), (column, bottom - y).into());
        out.drawn.insert(
            card_fill_at,
            Drawn::Fill(card, m::A_CARDS_CORNER, palette.surface),
        );
        out.drawn.push(Drawn::Border(
            card,
            m::A_CARDS_CORNER,
            if how_long == shows.focus {
                palette.ink
            } else {
                palette.edge
            },
        ));
        targets.push((OnTheApplicationSheet::For(how_long), card));
        y = bottom;
    }

    // **The note, naming Escape.**
    y += px(BETWEEN);
    let note = say(words::ONLY_THIS_APPLICATION, &named);
    let shaped = laying.words(
        &note,
        TheType::Body,
        column,
        (palette.surface, palette.secondary),
    );
    y = text(&mut out, inside, y, shaped, &note);
    y += px(BETWEEN);

    // **The buttons, from the left**: *Not now*, then the navy one.
    let mut left_edge = inside;
    for (on, word, least, navy) in [
        (
            OnTheApplicationSheet::NotNow,
            words::NOT_NOW,
            NOT_NOW_IS_AT_LEAST,
            false,
        ),
        (
            OnTheApplicationSheet::LetAloUseIt,
            words::LET_ALO_USE_IT,
            THE_NAVY_ONE_IS_AT_LEAST,
            true,
        ),
    ] {
        let label_words = say(word, &named);
        let (ground, ink) = if navy {
            (palette.ink, palette.surface)
        } else {
            (palette.surface, palette.ink)
        };
        let label = laying
            .words(&label_words, TheType::Label, column, (ground, ink))
            .cropped_to_its_ink();
        let button_wide = (label.width_of_box + 2 * px(BESIDE_A_LABEL)).max(px(least));
        let area = Rectangle::new((left_edge, y).into(), (button_wide, px(m::A_TARGET)).into());
        if area.loc.x + area.size.w > inside + column {
            return Err(RenderError::ApprovalScene);
        }
        out.drawn.push(Drawn::Fill(area, m::A_CARDS_CORNER, ground));
        if !navy {
            out.drawn
                .push(Drawn::Border(area, m::A_CARDS_CORNER, palette.edge));
        }
        let x = area.loc.x + (area.size.w - label.width_of_box) / 2;
        let at = area.loc.y + (area.size.h - label.height) / 2;
        text(&mut out, x, at, label, &label_words);
        targets.push((on, area));
        left_edge = area.loc.x + area.size.w + px(BETWEEN_THE_BUTTONS);
    }
    // **Where Enter lands**: the navy button is ringed, because Enter grants
    // the length the keyboard is on through it.
    if let Some((_, navy)) = targets
        .iter()
        .find(|(on, _)| *on == OnTheApplicationSheet::LetAloUseIt)
    {
        let outside = px(THE_RING_IS_OUTSIDE);
        out.drawn.push(Drawn::Border(
            Rectangle::new(
                (navy.loc.x - outside, navy.loc.y - outside).into(),
                (navy.size.w + 2 * outside, navy.size.h + 2 * outside).into(),
            ),
            m::A_CARDS_CORNER + THE_RING_IS_OUTSIDE,
            palette.ink,
        ));
    }
    let tall = y + px(m::A_TARGET) + inside;
    if tall > size.1 - 2 * px(BESIDE_THE_SHEET) {
        return Err(RenderError::ApprovalScene);
    }

    // **Then placed**, centred on the display, raised and painted.
    let left = (size.0 - wide) / 2;
    let top = (size.1 - tall) / 2;
    let sheet = Rectangle::new((left, top).into(), (wide, tall).into());
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
    let moved: Vec<Drawn> = out
        .drawn
        .into_iter()
        .map(|one| crate::the_open_with_sheet_drawn::shifted(one, left))
        .collect();
    painted(&moved, &mut canvas, top, measure.factor());
    let at = |area: Area| Rectangle::new((area.loc.x + left, area.loc.y + top).into(), area.size);
    Ok(ApplicationSheetPicture {
        size,
        washed: canvas.washed(),
        sheet,
        targets: targets
            .into_iter()
            .map(|(on, area)| (on, at(area)))
            .collect(),
        texts: out.texts.into_iter().map(|one| one.text).collect(),
    })
}

#[cfg(test)]
#[path = "the_application_sheet_drawn_tests.rs"]
mod tests;
