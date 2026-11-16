//! **The sheet alo asks for an application on, in the bytes a display is
//! handed** — not a picture of the sheet alone.
//!
//! The desktop is laid out by `crate::nested_desktop::frame_pictures` as every
//! frame lays it out, the sheet is put in the native layers in the place
//! `crate::direct_desktop` puts it, painted by the compositor's own software
//! renderer, and read back from the injected card. A surface that passed its
//! own tests and never reached the frame fails here.
//!
//! When `ALO_APPLICATION_SHEET_PICTURES` names a folder each frame is written
//! there as a PNG, which is a check somebody runs by hand. What runs on every
//! build is that the sheet, and the length the keyboard is on, change the
//! frame the card is handed.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::Instant;

use super::super::{fixture, scanout_tests};
use super::{ADesktop, row_of};
use crate::direct_target::Target;
use crate::software_scanout::SoftwarePainter;
use crate::the_application_sheet::{AnApplicationWanted, ApplicationSheetKey, TheApplicationSheet};
use crate::the_tasks_a_person_opened::WhichTask;

/// The sheet's words, and everything else the machine says.
fn words() -> alo_strings::Strings {
    let mut vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    crate::the_application_sheets_words::declare_the_application_sheets_words(&mut vocabulary)
        .unwrap();
    alo_strings::Strings::of(vocabulary)
}

/// The frame a card is handed for the desktop, with `sheet` over it when one
/// is up, as XRGB bytes with the fixture's padded rows, and the display's size.
fn handed(sheet: &TheApplicationSheet, dark: bool) -> (Vec<u8>, (i32, i32)) {
    handed_with_its_raster(sheet, dark).0
}

/// The same, and where the sheet's raster was, shadow and all.
fn handed_with_its_raster(
    sheet: &TheApplicationSheet,
    dark: bool,
) -> ((Vec<u8>, (i32, i32)), Option<(i32, i32, i32, i32)>) {
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
    let mut look = crate::settings_testing::light();
    if dark {
        look.scheme = alo_appearance::Scheme::Dark;
        look.accent = alo_appearance::Accent::Indigo.on(alo_appearance::Scheme::Dark);
    }
    // **The sheet as `direct_desktop` draws it**: laid out on the display, at
    // its scale.
    let drawn = sheet.shows().map(|shows| {
        crate::the_application_sheet_drawn::drawn(
            shows,
            &words(),
            &mut labels,
            (size.w, size.h),
            look,
            crate::display_scale::DisplayScale::of_per_cent(100),
        )
        .unwrap()
    });
    crate::presentation::NativeTarget::submit_native_layers(
        &mut target,
        &[],
        &[],
        &crate::Cursor::Default,
        crate::scene_native::NativeLayers {
            application_sheet: drawn.as_ref().map(|drawn| &drawn.washed),
            ..pictures.layers(&[])
        },
    )
    .unwrap();
    let raster = drawn.as_ref().map(|drawn| {
        let area = drawn.washed.area;
        (area.loc.x, area.loc.y, area.size.w, area.size.h)
    });
    let pixels = log.borrow().pixels.clone();
    drop(target);
    ((pixels, (size.w, size.h)), raster)
}

/// Write the frame as a PNG, where a folder was named for it.
fn written((pixels, (wide, tall)): &(Vec<u8>, (i32, i32)), name: &str) {
    let Some(folder) = std::env::var_os("ALO_APPLICATION_SHEET_PICTURES") else {
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
    image
        .save(std::path::Path::new(&folder).join(name))
        .unwrap();
}

/// **The sheet reaches the frame a display is handed**, and so does the
/// length the keyboard is on: the desktop alone, the sheet up on *This task*,
/// and the sheet on *Remember* are three different frames.
#[test]
fn the_sheet_reaches_the_frame_a_display_is_handed() {
    let mut sheet = TheApplicationSheet::default();
    let nothing = handed(&sheet, false);
    written(&nothing, "0-the-desktop.png");
    let _ = sheet.wanted(
        AnApplicationWanted::by(WhichTask::numbered(0), "com.vscodium.codium", "VSCodium"),
        Instant::now(),
    );
    let this_task = handed(&sheet, false);
    written(&this_task, "1-this-task.png");
    assert_ne!(nothing.0, this_task.0, "the sheet did not reach the frame");
    let _ = sheet.pressed(ApplicationSheetKey::Last, Instant::now());
    let remember = handed(&sheet, false);
    written(&remember, "2-remember.png");
    assert_ne!(
        this_task.0, remember.0,
        "the length the keyboard is on did not reach the frame"
    );
    let dark = handed(&sheet, true);
    written(&dark, "3-remember-dark.png");
    assert_ne!(
        remember.0, dark.0,
        "the person's dark scheme did not reach the sheet"
    );
}

/// **The sheet's shadow only darkens what is under it**: walking in from its
/// raster's edge at its middle, in the dark scheme, every pixel before the
/// dark sheet is no lighter than the desktop alone. Straight alpha handed to a
/// renderer that blends premultiplied drew the whole shadow as a pale box
/// (2026-11-16), and this is what notices it.
#[test]
fn the_sheets_shadow_only_darkens_what_is_under_it() {
    let (nothing, _) = handed_with_its_raster(&TheApplicationSheet::default(), true);
    let mut sheet = TheApplicationSheet::default();
    let _ = sheet.wanted(
        AnApplicationWanted::by(WhichTask::numbered(0), "com.vscodium.codium", "VSCodium"),
        Instant::now(),
    );
    let (with, raster) = handed_with_its_raster(&sheet, true);
    let (left, top, wide, tall) = raster.unwrap();
    let y = top + tall / 2;
    let lightness = |(pixels, (across, _)): &(Vec<u8>, (i32, i32)), x: i32| -> u32 {
        let at = usize::try_from(y).unwrap() * row_of(*across) + usize::try_from(x).unwrap() * 4;
        pixels[at..at + 3].iter().map(|it| u32::from(*it)).sum()
    };
    let mut walked = 0;
    for x in left..left + wide / 2 {
        let (here, under) = (lightness(&with, x), lightness(&nothing, x));
        // **The dark sheet begins** where it is far darker than the desktop.
        if here + 300 < under {
            break;
        }
        assert!(
            here <= under,
            "{here} at ({x}, {y}), beside the sheet, where the desktop alone is {under}: a pale box"
        );
        walked += 1;
    }
    assert!(
        walked > 0 && walked < wide / 2,
        "the walk never reached the sheet ({walked})"
    );
}
