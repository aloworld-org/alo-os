//! The sheet where a person picks the language alo OS speaks to them in.
//!
//! Frame `09b · Choose language`, drawn on the composition six captured frames
//! share — `crate::setup_sheet_layout` for where things go and
//! `crate::setup_sheet_surface` for the cards under them. Nothing about the
//! geometry is this file's; what is this file's is which words go in which
//! box, and which row wears the chosen border.
//!
//! # The two lines of a row
//!
//! A row reads the language's own name over its English one — `Nederlands`
//! above `Dutch`, `Ελληνικά` above `Greek`. The first identifies the row and
//! is drawn in the ink at 16/25; the second is `text/secondary` at 14/22 and
//! is there for somebody who knows only the English name. The design settled
//! that and it is better than showing either alone, which is what
//! `crate::the_languages_offered` was first written to do.
//!
//! # The chosen row says so in a word as well as a border
//!
//! `docs/design/the-first-start.md`: *selecting is shown by a navy border
//! **and the word `Selected`** — never colour alone*, and **teal never means
//! chosen**. So the chosen row's first line gains ` · Selected` and its card
//! gains the two-pixel border, and a person who cannot see the border reads
//! the word. Both come from one answer — `chosen_in_the_window` — so the word
//! and the border cannot disagree about which row it is.
//!
//! # No introduction, which is a shape the layout already knows
//!
//! `09b` has no introduction: its choices begin at 136, where an introduction
//! would have been, with no gap left for it. `setup_sheet_laid_out` already
//! takes a height of zero as *there is none* rather than as a missing figure,
//! which `a-sheet-without-a-consequence` built for the consequence and this
//! reuses unchanged.

use cosmic_text::FontSystem;
use smithay::utils::{Rectangle, Scale};

use crate::a_sheet_that_fits::{ItFits, the_sheet_that_fits};
use crate::painted_text::sentence;
use crate::setup_sheet_layout::TheTextIsTall;
use crate::sign_in_raster::{Measure, SignInPicture};
use crate::sign_in_surface::TheColours;
use crate::the_languages_offered::TheLanguagesOffered;
use crate::{RenderError, first_start_words as say};

/// What this sheet is showing.
#[derive(Debug)]
pub(crate) struct LanguageSheet<'a> {
    /// The languages, where the cursor is, and what has been chosen.
    pub(crate) offered: &'a TheLanguagesOffered,
}

/// Draw the language sheet.
///
/// # Errors
/// [`RenderError::SignInScene`] for a screen too small to hold a sheet, which
/// is a refusal rather than a column drawn over its own footer.
pub(crate) fn drawn(
    sheet: &LanguageSheet<'_>,
    fonts: &mut FontSystem,
    words: &alo_strings::Strings,
    size: (i32, i32),
    colours: TheColours,
    measure: &Measure,
) -> Result<SignInPicture, RenderError> {
    let height = size.1;
    let nothing = alo_strings::Filling::nothing();
    let say_it = |word: alo_strings::Word| words.say(&word.key(), &nothing);

    let eyebrow = say_it(say::LANGUAGE_EYEBROW);
    let title = say_it(say::LANGUAGE_TITLE);
    let go_on = say_it(say::USE_THIS_LANGUAGE);
    let go_back = say_it(say::BACK);
    let selected = say_it(say::SELECTED);

    // **Shaped before it is placed**, because the heading wraps in a language
    // this code has never seen and its height is what decides where the rows
    // begin — and **fitted to the screen**, so a short one shows fewer rows of
    // the list rather than no sheet (`crate::a_sheet_that_fits`).
    let Some(ItFits {
        laid,
        title: shaped_title,
        rows: fit,
    }) = the_sheet_that_fits(
        fonts,
        title.text(),
        colours,
        measure,
        size,
        TheTextIsTall {
            title: 0,
            // No introduction and no consequence on this sheet; a height of
            // zero is how the layout is told there is none.
            introduction: 0,
            consequence: 0,
            choices: sheet.offered.window().len(),
            actions: 2,
            rows: crate::setup_sheet_layout::TheRowsAre::Apart,
        },
        1,
    )
    else {
        return Err(RenderError::SignInScene);
    };
    let in_sight = sheet.offered.window_of(fit);
    let rows = in_sight.rows;

    let mut drawn = SignInPicture {
        size,
        surface: crate::setup_sheet_surface::setup_sheet_surface(
            &laid.logical,
            size,
            colours,
            crate::setup_sheet_surface::TheMarks {
                chosen: in_sight.chosen,
                // **Both marks, from one state.** The ring says where a
                // person is and the border says what they picked; drawing
                // only the second is what left an arrow key with nothing to
                // show for itself.
                ring: match sheet.offered.ring() {
                    crate::the_languages_offered::WhereTheRingIs::OnTheList => in_sight
                        .cursor
                        .map(crate::setup_sheet_surface::TheRing::OnAChoice),
                    crate::the_languages_offered::WhereTheRingIs::OnTheWayOn => {
                        Some(crate::setup_sheet_surface::TheRing::OnAnAction(0))
                    }
                },
            },
            Scale::from(measure.display().factor()),
        ),
        solids: Vec::new(),
        inked: Vec::new(),
        controls: Vec::new(),
    };
    let at = laid.pixels;
    // **What a press can land on is what was just drawn**, in its pixels.
    drawn.controls =
        crate::a_press_on_a_setup_sheet::the_controls_of(&at, |which| in_sight.first + which);

    crate::painted_text::line(
        &mut drawn.inked,
        height,
        fonts,
        say_it(say::STAGE).text(),
        at.stage,
        measure.label_metrics(),
        colours.ground,
        colours.muted,
    );
    crate::painted_text::line(
        &mut drawn.inked,
        height,
        fonts,
        eyebrow.text(),
        at.eyebrow,
        measure.label_metrics(),
        colours.ground,
        colours.muted,
    );
    // Drawn from the shape measured above rather than shaped twice, so what
    // was measured is what appears.
    drawn.inked.extend(shaped_title.cropped_to_its_ink().placed(
        at.title.loc.x,
        at.title.loc.y,
        height,
    ));

    // The rows, each its own name over its English one.
    let inside = measure.px(20);
    for (which, card) in at.choices.iter().enumerate() {
        let Some(row) = rows.get(which) else { continue };
        // **One answer decides the word and the border.** Both read
        // `chosen_in_the_window`, so a row cannot wear the border without the
        // word or the word without the border.
        let chosen = in_sight.chosen == Some(which);
        let first = if chosen {
            format!("{}  ·  {}", row.calls_itself, selected.text())
        } else {
            row.calls_itself.to_owned()
        };
        crate::painted_text::line(
            &mut drawn.inked,
            height,
            fonts,
            &first,
            Rectangle::new(
                (card.loc.x + inside, card.loc.y + measure.px(16)).into(),
                (card.size.w - 2 * inside, measure.px(25)).into(),
            ),
            measure.metrics(),
            colours.entry,
            colours.ink,
        );
        crate::painted_text::line(
            &mut drawn.inked,
            height,
            fonts,
            row.in_english,
            Rectangle::new(
                (card.loc.x + inside, card.loc.y + measure.px(45)).into(),
                (card.size.w - 2 * inside, measure.px(22)).into(),
            ),
            measure.label_metrics(),
            colours.entry,
            colours.secondary,
        );
    }

    // The way on is filled, so its label is drawn in the ground; the way back
    // is an outline, so its label is drawn in the ink.
    for (button, (text, on_the_ink)) in at
        .actions
        .iter()
        .zip([(go_on.text(), true), (go_back.text(), false)])
    {
        let (ground, ink) = if on_the_ink {
            (colours.ink, colours.ground)
        } else {
            (colours.entry, colours.ink)
        };
        let shaped = sentence(
            fonts,
            text,
            button.size.w,
            measure.label_metrics(),
            ground,
            ink,
        )
        .cropped_to_its_ink();
        let x = button.loc.x + (button.size.w - shaped.width).max(0) / 2;
        let y = button.loc.y + (button.size.h - shaped.height).max(0) / 2;
        drawn.inked.extend(shaped.placed(x, y, height));
    }

    crate::painted_text::line(
        &mut drawn.inked,
        height,
        fonts,
        say_it(say::FOOTER).text(),
        at.footer,
        measure.label_metrics(),
        colours.ground,
        colours.muted,
    );

    Ok(drawn)
}

#[cfg(test)]
#[path = "language_sheet_tests.rs"]
mod tests;
