//! The sheet where a person says which region the machine is in — frame
//! `09c · Choose region`, on the composition every setup sheet shares
//! (`crate::setup_sheet_layout` for where things go, `crate::setup_sheet_surface`
//! for the cards). What is this file's is which words go in which box.
//!
//! The rows are `crate::the_regions_offered`'s: the design's two —
//! `Belgium · Selected` over *Selected region*, and *Other region* over
//! *Choose another country or region.* — or, behind *Other region*, a window
//! of every region, each named in the person's language, the chosen one
//! wearing the border and the word *Selected* as the language sheet's does.

use cosmic_text::FontSystem;
use smithay::utils::{Rectangle, Scale};

use crate::painted_text::sentence;
use crate::setup_sheet_in_pixels::{laid_out, the_column_in_pixels};
use crate::setup_sheet_layout::TheTextIsTall;
use crate::sign_in_raster::{Measure, SignInPicture};
use crate::sign_in_surface::TheColours;
use crate::the_languages_offered::WhereTheRingIs;
use crate::the_regions_offered::{Showing, TheRegionsOffered};
use crate::{RenderError, first_start_words as say};

/// Draw the region sheet.
///
/// # Errors
/// [`RenderError::SignInScene`] for a screen too small to hold a sheet.
pub(crate) fn drawn(
    offered: &TheRegionsOffered,
    fonts: &mut FontSystem,
    words: &alo_strings::Strings,
    size: (i32, i32),
    colours: TheColours,
    measure: &Measure,
) -> Result<SignInPicture, RenderError> {
    let (width, height) = size;
    let nothing = alo_strings::Filling::nothing();
    let say_it = |word: alo_strings::Word| words.say(&word.key(), &nothing);

    let title = say_it(say::REGION_TITLE);
    let introduction = say_it(say::REGION_INTRODUCTION);
    let selected = say_it(say::SELECTED);

    let column = the_column_in_pixels(width, measure.display());
    let shaped_title = sentence(
        fonts,
        title.text(),
        column,
        measure.title_metrics(),
        colours.ground,
        colours.ink,
    );
    let shaped_introduction = sentence(
        fonts,
        introduction.text(),
        column,
        measure.metrics(),
        colours.ground,
        colours.ink,
    );

    let rows = offered.rows();
    let Some(laid) = laid_out(
        size,
        TheTextIsTall {
            title: shaped_title.height,
            introduction: shaped_introduction.height,
            consequence: 0,
            choices: rows.len(),
            actions: 2,
        },
        measure.display(),
    ) else {
        return Err(RenderError::SignInScene);
    };

    let chosen = offered.chosen_in_the_rows();
    let mut drawn = SignInPicture {
        size,
        surface: crate::setup_sheet_surface::setup_sheet_surface(
            &laid.logical,
            size,
            colours,
            crate::setup_sheet_surface::TheMarks {
                chosen,
                ring: match offered.ring() {
                    WhereTheRingIs::OnTheList => offered
                        .cursor_in_the_rows()
                        .map(crate::setup_sheet_surface::TheRing::OnAChoice),
                    WhereTheRingIs::OnTheWayOn => {
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
    drawn.controls = crate::a_press_on_a_setup_sheet::the_controls_of(&at, |which| {
        offered.window_of(rows.len()).first + which
    });

    for (text, place) in [
        (say_it(say::STAGE), at.stage),
        (say_it(say::REGION_EYEBROW), at.eyebrow),
    ] {
        crate::painted_text::line(
            &mut drawn.inked,
            height,
            fonts,
            text.text(),
            place,
            measure.label_metrics(),
            colours.ground,
            colours.muted,
        );
    }
    drawn.inked.extend(shaped_title.cropped_to_its_ink().placed(
        at.title.loc.x,
        at.title.loc.y,
        height,
    ));
    drawn
        .inked
        .extend(shaped_introduction.cropped_to_its_ink().placed(
            at.introduction.loc.x,
            at.introduction.loc.y,
            height,
        ));

    // **One answer decides the word and the border**, as on the language
    // sheet: both read `chosen_in_the_rows`.
    let inside = measure.px(20);
    for (which, card) in at.choices.iter().enumerate() {
        let Some(row) = rows.get(which) else { continue };
        let (first, second) = match row {
            Some(name) if chosen == Some(which) => (
                format!("{name}  ·  {}", selected.text()),
                match offered.showing() {
                    Showing::TheTwo => say_it(say::SELECTED_REGION).text().to_owned(),
                    Showing::EveryRegion => String::new(),
                },
            ),
            Some(name) => ((*name).to_owned(), String::new()),
            None => (
                say_it(say::OTHER_REGION).text().to_owned(),
                say_it(say::OTHER_REGION_HINT).text().to_owned(),
            ),
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
        if !second.is_empty() {
            crate::painted_text::line(
                &mut drawn.inked,
                height,
                fonts,
                &second,
                Rectangle::new(
                    (card.loc.x + inside, card.loc.y + measure.px(45)).into(),
                    (card.size.w - 2 * inside, measure.px(22)).into(),
                ),
                measure.label_metrics(),
                colours.entry,
                colours.secondary,
            );
        }
    }

    // The way on is filled, so its label is in the ground; the way back is an
    // outline, so its label is in the ink.
    let go_on = say_it(say::USE_THIS_REGION);
    let go_back = say_it(say::BACK);
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
