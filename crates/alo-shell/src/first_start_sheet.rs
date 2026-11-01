//! The first account's sheets, filled and drawn — as the owner approved them on
//! 2026-11-01: *What should we call you?* (`562:462`) and *Add a PIN?*
//! (`562:508`), and the password sheet *Use password only* leads to.
//!
//! `crate::setup_sheet_layout` places a sheet, `crate::setup_sheet_surface`
//! draws its card, its fields and its buttons, and
//! `crate::setup_sheet_painting::a_sheet` letters what every sheet has. This is
//! what is in the rows: each a label over a field, in one card.
//!
//! # Two sheets, because one cannot make an account
//!
//! `alo_greeting::makes_an_account` takes a name **and** a secret, and the
//! design asks for them on two sheets; `crate::the_first_account` is the state
//! between them, and the road to a password instead of a PIN.
//!
//! # The profile picture is not here
//!
//! The approved name sheet draws a second row, *Profile picture · optional*.
//! **The owner removed it for now** (2026-11-01), so the card holds the name
//! alone.
//!
//! # A secret is never drawn
//!
//! A field with a secret in it shows four dots however much is typed, and an
//! empty one its invitation: the rule `crate::sign_in_raster` keeps.
//!
//! # The words are the vocabulary's
//!
//! Every line comes from `crate::first_start_words` through the person's own
//! `Strings`. Nothing here is English.

use cosmic_text::FontSystem;
use smithay::utils::{Physical, Rectangle};

use crate::painted::Solid;
use crate::painted_text::sentence;
use crate::setup_sheet_layout::TheRowsAre;
use crate::setup_sheet_painting::{TheSheetSays, a_sheet};
use crate::setup_sheet_surface::{TheFields, TheMarks, TheRing};
use crate::sign_in_raster::{Measure, SignInPicture};
use crate::sign_in_surface::TheColours;
use crate::the_first_account::{ASecret, OnTheSheet, TheAccountSheet, TheAccountShown};
use crate::the_first_starts_icons::AFirstStartIcon;
use crate::{RenderError, first_start_words as say};

/// How tall a row holding a field is: `562:484`'s 106.
const A_FIELDS_ROW_IS_TALL: i32 = 106;

/// What a secret is shown as, however long: four dots.
const A_SECRET_IS_SHOWN_AS: &str = "\u{2022}\u{2022}\u{2022}\u{2022}";

/// What a first-account sheet is showing.
pub(crate) struct FirstStartSheet<'a> {
    /// The account being asked for, as it is drawn.
    pub(crate) account: TheAccountShown<'a>,
    /// What this machine last said about making an account.
    pub(crate) said: Option<&'a alo_strings::Said>,
}

/// One of a sheet's rows: its label, what its field shows, and whether that is
/// the invitation rather than something typed.
struct ARow {
    /// The label over the field.
    label: String,
    /// What the field shows.
    shows: String,
    /// Whether that is the invitation to type rather than what was typed.
    inviting: bool,
}

/// Draw the sheet that is showing.
///
/// # Errors
/// [`RenderError::SignInScene`] for a screen too small to hold a sheet, which
/// is a refusal rather than a column drawn over its own footer.
pub(crate) fn drawn(
    sheet: &FirstStartSheet<'_>,
    fonts: &mut FontSystem,
    words: &alo_strings::Strings,
    size: (i32, i32),
    colours: TheColours,
    measure: &Measure,
) -> Result<SignInPicture, RenderError> {
    let nothing = alo_strings::Filling::nothing();
    let say_it = |word: alo_strings::Word| words.say(&word.key(), &nothing).into_text();
    let account = sheet.account;
    let (secret_typed, again_typed) = account.typed;
    let a_secret = |label: alo_strings::Word, typed: bool, hint: alo_strings::Word| ARow {
        label: say_it(label),
        shows: if typed {
            A_SECRET_IS_SHOWN_AS.to_owned()
        } else {
            say_it(hint)
        },
        inviting: !typed,
    };
    let (eyebrow, title, introduction, rows, note, actions) = match account.sheet {
        TheAccountSheet::Name => (
            say_it(say::NAME_EYEBROW),
            say_it(say::NAME_TITLE),
            say_it(say::NAME_INTRODUCTION),
            vec![ARow {
                label: say_it(say::NAME_CHOICE),
                shows: if account.name.is_empty() {
                    say_it(say::NAME_HINT)
                } else {
                    account.name.to_owned()
                },
                inviting: account.name.is_empty(),
            }],
            (AFirstStartIcon::Computer, say_it(say::NAME_CONSEQUENCE)),
            [say_it(say::CONTINUE), say_it(say::BACK)],
        ),
        TheAccountSheet::Secret(ASecret::Pin) => (
            say_it(say::PIN_EYEBROW),
            say_it(say::PIN_TITLE),
            say_it(say::PIN_INTRODUCTION),
            vec![
                a_secret(say::PIN_CHOICE, secret_typed, say::PIN_HINT),
                a_secret(say::PIN_AGAIN, again_typed, say::AGAIN_HINT),
            ],
            (AFirstStartIcon::Lock, say_it(say::PIN_CONSEQUENCE)),
            [say_it(say::SAVE), say_it(say::USE_PASSWORD_ONLY)],
        ),
        TheAccountSheet::Secret(ASecret::Password) => (
            say_it(say::PASSWORD_EYEBROW),
            say_it(say::PASSWORD_TITLE),
            say_it(say::PASSWORD_INTRODUCTION),
            vec![
                a_secret(say::PASSWORD_CHOICE, secret_typed, say::PASSWORD_HINT),
                a_secret(say::PASSWORD_AGAIN, again_typed, say::AGAIN_HINT),
            ],
            (AFirstStartIcon::Lock, say_it(say::PASSWORD_CONSEQUENCE)),
            [say_it(say::SAVE_PASSWORD), say_it(say::BACK)],
        ),
    };
    // **What the machine last said, or that the two differed, before what the
    // sheet means** — one line under the card, which is where a person looks
    // after pressing the way on.
    let (icon, means) = note;
    let under = sheet.said.map_or_else(
        || {
            if account.they_differed {
                say_it(say::THEY_DIFFER)
            } else {
                means.clone()
            }
        },
        |said| said.text().to_owned(),
    );
    let (focused, ring) = match account.on {
        OnTheSheet::Field(it) => (Some(it), None),
        OnTheSheet::WayOn => (None, Some(TheRing::OnAnAction(0))),
        OnTheSheet::WayBack => (None, Some(TheRing::OnAnAction(1))),
    };
    let how_many = rows.len();
    let [go_on, go_back] = &actions;
    let (mut drawn, at, _) = a_sheet(
        &TheSheetSays {
            eyebrow: &eyebrow,
            title: &title,
            introduction: &introduction,
            actions: [go_on, go_back],
            rows: TheRowsAre::Grouped(A_FIELDS_ROW_IS_TALL),
            note: Some((icon, &under)),
        },
        (how_many, how_many),
        |_| TheMarks {
            chosen: None,
            ring,
            fields: TheFields::InEveryRow { focused },
        },
        fonts,
        words,
        size,
        colours,
        measure,
    )?;
    // **What a press can land on is what was just drawn**, in its pixels.
    drawn.controls = crate::a_press_on_a_setup_sheet::the_controls_of(&at, |which| which);
    let height = drawn.size.1;
    for (which, (row, card)) in rows.iter().zip(at.choices.iter()).enumerate() {
        // The label, 15 in and 15 down, in `text/secondary` (`562:485`).
        let label = sentence(
            fonts,
            &row.label,
            (card.size.w - measure.px(30)).max(1),
            measure.field_label_metrics(),
            colours.entry,
            colours.secondary,
        )
        .cropped_to_its_ink();
        drawn.inked.extend(label.placed(
            card.loc.x + measure.px(15),
            card.loc.y + measure.px(15),
            height,
        ));
        // What the field shows, 16 into it and centred down it.
        let field = the_field_in_pixels(*card, measure.display());
        let shown = sentence(
            fonts,
            &row.shows,
            (field.size.w - measure.px(32)).max(1),
            measure.metrics(),
            colours.entry,
            if row.inviting {
                colours.muted
            } else {
                colours.ink
            },
        )
        .cropped_to_its_ink();
        let x = field.loc.x + measure.px(16);
        let y = field.loc.y + (field.size.h - shown.height).max(0) / 2;
        let typed_wide = if row.inviting { 0 } else { shown.width };
        drawn.inked.extend(shown.placed(x, y, height));
        // **The caret where the keyboard is**, after what was typed.
        if focused == Some(which) {
            drawn.solids.push(Solid {
                area: Rectangle::new(
                    (
                        x + typed_wide + measure.px(1),
                        field.loc.y + (field.size.h - measure.px(20)) / 2,
                    )
                        .into(),
                    (measure.px(2), measure.px(20)).into(),
                ),
                colour: colours.ink,
            });
        }
    }
    Ok(drawn)
}

/// The field inside a row, in the picture's pixels — the rule
/// `crate::setup_sheet_layout::the_field_in` keeps in logical units.
fn the_field_in_pixels(
    row: Rectangle<i32, Physical>,
    display: crate::display_scale::DisplayScale,
) -> Rectangle<i32, Physical> {
    Rectangle::new(
        (row.loc.x + display.px(15), row.loc.y + display.px(41)).into(),
        ((row.size.w - display.px(32)).max(0), display.px(48)).into(),
    )
}

#[cfg(test)]
#[path = "first_start_sheet_tests.rs"]
mod tests;
