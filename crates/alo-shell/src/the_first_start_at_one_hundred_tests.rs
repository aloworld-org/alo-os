//! **At 100 per cent the first-start sheets are the pixels they were** before
//! they followed the screen's own density.
//!
//! Held to hashes taken from `main` at cdce3e15, the tree before that change, by the same
//! code, twice, with the same answers both times. Drawn with the shell's
//! bundled faces only (`WindowControlLabels`), so a machine's own fonts cannot
//! move them.
//!
//! **Re-taken on 2026-11-01** for the first start the owner approved that day
//! (`562:342`, `562:462`, `562:508`), which changed every sheet on purpose:
//! from the gate's run of this tree, which printed them. **They pin what is
//! drawn; they do not say it is right** — that is the comparison by eye with
//! the design, through `the_approved_frames_are_drawn`'s pictures, which a
//! person makes by hand and this test does not.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None, Err or index is the failure being reported"
)]

use alo_accounts::Accounts;
use alo_appearance::{Scheme, TextScale};
use alo_greeting::{Greeting, Knocking, NotAnswered};
use alo_sessiond::{Answered, Knock};

use crate::a_picture_hashed::Hashed;
use crate::display_scale::DisplayScale;
use crate::sign_in_raster::{SignInPicture, WhoseScreen, picture};
use crate::{SignInKey, SignInScreen, Signing, WindowControlLabels};

/// A door that opens for anybody who reaches it.
#[derive(Debug)]
struct ADoor;

impl Knocking for ADoor {
    fn knock(&self, _: Knock) -> Result<Answered, NotAnswered> {
        Ok(Answered::Opened)
    }
}

/// The ordinary light look.
pub(super) fn light() -> crate::SignInLook {
    crate::SignInLook {
        contrast: crate::Contrast::AsDesigned,
        scheme: Scheme::Light,
        scale: TextScale::ordinary(),
    }
}

/// A machine that knows nobody, walked through these keys.
fn a_fresh_machine_after(keys: &[SignInKey]) -> (tempfile::TempDir, SignInScreen<ADoor>) {
    let kept = tempfile::tempdir().unwrap();
    let store = kept.path().join("accounts.toml");
    alo_accounts::kept(&store, &Accounts::none().unwrap()).unwrap();
    let greeting = Greeting::at(&store, 1000, ADoor).unwrap();
    let words = crate::what_this_machine_can_say().unwrap();
    let mut screen = SignInScreen::of(Ok(greeting), words);
    for key in keys {
        screen = match screen.pressed(*key) {
            Signing::Still(screen) => *screen,
            Signing::HandedOver(_) => panic!("a first-start sheet signed somebody in"),
        };
    }
    (kept, screen)
}

/// Every sheet a fresh machine shows before anybody is on it, by the keys that
/// reach it: the overview, language, region and keyboard, the name with a
/// letter typed and the PIN with a digit typed.
pub(super) fn every_sheet() -> [Vec<SignInKey>; 6] {
    use SignInKey::{Enter, Later, Letter, OtherField};
    [
        vec![],
        vec![Enter],
        vec![Later, Enter],
        vec![Later, Later, Enter],
        vec![OtherField, Enter, Letter('a')],
        vec![OtherField, Enter, Letter('a'), OtherField, Letter('1')],
    ]
}

/// That sheet, drawn at `size` and `display`.
pub(super) fn the_sheet(
    keys: &[SignInKey],
    size: (i32, i32),
    display: DisplayScale,
) -> Result<SignInPicture, crate::RenderError> {
    let (_kept, screen) = a_fresh_machine_after(keys);
    let mut labels = WindowControlLabels::new().unwrap();
    picture(
        screen.shows(),
        &mut labels,
        screen.words(),
        size,
        light(),
        WhoseScreen::ItsOwn,
        display,
    )
}

/// Everything a picture paints, hashed.
pub(super) fn hash_of(drawn: &SignInPicture) -> u64 {
    let mut hashed = Hashed::begun();
    if let Some(surface) = drawn.surface.as_ref() {
        hashed.washed(surface);
    }
    hashed.solids(&drawn.solids);
    hashed.inked(&drawn.inked);
    hashed.done()
}

#[test]
fn at_one_hundred_every_sheet_is_the_pixels_main_drew() {
    let mut hashes = Vec::new();
    for keys in every_sheet() {
        for size in [(1440, 960), (2560, 1440)] {
            hashes.push(hash_of(&the_sheet(&keys, size, DisplayScale::ONE).unwrap()));
        }
    }
    // The control: six different sheets, so the keys reached six places.
    let mut different = hashes.clone();
    different.sort_unstable();
    different.dedup();
    assert_eq!(different.len(), hashes.len(), "{hashes:#x?}");
    assert_eq!(
        hashes,
        [
            0xc7d7_1af9_762c_2ae4,
            0x4828_56a8_a24c_3b79,
            0xd34e_c7c5_02a7_df72,
            0xb9d4_411e_0ed1_ab93,
            0x69d8_8724_8af0_f822,
            0xd10c_dfb8_0b86_ef03,
            0x784d_ce7a_811d_65ec,
            0x5e04_9bcd_cdb4_3583,
            0x0ef0_7432_0eb6_a38e,
            0x2bb2_94b5_8d05_d591,
            0x8b98_b033_7975_07bb,
            0x5a32_810b_1798_4182,
        ],
        "a first-start sheet at 100 per cent is not the pixels main drew: {hashes:x?}"
    );
}

/// **The approved frames, drawn, for the owner to compare** — *Start with your
/// language* `562:342` at 1440 × 960 and `562:402` at 1280 × 800, *What should
/// we call you?* `562:462`, and *Add a PIN?* `562:508`.
///
/// With `ALO_ROUND_2_PICTURES` set to a folder each is written there as a PNG,
/// which is how the rendered-against-design pictures are made: a check somebody
/// runs by hand. What runs on every build is that each draws.
#[test]
fn the_approved_frames_are_drawn() {
    use SignInKey::{Enter, Letter, OtherField};
    for (keys, size, name) in [
        (
            vec![],
            (1440, 960),
            "first-start-language-improved-1440x960-light.png",
        ),
        (
            vec![],
            (1280, 800),
            "first-start-language-improved-1280x800-light.png",
        ),
        (
            vec![OtherField, Enter, Letter('a')],
            (1440, 960),
            "first-start-your-name-improved-1440x960-light.png",
        ),
        (
            vec![OtherField, Enter, Letter('a'), Enter, Letter('1')],
            (1440, 960),
            "first-start-add-a-pin-improved-1440x960-light.png",
        ),
    ] {
        let drawn = the_sheet(&keys, size, DisplayScale::ONE).unwrap();
        let Some(folder) = std::env::var_os("ALO_ROUND_2_PICTURES") else {
            continue;
        };
        let (wide, tall) = (
            u32::try_from(size.0).unwrap(),
            u32::try_from(size.1).unwrap(),
        );
        let mut image = image::RgbImage::from_pixel(wide, tall, image::Rgb([0xFA, 0xF7, 0xF2]));
        let mut put = |x: i32, y: i32, colour: [u8; 3], alpha: f32| {
            let (Ok(x), Ok(y)) = (u32::try_from(x), u32::try_from(y)) else {
                return;
            };
            if x >= wide || y >= tall {
                return;
            }
            let under = image.get_pixel(x, y).0;
            let over = |which: usize| {
                (f32::from(colour[which]) * alpha + f32::from(under[which]) * (1.0 - alpha)).round()
                    as u8
            };
            image.put_pixel(x, y, image::Rgb([over(0), over(1), over(2)]));
        };
        if let Some(surface) = drawn.surface.as_ref() {
            for (at, pixel) in surface.pixels.as_chunks::<4>().0.iter().enumerate() {
                let at = i32::try_from(at).unwrap();
                put(
                    surface.area.loc.x + at % surface.area.size.w,
                    surface.area.loc.y + at / surface.area.size.w,
                    [pixel[0], pixel[1], pixel[2]],
                    f32::from(pixel[3]) / 255.0,
                );
            }
        }
        for solid in &drawn.solids {
            for y in solid.area.loc.y..solid.area.loc.y + solid.area.size.h {
                for x in solid.area.loc.x..solid.area.loc.x + solid.area.size.w {
                    put(x, y, solid.colour, 1.0);
                }
            }
        }
        for inked in &drawn.inked {
            for (at, pixel) in inked.pixels.iter().enumerate() {
                let at = i32::try_from(at).unwrap();
                put(
                    inked.area.loc.x + at % inked.area.size.w,
                    inked.area.loc.y + at / inked.area.size.w,
                    *pixel,
                    1.0,
                );
            }
        }
        image
            .save(std::path::Path::new(&folder).join(name))
            .unwrap();
    }
}
