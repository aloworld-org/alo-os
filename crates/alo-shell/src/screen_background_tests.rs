//! One screen's background: the colour or the picture `alo-appearance` chose
//! for that screen, fitted to its own room and warmed by its own night light.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_appearance::{Picture, Token};
use alo_displays::Warmth;
use image::ImageEncoder;

use super::*;
use crate::screens_testing::{a_cold_evening, a_warm_evening};

/// Nowhere a shipped image could be found.
fn no_wallpapers() -> &'static Path {
    Path::new("/nonexistent/alo/wallpapers")
}

/// The session has just begun, which is what a rotating folder is asked with.
fn just_begun() -> Duration {
    Duration::ZERO
}

/// What night light does to a screen's channels with it off.
fn cold() -> Warming {
    Warming::at(a_cold_evening().warmth())
}

/// What it does at 2700 K.
fn warm() -> Warming {
    Warming::at(a_warm_evening().warmth())
}

/// **A colour background is one shape over the whole screen, in the colour the
/// person chose** — and with night light on it is that colour as a warmed
/// screen shows it, which is `alo-displays`' arithmetic and not a second copy
/// of it here.
#[test]
fn a_colour_background_is_the_whole_screen_in_the_colour_that_was_chosen() {
    let chosen = Background::from(Token::Navy.colour());
    let size = (1920, 1080);

    let by_day =
        ScreenBackground::prepare(&chosen, cold(), size, just_begun(), no_wallpapers()).unwrap();
    assert!(by_day.inked.is_empty());
    assert_eq!(by_day.solids.len(), 1);
    let solid = by_day.solids.first().unwrap();
    assert_eq!((solid.area.size.w, solid.area.size.h), size);
    assert_eq!((solid.area.loc.x, solid.area.loc.y), (0, 0));
    assert_eq!(solid.colour, as_painted(Token::Navy.colour()));

    let tonight =
        ScreenBackground::prepare(&chosen, warm(), size, just_begun(), no_wallpapers()).unwrap();
    let warmed = tonight.solids.first().unwrap();
    assert_eq!(
        warmed.colour,
        as_painted(warm().applied_to(Token::Navy.colour()))
    );
    assert!(warmed.colour.get(2) < solid.colour.get(2));
}

/// **A picture background is the picture, fitted to this screen's room, one
/// pixel per pixel** — and every one of those pixels is warmed the same way a
/// flat colour is, so a photograph and a plain ground turn together in the
/// evening rather than one of them staying cold.
#[test]
fn a_picture_background_is_fitted_to_this_screen_and_warmed_pixel_by_pixel() {
    let folder = tempfile::tempdir().unwrap();
    let at = folder.path().join("white.png");
    let mut bytes = Vec::new();
    image::codecs::png::PngEncoder::new(&mut bytes)
        .write_image(&[u8::MAX; 4 * 4 * 3], 4, 4, image::ExtendedColorType::Rgb8)
        .unwrap();
    std::fs::write(&at, bytes).unwrap();
    let chosen = Background::from(Picture::file(at).unwrap());
    let size = (320, 200);

    let by_day =
        ScreenBackground::prepare(&chosen, cold(), size, just_begun(), no_wallpapers()).unwrap();
    assert!(by_day.solids.is_empty());
    assert_eq!(by_day.inked.len(), 1);
    let painted = by_day.inked.first().unwrap();
    assert_eq!((painted.area.size.w, painted.area.size.h), size);
    assert_eq!(painted.pixels.len(), 320 * 200);

    let tonight =
        ScreenBackground::prepare(&chosen, warm(), size, just_begun(), no_wallpapers()).unwrap();
    let after = tonight.inked.first().unwrap();
    assert_eq!(after.pixels.len(), painted.pixels.len());
    let mut any_blue_taken = false;
    for (was, now) in painted.pixels.iter().zip(after.pixels.iter()) {
        for (was, now) in was.iter().zip(now.iter()) {
            assert!(now <= was, "warming a screen never adds light");
        }
        if now.get(2) < was.get(2) {
            any_blue_taken = true;
        }
    }
    assert!(any_blue_taken, "not one pixel was warmed");
}

/// **A screen a background cannot be fitted to is refused, in the desktop's own
/// refusal.** A room with no pixels and a room larger than any screen are both
/// `RenderError::DesktopScene` — never the lock screen's refusal, because the
/// frame being refused is a desktop frame.
#[test]
fn a_screen_a_background_cannot_be_fitted_to_is_refused_as_a_desktop_frame() {
    let chosen = Background::from(Token::Navy.colour());
    for size in [(0, 1080), (1920, 0), (-1, 1080), (LARGEST_SIDE + 1, 1080)] {
        let refused =
            ScreenBackground::prepare(&chosen, cold(), size, just_begun(), no_wallpapers());
        assert!(
            matches!(refused, Err(RenderError::DesktopScene)),
            "{size:?} was not refused as a desktop frame"
        );
    }
}

/// **A chosen picture that is not there refuses the desktop frame**, rather
/// than drawing a screen with nothing behind its windows.
#[test]
fn a_chosen_picture_that_is_not_there_refuses_the_desktop_frame() {
    let chosen = Background::from(Picture::shipped("nothing-was-ever-installed").unwrap());
    let refused =
        ScreenBackground::prepare(&chosen, cold(), (640, 480), just_begun(), no_wallpapers());
    assert!(matches!(refused, Err(RenderError::DesktopScene)));
}

/// **The neutral warming changes nothing**, so a machine with night light off
/// draws the colour that was chosen and not one worked out from it.
#[test]
fn the_neutral_warming_changes_nothing() {
    assert!(Warmth::neutral().changes_nothing());
    for token in [Token::Navy, Token::Cream, Token::Terracotta] {
        let prepared = ScreenBackground::prepare(
            &Background::from(token.colour()),
            cold(),
            (64, 64),
            just_begun(),
            no_wallpapers(),
        )
        .unwrap();
        assert_eq!(
            prepared.solids.first().unwrap().colour,
            as_painted(token.colour())
        );
    }
}
