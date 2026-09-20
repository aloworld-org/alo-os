//! The laid-out screen, read as pixels: what is on it and what never is.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;
use crate::{SignInKey, SignInScreen, Signing};
use alo_accounts::Accounts;
use alo_appearance::Token;
use alo_greeting::{Greeting, Knocking, NotAnswered};
use alo_sessiond::{Answered, Knock};
use alo_strings::Strings;

/// A door that is never reached in these tests: nobody presses Enter.
#[derive(Debug)]
struct NoDoor;

impl Knocking for NoDoor {
    fn knock(&self, _: Knock) -> Result<Answered, NotAnswered> {
        Ok(Answered::Opened)
    }
}

/// The ordinary light look.
fn light() -> SignInLook {
    SignInLook {
        contrast: Contrast::AsDesigned,
        scheme: Scheme::Light,
        scale: TextScale::ordinary(),
    }
}

/// A screen with `name` typed, then `password` typed in the password field.
fn typed(name: &str, password: &str) -> SignInScreen<NoDoor> {
    let mut store = Accounts::none().unwrap();
    store.created("ada", 1000, "irrelevant").unwrap();
    let words = Strings::of(alo_saying::everything_this_machine_can_say().unwrap());
    let mut screen = SignInScreen::of(Ok(Greeting::of(store, 1000, NoDoor)), words);
    let mut keys: Vec<SignInKey> = name.chars().map(SignInKey::Letter).collect();
    keys.push(SignInKey::OtherField);
    keys.extend(password.chars().map(SignInKey::Letter));
    for key in keys {
        screen = match screen.pressed(key) {
            Signing::Still(screen) => Some(*screen),
            Signing::HandedOver(_) => None,
        }
        .unwrap();
    }
    screen
}

/// The picture of a screen at an ordinary output size.
fn drawn(screen: &SignInScreen<NoDoor>, look: SignInLook) -> SignInPicture {
    let mut labels = WindowControlLabels::new().unwrap();
    picture(screen.shows(), &mut labels, (800, 600), look).unwrap()
}

/// **The password is never drawn**: a screen with one password typed and a
/// screen with a different, much longer one are the same picture, pixel for
/// pixel — not even the length reaches the screen.
#[test]
fn the_password_is_never_drawn_not_even_its_length() {
    for look in [
        light(),
        SignInLook {
            contrast: Contrast::AsDesigned,
            scheme: Scheme::Dark,
            scale: TextScale::ordinary(),
        },
    ] {
        let short = drawn(&typed("ada", "x"), look);
        let long = drawn(
            &typed("ada", "correct horse battery staple, and then some"),
            look,
        );
        assert_eq!(
            short, long,
            "the picture depends on what password was typed"
        );
    }
}

/// **Something typed is still visible as something**, so a person can see
/// their keystrokes arrived — the one fact about the password that is drawn.
#[test]
fn that_a_password_is_typed_is_visible() {
    assert_ne!(
        drawn(&typed("ada", ""), light()),
        drawn(&typed("ada", "x"), light())
    );
}

/// **The name is drawn as typed**, so two names are two pictures.
#[test]
fn the_name_is_drawn() {
    assert_ne!(
        drawn(&typed("ada", ""), light()),
        drawn(&typed("grace", ""), light())
    );
}

/// **Terracotta is not on this screen**: it means the agent, and nobody has
/// signed in to have one.
#[test]
fn terracotta_is_not_on_the_sign_in_screen() {
    let terracotta = {
        let colour = Token::Terracotta.colour();
        [colour.red(), colour.green(), colour.blue()]
    };
    for scheme in [Scheme::Light, Scheme::Dark] {
        let look = SignInLook {
            contrast: Contrast::AsDesigned,
            scheme,
            scale: TextScale::ordinary(),
        };
        let picture = drawn(&typed("ada", "pw"), look);
        assert!(
            picture
                .solids
                .iter()
                .all(|solid| solid.colour != terracotta)
        );
        assert!(
            picture
                .inked
                .iter()
                .all(|inked| inked.pixels.iter().all(|pixel| *pixel != terracotta))
        );
    }
}

/// **Everything drawn is inside the output and whole**, at the ordinary
/// scale and at a large one.
#[test]
fn everything_drawn_is_inside_the_output() {
    for percent in [100, 200] {
        let look = SignInLook {
            contrast: Contrast::AsDesigned,
            scheme: Scheme::Light,
            scale: TextScale::percent(percent).unwrap(),
        };
        let picture = drawn(&typed(&"n".repeat(200), "pw"), look);
        let output = Rectangle::<i32, Physical>::from_size((800, 600).into());
        for area in picture
            .solids
            .iter()
            .map(|solid| solid.area)
            .chain(picture.inked.iter().map(|inked| inked.area))
        {
            assert_eq!(
                area.intersection(output),
                Some(area),
                "{area:?} leaves the output"
            );
        }
        for inked in &picture.inked {
            assert_eq!(
                inked.pixels.len(),
                (inked.area.size.w * inked.area.size.h) as usize
            );
        }
    }
}

/// **An output too small to hold two fields, or larger than any output, is
/// refused** rather than drawn wrong.
#[test]
fn an_output_that_cannot_hold_the_screen_is_refused() {
    let screen = typed("ada", "");
    let mut labels = WindowControlLabels::new().unwrap();
    for size in [(100, 600), (800, 90), (0, 0), (-5, 600), (20_000, 600)] {
        assert!(
            matches!(
                picture(screen.shows(), &mut labels, size, light()),
                Err(RenderError::SignInScene)
            ),
            "{size:?}"
        );
    }
}

/// **The waiting field is told apart by its shape, not by colour alone**:
/// moving between fields changes the picture while every colour on it stays
/// one of the same few.
#[test]
fn the_waiting_field_is_told_apart_by_shape() {
    let at_password = typed("ada", "");
    let at_name = match at_password.pressed(SignInKey::OtherField) {
        Signing::Still(screen) => Some(*screen),
        Signing::HandedOver(_) => None,
    }
    .unwrap();
    let one = drawn(&at_name, light());
    let other = drawn(&typed("ada", ""), light());
    assert_ne!(one, other);
    let colours = |picture: &SignInPicture| {
        let mut colours: Vec<[u8; 3]> = picture.solids.iter().map(|solid| solid.colour).collect();
        colours.sort_unstable();
        colours.dedup();
        colours
    };
    assert_eq!(colours(&one), colours(&other));
}

/// **High contrast is the same screen in the other palette.** Every box is
/// where it was — turning it on changes no layout, because there is no second
/// screen for a person who needs it — and every flat colour drawn is one
/// `alo_access::HighContrast` decided, so a token written into this file by
/// hand would fail here rather than survive as a patch of the design's cream
/// on a screen meant to be readable.
#[test]
fn high_contrast_draws_the_same_screen_in_the_palette_that_crate_decided() {
    let screen = typed("ada", "secret");
    for scheme in [Scheme::Light, Scheme::Dark] {
        let scale = TextScale::ordinary();
        let designed = drawn(
            &screen,
            SignInLook {
                scheme,
                scale,
                contrast: Contrast::AsDesigned,
            },
        );
        let high = drawn(
            &screen,
            SignInLook {
                scheme,
                scale,
                contrast: Contrast::High,
            },
        );
        let boxes = |picture: &SignInPicture| {
            picture
                .solids
                .iter()
                .map(|solid| solid.area)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            boxes(&designed),
            boxes(&high),
            "{scheme:?}: high contrast moved something"
        );
        let palette = crate::access_contrast::every_colour_of(
            Contrast::High,
            scheme,
            Token::Terracotta.colour(),
        );
        for solid in &high.solids {
            assert!(
                palette.contains(&solid.colour),
                "{scheme:?}: {:?} is drawn in {:?}, which is in no palette this crate may use",
                solid.area,
                solid.colour
            );
        }
        assert_ne!(
            designed.solids.first().map(|solid| solid.colour),
            high.solids.first().map(|solid| solid.colour),
            "{scheme:?}: the ground did not change at all"
        );
    }
}
