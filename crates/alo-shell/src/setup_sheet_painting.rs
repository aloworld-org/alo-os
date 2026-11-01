//! **What every setup sheet letters the same way**: its stage, eyebrow,
//! heading and introduction; a row's two lines; the way on and the way back;
//! the footer.
//!
//! The overview (`09`) and the keyboard sheet (`09a`) letter through this, so
//! the two cannot drift apart from each other in where a line sits or which
//! ink it is in. Where things go is `crate::setup_sheet_layout`'s; the cards
//! under them are `crate::setup_sheet_surface`'s; what each sheet says is its
//! own file's.

use cosmic_text::FontSystem;
use smithay::utils::{Physical, Rectangle, Scale};

use crate::a_sheet_that_fits::{ItFits, the_sheet_that_fits};
use crate::painted_text::sentence;
use crate::setup_sheet_in_pixels::{SetupSheetInPixels, the_column_in_pixels};
use crate::setup_sheet_layout::TheTextIsTall;
use crate::setup_sheet_surface::TheMarks;
use crate::sign_in_raster::{Measure, SignInPicture};
use crate::sign_in_surface::TheColours;
use crate::{RenderError, first_start_words as say};

/// The words around a sheet's rows.
pub(crate) struct TheSheetSays<'a> {
    /// The small line above the heading.
    pub(crate) eyebrow: &'a str,
    /// The heading.
    pub(crate) title: &'a str,
    /// The line under it.
    pub(crate) introduction: &'a str,
    /// The way on, then the way back.
    pub(crate) actions: [&'a str; 2],
    /// How the rows sit: each its own card, or grouped in one.
    pub(crate) rows: crate::setup_sheet_layout::TheRowsAre,
    /// The note under the rows and the icon before it, where the sheet has
    /// one (`562:391`).
    pub(crate) note: Option<(crate::the_first_starts_icons::AFirstStartIcon, &'a str)>,
}

/// How far the note's words sit in from its icon's start: `562:395` at 28.
const THE_NOTES_WORDS_START_AT: u32 = 28;

/// The note's icon: 16, 4 in (`562:392`).
const THE_NOTES_ICON_IS: (u32, u32) = (16, 4);

/// *Accessibility*'s icon, 16 at (17, 14), and its word at 41 (`562:350`).
const ACCESSIBILITYS_ICON: (u32, u32, u32) = (16, 17, 14);

/// Where *Accessibility*'s word starts in its control.
const ACCESSIBILITYS_WORD_AT: u32 = 41;

/// *Accessibility* in its control: the icon, then the word, in the ink on the
/// control's own surface (`562:350`).
pub(crate) fn accessibility(
    drawn: &mut SignInPicture,
    fonts: &mut FontSystem,
    words: &alo_strings::Strings,
    at: Rectangle<i32, Physical>,
    colours: TheColours,
    measure: &Measure,
) {
    let (side, x, y) = ACCESSIBILITYS_ICON;
    let px = |logical: u32| measure.px(i32::try_from(logical).unwrap_or(i32::MAX));
    drawn.inked.extend(
        crate::the_first_starts_icons::AFirstStartIcon::Accessibility.inked(
            Rectangle::new(
                (at.loc.x + px(x), at.loc.y + px(y)).into(),
                (px(side), px(side)).into(),
            ),
            colours.entry,
            colours.ink,
        ),
    );
    let word = words.say(
        &crate::sign_in_words::ACCESSIBILITY.key(),
        &alo_strings::Filling::nothing(),
    );
    let shaped = sentence(
        fonts,
        word.text(),
        (at.size.w - px(ACCESSIBILITYS_WORD_AT)).max(1),
        measure.label_metrics(),
        colours.entry,
        colours.ink,
    )
    .cropped_to_its_ink();
    let down = (at.size.h - shaped.height).max(0) / 2;
    let height = drawn.size.1;
    drawn.inked.extend(shaped.placed(
        at.loc.x + px(ACCESSIBILITYS_WORD_AT),
        at.loc.y + down,
        height,
    ));
}

/// The note under a sheet's rows: its icon, then its words, in the muted ink on
/// the sheet's ground (`562:391`).
pub(crate) fn a_note(
    drawn: &mut SignInPicture,
    fonts: &mut FontSystem,
    (icon, said): (crate::the_first_starts_icons::AFirstStartIcon, &str),
    at: Rectangle<i32, Physical>,
    colours: TheColours,
    measure: &Measure,
) {
    let px = |logical: u32| measure.px(i32::try_from(logical).unwrap_or(i32::MAX));
    let (side, inset) = THE_NOTES_ICON_IS;
    drawn.inked.extend(icon.inked(
        Rectangle::new(
            (at.loc.x + px(inset), at.loc.y + px(3)).into(),
            (px(side), px(side)).into(),
        ),
        colours.ground,
        colours.muted,
    ));
    let shaped = sentence(
        fonts,
        said,
        (at.size.w - px(THE_NOTES_WORDS_START_AT)).max(1),
        measure.metrics_of_a_row(),
        colours.ground,
        colours.muted,
    )
    .cropped_to_its_ink();
    let height = drawn.size.1;
    drawn.inked.extend(shaped.placed(
        at.loc.x + px(THE_NOTES_WORDS_START_AT),
        at.loc.y,
        height,
    ));
}

/// **A sheet with up to `rows` rows, its words lettered and its cards drawn**
/// — and where each row's card is, in the picture's pixels, for the sheet to
/// letter its rows into, and how many rows there was room for.
///
/// **Fitted to the screen** (`crate::a_sheet_that_fits`): a screen too short
/// for every row draws as many as fit and never fewer than `fewest`, and
/// `marks` is asked for the marks once that number is known, because which
/// row wears the ring depends on which rows are in sight.
///
/// At the display scale `measure` was made for: laid out at the output's
/// pixels through it and drawn at its size (`crate::setup_sheet_in_pixels`).
///
/// # Errors
/// [`RenderError::SignInScene`] for a screen too small to hold a sheet.
#[expect(
    clippy::too_many_arguments,
    reason = "each is a different thing about one sheet, and a struct of them would be named \
              after this call"
)]
pub(crate) fn a_sheet(
    says: &TheSheetSays<'_>,
    (rows, fewest): (usize, usize),
    marks: impl FnOnce(usize) -> TheMarks,
    fonts: &mut FontSystem,
    words: &alo_strings::Strings,
    size: (i32, i32),
    colours: TheColours,
    measure: &Measure,
) -> Result<(SignInPicture, SetupSheetInPixels, usize), RenderError> {
    let (width, height) = size;
    let display = measure.display();
    let column = the_column_in_pixels(width, display);
    let shaped_introduction = sentence(
        fonts,
        says.introduction,
        column,
        measure.metrics(),
        colours.ground,
        colours.ink,
    );
    let Some(ItFits {
        laid,
        title: shaped_title,
        rows,
    }) = the_sheet_that_fits(
        fonts,
        says.title,
        colours,
        measure,
        size,
        TheTextIsTall {
            title: 0,
            introduction: shaped_introduction.height,
            consequence: 0,
            choices: rows,
            actions: 2,
            rows: crate::setup_sheet_layout::TheRowsAre::Apart,
        },
        fewest,
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
            marks(rows),
            Scale::from(display.factor()),
        ),
        solids: Vec::new(),
        inked: Vec::new(),
        controls: Vec::new(),
    };
    let at = laid.pixels;
    let nothing = alo_strings::Filling::nothing();
    let stage = words.say(&say::STAGE.key(), &nothing);
    let footer = words.say(&say::FOOTER.key(), &nothing);
    for (text, place) in [
        (stage.text(), at.stage),
        (says.eyebrow, at.eyebrow),
        (footer.text(), at.footer),
    ] {
        crate::painted_text::line(
            &mut drawn.inked,
            height,
            fonts,
            text,
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
    // The way on is filled, so its label is in the ground; the way back is an
    // outline, so its label is in the ink.
    for (button, (text, on_the_ink)) in at
        .actions
        .iter()
        .zip([(says.actions[0], true), (says.actions[1], false)])
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
    Ok((drawn, at, rows))
}

/// **A row's two lines**, the first in the ink at 16/25 and the second in
/// `text/secondary` at 14/22, inside its card — the language sheet's rows,
/// measured off `09b`.
pub(crate) fn two_lines(
    drawn: &mut SignInPicture,
    fonts: &mut FontSystem,
    card: Rectangle<i32, Physical>,
    lines: (&str, &str),
    colours: TheColours,
    measure: &Measure,
) {
    let height = drawn.size.1;
    let inside = measure.px(20);
    let width = card.size.w - 2 * inside;
    crate::painted_text::line(
        &mut drawn.inked,
        height,
        fonts,
        lines.0,
        Rectangle::new(
            (card.loc.x + inside, card.loc.y + measure.px(16)).into(),
            (width, measure.px(25)).into(),
        ),
        measure.metrics(),
        colours.entry,
        colours.ink,
    );
    crate::painted_text::line(
        &mut drawn.inked,
        height,
        fonts,
        lines.1,
        Rectangle::new(
            (card.loc.x + inside, card.loc.y + measure.px(45)).into(),
            (width, measure.px(22)).into(),
        ),
        measure.label_metrics(),
        colours.entry,
        colours.secondary,
    );
}
