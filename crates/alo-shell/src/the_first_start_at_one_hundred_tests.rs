//! **At 100 per cent the first-start sheets are the pixels they were** before
//! they followed the screen's own density.
//!
//! Held to hashes taken from `main` at cdce3e15, the tree before that change, by the same
//! code, twice, with the same answers both times. Drawn with the shell's
//! bundled faces only (`WindowControlLabels`), so a machine's own fonts cannot
//! move them.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
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
            0x65fb_5e06_9413_07e9,
            0x0306_039b_8963_6f44,
            0xa730_dc0b_ce66_c79e,
            0xdd58_b88b_6c1e_865b,
            0x38f9_e8e2_3110_e10e,
            0x709c_4fc5_8036_790b,
            0xfef5_f13c_f256_4ec6,
            0xe9ae_26d0_30e1_f18d,
            0x1b08_7f87_c01f_501e,
            0xa7ee_980d_1017_99fd,
            0x401c_dd65_b3ab_db9f,
            0x4a19_2fe8_edb5_21c0,
        ],
        "a first-start sheet at 100 per cent is not the pixels main drew: {hashes:#x?}"
    );
}
