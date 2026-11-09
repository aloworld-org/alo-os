//! *How much can alo do?*, drawn — on the composition the language sheet and
//! the design's access frames share (`183:10695`, `183:10699`, `183:10703`):
//! the strip, the heading, three cards, the note under them, the one action,
//! *Accessibility* and the footer.
//!
//! `crate::how_much_alo_can_do` is what is chosen and where the ring is; this
//! is which words go in which box. Where things go is
//! `crate::setup_sheet_layout`'s, and the cards are `crate::setup_sheet_surface`'s.
//!
//! # Where this departs from the frames, by the owner's answers of 2026-11-09
//!
//! Recorded in full in `docs/design/the-first-start.md`; in short — the title,
//! the three names and Full access's line are ADR 0149's; the other two lines
//! are Settings'; there is no eyebrow and no introduction; and the one action
//! is *Continue*. **The cards grow to their words** (`TheRowsAre::ApartEachAs`),
//! because the lines the owner chose are longer than the frames' one-liners and
//! an 85-tall card would cut the cost of full access off mid-sentence — the one
//! sentence on this sheet that must be read whole.
//!
//! # The chosen card says so in a word as well as a border
//!
//! ` · Selected` after its name, from the same answer that draws the border,
//! as on the language sheet.

use cosmic_text::FontSystem;
use smithay::utils::{Rectangle, Scale};

use alo_capability::HowCodeRuns;

use crate::a_sheet_that_fits::{ItFits, the_sheet_that_fits};
use crate::how_much_alo_can_do::{HowMuchAloCanDo, THE_THREE, TheRingIs};
use crate::painted_text::sentence;
use crate::setup_sheet_in_pixels::the_column_in_pixels;
use crate::setup_sheet_layout::{HowClose, TheRowsAre, TheTextIsTall};
use crate::setup_sheet_surface::{TheFields, TheMarks, TheRing};
use crate::sign_in_raster::{Measure, SignInPicture};
use crate::sign_in_surface::TheColours;
use crate::the_first_starts_icons::AFirstStartIcon;
use crate::{RenderError, first_start_words as say, the_settings_words as settings};

/// How far a card's words sit in from its sides: the language sheet's 20.
const INSIDE: i32 = 20;

/// Where a card's second line starts, down from its top: the language sheet's
/// 45 (16 down, a 25-tall first line, 4 between).
const THE_SECOND_LINE_IS_AT: i32 = 45;

/// What a card keeps under its last line: the 16 the first line has above.
const UNDER_THE_LAST_LINE: i32 = 16;

/// What the sheet shows, beside what is chosen.
#[derive(Debug, Clone, Copy)]
pub(crate) struct HowMuchSheet<'a> {
    /// What is chosen, where the ring is, and whether saving failed.
    pub(crate) asking: &'a HowMuchAloCanDo,
    /// Whether the sealed box can start a program on this machine — Settings'
    /// own reading, so the sheet says what Settings says under it.
    pub(crate) the_sealed_box_runs_here: bool,
}

/// What a card says under its name.
fn what_it_means(
    level: HowCodeRuns,
    sheet: &HowMuchSheet<'_>,
    say_it: &dyn Fn(alo_strings::Word) -> String,
) -> String {
    match level {
        HowCodeRuns::AtFullTrust => say_it(say::ACCESS_FULL_COSTS),
        HowCodeRuns::AskingEachTime => say_it(settings::ASKING_IS),
        HowCodeRuns::InASealedBox if sheet.the_sealed_box_runs_here => {
            say_it(settings::SEALED_BOX_IS)
        }
        HowCodeRuns::InASealedBox => format!(
            "{} {}",
            say_it(settings::SEALED_BOX_IS),
            say_it(alo_capability::words::THE_SEALED_BOX_IS_NOT_HERE_YET)
        ),
    }
}

/// What a card is called.
const fn the_name_of(level: HowCodeRuns) -> alo_strings::Word {
    match level {
        HowCodeRuns::AtFullTrust => say::ACCESS_FULL,
        HowCodeRuns::AskingEachTime => say::ACCESS_ASK,
        HowCodeRuns::InASealedBox => say::ACCESS_SEALED,
    }
}

/// Draw the sheet.
///
/// # Errors
/// [`RenderError::SignInScene`] for a screen too small to hold all three
/// cards — never fewer than three, because a choice of how much alo may do
/// with one of its answers out of sight is not that choice.
pub(crate) fn drawn(
    sheet: &HowMuchSheet<'_>,
    fonts: &mut FontSystem,
    words: &alo_strings::Strings,
    size: (i32, i32),
    colours: TheColours,
    measure: &Measure,
) -> Result<SignInPicture, RenderError> {
    let height = size.1;
    let nothing = alo_strings::Filling::nothing();
    let say_it = |word: alo_strings::Word| words.say(&word.key(), &nothing).into_text();
    let display = measure.display();
    let selected = say_it(say::SELECTED);
    let asking = sheet.asking;

    // **Each card's second line, shaped first**, because how far it wraps in
    // a language this code has never seen is what decides how tall the cards
    // are.
    let column = the_column_in_pixels(size.0, display);
    let wide = (column - 2 * measure.px(INSIDE)).max(1);
    let meanings: Vec<_> = THE_THREE
        .iter()
        .map(|level| {
            sentence(
                fonts,
                &what_it_means(*level, sheet, &say_it),
                wide,
                measure.label_metrics(),
                colours.entry,
                colours.secondary,
            )
        })
        .collect();
    // Each card as tall as its own words, rounded up so the last line never
    // touches the card's edge.
    let mut cards_are = [0; 3];
    for (card, meaning) in cards_are.iter_mut().zip(&meanings) {
        *card = THE_SECOND_LINE_IS_AT
            + UNDER_THE_LAST_LINE
            + i32::try_from(display.logical(meaning.height)).unwrap_or(i32::MAX)
            + 1;
    }

    let note = if asking.was_not_saved() {
        (AFirstStartIcon::Info, say_it(say::ACCESS_NOT_SAVED))
    } else {
        (AFirstStartIcon::Info, say_it(say::ACCESS_CONSEQUENCE))
    };
    let note_height = sentence(
        fonts,
        &note.1,
        (column - measure.px(28)).max(1),
        measure.metrics_of_a_row(),
        colours.ground,
        colours.muted,
    )
    .height;

    let title = say_it(say::ACCESS_TITLE);
    let tall = TheTextIsTall {
        title: 0,
        // **No introduction**, the owner's answer; a height of zero is how the
        // layout is told there is none.
        introduction: 0,
        consequence: note_height,
        choices: THE_THREE.len(),
        actions: 1,
        rows: TheRowsAre::ApartEachAs(cards_are),
    };
    // **As captured, then tightened, then started higher** — never with a
    // card out of sight (`HowClose::TightenedToTheTop` says why this sheet
    // alone has the third).
    let (laid, shaped_title) =
        match the_sheet_that_fits(fonts, &title, colours, measure, size, tall, THE_THREE.len()) {
            Some(ItFits {
                laid,
                title: shaped,
                rows,
            }) if rows == THE_THREE.len() => (laid, shaped),
            Some(_) | None => {
                let shaped = sentence(
                    fonts,
                    &title,
                    column,
                    measure.tight_title_metrics(),
                    colours.ground,
                    colours.ink,
                );
                let laid = crate::setup_sheet_in_pixels::laid_out_as(
                    size,
                    TheTextIsTall {
                        title: shaped.height,
                        ..tall
                    },
                    display,
                    HowClose::TightenedToTheTop,
                )
                .ok_or(RenderError::SignInScene)?;
                (laid, shaped)
            }
        };
    let chosen = THE_THREE.iter().position(|it| *it == asking.chosen());

    let mut drawn = SignInPicture {
        size,
        surface: crate::setup_sheet_surface::setup_sheet_surface(
            &laid.logical,
            size,
            colours,
            TheMarks {
                chosen,
                // **Both marks, from one state**: the ring where a person is,
                // the border on what they picked.
                ring: Some(match asking.ring() {
                    TheRingIs::OnAChoice(it) => TheRing::OnAChoice(it),
                    TheRingIs::OnContinue => TheRing::OnAnAction(0),
                }),
                fields: TheFields::None,
            },
            Scale::from(display.factor()),
        ),
        solids: Vec::new(),
        inked: Vec::new(),
        controls: Vec::new(),
    };
    let at = laid.pixels;
    // **What a press can land on is what was just drawn**, in its pixels.
    drawn.controls = crate::a_press_on_a_setup_sheet::the_controls_of(&at, |which| which);

    for (text, place) in [
        (say_it(say::STAGE), at.stage),
        (say_it(say::FOOTER), at.footer),
    ] {
        crate::painted_text::line(
            &mut drawn.inked,
            height,
            fonts,
            &text,
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

    let inside = measure.px(INSIDE);
    for (which, ((card, level), meaning)) in
        at.choices.iter().zip(THE_THREE).zip(meanings).enumerate()
    {
        // **One answer decides the word and the border.**
        let name = say_it(the_name_of(level));
        let first = if chosen == Some(which) {
            format!("{name}  ·  {selected}")
        } else {
            name
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
        drawn.inked.extend(meaning.cropped_to_its_ink().placed(
            card.loc.x + inside,
            card.loc.y + measure.px(THE_SECOND_LINE_IS_AT),
            height,
        ));
    }

    // **Continue, filled**, so its label is in the ground.
    if let Some(button) = at.actions.first() {
        let shaped = sentence(
            fonts,
            &say_it(say::CONTINUE),
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

    crate::setup_sheet_painting::a_note(
        &mut drawn,
        fonts,
        (note.0, &note.1),
        at.consequence,
        colours,
        measure,
    );
    crate::setup_sheet_painting::accessibility(
        &mut drawn,
        fonts,
        words,
        at.accessibility,
        colours,
        measure,
    );
    Ok(drawn)
}

#[cfg(test)]
#[path = "how_much_alo_can_do_sheet_tests.rs"]
mod tests;
