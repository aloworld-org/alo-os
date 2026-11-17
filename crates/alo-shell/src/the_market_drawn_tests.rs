//! **The market's labels in the bytes a display is handed** — not a picture of
//! the market alone.
//!
//! The market is placed by `crate::where_the_market_opens` on the display, put
//! on the desktop frame exactly as `crate::direct_desktop` puts it, laid out by
//! `crate::nested_desktop::frame_pictures`, painted by the compositor's own
//! software renderer and read back from the injected card.
//!
//! The catalogue is the decided one with one entry added under *Making music*,
//! so all three labels have a row to be on: SoundCraft, an early version.
//! Narrowed to *Making music*, the window shows Ardour and MuseScore (*not
//! verified*), Bitwig Studio (*not open source*), REAPER (*not open source ·
//! not verified*) and SoundCraft (*early version*) — every one but SoundCraft
//! exactly as the decided catalogue lists it.
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

/// The decided catalogue, and SoundCraft under *Making music*.
fn a_catalogue_with_every_label() -> Catalogue {
    let text = format!(
        "{}\n[[application]]\ncategory = \"music-making\"\nidentifier = \
         \"ai.storyteller.soundcraft\"\nsource = \"flathub\"\nversion = \"0.20.1\"\nlicence = \
         \"Apache-2.0\"\nverified = true\nearly = true\n",
        alo_software::catalogue::THE_CATALOGUE
    );
    Catalogue::read(&text, &Shipped::decided().unwrap()).unwrap()
}

/// A desktop with the market open over it, narrowed to *Making music*.
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
        // Everything, then the kinds in order: Making music is the eleventh.
        let eleventh = alo_software::Category::EVERY
            .iter()
            .position(|kind| *kind == alo_software::Category::MusicMaking)
            .unwrap()
            + 1;
        for _ in 0..eleventh {
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

/// **Each label reaches the frame**: the market narrowed to *Making music* with
/// the labelled entries is a different frame from the same market with every
/// entry made open, published by its maker and not early — so the labels,
/// and not something else, are what differs.
#[test]
fn the_three_labels_reach_the_frame_a_display_is_handed() {
    let labelled = handed(&WithTheMarket::open(a_catalogue_with_every_label()));
    written(&labelled, "market-labels.png");

    // Every entry made open, verified and not early, line by line.
    let plain_text: String = format!(
        "{}\n[[application]]\ncategory = \"music-making\"\nidentifier = \
         \"ai.storyteller.soundcraft\"\nsource = \"flathub\"\nversion = \"0.20.1\"\nlicence = \
         \"Apache-2.0\"\nverified = true\nearly = false\n",
        alo_software::catalogue::THE_CATALOGUE
    )
    .lines()
    .map(|line| {
        if line.starts_with("licence = \"LicenseRef-proprietary") {
            "licence = \"MIT\""
        } else if line == "verified = false" {
            "verified = true"
        } else if line == "early = true" {
            "early = false"
        } else {
            line
        }
    })
    .collect::<Vec<_>>()
    .join("\n");
    let plain = Catalogue::read(&plain_text, &Shipped::decided().unwrap()).unwrap();
    assert!(
        plain.every().iter().all(|one| one.labels().is_empty()),
        "the premise: nothing in the plain catalogue has a label"
    );
    let unlabelled = handed(&WithTheMarket::open(plain));
    written(&unlabelled, "market-no-labels.png");

    assert_ne!(
        labelled.0, unlabelled.0,
        "the labels did not reach the frame"
    );
}
