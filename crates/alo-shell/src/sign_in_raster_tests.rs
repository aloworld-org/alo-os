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
    picture(
        screen.shows(),
        &mut labels,
        (800, 600),
        look,
        super::WhoseScreen::ItsOwn,
    )
    .unwrap()
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

/// **The person this machine signs in is named on the screen**, so two
/// machines with two people are two pictures.
///
/// This asked whether the **typed** name was drawn until 2026-10-12. The
/// full-screen design has one entry — the PIN — and greets the person it is
/// authenticating above it, so there is no name field to type into and
/// nothing about typing changes this. Drawing the typed name is what left a
/// booted machine greeting nobody, because nothing is typed when the screen
/// appears.
#[test]
fn the_persons_own_name_is_drawn() {
    assert_ne!(whose_screen("ada"), whose_screen("grace"));
}

/// The screen of a machine whose one account has this name.
fn whose_screen(name: &str) -> crate::sign_in_raster::SignInPicture {
    let mut store = Accounts::none().unwrap();
    store.created(name, 1000, "irrelevant").unwrap();
    let words = Strings::of(alo_saying::everything_this_machine_can_say().unwrap());
    drawn(
        &SignInScreen::of(Ok(Greeting::of(store, 1000, NoDoor)), words),
        light(),
    )
}

/// **Deep teal is not on this screen**: it means the agent, and nobody has
/// signed in to have one.
#[test]
fn deep_teal_is_not_on_the_sign_in_screen() {
    let deep_teal = {
        let colour = Token::DeepTeal.colour();
        [colour.red(), colour.green(), colour.blue()]
    };
    for scheme in [Scheme::Light, Scheme::Dark] {
        let look = SignInLook {
            contrast: Contrast::AsDesigned,
            scheme,
            scale: TextScale::ordinary(),
        };
        let picture = drawn(&typed("ada", "pw"), look);
        assert!(picture.solids.iter().all(|solid| solid.colour != deep_teal));
        assert!(
            picture
                .inked
                .iter()
                .all(|inked| inked.pixels.iter().all(|pixel| *pixel != deep_teal))
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
                picture(
                    screen.shows(),
                    &mut labels,
                    size,
                    light(),
                    super::WhoseScreen::ItsOwn,
                ),
                Err(RenderError::SignInScene)
            ),
            "{size:?}"
        );
    }
}

/// **The waiting field is told apart by its shape, not by colour alone**:
/// moving between fields changes the picture while every colour on it stays
/// one of the same few.
///
/// **Asked of the embedded form, which is where two fields still are.** The
/// sign-in screen was redrawn to `176-10686.xml` on 2026-10-12 and has one
/// entry, so there is no second field to move to and this property is not
/// about it. The lock screen still embeds the two, and the guarantee is kept
/// where it still means something rather than deleted with the design that
/// used to carry it.
#[test]
fn the_waiting_field_is_told_apart_by_shape() {
    let at_password = typed("ada", "");
    let at_name = match at_password.pressed(SignInKey::OtherField) {
        Signing::Still(screen) => Some(*screen),
        Signing::HandedOver(_) => None,
    }
    .unwrap();
    let embedded = |screen: &SignInScreen<_>| {
        let mut labels = WindowControlLabels::new().unwrap();
        picture(
            screen.shows(),
            &mut labels,
            (800, 600),
            light(),
            super::WhoseScreen::InsideAnother,
        )
        .unwrap()
    };
    let one = embedded(&at_name);
    let other = embedded(&typed("ada", ""));
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
            Token::DeepTeal.colour(),
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

/// **No inked box is wider than the words in it.**
///
/// A shaped box is as wide as the width it was given to wrap inside, and
/// carries its ground colour everywhere the ink is not. On a flat screen that
/// is invisible. On this one the raster draws the design's folded light, and
/// an uncropped box is a **bar** laid across it — which is what a booted
/// machine showed on 2026-10-12: the person's name, and then a white rectangle
/// running from it to halfway across the display.
///
/// It also broke the centring. A caller centring by `(box - ink) / 2` places
/// the ink correctly and then draws the rest of the box off to one side.
#[test]
fn no_inked_box_is_wider_than_its_own_words() {
    let drawn = whose_screen("disan");
    for ink in &drawn.inked {
        let lit = ink
            .pixels
            .iter()
            .filter(|p| u32::from(p[0]) + u32::from(p[1]) + u32::from(p[2]) < 300)
            .count();
        assert!(
            lit > 0,
            "an inked box at {:?} carries no ink at all: it is a rectangle of ground \
             colour on the design's own artwork",
            ink.area
        );
        // Generous: a box may be a little wider than the ink it holds, because
        // a glyph's own side bearings belong to it. A box four times the ink
        // is the fault this catches.
        let area = (ink.area.size.w * ink.area.size.h) as usize;
        assert!(
            area < lit * 40,
            "the box at {:?} is {area} pixels around {lit} of ink: it is mostly ground, \
             and that ground is drawn over the wallpaper",
            ink.area
        );
    }
}
