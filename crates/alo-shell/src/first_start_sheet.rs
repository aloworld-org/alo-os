//! A first-start sheet, filled and drawn.
//!
//! `crate::setup_sheet_layout` says where everything on a setup sheet goes and
//! `crate::setup_sheet_surface` draws its cards and buttons. This is what puts
//! words in them — the two account sheets of the journey `ROOTS.md`
//! inventories, which are the ones standing between a machine that knows
//! nobody and a person who can use it.
//!
//! # Two sheets, because one cannot make an account
//!
//! `alo_greeting::makes_an_account` takes a name **and** a password, and the
//! design splits them: *10 · Your name* asks for the name and *11a · Add a
//! PIN* asks for the secret. So the smallest thing that actually makes an
//! account is both sheets and the step between them, and a single sheet would
//! have been a surface that could not finish.
//!
//! Which sheet is showing is read from where the keyboard is: the name field
//! means the first, the password field means the second.
//!
//! # The words are the vocabulary's
//!
//! Every line comes from `crate::first_start_words` through the person's own
//! `Strings`. Nothing here is English.

use cosmic_text::FontSystem;
use smithay::utils::{Rectangle, Scale};

use crate::a_sheet_that_fits::{ItFits, the_sheet_that_fits};
use crate::painted::Solid;
use crate::painted_text::sentence;
use crate::setup_sheet_in_pixels::the_column_in_pixels;
use crate::setup_sheet_layout::TheTextIsTall;
use crate::sign_in_raster::{Measure, SignInPicture};
use crate::sign_in_surface::TheColours;
use crate::{RenderError, first_start_words as say};

/// What a first-start sheet is showing.
pub(crate) struct FirstStartSheet<'a> {
    /// Whether the PIN sheet is up rather than the name sheet.
    pub(crate) on_the_pin: bool,
    /// The name as typed, which the name sheet shows in its own row.
    pub(crate) typed: &'a str,
    /// Whether anything is typed in the secret. **Never how much.**
    pub(crate) password_typed: bool,
    /// What this machine last said about making an account.
    pub(crate) said: Option<&'a alo_strings::Said>,
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
    let (width, height) = size;
    let nothing = alo_strings::Filling::nothing();
    let say_it = |word: alo_strings::Word| words.say(&word.key(), &nothing);

    // The words first, because their height decides where everything sits.
    let (eyebrow, title, introduction, choice, hint, consequence, action) = if sheet.on_the_pin {
        (
            say_it(say::PIN_EYEBROW),
            say_it(say::PIN_TITLE),
            say_it(say::PIN_INTRODUCTION),
            say_it(say::PIN_CHOICE),
            say_it(say::PIN_HINT),
            say_it(say::PIN_CONSEQUENCE),
            say_it(say::SAVE),
        )
    } else {
        (
            say_it(say::NAME_EYEBROW),
            say_it(say::NAME_TITLE),
            say_it(say::NAME_INTRODUCTION),
            say_it(say::NAME_CHOICE),
            say_it(say::NAME_HINT),
            say_it(say::NAME_CONSEQUENCE),
            say_it(say::CONTINUE),
        )
    };

    // **Shaped before it is placed.** The title wraps and the introduction
    // wraps, in a language this code has never seen, so their heights are
    // measured and handed to the layout rather than assumed — which is what
    // stops a longer heading being drawn over the choices.
    let column = the_column_in_pixels(width, measure.display());
    let shaped_introduction = sentence(
        fonts,
        introduction.text(),
        column,
        measure.metrics(),
        colours.ground,
        colours.ink,
    );

    // **The consequence is measured too.** It is one line in the design and
    // two in a language that needs more words, and a fixed 22 drew the sentence
    // straight through the button below it — which is what a fresh machine
    // showed the first time this was rendered.
    let under = sheet.said.map_or_else(
        || consequence.text().to_owned(),
        |said| said.text().to_owned(),
    );
    let shaped_consequence = sentence(
        fonts,
        &under,
        column,
        measure.metrics(),
        colours.ground,
        colours.muted,
    );

    // **Fitted to the screen**, which on these sheets can only tighten: the
    // one row is the field a person types into (`crate::a_sheet_that_fits`).
    let Some(ItFits {
        laid,
        title: shaped_title,
        ..
    }) = the_sheet_that_fits(
        fonts,
        title.text(),
        colours,
        measure,
        size,
        TheTextIsTall {
            title: 0,
            introduction: shaped_introduction.height,
            consequence: shaped_consequence.height,
            // One row on each sheet takes what is typed. The design's second
            // row is a profile picture and a confirmation, neither of which
            // this screen can do yet, so it draws the one it can act on.
            choices: 1,
            actions: 1,
            rows: crate::setup_sheet_layout::TheRowsAre::Apart,
        },
        1,
    )
    else {
        return Err(RenderError::SignInScene);
    };

    let mut drawn = SignInPicture {
        size,
        surface: crate::setup_sheet_surface::setup_sheet_surface(
            &laid.logical,
            size,
            colours,
            crate::setup_sheet_surface::TheMarks::default(),
            Scale::from(measure.display().factor()),
        ),
        solids: Vec::new(),
        inked: Vec::new(),
        controls: Vec::new(),
    };
    let at = laid.pixels;
    // **What a press can land on is what was just drawn**, in its pixels.
    drawn.controls = crate::a_press_on_a_setup_sheet::the_controls_of(&at, |which| which);

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
    // The title and the introduction were already shaped above, to place
    // everything below them; drawn here from the same shapes rather than
    // shaped twice, so what was measured is what appears.
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

    // The one row that takes what is typed: its label, then either what was
    // typed or the invitation to type.
    if let Some(card) = at.choices.first() {
        let inside = measure.px(21);
        crate::painted_text::line(
            &mut drawn.inked,
            height,
            fonts,
            choice.text(),
            Rectangle::new(
                (card.loc.x + inside, card.loc.y + measure.px(17)).into(),
                (card.size.w - 2 * inside, measure.px(25)).into(),
            ),
            measure.metrics(),
            colours.entry,
            colours.ink,
        );
        // **What is typed, and on the PIN sheet never how much.** The name is
        // shown because a person is naming themselves and must see it; the
        // secret is shown as one band however long it is, which is the rule
        // `crate::sign_in_raster` already keeps.
        let below = Rectangle::new(
            (card.loc.x + inside, card.loc.y + measure.px(46)).into(),
            (card.size.w - 2 * inside, measure.px(22)).into(),
        );
        if sheet.on_the_pin {
            if sheet.password_typed {
                let dot = measure.px(8);
                for which in 0..4 {
                    drawn.solids.push(Solid {
                        area: Rectangle::new(
                            (below.loc.x + which * (dot + measure.px(8)), below.loc.y).into(),
                            (dot, dot).into(),
                        ),
                        colour: colours.ink,
                    });
                }
            } else {
                crate::painted_text::line(
                    &mut drawn.inked,
                    height,
                    fonts,
                    hint.text(),
                    below,
                    measure.metrics(),
                    colours.entry,
                    colours.muted,
                );
            }
        } else if sheet.typed.is_empty() {
            crate::painted_text::line(
                &mut drawn.inked,
                height,
                fonts,
                hint.text(),
                below,
                measure.metrics(),
                colours.entry,
                colours.muted,
            );
        } else {
            crate::painted_text::line(
                &mut drawn.inked,
                height,
                fonts,
                sheet.typed,
                below,
                measure.metrics(),
                colours.entry,
                colours.ink,
            );
        }
    }

    // What this sheet's answer means, or what the machine last said about an
    // attempt — drawn from the shape that was measured, so what decided the
    // layout is what appears in it.
    drawn
        .inked
        .extend(shaped_consequence.cropped_to_its_ink().placed(
            at.consequence.loc.x,
            at.consequence.loc.y,
            height,
        ));

    // The way on, lettered on its filled ground. **Centred**, because a label
    // placed at a button's own left sits against its corner.
    if let Some(button) = at.actions.first() {
        let shaped = sentence(
            fonts,
            action.text(),
            button.size.w,
            measure.label_metrics(),
            colours.ink,
            colours.ground,
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
        say_it(crate::sign_in_words::ACCESSIBILITY).text(),
        at.accessibility,
        measure.label_metrics(),
        colours.ground,
        colours.ink,
    );
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
#[path = "first_start_sheet_tests.rs"]
mod tests;
