//! **The password sheet, laid out for one display**: what it is for, the one
//! hidden field, and the line that says what to do or what went wrong
//! (ADR 0150).
//!
//! # A departure from the design, written down
//!
//! The design file has no frame for this sheet (ADR 0149, *Not decided
//! here*). Until it has, the sheet is the approval surface's panel — the same
//! measures, palette and frame (`crate::approval_raster`), in the same layer,
//! above every window and below the egress indicator — because a person
//! should meet the moment alo asks for their password in the same panel as the
//! moment alo asks for their OK. It is drawn into an
//! `crate::approval_raster::ApprovalPicture` and painted where that is.
//!
//! # Nothing about the password is drawn but whether there is one
//!
//! The field shows eight marks once anything is typed and a caret before,
//! never one mark per letter: how long a password is, is a fact about it.
//!
//! # Drawn whole or not at all
//!
//! The command is what the person is giving their password for, so it is
//! drawn whole, wrapped and never cut. A display too small to hold the sheet
//! refuses the frame, and the sheet then cancels the question
//! (`crate::the_password_sheet::ThePasswordSheet::could_not_be_drawn`) rather
//! than hold every key with nothing on the screen.

use alo_strings::{Filling, Strings};
use smithay::utils::Rectangle;

use crate::approval_raster::{ApprovalPicture, Measure, Palette, framed, placed};
use crate::painted::Solid;
use crate::painted_text::sentence;
use crate::the_password_sheet::SheetShows;
use crate::the_password_sheets_words as words;
use crate::{RenderError, WindowControlLabels};

/// How many marks stand for a password, whatever its length.
const MARKS: i32 = 8;

/// The largest output side, in pixels, the sheet is laid out for.
const LARGEST_SIDE: i32 = 16_384;

/// How the sheet looks, as the person's appearance decides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TheSheetsLook {
    /// Light or dark.
    pub(crate) scheme: alo_appearance::Scheme,
    /// The person's text size.
    pub(crate) scale: alo_appearance::TextScale,
    /// The design's palette, or high contrast's.
    pub(crate) contrast: crate::Contrast,
}

/// **The sheet for an output of `size` drawn at `display`.**
///
/// # Errors
/// [`RenderError::ApprovalScene`] when the output is larger than any this is
/// laid out for, too narrow for the panel, or too short to hold it whole.
pub(crate) fn drawn(
    shows: SheetShows<'_>,
    strings: &Strings,
    labels: &mut WindowControlLabels,
    size: (i32, i32),
    look: TheSheetsLook,
    display: crate::display_scale::DisplayScale,
) -> Result<ApprovalPicture, RenderError> {
    let (width, height) = size;
    if width > LARGEST_SIDE || height > LARGEST_SIDE {
        return Err(RenderError::ApprovalScene);
    }
    let measure = Measure::at(look.scale, display);
    let palette = Palette::of(look.scheme, look.contrast);
    let margin = measure.px(16);
    let pad = measure.px(20);
    let panel_width = (width - 2 * margin).min(measure.px(560));
    let inner = panel_width - 2 * pad;
    if inner < measure.px(160) {
        return Err(RenderError::ApprovalScene);
    }
    let say = |word: alo_strings::Word| {
        strings
            .say(&word.key(), &Filling::nothing())
            .text()
            .to_owned()
    };
    let (heading, what) = words::the_question(shows.what_for);
    let heading = say(heading);
    let under = say(words::the_line_under(shows.under));
    let fonts = &mut labels.fonts;
    let mut line = |text: &str| {
        sentence(
            fonts,
            text,
            inner,
            measure.metrics(),
            palette.ground,
            palette.ink,
        )
    };
    let heading_shaped = line(&heading);
    let what_shaped = line(what);
    let under_shaped = line(&under);

    let gap = measure.px(8);
    let field_height = measure.px(40);
    let panel_height = pad
        + heading_shaped.height
        + gap
        + what_shaped.height
        + 2 * gap
        + field_height
        + gap
        + under_shaped.height
        + pad;
    if panel_height > height - 2 * margin {
        return Err(RenderError::ApprovalScene);
    }
    let left = (width - panel_width) / 2;
    let top = (height - panel_height) / 2;
    let panel = Rectangle::new((left, top).into(), (panel_width, panel_height).into());
    let mut picture = ApprovalPicture {
        size,
        panel: Some(panel),
        agent: None,
        sentence: None,
        answers: Vec::new(),
        solids: Vec::new(),
        inked: Vec::new(),
    };
    framed(&mut picture.solids, panel, measure.px(2), palette);

    let mut y = top + pad;
    let heading_height = heading_shaped.height;
    picture.inked.extend(placed(heading_shaped, left + pad, y));
    y += heading_height + gap;
    let what_area = Rectangle::new(
        (left + pad, y).into(),
        (what_shaped.width_of_box, what_shaped.height).into(),
    );
    y += what_shaped.height + 2 * gap;
    picture
        .inked
        .extend(placed(what_shaped, left + pad, what_area.loc.y));
    picture.sentence = Some((what.clone(), what_area));

    let field = Rectangle::new((left + pad, y).into(), (inner, field_height).into());
    framed(&mut picture.solids, field, measure.px(1), palette);
    the_field_holds(
        &mut picture.solids,
        field,
        shows.typed_something,
        measure,
        palette,
    );
    y += field_height + gap;
    picture.inked.extend(placed(under_shaped, left + pad, y));
    Ok(picture)
}

/// Inside the field: eight marks once anything is typed, a caret before.
fn the_field_holds(
    solids: &mut Vec<Solid>,
    field: Rectangle<i32, smithay::utils::Physical>,
    typed_something: bool,
    measure: Measure,
    palette: Palette,
) {
    let inset = measure.px(12);
    if typed_something {
        let mark = measure.px(8);
        let step = mark + measure.px(6);
        let y = field.loc.y + (field.size.h - mark) / 2;
        for which in 0..MARKS {
            solids.push(Solid {
                area: alo_drawing::area::Area::new(
                    (field.loc.x + inset + which * step, y).into(),
                    (mark, mark).into(),
                ),
                colour: palette.ink,
            });
        }
    } else {
        let tall = measure.px(20);
        solids.push(Solid {
            area: alo_drawing::area::Area::new(
                (field.loc.x + inset, field.loc.y + (field.size.h - tall) / 2).into(),
                (measure.px(2), tall).into(),
            ),
            colour: palette.ink,
        });
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_password_sheet_drawn_tests.rs"]
mod tests;
