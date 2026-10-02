//! One screen's surface: the colour `alo-appearance` chose for that screen,
//! across the whole of its own room and warmed by its own night light.
//!
//! It used to be *the colour or the picture*, and three of these tests were
//! about the picture. ADR 0075 removed it, and what is left is what was always
//! about the surface.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_appearance::Token;
use alo_displays::Warmth;

use super::*;
use crate::screens_testing::{a_cold_evening, a_warm_evening};

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
/// The size these tests ask about, as the room it stands for.
///
/// They were written when `prepare` took a pair, and each of them means *a
/// screen with this much room in it* — so the helper is a rename rather than a
/// conversion, and it is here rather than inline so the meaning is stated once.
fn a_room((across, along): (i32, i32)) -> crate::TheRoom {
    crate::TheRoom::of_laid_out_units(across, along)
}

#[test]
fn a_colour_background_is_the_whole_screen_in_the_colour_that_was_chosen() {
    let chosen = Background::from(Token::Navy.colour());
    let size = (1920, 1080);

    let by_day = ScreenBackground::prepare(&chosen, cold(), a_room(size)).unwrap();
    assert_eq!(by_day.solids.len(), 1);
    let solid = by_day.solids.first().unwrap();
    assert_eq!((solid.area.size.w, solid.area.size.h), size);
    assert_eq!((solid.area.loc.x, solid.area.loc.y), (0, 0));
    assert_eq!(solid.colour, as_painted(Token::Navy.colour()));

    let tonight = ScreenBackground::prepare(&chosen, warm(), a_room(size)).unwrap();
    let warmed = tonight.solids.first().unwrap();
    assert_eq!(
        warmed.colour,
        as_painted(warm().applied_to(Token::Navy.colour()))
    );
    assert!(warmed.colour.get(2) < solid.colour.get(2));
}

/// **A screen a background cannot be fitted to is refused, in the desktop's own
/// refusal.** A room with no pixels and a room larger than any screen are both
/// `RenderError::DesktopScene` — never the lock screen's refusal, because the
/// frame being refused is a desktop frame.
#[test]
fn a_screen_a_background_cannot_be_fitted_to_is_refused_as_a_desktop_frame() {
    let chosen = Background::from(Token::Navy.colour());
    for size in [
        (0, 1080),
        (1920, 0),
        (-1, 1080),
        (crate::lock_background::LARGEST_SIDE + 1, 1080),
    ] {
        let refused = ScreenBackground::prepare(&chosen, cold(), a_room(size));
        assert!(
            matches!(refused, Err(RenderError::DesktopScene)),
            "{size:?} was not refused as a desktop frame"
        );
    }
}

/// **The neutral warming changes nothing**, so a machine with night light off
/// draws the colour that was chosen and not one worked out from it.
#[test]
fn the_neutral_warming_changes_nothing() {
    assert!(Warmth::neutral().changes_nothing());
    for token in [Token::Navy, Token::Cream, Token::DeepTeal] {
        let prepared = ScreenBackground::prepare(
            &Background::from(token.colour()),
            cold(),
            crate::TheRoom::of_laid_out_units(64, 64),
        )
        .unwrap();
        assert_eq!(
            prepared.solids.first().unwrap().colour,
            as_painted(token.colour())
        );
    }
}
