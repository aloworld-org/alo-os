//! The laid-out screen, read as pixels: what is on it and what never is.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;
use crate::{SignInKey, SignInScreen, Signing};
use alo_accounts::Accounts;
use alo_appearance::Token;
use alo_greeting::{Greeting, Knocking, NotAnswered};
use alo_sessiond::{Answered, Knock};

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
///
/// **The greeting is for 1001 and the account is 1000**, so `whose_name` is
/// `None`: this machine has an account and cannot say whose, which is the
/// screen that **asks for a name** and draws both fields. These tests type a
/// name and cross to the password, which is that screen's interaction.
///
/// It was 1000 until 2026-10-16, which made it the screen a person comes
/// back to -- one field, no name asked for. Every name typed went into the
/// password, so `typed("ada", "")` and `typed("ada", "x")` both read as
/// *something typed*, drew the same picture, and the test that says a typed
/// password is visible compared a thing with itself.
fn typed(name: &str, password: &str) -> SignInScreen<NoDoor> {
    let mut store = Accounts::none().unwrap();
    store.created("ada", 1000, "irrelevant").unwrap();
    let words = crate::what_this_machine_can_say().unwrap();
    let mut screen = SignInScreen::of(Ok(Greeting::of(store, 1001, NoDoor)), words);
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
        screen.words(),
        (800, 600),
        look,
        super::WhoseScreen::ItsOwn,
        crate::display_scale::DisplayScale::ONE,
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
    let words = crate::what_this_machine_can_say().unwrap();
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
                    screen.words(),
                    size,
                    light(),
                    super::WhoseScreen::ItsOwn,
                    crate::display_scale::DisplayScale::ONE,
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
            screen.words(),
            (800, 600),
            light(),
            super::WhoseScreen::InsideAnother,
            crate::display_scale::DisplayScale::ONE,
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
        let lit = marked(ink);
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

/// **Every label and every mark the design draws puts ink on the screen.**
///
/// Not "it was placed": the pixels are read back. A label whose key is not in
/// the vocabulary renders as `«the.key»`, and a glyph no bundled face carries
/// renders as a tofu box or as nothing at all — both of which this screen has
/// shown on a real machine within one day. Neither is visible to a test that
/// counts calls.
///
/// The power control is checked in the surface raster rather than the ink,
/// because `U+23FB` is in neither bundled face and it is drawn by
/// `crate::sign_in_surface` as a ring with a bar standing in it.
#[test]
fn every_label_and_mark_is_drawn_and_not_merely_placed() {
    let drawn = whose_screen("disan");
    let at = crate::sign_in_layout::sign_in_laid_out(smithay::utils::Size::from((800, 600)))
        .expect("this size lays out");

    let dark = marked;
    for (what, box_of) in [
        ("the greeting", at.account),
        ("the submit arrow", at.submit),
        ("back to lock", at.back),
        ("other ways to sign in", at.other_ways),
        ("accessibility", at.accessibility),
        ("switch person", at.switch_person),
    ] {
        let found: usize = drawn
            .inked
            .iter()
            .filter(|ink| {
                let same: smithay::utils::Rectangle<i32, smithay::utils::Physical> =
                    smithay::utils::Rectangle::new(
                        (box_of.loc.x, box_of.loc.y).into(),
                        (box_of.size.w, box_of.size.h).into(),
                    );
                ink.area.intersection(same).is_some()
            })
            .map(dark)
            .sum();
        assert!(
            found > 0,
            "{what} has no ink anywhere in {box_of:?}: it is laid out and not drawn"
        );
    }

    // The power control, in the raster.
    let surface = drawn.surface.as_ref().expect("the design draws a surface");
    let w = surface.area.size.w;
    let mut marked = 0_usize;
    for y in at.power.loc.y..(at.power.loc.y + at.power.size.h) {
        for x in at.power.loc.x..(at.power.loc.x + at.power.size.w) {
            let i = ((y * w + x) * 4) as usize;
            let (Some(r), Some(g), Some(b)) = (
                surface.pixels.get(i),
                surface.pixels.get(i + 1),
                surface.pixels.get(i + 2),
            ) else {
                continue;
            };
            if u32::from(*r) + u32::from(*g) + u32::from(*b) < 400 {
                marked += 1;
            }
        }
    }
    assert!(
        marked > 20,
        "the power control has only {marked} marked pixels: it is laid out, answered by the \
         hit test, and not drawn"
    );
}

/// How many pixels of an inked box are ink rather than the ground it was
/// shaped against.
///
/// **Not "how many are dark".** The placeholder inside the entry is drawn in
/// `text/muted` — `#596B78`, whose channels sum to 316 — so a darkness
/// threshold counted a correctly drawn placeholder as an empty box. A box's
/// own first pixel is its ground, and ink is whatever differs from it.
fn marked(ink: &crate::painted::Inked) -> usize {
    let Some(ground) = ink.pixels.first().copied() else {
        return 0;
    };
    ink.pixels
        .iter()
        .filter(|p| {
            u32::from(p[0]).abs_diff(u32::from(ground[0]))
                + u32::from(p[1]).abs_diff(u32::from(ground[1]))
                + u32::from(p[2]).abs_diff(u32::from(ground[2]))
                > 24
        })
        .count()
}

/// The screen of a machine whose one account is `ada`, on an output of `size`
/// drawn at `display`.
fn adas_screen_at(size: (i32, i32), display: crate::display_scale::DisplayScale) -> SignInPicture {
    let mut store = Accounts::none().unwrap();
    store.created("ada", 1000, "irrelevant").unwrap();
    let words = crate::what_this_machine_can_say().unwrap();
    let screen = SignInScreen::of(Ok(Greeting::of(store, 1000, NoDoor)), words);
    let mut labels = WindowControlLabels::new().unwrap();
    picture(
        screen.shows(),
        &mut labels,
        screen.words(),
        size,
        light(),
        super::WhoseScreen::ItsOwn,
        display,
    )
    .unwrap()
}

/// **At 200 per cent everything is drawn twice the size, in twice the
/// place.** A 3840 × 2160 screen at 200 against a 1920 × 1080 one at 100: the
/// same pieces of text, each about twice as large and twice as far from the
/// corner, the dots of a typed PIN twice as large, and the rounded shapes
/// covering the whole output.
///
/// *About*, because a line of type drawn twice the size is not exactly twice
/// the pixels of its ink: a glyph's edge rounds to a pixel either way. A
/// tenth and four pixels is far smaller than the halving this replaces.
#[test]
fn at_two_hundred_everything_is_twice_the_size() {
    let one = crate::display_scale::DisplayScale::ONE;
    let two = crate::display_scale::DisplayScale::of_per_cent(200);
    let sparse = adas_screen_at((1920, 1080), one);
    let dense = adas_screen_at((3840, 2160), two);

    let surface = dense
        .surface
        .as_ref()
        .expect("the dense screen has a surface");
    assert_eq!(
        (surface.area.size.w, surface.area.size.h),
        (3840, 2160),
        "the rounded shapes do not cover the whole dense output"
    );
    assert_eq!(
        sparse.inked.len(),
        dense.inked.len(),
        "a different number of lines was drawn at 200"
    );
    let about_twice = |at_one: i32, at_two: i32| (at_two - 2 * at_one).abs() <= 4 + at_one / 5;
    for (small, large) in sparse.inked.iter().zip(&dense.inked) {
        let (a, b) = (small.area, large.area);
        assert!(
            about_twice(a.loc.x, b.loc.x)
                && about_twice(a.loc.y, b.loc.y)
                && about_twice(a.size.w, b.size.w)
                && about_twice(a.size.h, b.size.h),
            "a line at {a:?} at 100 is at {b:?} at 200, not twice"
        );
        assert!(
            marked(large) > 2 * marked(small),
            "a line at {b:?} has no more ink at 200 than at 100"
        );
    }
}

/// **A screen too small for the scale is refused, not drawn over itself**:
/// 640 × 480 at 200 is 320 × 240 logical, under the smallest the screen lays
/// out for. `crate::the_sign_in_screens_scale` holds the scale down so this
/// is never asked of a real screen; the raster refuses it anyway.
#[test]
fn a_screen_too_small_for_its_scale_is_refused() {
    let words = crate::what_this_machine_can_say().unwrap();
    let mut store = Accounts::none().unwrap();
    store.created("ada", 1000, "irrelevant").unwrap();
    let screen = SignInScreen::of(Ok(Greeting::of(store, 1000, NoDoor)), words);
    let mut labels = WindowControlLabels::new().unwrap();
    assert!(matches!(
        picture(
            screen.shows(),
            &mut labels,
            screen.words(),
            (640, 480),
            light(),
            super::WhoseScreen::ItsOwn,
            crate::display_scale::DisplayScale::of_per_cent(200),
        ),
        Err(RenderError::SignInScene)
    ));
}

/// FNV-1a over a value's `Debug`, which is every pixel, rectangle and colour a
/// picture holds. Written here rather than taken from `std`, whose hasher is
/// not promised to stay the same between Rust releases.
fn fnv_of(value: &impl std::fmt::Debug) -> u64 {
    struct Fnv(u64);
    impl std::fmt::Write for Fnv {
        fn write_str(&mut self, written: &str) -> std::fmt::Result {
            for byte in written.bytes() {
                self.0 ^= u64::from(byte);
                self.0 = self.0.wrapping_mul(0x0100_0000_01b3);
            }
            Ok(())
        }
    }
    let mut hashed = Fnv(0xcbf2_9ce4_8422_2325);
    std::fmt::Write::write_fmt(&mut hashed, format_args!("{value:?}")).unwrap();
    hashed.0
}

/// **At 100 per cent the screen is the pixels it was before it followed the
/// screen's scale**, held to hashes taken from `main` at 99e9007d — the tree
/// before #1017 — by the same code on 2026-10-29, twice, with the same
/// answers both times. Drawn with the shell's bundled faces only
/// (`WindowControlLabels`), so the machine's own fonts cannot move them.
///
/// Three screens at two sizes: a returning person with nothing typed and with
/// a PIN typed, and a machine that asks for a name with one letter of it typed.
#[test]
fn at_one_hundred_the_pixels_are_mains() {
    let mut labels = WindowControlLabels::new().unwrap();
    let mut hashes = Vec::new();
    for (uid, keys) in [
        (1000, vec![]),
        (1000, vec![SignInKey::Letter('x')]),
        (1001, vec![SignInKey::Letter('a')]),
    ] {
        let mut store = Accounts::none().unwrap();
        store.created("ada", 1000, "irrelevant").unwrap();
        let words = crate::what_this_machine_can_say().unwrap();
        let mut screen = SignInScreen::of(Ok(Greeting::of(store, uid, NoDoor)), words);
        for key in keys {
            screen = match screen.pressed(key) {
                Signing::Still(screen) => *screen,
                Signing::HandedOver(_) => panic!("a key signed somebody in"),
            };
        }
        for size in [(800, 600), (1920, 1080)] {
            let drawn = picture(
                screen.shows(),
                &mut labels,
                screen.words(),
                size,
                light(),
                super::WhoseScreen::ItsOwn,
                crate::display_scale::DisplayScale::ONE,
            )
            .unwrap();
            hashes.push(fnv_of(&drawn));
        }
    }
    assert_eq!(
        hashes,
        [
            0xe3c4_51de_9d6f_927b,
            0xdd31_62a6_bc81_62b9,
            0xb5f9_6082_9d80_1781,
            0x8c0b_fe33_57b1_b364,
            0x0623_194d_c787_8f03,
            0x33d2_c3f3_7c7c_319f,
        ],
        "a screen at 100 per cent is not the pixels main drew"
    );
}
