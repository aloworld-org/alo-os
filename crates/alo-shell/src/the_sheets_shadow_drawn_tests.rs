//! **A sheet's shadow only darkens what is under it**, in the bytes a display
//! is handed — the *Open with* sheet and the GitHub sign-in sheet, drawn
//! through `crate::nested_desktop::frame_pictures` and the compositor's own
//! software renderer, as every frame is.
//!
//! **Why this exists.** Both sheets handed the renderer their raster in
//! straight alpha, while the renderer blends premultiplied colour. A pixel of
//! the shadow's area that is almost transparent then adds its colour to the
//! desktop under it, and the whole shadow drew as a pale rounded box around
//! the sheet. Settings, the record and the bar premultiply; these two did not.
//! Seen on 2026-11-16, drawing a third sheet copied from them.
//!
//! The measure: a pixel inside the raster but outside the sheet, beside its
//! middle, is **no lighter** with the sheet up than without it.
//!
//! When `ALO_SHEET_SHADOW_PICTURES` names a folder each frame is written there
//! as a PNG, which is a check somebody runs by hand. What runs on every build
//! is the measure.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::net::UnixStream;
use std::time::Instant;

use super::super::{fixture, scanout_tests};
use super::{ADesktop, row_of};
use crate::direct_target::Target;
use crate::software_scanout::SoftwarePainter;

/// Every word the machine says, and both sheets'.
fn words() -> alo_strings::Strings {
    let mut vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    crate::the_settings_words::declare_the_settings_words(&mut vocabulary).unwrap();
    crate::the_github_sheets_words::declare_the_github_sheets_words(&mut vocabulary).unwrap();
    crate::the_open_with_sheets_words::declare_the_open_with_sheets_words(&mut vocabulary).unwrap();
    alo_strings::Strings::of(vocabulary)
}

/// Which sheet is up.
#[derive(Clone, Copy)]
enum Up {
    /// Nothing: the desktop alone.
    Nothing,
    /// The *Open with* sheet.
    OpenWith,
    /// The GitHub sign-in sheet, waiting for its code.
    GitHub,
}

/// A frame as the card is handed it: its bytes, the display's size, and the
/// raster the sheet was drawn in, where one was.
struct Handed {
    /// XRGB bytes with the fixture's padded rows.
    pixels: Vec<u8>,
    /// The display's size.
    size: (i32, i32),
    /// Where the sheet's raster is, shadow and all.
    raster: Option<crate::settings_canvas::Area>,
}

impl Handed {
    /// How light the pixel at (`x`, `y`) is: its three channels summed.
    fn lightness(&self, x: i32, y: i32) -> u32 {
        let at =
            usize::try_from(y).unwrap() * row_of(self.size.0) + usize::try_from(x).unwrap() * 4;
        self.pixels[at..at + 3]
            .iter()
            .map(|it| u32::from(*it))
            .sum()
    }
}

/// The frame with `up` over the desktop, light or dark.
fn handed(up: Up, dark: bool) -> Handed {
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
    let strings = words();
    let display = crate::display_scale::DisplayScale::of_per_cent(100);
    let (_portal, open_with) = an_open_with_sheet();
    let mut github = crate::the_github_sheet::TheGitHubSheet::default();
    assert!(github.opened());
    let open_with_drawn = matches!(up, Up::OpenWith).then(|| {
        crate::the_open_with_sheet_drawn::drawn(
            open_with.shows().unwrap(),
            &strings,
            &mut labels,
            (size.w, size.h),
            look,
            display,
            &|identifier| (identifier.to_owned(), None),
        )
        .unwrap()
    });
    let github_drawn = matches!(up, Up::GitHub).then(|| {
        crate::the_github_sheet_drawn::drawn(
            github.shows().unwrap(),
            &strings,
            &mut labels,
            (size.w, size.h),
            look,
            display,
        )
        .unwrap()
    });
    let raster = open_with_drawn
        .as_ref()
        .map(|drawn| drawn.washed.area)
        .or(github_drawn.as_ref().map(|drawn| drawn.washed.area))
        .map(|area| {
            smithay::utils::Rectangle::new(
                (area.loc.x, area.loc.y).into(),
                (area.size.w, area.size.h).into(),
            )
        });
    crate::presentation::NativeTarget::submit_native_layers(
        &mut target,
        &[],
        &[],
        &crate::Cursor::Default,
        crate::scene_native::NativeLayers {
            open_with: open_with_drawn.as_ref(),
            github_sheet: github_drawn.as_ref().map(|drawn| &drawn.washed),
            ..pictures.layers(&[])
        },
    )
    .unwrap();
    let pixels = log.borrow().pixels.clone();
    drop(target);
    Handed {
        pixels,
        size: (size.w, size.h),
        raster,
    }
}

/// An *Open with* sheet up for Papers asking to open a web link; the portal's
/// end is handed back to be kept open.
fn an_open_with_sheet() -> (UnixStream, crate::the_open_with_sheet::TheOpenWithSheet) {
    use alo_applications::{Application, Link};
    let (portal, desktop) = UnixStream::pair().unwrap();
    desktop.set_nonblocking(true).unwrap();
    let asked = alo_acting::ToChooseALink::of(
        Application::identified("org.gnome.Papers").unwrap(),
        Link::checked("https://example.com/invoice").unwrap(),
        vec![
            Application::identified("org.mozilla.firefox").unwrap(),
            Application::identified("io.gitlab.librewolf-community").unwrap(),
        ],
    )
    .unwrap();
    let mut sheet = crate::the_open_with_sheet::TheOpenWithSheet::none();
    sheet.asked(
        vec![crate::the_open_with_sheet::AskedToChoose::of(
            asked, desktop,
        )],
        true,
        Instant::now(),
    );
    (portal, sheet)
}

/// Write the frame as a PNG, where a folder was named for it.
fn written(handed: &Handed, name: &str) {
    let Some(folder) = std::env::var_os("ALO_SHEET_SHADOW_PICTURES") else {
        return;
    };
    let (wide, tall) = handed.size;
    let row = row_of(wide);
    let mut image =
        image::RgbImage::new(u32::try_from(wide).unwrap(), u32::try_from(tall).unwrap());
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let at = usize::try_from(y).unwrap() * row + usize::try_from(x).unwrap() * 4;
        let bgr = handed.pixels.get(at..at + 3).unwrap_or(&[0, 0, 0]);
        *pixel = image::Rgb([bgr[2], bgr[1], bgr[0]]);
    }
    image
        .save(std::path::Path::new(&folder).join(name))
        .unwrap();
}

/// How much darker the dark sheet's own surface is than the desktop under it:
/// what says the walk below has reached the sheet. `navy/900` against the
/// canvas colour differs by far more than this in every channel summed.
const THE_SHEET_BEGINS: u32 = 300;

/// **Walking in from the raster's edge at its middle, every pixel before the
/// sheet is no lighter than the desktop alone** — in the dark scheme, where
/// the sheet's own surface is dark and so is told apart from a pale box by
/// lightness alone. A shadow only darkens; a box of straight alpha blended as
/// premultiplied lightens.
#[test]
fn a_sheets_shadow_only_darkens_what_is_under_it() {
    let nothing = handed(Up::Nothing, true);
    for (up, named) in [(Up::OpenWith, "open-with"), (Up::GitHub, "github")] {
        written(&handed(up, false), &format!("{named}-light.png"));
        let with = handed(up, true);
        written(&with, &format!("{named}-dark.png"));
        let raster = with.raster.unwrap();
        let y = raster.loc.y + raster.size.h / 2;
        let mut walked = 0;
        for x in raster.loc.x..raster.loc.x + raster.size.w / 2 {
            let (here, under) = (with.lightness(x, y), nothing.lightness(x, y));
            if here + THE_SHEET_BEGINS < under {
                break;
            }
            assert!(
                here <= under,
                "the {named} sheet drew {here} at ({x}, {y}), beside it, where the desktop alone is {under}: a pale box, not a shadow"
            );
            walked += 1;
        }
        assert!(
            walked > 0 && walked < raster.size.w / 2,
            "the {named} sheet: the walk never reached the sheet, or began on it ({walked})"
        );
    }
}
