//! **What the screen reader came to, drawn where a person looks for it** —
//! on the first-start sheets and the sign-in screen, one line above
//! *Accessibility* ([ADR 0155](../../../docs/decisions/0155-the-screens-before-sign-in-speak.md) §7).
//!
//! A reader that could not start cannot say so itself, and until this change
//! the reason went to the journal only, so a sighted person helping somebody
//! set a machine up saw nothing at all. Now the sentence — *did not start*,
//! *not on this computer*, *nothing to play it on* — is drawn above the button
//! that turned it on, in the ink, and is told to any reader that is running
//! (`crate::the_screens_before_sign_in_as_read`).
//!
//! **Placed from what was drawn**: on a sheet, from the rectangle the sheet
//! drew *Accessibility* in; on the sign-in screen, from that screen's own
//! layout (`crate::sign_in_layout`). It is left-aligned with the button, as
//! wide as the screen allows up to a column's width, and never above the top.
//! The design draws nothing here; this is a deliberate addition, written down
//! in `docs/design/the-first-start.md`.

use cosmic_text::FontSystem;
use smithay::utils::{Logical, Physical, Rectangle, Scale, Size};

use crate::a_press_on_a_setup_sheet::OnASheet;
use crate::display_scale::DisplayScale;
use crate::painted_text::sentence;
use crate::sign_in_raster::{Measure, SignInLook, SignInPicture};
use crate::sign_in_surface::TheColours;

/// The widest the line is, in logical pixels: a sheet's column.
const AT_MOST: i32 = 480;

/// How far it sits above *Accessibility*, in logical pixels.
const ABOVE_BY: i32 = 12;

/// **Draw `said` above *Accessibility* in `drawn`.** A picture with no
/// *Accessibility* to place it by is left as it was.
pub(crate) fn above_accessibility(
    drawn: &mut SignInPicture,
    said: &str,
    fonts: &mut FontSystem,
    look: SignInLook,
    display: DisplayScale,
) {
    let accessibility = drawn
        .controls
        .iter()
        .find(|control| control.what == OnASheet::Accessibility)
        .map(|control| control.area)
        .or_else(|| on_the_sign_in_screen(drawn.size, display));
    let Some(at) = accessibility else {
        return;
    };
    let measure = Measure::at(look.scale, display);
    let colours = TheColours::of(look.contrast, look.scheme);
    let width = (drawn.size.0 - at.loc.x - measure.px(16))
        .min(measure.px(AT_MOST))
        .max(1);
    let shaped = sentence(
        fonts,
        said,
        width,
        measure.label_metrics(),
        colours.ground,
        colours.ink,
    )
    .cropped_to_its_ink();
    let y = (at.loc.y - measure.px(ABOVE_BY) - shaped.height).max(0);
    let height = drawn.size.1;
    drawn.inked.extend(shaped.placed(at.loc.x, y, height));
}

/// Where the sign-in screen draws *Accessibility*, in pixels, for a picture of
/// `size` at `display`.
fn on_the_sign_in_screen(
    size: (i32, i32),
    display: DisplayScale,
) -> Option<Rectangle<i32, Physical>> {
    let logical = Size::<i32, Logical>::from((
        i32::try_from(display.logical(size.0)).unwrap_or(i32::MAX),
        i32::try_from(display.logical(size.1)).unwrap_or(i32::MAX),
    ));
    let laid = crate::sign_in_layout::sign_in_laid_out(logical)?;
    Some(
        crate::sign_in_in_pixels::SignInInPixels::of(&laid, Scale::from(display.factor()))
            .accessibility,
    )
}
