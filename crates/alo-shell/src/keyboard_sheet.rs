//! The sheet where a person picks the keyboard they type on — frame `09a ·
//! Keyboard choice`: three keyboards, each its name over the country it is
//! named for, the chosen one wearing the border and the word *Selected*; then
//! *Test your keyboard*, where what is typed appears.
//!
//! Which keyboards, and how a fourth is reached, are
//! `crate::the_keyboard_picker`'s; their names are `xkeyboard-config`'s in the
//! person's language, and their countries CLDR's.

use cosmic_text::FontSystem;

use crate::setup_sheet_painting::{TheSheetSays, a_sheet, two_lines};
use crate::setup_sheet_surface::{TheMarks, TheRing as TheCardRing};
use crate::sign_in_raster::{Measure, SignInPicture};
use crate::sign_in_surface::TheColours;
use crate::the_first_start::TheFirstStart;
use crate::the_keyboard_picker::{TheKeyboardPicker, TheRing};
use crate::{RenderError, first_start_words as say};

/// Draw the keyboard sheet.
///
/// # Errors
/// [`RenderError::SignInScene`] for a screen too small to hold a sheet.
pub(crate) fn drawn(
    first_start: &TheFirstStart,
    picker: &TheKeyboardPicker,
    fonts: &mut FontSystem,
    words: &alo_strings::Strings,
    size: (i32, i32),
    colours: TheColours,
    measure: &Measure,
) -> Result<SignInPicture, RenderError> {
    let nothing = alo_strings::Filling::nothing();
    let say_it = |word: alo_strings::Word| words.say(&word.key(), &nothing).into_text();
    let (eyebrow, title, introduction) = (
        say_it(say::KEYBOARD_EYEBROW),
        say_it(say::KEYBOARD_TITLE),
        say_it(say::KEYBOARD_INTRODUCTION),
    );
    let (go_on, go_back) = (say_it(say::USE_THIS_KEYBOARD), say_it(say::BACK));
    // **As many keyboards as the screen has room for, and never none.** Three
    // keyboards, the test row, a heading and an introduction need 730 logical
    // pixels down and a 1280 × 800 screen has 716 above *Accessibility*, so on
    // main this sheet was refused there — and a refused frame ended the
    // sign-in screen with nobody signed in. A shorter screen draws fewer
    // keyboards at a time; the window follows the cursor, so the arrows still
    // reach every one, and the test row is always drawn.
    let mut shown = picker.window().len();
    let (mut drawn, at, in_sight) = loop {
        let in_sight = picker.window_of(shown);
        let the_test = in_sight.rows.len();
        let ring = match picker.ring() {
            TheRing::OnTheKeyboards => in_sight.cursor.map(TheCardRing::OnAChoice),
            TheRing::OnTheTest => Some(TheCardRing::OnAChoice(the_test)),
            TheRing::OnTheWayOn => Some(TheCardRing::OnAnAction(0)),
        };
        match a_sheet(
            &TheSheetSays {
                eyebrow: &eyebrow,
                title: &title,
                introduction: &introduction,
                actions: [&go_on, &go_back],
            },
            the_test + 1,
            TheMarks {
                chosen: in_sight.chosen,
                ring,
            },
            fonts,
            words,
            size,
            colours,
            measure,
        ) {
            Ok((drawn, at)) => break (drawn, at, in_sight),
            Err(_) if shown > 1 => shown -= 1,
            Err(refused) => return Err(refused),
        }
    };
    let (window, chosen) = (in_sight.rows, in_sight.chosen);
    // **A press names the keyboard it is on**, by where it is in the whole
    // list, because the window drawn depends on the room there was.
    drawn.controls = crate::a_press_on_a_setup_sheet::the_controls_of(&at, |which| {
        if which < window.len() {
            in_sight.first + which
        } else {
            TheKeyboardPicker::THE_TEST
        }
    });

    let language = first_start.the_language();
    let selected = say_it(say::SELECTED);
    for (which, card) in at.choices.iter().enumerate() {
        let lines = match window.get(which) {
            Some(keyboard) => {
                let name = first_start.the_name_of(keyboard);
                // **One answer decides the word and the border**, as on the
                // language sheet: both read `chosen_in_the_window`.
                let first = if chosen == Some(which) {
                    format!("{name}  ·  {selected}")
                } else {
                    name
                };
                let country = alo_keyboards::TheKeyboardNames::its_country(keyboard)
                    .and_then(|code| alo_formats::region_names::the_name_in(&code, language))
                    .unwrap_or_default();
                (first, country)
            }
            None => (
                say_it(say::TEST_YOUR_KEYBOARD),
                if picker.typed().is_empty() {
                    say_it(say::TEST_HINT)
                } else {
                    picker.typed().to_owned()
                },
            ),
        };
        two_lines(
            &mut drawn,
            fonts,
            *card,
            (&lines.0, &lines.1),
            colours,
            measure,
        );
    }
    Ok(drawn)
}

#[cfg(test)]
#[path = "keyboard_sheet_tests.rs"]
mod tests;
