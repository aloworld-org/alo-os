//! **The GitHub sign-in sheet, laid out for one display** — `sheet-sign-in`
//! and `sheet-signed-in`, approved by the owner on 2026-11-13 *"as drawn"*
//! (`docs/design/github-sign-in.md`, *The sheet*).
//!
//! # Drawn as the *Open with* sheet is
//!
//! The proposal measured itself off the *Open with* sheet
//! (`crate::the_open_with_sheet_drawn`), so it is laid out by the same parts:
//! Settings' type, palette and risk box (`crate::settings_drawn_controls::a_risk`),
//! painted onto a `crate::settings_canvas::Canvas`, raised with the FLOATING
//! shadow and centred on the display. **Nothing behind it dims** — the owner's
//! ruling, *"No dimming, like Open with"*, over the proposal's pictures.
//!
//! Every figure is a logical measure taken through the display's scale and the
//! person's text size (`CLAUDE.md`, *nothing is built to one screen size*); the
//! sheet is never wider than the display allows, and a display that cannot
//! hold it whole is refused rather than drawn in part.
//!
//! # Where the keyboard is, is drawn
//!
//! The button the keyboard is on is ringed in `text/primary`, a hairline
//! outside its edge, so a ring on the navy button is not lost against it.

use alo_strings::{Filling, Strings, Word};
use smithay::utils::Rectangle;

use crate::desktop_look::Measure;
use crate::display_scale::DisplayScale;
use crate::painted::Washed;
use crate::painted_text::Shaped;
use crate::settings_canvas::{AShadow, Area, Canvas};
use crate::settings_drawn_page::{Drawn, Laying, ThePageDrawn, painted, text};
use crate::settings_measures as m;
use crate::settings_page::ARisk;
use crate::settings_palette::SettingsPalette;
use crate::settings_type::TheType;
use crate::the_github_sheet::{GitHubSheetShows, OnTheGitHubSheet, TheSheetIs};
use crate::the_github_sheets_words as words;
use crate::the_settings_icons::ASettingsIcon;
use crate::the_settings_words as w;
use crate::{RenderError, SettingsLook, WindowControlLabels};

/// The sheet's width, the *Open with* sheet's (`25:57`).
const THE_SHEET_IS_WIDE: u32 = 480;
/// Inside the sheet, to everything on it.
const INSIDE_THE_SHEET: u32 = 25;
/// From the title to the line under it.
const UNDER_THE_TITLE: u32 = 14;
/// From the line to the code's card, and — signed in — to the risk.
const ABOVE_THE_CARD: u32 = 11;
/// The code's card: 16 above and below a 48 row.
const THE_CARD_IS_TALL: u32 = 16 + 48 + 16;
/// Inside the code's card, before the code.
const BEFORE_THE_CODE: u32 = 20;
/// Inside the code's card, above, below and after *Copy*.
const AROUND_COPY: u32 = 16;
/// From the card to the risk, and from the risk to the buttons.
const ABOVE_THE_RISK_AND_THE_BUTTONS: u32 = 14;
/// Between *Cancel* and the navy button.
const BETWEEN_THE_BUTTONS: u32 = 8;
/// Beside a footer button's label (`the_open_with_sheet_drawn`'s).
const BESIDE_A_LABEL: u32 = 18;
/// The check before *Signed in.*, and the room after it.
const THE_CHECK: u32 = 20;
/// After the check.
const AFTER_THE_CHECK: u32 = 8;
/// The keyboard's ring, outside a button's edge.
const THE_RING_IS_OUTSIDE: u32 = 3;
/// The least the display leaves beside the sheet.
const BESIDE_THE_SHEET: u32 = 16;
/// The narrowest column the sheet's words are laid out in.
const THE_NARROWEST_COLUMN: u32 = 240;

/// The sheet drawn, for one display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GitHubSheetPicture {
    /// The display size it was laid out for.
    pub(crate) size: (i32, i32),
    /// The sheet and everything on it, as one raster.
    pub(crate) washed: Washed,
    /// The sheet itself, on the display.
    pub(crate) sheet: Area,
    /// Every button, where it is drawn on the display.
    pub(crate) buttons: Vec<(OnTheGitHubSheet, Area)>,
    /// Every piece of text drawn and where, for a test to read back.
    pub(crate) texts: Vec<(String, Area)>,
}

/// **The sheet for a display of `size` at `display`.**
///
/// # Errors
/// [`RenderError::ApprovalScene`] when the display is too small to hold the
/// sheet whole, or the person's accent is not one they may choose — and the
/// sign-in is then stopped rather than every key held with nothing on the
/// screen (`crate::the_github_sheet::TheGitHubSheet::could_not_be_drawn`).
pub(crate) fn drawn(
    shows: GitHubSheetShows<'_>,
    strings: &Strings,
    labels: &mut WindowControlLabels,
    size: (i32, i32),
    look: SettingsLook,
    display: DisplayScale,
) -> Result<GitHubSheetPicture, RenderError> {
    let palette = SettingsPalette::of(look).map_err(|_| RenderError::ApprovalScene)?;
    let measure = Measure::of(look.scale).on(display);
    let px = |logical: u32| measure.px(i32::try_from(logical).unwrap_or(i32::MAX));
    let wide = px(THE_SHEET_IS_WIDE).min(size.0 - 2 * px(BESIDE_THE_SHEET));
    let inside = px(INSIDE_THE_SHEET);
    let column = wide - 2 * inside;
    if column < px(THE_NARROWEST_COLUMN) {
        return Err(RenderError::ApprovalScene);
    }
    let said = |word: Word| strings.say(&word.key(), &Filling::nothing()).into_text();
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
    let mut buttons: Vec<(OnTheGitHubSheet, Area)> = Vec::new();

    // **The title**: *Sign in to GitHub*, or who is signed in.
    let title = match shows.is {
        TheSheetIs::SignedInAs(login) => strings
            .say(
                &words::SIGNED_IN_TO_GITHUB_AS.key(),
                &Filling::of("login", login.clone()),
            )
            .into_text(),
        TheSheetIs::ACommandNeededIt
        | TheSheetIs::WaitingForTheCode
        | TheSheetIs::TheCodeIs { .. } => said(words::SIGN_IN_TO_GITHUB),
    };
    let shaped = laying.words(
        &title,
        TheType::Title,
        column,
        (palette.surface, palette.ink),
    );
    let mut y = text(&mut out, inside, inside, shaped, &title);
    y += px(UNDER_THE_TITLE);

    match shows.is {
        // **Signed in**: a check in `status/positive`, and where to sign out.
        TheSheetIs::SignedInAs(_) => {
            let sign = px(THE_CHECK);
            let after = sign + px(AFTER_THE_CHECK);
            let line = said(words::SIGNED_IN_AND_WHERE_TO_SIGN_OUT);
            let shaped = laying.words(
                &line,
                TheType::Body,
                column - after,
                (palette.surface, palette.secondary),
            );
            let first_line = px(TheType::Body.line()).min(shaped.height);
            out.drawn.push(Drawn::Icon(
                ASettingsIcon::CircleCheck,
                Rectangle::new(
                    (inside, y + (first_line - sign) / 2).into(),
                    (sign, sign).into(),
                ),
                palette.positive,
            ));
            y = text(&mut out, inside + after, y, shaped, &line);
            y += px(ABOVE_THE_CARD);
        }
        // **It rose by itself**: what failed and that alo will not try again,
        // and no card — nothing has started, so there is no code to show.
        TheSheetIs::ACommandNeededIt => {
            let line = said(words::A_COMMAND_NEEDED_IT);
            let shaped = laying.words(
                &line,
                TheType::Body,
                column,
                (palette.surface, palette.secondary),
            );
            y = text(&mut out, inside, y, shaped, &line);
            y += px(ABOVE_THE_RISK_AND_THE_BUTTONS);
        }
        // **Signing in**: the sentence, and the code in its card.
        TheSheetIs::WaitingForTheCode | TheSheetIs::TheCodeIs { .. } => {
            let line = said(w::GITHUB_WILL_ASK_FOR_THIS_CODE);
            let shaped = laying.words(
                &line,
                TheType::Body,
                column,
                (palette.surface, palette.secondary),
            );
            y = text(&mut out, inside, y, shaped, &line);
            y += px(ABOVE_THE_CARD);
            let card = Rectangle::new((inside, y).into(), (column, px(THE_CARD_IS_TALL)).into());
            out.drawn
                .push(Drawn::Fill(card, m::A_CARDS_CORNER, palette.surface));
            out.drawn
                .push(Drawn::Border(card, m::A_CARDS_CORNER, palette.edge));
            let middle = y + card.size.h / 2;
            let before = inside + px(BEFORE_THE_CODE);
            match shows.is {
                TheSheetIs::TheCodeIs { code, .. } => {
                    let copy_words = said(words::COPY);
                    let label = laying
                        .words(
                            &copy_words,
                            TheType::Label,
                            column,
                            Ground::Quiet.colours(&palette),
                        )
                        .cropped_to_its_ink();
                    let copy_wide = label.width_of_box + 2 * px(m::BESIDE_A_BUTTONS_LABEL);
                    let copy = Rectangle::new(
                        (
                            inside + column - px(AROUND_COPY) - copy_wide,
                            y + px(AROUND_COPY),
                        )
                            .into(),
                        (copy_wide, px(m::A_TARGET)).into(),
                    );
                    a_button(
                        &mut out,
                        (copy, Ground::Quiet, &palette),
                        label,
                        &copy_words,
                    );
                    buttons.push((OnTheGitHubSheet::Copy, copy));
                    let room = copy.loc.x - px(AROUND_COPY) - before;
                    if room <= 0 {
                        return Err(RenderError::ApprovalScene);
                    }
                    let shaped =
                        laying.words(code, TheType::ACode, room, (palette.surface, palette.ink));
                    let at = middle - shaped.height / 2;
                    text(&mut out, before, at, shaped, code);
                }
                // **Waiting for the code**: *Checking…*, the word the Accounts
                // page says while it waits too, and no *Copy* — there is
                // nothing yet to copy.
                TheSheetIs::ACommandNeededIt
                | TheSheetIs::WaitingForTheCode
                | TheSheetIs::SignedInAs(_) => {
                    let checking = said(w::CHECKING);
                    let shaped = laying.words(
                        &checking,
                        TheType::Body,
                        column - 2 * px(BEFORE_THE_CODE),
                        (palette.surface, palette.muted),
                    );
                    let at = middle - shaped.height / 2;
                    text(&mut out, before, at, shaped, &checking);
                }
            }
            y += card.size.h + px(ABOVE_THE_RISK_AND_THE_BUTTONS);
        }
    }

    // **What it risks**, in the owner's words, in both states.
    y = crate::settings_drawn_controls::a_risk(
        &mut out,
        &mut laying,
        (inside, y, column),
        &ARisk {
            heading: said(w::WHAT_IT_RISKS),
            words: said(w::GITHUB_RISKS),
        },
    );
    y += px(ABOVE_THE_RISK_AND_THE_BUTTONS);

    // **The footer's buttons**, right to left: the navy one at the right edge.
    let mut right = inside + column;
    for on in shows.is.buttons().iter().rev() {
        let (label_words, ground) = match on {
            OnTheGitHubSheet::Copy => continue,
            OnTheGitHubSheet::Cancel => (said(words::CANCEL), Ground::Quiet),
            OnTheGitHubSheet::Done => (said(words::DONE), Ground::Navy),
            OnTheGitHubSheet::SignIn => (said(words::SIGN_IN), Ground::Navy),
            OnTheGitHubSheet::OpenThePage => {
                let page = match shows.is {
                    TheSheetIs::TheCodeIs { page, .. } => page.as_str(),
                    TheSheetIs::ACommandNeededIt
                    | TheSheetIs::WaitingForTheCode
                    | TheSheetIs::SignedInAs(_) => "",
                };
                (
                    strings
                        .say(
                            &words::OPEN_THE_PAGE.key(),
                            &Filling::of("page", page.trim_start_matches("https://")),
                        )
                        .into_text(),
                    Ground::Navy,
                )
            }
        };
        let label = laying
            .words(
                &label_words,
                TheType::Label,
                column,
                ground.colours(&palette),
            )
            .cropped_to_its_ink();
        let button_wide = label.width_of_box + 2 * px(BESIDE_A_LABEL);
        let area = Rectangle::new(
            (right - button_wide, y).into(),
            (button_wide, px(m::A_TARGET)).into(),
        );
        if area.loc.x < inside {
            return Err(RenderError::ApprovalScene);
        }
        a_button(&mut out, (area, ground, &palette), label, &label_words);
        buttons.push((*on, area));
        right = area.loc.x - px(BETWEEN_THE_BUTTONS);
    }
    let tall = y + px(m::A_TARGET) + inside;
    if tall > size.1 - 2 * px(BESIDE_THE_SHEET) {
        return Err(RenderError::ApprovalScene);
    }
    // **Where the keyboard is.**
    if let Some((_, focused)) = buttons.iter().find(|(on, _)| *on == shows.focus) {
        let outside = px(THE_RING_IS_OUTSIDE);
        out.drawn.push(Drawn::Border(
            Rectangle::new(
                (focused.loc.x - outside, focused.loc.y - outside).into(),
                (focused.size.w + 2 * outside, focused.size.h + 2 * outside).into(),
            ),
            m::A_CARDS_CORNER + THE_RING_IS_OUTSIDE,
            palette.ink,
        ));
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
    Ok(GitHubSheetPicture {
        size,
        washed: canvas.washed(),
        sheet,
        buttons: buttons
            .into_iter()
            .map(|(on, area)| (on, at(area)))
            .collect(),
        texts: out
            .texts
            .into_iter()
            .map(|one| (one.text, at(one.area)))
            .collect(),
    })
}

/// What a button is drawn on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ground {
    /// The Primary button: `text/primary`, its label in `bg/surface`.
    Navy,
    /// The Quiet button: `bg/surface` with a `border/default` edge.
    Quiet,
}

impl Ground {
    /// The label's ground and ink.
    const fn colours(self, palette: &SettingsPalette) -> ([u8; 3], [u8; 3]) {
        match self {
            Self::Navy => (palette.ink, palette.surface),
            Self::Quiet => (palette.surface, palette.ink),
        }
    }
}

/// One button at `area`: its ground, its edge where it is quiet, and its label
/// in the middle.
fn a_button(
    out: &mut ThePageDrawn,
    (area, ground, palette): (Area, Ground, &SettingsPalette),
    label: Shaped,
    words: &str,
) {
    let x = area.loc.x + (area.size.w - label.width_of_box) / 2;
    let y = area.loc.y + (area.size.h - label.height) / 2;
    let (fill, _) = ground.colours(palette);
    out.drawn.push(Drawn::Fill(area, m::A_CARDS_CORNER, fill));
    if ground == Ground::Quiet {
        out.drawn
            .push(Drawn::Border(area, m::A_CARDS_CORNER, palette.edge));
    }
    text(out, x, y, label, words);
}

#[cfg(test)]
#[path = "the_github_sheet_drawn_tests.rs"]
mod tests;
