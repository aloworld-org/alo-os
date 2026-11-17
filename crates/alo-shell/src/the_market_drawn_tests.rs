//! **The market's labels in the bytes a display is handed** — not a picture of
//! the market alone.
//!
//! The market is placed by `crate::where_the_market_opens` on the display, put
//! on the desktop frame exactly as `crate::direct_desktop` puts it, laid out by
//! `crate::nested_desktop::frame_pictures`, painted by the compositor's own
//! software renderer and read back from the injected card.
//!
//! The catalogue is the decided one with two entries added under *Audio* so all
//! three labels have a row to be on: REAPER, which Flathub lists as
//! proprietary and unverified, and SoundCraft, an early version. Narrowed to
//! *Audio*, the window shows Audacity (*not verified*), REAPER (*not open
//! source · not verified*) and SoundCraft (*early version*).
//!
//! When `ALO_MARKET_PICTURES` names a folder, the frame is written there as a
//! PNG, which is a check somebody runs by hand. What runs on every build is
//! that each label changes the frame the card is handed.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_software::{Catalogue, Shipped};

use super::super::{fixture, scanout_tests};
use super::{ADesktop, row_of};
use crate::direct_target::Target;
use crate::market_window::MarketWindow;
use crate::software_scanout::SoftwarePainter;
use crate::{MarketKey, TheMarketsWords};

/// The decided catalogue, and two more under *Audio*.
fn a_catalogue_with_every_label() -> Catalogue {
    let text = format!(
        "{}\n[[application]]\ncategory = \"audio\"\nidentifier = \"fm.reaper.Reaper\"\nsource = \
         \"flathub\"\nversion = \"7.85\"\nlicence = \
         \"LicenseRef-proprietary=https://www.reaper.fm/dist-agreement.php\"\nverified = \
         false\nearly = false\n\n[[application]]\ncategory = \"audio\"\nidentifier = \
         \"ai.storyteller.soundcraft\"\nsource = \"flathub\"\nversion = \"0.20.1\"\nlicence = \
         \"Apache-2.0\"\nverified = true\nearly = true\n",
        alo_software::catalogue::THE_CATALOGUE
    );
    Catalogue::read(&text, &Shipped::decided().unwrap()).unwrap()
}

/// A desktop with the market open over it, narrowed to *Audio*.
struct WithTheMarket {
    desktop: ADesktop,
    window: MarketWindow,
    words: TheMarketsWords,
}

impl WithTheMarket {
    fn open(catalogue: Catalogue) -> Self {
        let desktop = ADesktop::with(None);
        let words = TheMarketsWords::said(&desktop.strings);
        let mut window = MarketWindow::closed();
        window.opened(catalogue, Vec::new());
        // Everything, then the eight kinds in order: Audio is the seventh.
        for _ in 0..7 {
            window.pressed(MarketKey::After);
        }
        Self {
            desktop,
            window,
            words,
        }
    }
}

impl crate::TheDesktop for WithTheMarket {
    fn now(&self) -> crate::DesktopFrame<'_> {
        let mut frame = crate::TheDesktop::now(&self.desktop);
        let (wide, tall) = (1280, 720);
        let view = crate::where_a_task_window_opens::as_a_frame_draws_it(
            smithay::utils::Rectangle::new((0, 0).into(), (wide, tall).into()),
        );
        frame.market = crate::where_the_market_opens::where_the_market_opens(
            view,
            &self.window.shows(),
            frame.look.scale(),
            crate::display_scale::DisplayScale::ONE,
        )
        .map(|at| crate::market_raster::AMarketOnTheDisplay {
            at,
            showing: self.window.shows(),
            words: &self.words,
        });
        assert!(frame.market.is_some(), "the market had nowhere to be drawn");
        frame
    }
}

/// The frame a card is handed for this desktop, as XRGB bytes with the
/// fixture's padded rows, and the display's size.
fn handed(desktop: &impl crate::TheDesktop) -> (Vec<u8>, (i32, i32)) {
    use crate::FrameTarget;

    let (device, log) = fixture(&[], 0);
    let mut labels = crate::WindowControlLabels::new().unwrap();
    let mut target = Target::new(
        SoftwarePainter::new().unwrap(),
        device,
        scanout_tests::output(),
    );
    let size = target.size();
    let pictures = crate::nested_desktop::frame_pictures(
        crate::TheDesktop::now(desktop),
        None,
        None,
        &mut labels,
        (size.w, size.h),
        None,
    )
    .unwrap();
    crate::presentation::NativeTarget::submit_native_layers(
        &mut target,
        &[],
        &[],
        &crate::Cursor::Default,
        pictures.layers(&[]),
    )
    .unwrap();
    let pixels = log.borrow().pixels.clone();
    drop(target);
    (pixels, (size.w, size.h))
}

/// Write the frame as a PNG, where a folder was named for it.
fn written((pixels, (wide, tall)): &(Vec<u8>, (i32, i32)), name: &str) {
    let Some(folder) = std::env::var_os("ALO_MARKET_PICTURES") else {
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

/// **Each label reaches the frame**: the market narrowed to *Audio* with the
/// labelled entries is a different frame from the same market with every
/// entry made open, published by its maker and not early — so the labels,
/// and not something else, are what differs.
#[test]
fn the_three_labels_reach_the_frame_a_display_is_handed() {
    let labelled = handed(&WithTheMarket::open(a_catalogue_with_every_label()));
    written(&labelled, "market-labels.png");

    let plain_text = format!(
        "{}\n[[application]]\ncategory = \"audio\"\nidentifier = \"fm.reaper.Reaper\"\nsource = \
         \"flathub\"\nversion = \"7.85\"\nlicence = \"MIT\"\nverified = true\nearly = \
         false\n\n[[application]]\ncategory = \"audio\"\nidentifier = \
         \"ai.storyteller.soundcraft\"\nsource = \"flathub\"\nversion = \"0.20.1\"\nlicence = \
         \"Apache-2.0\"\nverified = true\nearly = false\n",
        alo_software::catalogue::THE_CATALOGUE.replace(
            "identifier = \"org.audacityteam.Audacity\"\nsource = \"flathub\"\nversion = \
             \"3.7.8\"\nlicence = \"GPL-3.0-only\"\nverified = false",
            "identifier = \"org.audacityteam.Audacity\"\nsource = \"flathub\"\nversion = \
             \"3.7.8\"\nlicence = \"GPL-3.0-only\"\nverified = true"
        )
    );
    let plain = Catalogue::read(&plain_text, &Shipped::decided().unwrap()).unwrap();
    assert!(
        plain
            .under(alo_software::Category::Audio)
            .all(|one| one.labels().is_empty()),
        "the premise: nothing on the Audio shelf has a label"
    );
    let unlabelled = handed(&WithTheMarket::open(plain));
    written(&unlabelled, "market-no-labels.png");

    assert_ne!(
        labelled.0, unlabelled.0,
        "the labels did not reach the frame"
    );
}
