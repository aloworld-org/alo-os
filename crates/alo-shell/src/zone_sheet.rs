//! The sheet that asks which time zone, in a region that has several —
//! `crate::the_zone_picker`'s rows, each a zone's exemplar city in the
//! person's language, the chosen one wearing the border and the word
//! *Selected*.
//!
//! **No frame of the design draws it.** It is the setup sheets' composition,
//! lettered through `crate::setup_sheet_painting` as the overview and the
//! keyboard sheet are, with the region sheet's eyebrow because it is the same
//! step: where the machine is.

use cosmic_text::FontSystem;

use crate::setup_sheet_painting::{TheSheetSays, a_sheet, two_lines};
use crate::setup_sheet_surface::{TheMarks, TheRing};
use crate::sign_in_raster::{Measure, SignInPicture};
use crate::sign_in_surface::TheColours;
use crate::the_languages_offered::WhereTheRingIs;
use crate::the_zone_picker::TheZonePicker;
use crate::{RenderError, first_start_words as say};

/// Draw the zone sheet.
///
/// # Errors
/// [`RenderError::SignInScene`] for a screen too small to hold a sheet.
pub(crate) fn drawn(
    picker: &TheZonePicker,
    fonts: &mut FontSystem,
    words: &alo_strings::Strings,
    size: (i32, i32),
    colours: TheColours,
    measure: &Measure,
) -> Result<SignInPicture, RenderError> {
    let nothing = alo_strings::Filling::nothing();
    let say_it = |word: alo_strings::Word| words.say(&word.key(), &nothing).into_text();
    let (eyebrow, title, introduction) = (
        say_it(say::REGION_EYEBROW),
        say_it(say::ZONE_TITLE),
        say_it(say::ZONE_INTRODUCTION),
    );
    let (go_on, go_back) = (say_it(say::USE_THIS_ZONE), say_it(say::BACK));
    let window = picker.window();
    let ring = match picker.ring() {
        WhereTheRingIs::OnTheList => picker.cursor_in_the_window().map(TheRing::OnAChoice),
        WhereTheRingIs::OnTheWayOn => Some(TheRing::OnAnAction(0)),
    };
    let chosen = picker.chosen_in_the_window();
    let (mut drawn, at) = a_sheet(
        &TheSheetSays {
            eyebrow: &eyebrow,
            title: &title,
            introduction: &introduction,
            actions: [&go_on, &go_back],
        },
        window.len(),
        TheMarks { chosen, ring },
        fonts,
        words,
        size,
        colours,
        measure,
    )?;
    let selected = say_it(say::SELECTED);
    for (which, (zone, card)) in window.iter().zip(at.choices.iter()).enumerate() {
        // **One answer decides the word and the border**, as on every sheet.
        let first = if chosen == Some(which) {
            format!("{}  ·  {selected}", zone.name)
        } else {
            zone.name.clone()
        };
        two_lines(&mut drawn, fonts, *card, (&first, ""), colours, measure);
    }
    Ok(drawn)
}
