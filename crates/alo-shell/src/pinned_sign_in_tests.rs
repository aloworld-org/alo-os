//! The sign-in screen that asks for a name, with one typed, held to the pixels
//! it drew before the UI kit moved anything
//! (`crate::the_desktops_pictures_pinned`).
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_accounts::Accounts;
use alo_appearance::Scheme;
use alo_greeting::{Greeting, Knocking, NotAnswered};
use alo_sessiond::{Answered, Knock};

use super::{Drawn, LOGICAL, every_way, held, in_pixels};
use crate::a_picture_hashed::Hashed;
use crate::sign_in_raster::{WhoseScreen, picture};
use crate::{Contrast, SignInKey, SignInLook, SignInScreen, Signing, WindowControlLabels};

/// A door nobody reaches: nothing here presses Enter.
#[derive(Debug)]
struct NoDoor;

impl Knocking for NoDoor {
    fn knock(&self, _: Knock) -> Result<Answered, NotAnswered> {
        Ok(Answered::Opened)
    }
}

/// A machine with one account greeting somebody else, so it asks for a name
/// and draws both fields, with *ada* typed in the first.
fn asking_with_a_name_typed() -> SignInScreen<NoDoor> {
    let mut store = Accounts::none().unwrap();
    store.created("ada", 1000, "irrelevant").unwrap();
    let words = crate::what_this_machine_can_say().unwrap();
    let mut screen = SignInScreen::of(Ok(Greeting::of(store, 1001, NoDoor)), words);
    for letter in "ada".chars() {
        screen = match screen.pressed(SignInKey::Letter(letter)) {
            Signing::Still(screen) => Some(*screen),
            Signing::HandedOver(_) => None,
        }
        .unwrap();
    }
    screen
}

/// **The sign-in screen draws the pixels it drew before**, at every scale and
/// text size.
#[test]
fn the_sign_in_screen_draws_the_pixels_it_drew_before() {
    let screen = asking_with_a_name_typed();
    let now = every_way(|display, text| {
        let mut labels = WindowControlLabels::new().unwrap();
        let drawn = picture(
            screen.shows(),
            &mut labels,
            screen.words(),
            in_pixels(LOGICAL, display),
            SignInLook {
                contrast: Contrast::AsDesigned,
                scheme: Scheme::Light,
                scale: text,
            },
            WhoseScreen::ItsOwn,
            display,
        )
        .unwrap();
        let mut hash = Hashed::begun();
        if let Some(surface) = drawn.surface.as_ref() {
            hash.washed(surface);
        }
        hash.solids(&drawn.solids);
        hash.inked(&drawn.inked);
        hash.done()
    });
    held("The sign-in screen", &now, &BEFORE);
}

/// What the sign-in screen drew before. See
/// `crate::the_desktops_pictures_pinned`.
const BEFORE: [Drawn; 6] = [
    Drawn {
        scale: 100,
        text: 100,
        hash: 0xdaaf_c237_3d90_0b96,
    },
    Drawn {
        scale: 100,
        text: 125,
        hash: 0xf63f_74ea_fd68_9c0a,
    },
    Drawn {
        scale: 150,
        text: 100,
        hash: 0x10f2_1a41_f9f7_4bdd,
    },
    Drawn {
        scale: 150,
        text: 125,
        hash: 0xf7e0_1ff3_0dcd_da01,
    },
    Drawn {
        scale: 200,
        text: 100,
        hash: 0x2b3d_ac1b_d81b_4783,
    },
    Drawn {
        scale: 200,
        text: 125,
        hash: 0x0464_4973_3157_9b65,
    },
];
