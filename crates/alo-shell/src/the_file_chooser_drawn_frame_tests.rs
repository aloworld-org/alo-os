//! **The file chooser, in the bytes a display is handed** — not a picture of
//! the sheet alone.
//!
//! The desktop is laid out as every frame lays it out, the sheet is put in the
//! native layers in the place `crate::direct_desktop` puts it, painted by the
//! compositor's own software renderer, and read back from the injected card. A
//! sheet that passed its own tests and never reached the frame fails here.
//!
//! When `ALO_FILE_CHOOSER_PICTURES` names a folder each frame is written there
//! as a PNG, which is a check somebody runs by hand. What runs on every build
//! is that the sheet, and where the keyboard is on it, change the frame.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Instant;

use alo_acting::{ToChoose, ToChooseAFile};
use alo_applications::Application;
use alo_file_chooser::{Folders, Key, Kind, Listing, NotListed};

use super::super::{fixture, scanout_tests};
use super::{ADesktop, row_of};
use crate::direct_target::Target;
use crate::software_scanout::SoftwarePainter;
use crate::the_file_chooser::{AskedForAFile, TheFileChooser};

/// The sheet's words, and everything else the machine says.
fn words() -> alo_strings::Strings {
    let mut vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    crate::the_file_choosers_words::declare_the_file_choosers_words(&mut vocabulary).unwrap();
    alo_strings::Strings::of(vocabulary)
}

/// Anna's home, as a person might have it.
struct AHome;

impl Folders for AHome {
    fn listed(&self, at: &Path) -> Result<Listing, NotListed> {
        if at == Path::new("/home/anna") {
            Ok(Listing::of(
                vec!["Documents".into(), "Downloads".into(), "Pictures".into()],
                vec!["march invoice.pdf".into(), "notes.txt".into()],
            ))
        } else {
            Err(NotListed::NotThere)
        }
    }

    fn holds(&self, at: &Path, name: &str) -> Option<Kind> {
        (at == Path::new("/home/anna") && name == "notes.txt").then_some(Kind::File)
    }
}

/// A frame as the card is handed it, and the display's size.
type AFrame = (Vec<u8>, (i32, i32));

/// The frame a card is handed for the desktop, with `sheet` over it when one
/// is up.
fn handed(sheet: &TheFileChooser, dark: bool) -> AFrame {
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
    let called = |identifier: &str| {
        if identifier == "org.gnome.TextEditor" {
            "Text Editor".to_owned()
        } else {
            identifier.to_owned()
        }
    };
    let drawn = sheet.shows().map(|shows| {
        crate::the_file_chooser_drawn::drawn(
            shows,
            &words(),
            &mut labels,
            (size.w, size.h),
            look,
            crate::display_scale::DisplayScale::of_per_cent(100),
            &called,
        )
        .unwrap()
    });
    crate::presentation::NativeTarget::submit_native_layers(
        &mut target,
        &[],
        &[],
        &crate::Cursor::Default,
        crate::scene_native::NativeLayers {
            file_chooser: drawn.as_ref().map(|drawn| &drawn.washed),
            ..pictures.layers(&[])
        },
    )
    .unwrap();
    let pixels = log.borrow().pixels.clone();
    drop(target);
    (pixels, (size.w, size.h))
}

/// Write the frame as a PNG, where a folder was named for it.
fn written((pixels, (wide, tall)): &AFrame, name: &str) {
    let Some(folder) = std::env::var_os("ALO_FILE_CHOOSER_PICTURES") else {
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

/// A sheet up for Text Editor; the portal's end is handed back to be kept.
fn a_sheet(to: ToChoose) -> (UnixStream, TheFileChooser) {
    let (portal, desktop) = UnixStream::pair().unwrap();
    desktop.set_nonblocking(true).unwrap();
    let asked = ToChooseAFile::of(
        Application::identified("org.gnome.TextEditor").unwrap(),
        to,
        PathBuf::from("/home/anna"),
    )
    .unwrap();
    let mut sheet = TheFileChooser::none();
    sheet.asked(
        vec![AskedForAFile::of(asked, desktop)],
        true,
        Instant::now(),
        &AHome,
    );
    (portal, sheet)
}

/// **The sheet reaches the frame a display is handed**, and so does where the
/// keyboard is: the desktop alone, the sheet opening, the keyboard moved, and
/// the sheet saving are four different frames.
#[test]
fn the_file_chooser_reaches_the_frame_a_display_is_handed() {
    let nothing = handed(&TheFileChooser::none(), false);
    written(&nothing, "0-the-desktop.png");
    let (_opening, mut sheet) = a_sheet(ToChoose::Open);
    let opening = handed(&sheet, false);
    written(&opening, "1-open-a-file.png");
    assert_ne!(nothing.0, opening.0, "the sheet did not reach the frame");
    sheet.pressed(Key::Down, Instant::now(), &AHome);
    let moved = handed(&sheet, false);
    written(&moved, "2-the-keyboard-moved.png");
    assert_ne!(
        opening.0, moved.0,
        "where the keyboard is did not reach the frame"
    );
    let (_saving, mut saving) = a_sheet(ToChoose::Save {
        name: "notes.txt".to_owned(),
    });
    saving.pressed(Key::Enter, Instant::now(), &AHome);
    let replacing = handed(&saving, false);
    written(&replacing, "3-save-a-file-replacing.png");
    let dark = handed(&saving, true);
    written(&dark, "4-save-a-file-dark.png");
    assert_ne!(
        replacing.0, dark.0,
        "the person's dark scheme did not reach the sheet"
    );
}
