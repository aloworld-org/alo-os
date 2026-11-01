//! The first sheet a machine with nobody on it shows — frame `09 · Language,
//! region and keyboard`: three rows, each its label over what is set now
//! (*Language* over *English*), then *Continue* and *Back*.
//!
//! What the rows name is `crate::the_first_start`'s answer; the region is
//! named in the person's language by CLDR (`alo_formats::region_names`), and
//! the keyboard by `xkeyboard-config` (`alo_keyboards::TheKeyboardNames`).

use cosmic_text::FontSystem;

use crate::setup_sheet_painting::{TheSheetSays, a_grouped_row, a_sheet};
use crate::setup_sheet_surface::{TheMarks, TheRing};
use crate::sign_in_raster::{Measure, SignInPicture};
use crate::sign_in_surface::TheColours;
use crate::the_first_start::{Row, TheFirstStart};
use crate::the_first_starts_icons::AFirstStartIcon;
use crate::the_languages_offered::WhereTheRingIs;
use crate::{RenderError, first_start_words as say};

/// How tall each of the overview's rows is: `562:364`'s 72.
const THE_ROW_IS_TALL: i32 = 72;

/// Draw the overview.
///
/// # Errors
/// [`RenderError::SignInScene`] for a screen too small to hold a sheet.
pub(crate) fn drawn(
    first_start: &TheFirstStart,
    fonts: &mut FontSystem,
    words: &alo_strings::Strings,
    size: (i32, i32),
    colours: TheColours,
    measure: &Measure,
) -> Result<SignInPicture, RenderError> {
    let nothing = alo_strings::Filling::nothing();
    let say_it = |word: alo_strings::Word| words.say(&word.key(), &nothing).into_text();
    let (eyebrow, title, introduction) = (
        say_it(say::LANGUAGE_EYEBROW),
        say_it(say::OVERVIEW_TITLE),
        say_it(say::OVERVIEW_INTRODUCTION),
    );
    let (go_on, go_back) = (say_it(say::CONTINUE), say_it(say::BACK));
    let note = say_it(say::OVERVIEW_NOTE);
    let on = Row::ALL
        .iter()
        .position(|it| *it == first_start.row())
        .unwrap_or_default();
    // The window of rows in sight, once the sheet has said how many fit.
    let in_sight = |rows| crate::a_window_onto_a_list::a_window(&Row::ALL, on, None, rows);
    let ring = |rows| match first_start.ring() {
        WhereTheRingIs::OnTheList => in_sight(rows).cursor.map(TheRing::OnAChoice),
        WhereTheRingIs::OnTheWayOn => Some(TheRing::OnAnAction(0)),
    };
    let (mut drawn, at, rows) = a_sheet(
        &TheSheetSays {
            eyebrow: &eyebrow,
            title: &title,
            introduction: &introduction,
            actions: [&go_on, &go_back],
            rows: crate::setup_sheet_layout::TheRowsAre::Grouped(THE_ROW_IS_TALL),
            note: Some((AFirstStartIcon::Info, &note)),
        },
        (Row::ALL.len(), 1),
        |rows| TheMarks {
            chosen: None,
            ring: ring(rows),
            fields: crate::setup_sheet_surface::TheFields::None,
        },
        fonts,
        words,
        size,
        colours,
        measure,
    )?;

    let window = in_sight(rows);
    drawn.controls =
        crate::a_press_on_a_setup_sheet::the_controls_of(&at, |which| window.first + which);
    let language = first_start.the_language();
    for (row, card) in window.rows.iter().zip(at.choices.iter()) {
        let (label, set_to) = match row {
            Row::Language => (
                say_it(say::ROW_LANGUAGE),
                first_start.the_language_calls_itself().to_owned(),
            ),
            Row::Region => {
                let region = first_start
                    .the_region()
                    .and_then(|it| alo_formats::region_names::the_name_in(it.code(), language))
                    .unwrap_or_default();
                // **The zone beside the region, where the region has several**
                // — so a person can see which they chose. The design draws the
                // region alone because its sample has one zone.
                let set_to = match first_start.the_zone_named() {
                    Some(zone) => format!("{region}  ·  {zone}"),
                    None => region,
                };
                (say_it(say::ROW_REGION), set_to)
            }
            Row::Keyboard => (
                say_it(say::ROW_KEYBOARD),
                first_start.the_name_of(&first_start.the_keyboard()),
            ),
        };
        let icon = match row {
            Row::Language => AFirstStartIcon::Language,
            Row::Region => AFirstStartIcon::Region,
            Row::Keyboard => AFirstStartIcon::Keyboard,
        };
        a_grouped_row(
            &mut drawn,
            fonts,
            *card,
            (icon, &label, &set_to),
            colours,
            measure,
        );
    }
    Ok(drawn)
}

#[cfg(test)]
#[path = "overview_sheet_tests.rs"]
mod tests;
