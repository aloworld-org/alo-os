//! **The list of applications a person gives alo one of, in the bytes a
//! display is handed** — not a picture of Settings alone.
//!
//! The Settings raster is laid out the way `crate::direct_desktop` lays it out
//! for a frame, put in the native layers over the desktop that
//! `crate::nested_desktop::frame_pictures` builds, painted by the compositor's
//! own software renderer, and read back from the injected card. A surface that
//! passed its own tests and never reached the frame — the fault that cost a
//! border on 2026-11-16 — fails here.
//!
//! When `ALO_APPLICATION_GRANT_PICTURES` names a folder the frame is written
//! there as a PNG, which is a check somebody runs by hand. What runs on every
//! build is that the list's rows change the frame the card is handed.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::super::{fixture, scanout_tests};
use super::{ADesktop, row_of};
use crate::an_application_is_picked_in_settings::ApplicationRow;
use crate::direct_target::Target;
use crate::settings_categories::SettingsCategory;
use crate::settings_pointed_at::PointedAt;
use crate::settings_testing::{
    CountingDoor, a_browser_and_a_mail_app, a_persons_machine, doors, light, noon, settings_words,
};
use crate::software_scanout::SoftwarePainter;
use crate::{SettingsKey, SettingsPress, SettingsRow, SettingsWindow};

/// A key that means `key` and nothing else.
fn press(window: &mut SettingsWindow, key: SettingsKey, door: &CountingDoor) {
    let strings = settings_words();
    window.pressed(
        SettingsPress {
            key,
            chord: None,
            filling: crate::a_key_fills_in_a_provider::FillingInKey::Nothing,
        },
        doors(&strings, door),
    );
}

/// Move the keyboard to `row` on the folders page, without pressing it.
fn focused_on(window: &mut SettingsWindow, row: &SettingsRow, door: &CountingDoor) {
    if window.focused(noon()).as_ref() == Some(row) {
        return;
    }
    let _ = window.pointed_at(&PointedAt::ACategory(SettingsCategory::FoldersAloCanUse), noon());
    press(window, SettingsKey::NextSection, door);
    for _ in 0..256 {
        if window.focused(noon()).as_ref() == Some(row) {
            return;
        }
        press(window, SettingsKey::Next, door);
    }
    panic!("{row:?} is not a row: {:?}", window.rows(noon()));
}

/// **Enter on `row` changes the frame**: the frame with the keyboard already
/// on the row, and the frame after Enter, differ — so a focus ring moving is
/// never what made them differ.
fn enter_changes_the_frame(
    window: &mut SettingsWindow,
    row: &SettingsRow,
    door: &CountingDoor,
    name: &str,
) -> (Vec<u8>, (i32, i32)) {
    focused_on(window, row, door);
    let before = handed(window);
    press(window, SettingsKey::Choose, door);
    let after = handed(window);
    written(&after, name);
    assert_ne!(before.0, after.0, "Enter on {row:?} did not reach the frame");
    after
}

/// The frame a card is handed for the desktop with Settings over it, as XRGB
/// bytes with the fixture's padded rows, and the display's size.
fn handed(window: &SettingsWindow) -> (Vec<u8>, (i32, i32)) {
    use crate::FrameTarget;

    let desktop = ADesktop::with(None);
    let (device, log) = fixture(&[], 0);
    let mut labels = crate::WindowControlLabels::new().unwrap();
    let mut target = Target::new(
        SoftwarePainter::new().unwrap(),
        device,
        scanout_tests::output(),
    );
    let size = target.size();
    let pictures = crate::nested_desktop::frame_pictures(
        crate::TheDesktop::now(&desktop),
        None,
        None,
        &mut labels,
        (size.w, size.h),
        None,
    )
    .unwrap();
    // **Settings as `direct_desktop` draws it on a display it has no room on
    // the canvas for**: laid out on the display, at its scale.
    let settings = crate::settings_raster::picture(
        window,
        &settings_words(),
        &mut labels,
        (size.w, size.h),
        light(),
        noon(),
        None,
        crate::display_scale::DisplayScale::of_per_cent(100),
    )
    .unwrap();
    crate::presentation::NativeTarget::submit_native_layers(
        &mut target,
        &[],
        &[],
        &crate::Cursor::Default,
        crate::scene_native::NativeLayers {
            settings: Some(&settings),
            ..pictures.layers(&[])
        },
    )
    .unwrap();
    let pixels = log.borrow().pixels.clone();
    drop(target);
    (pixels, (size.w, size.h))
}

/// Write the frame as a PNG, where a folder was named for it.
fn written((pixels, (wide, tall)): &(Vec<u8>, (i32, i32)), name: &str) {
    let Some(folder) = std::env::var_os("ALO_APPLICATION_GRANT_PICTURES") else {
        return;
    };
    let row = row_of(*wide);
    let mut image =
        image::RgbImage::new(u32::try_from(*wide).unwrap(), u32::try_from(*tall).unwrap());
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let at = usize::try_from(y).unwrap() * row + usize::try_from(x).unwrap() * 4;
        let bgr = pixels.get(at..at + 3).unwrap_or(&[0, 0, 0]);
        *pixel = image::Rgb([bgr[2], bgr[1], bgr[0]]);
    }
    image.save(std::path::Path::new(&folder).join(name)).unwrap();
}

/// **The folders page with *Give alo an application*, then the list it opens,
/// then the question of how long, each reach the frame**: every step changes
/// the bytes the card is handed, so none of them is a picture that never got
/// there.
#[test]
fn giving_alo_an_application_reaches_the_frame_a_display_is_handed() {
    let machine = a_persons_machine("an-application-drawn");
    let door = CountingDoor::revoking();
    let mut window = machine.opened();
    window.the_applications_are(a_browser_and_a_mail_app());
    let _ = window.pointed_at(&PointedAt::ACategory(SettingsCategory::FoldersAloCanUse), noon());
    written(&handed(&window), "1-folders-alo-can-use.png");
    enter_changes_the_frame(
        &mut window,
        &SettingsRow::GrantAnApplication,
        &door,
        "2-select-an-application.png",
    );
    enter_changes_the_frame(
        &mut window,
        &SettingsRow::ChoosingAnApplication(ApplicationRow::This("org.mozilla.firefox".to_owned())),
        &door,
        "3-how-long.png",
    );
    enter_changes_the_frame(
        &mut window,
        &SettingsRow::ChoosingAnApplication(ApplicationRow::For(
            crate::how_long_a_grant_lasts::HowLong::AnHour,
        )),
        &door,
        "4-given.png",
    );
    assert_eq!(door.knocks.get(), 1, "the grant the frames show was not made");
}
