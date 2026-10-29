//! What the overview draws, asked of the picture it produced.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a picture that is not there is the failure being reported"
)]

use super::drawn;
use crate::the_first_start::TheFirstStart;

/// The overview at a size, in the designed light palette.
fn sheet(
    first_start: &TheFirstStart,
    size: (i32, i32),
) -> Result<crate::sign_in_raster::SignInPicture, crate::RenderError> {
    let mut fonts = crate::window_control_label::WindowControlLabels::new()
        .expect("the shell's own fonts load");
    let words = crate::booting::what_this_machine_can_say().expect("this machine's words");
    drawn(
        first_start,
        &mut fonts.fonts,
        &words,
        size,
        crate::sign_in_surface::TheColours::of(
            crate::Contrast::AsDesigned,
            alo_appearance::Scheme::Light,
        ),
        &crate::sign_in_raster::Measure::of(alo_appearance::TextScale::ordinary()),
    )
}

/// **The overview is drawn at the design's size**, with ink in it.
#[test]
fn three_rows_each_name_what_is_set() {
    let first_start = TheFirstStart::arriving(alo_formats::RegionCode::named("BE").ok());
    let picture = sheet(&first_start, (1440, 960)).expect("the reference size holds the sheet");
    assert!(!picture.inked.is_empty());
}

/// **A screen too small for a sheet is refused**, not drawn over itself.
#[test]
fn a_screen_too_small_is_refused() {
    let first_start = TheFirstStart::arriving(None);
    assert!(sheet(&first_start, (200, 120)).is_err());
}
