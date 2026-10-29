//! What the keyboard sheet draws, asked of the picture it produced.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a picture that is not there is the failure being reported"
)]

use super::drawn;
use crate::the_first_start::TheFirstStart;
use crate::the_keyboard_picker::TheKeyboardPicker;

/// The keyboard sheet at the design's size, in the designed light palette.
fn sheet(picker: &TheKeyboardPicker) -> crate::sign_in_raster::SignInPicture {
    let mut fonts = crate::window_control_label::WindowControlLabels::new()
        .expect("the shell's own fonts load");
    let words = crate::booting::what_this_machine_can_say().expect("this machine's words");
    drawn(
        &TheFirstStart::arriving(None),
        picker,
        &mut fonts.fonts,
        &words,
        (1440, 960),
        crate::sign_in_surface::TheColours::of(
            crate::Contrast::AsDesigned,
            alo_appearance::Scheme::Light,
        ),
        &crate::sign_in_raster::Measure::of(alo_appearance::TextScale::ordinary()),
    )
    .expect("the reference size holds the sheet")
}

/// **What is typed on the test row is drawn**: more ink with letters in it
/// than with its hint, whose words are a different length.
#[test]
fn what_is_typed_on_the_test_row_is_drawn() {
    let mut picker = TheKeyboardPicker::offering(
        alo_keyboards::the_keyboards_offered(
            &alo_strings::Language::written("en").expect("English"),
        ),
        None,
    );
    let before: usize = sheet(&picker).inked.iter().map(|it| it.pixels.len()).sum();
    picker.ring_crosses();
    for letter in "WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW".chars() {
        picker.typed_a(letter);
    }
    let after: usize = sheet(&picker).inked.iter().map(|it| it.pixels.len()).sum();
    assert_ne!(
        before, after,
        "the test row drew the same with and without letters"
    );
}
