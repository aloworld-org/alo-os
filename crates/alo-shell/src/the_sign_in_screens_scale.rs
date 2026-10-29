//! How large the sign-in screen is drawn, worked out from the screen itself.
//!
//! # Why the screen and not the person
//!
//! Everywhere else a display's scale is the person's own setting, read from
//! their arrangement (`Server::the_scale_of_display`). The sign-in screen is
//! drawn **before anybody has signed in**, so there is nobody's setting to read
//! — and drawing it at one-to-one handed a person with a dense laptop panel a
//! screen at half the size of everything they meet after it.
//!
//! The owner decided on 2026-10-29: *work it out from the screen*. So this asks
//! `alo_displays::Scale::worked_out_for` — the same arithmetic a screen nobody
//! has arranged gets once somebody is signed in — with the pixels and the glass
//! the connector reports. This file decides nothing about sizes itself; it only
//! carries the two facts there and the answer back.
//!
//! Once somebody signs in, this has no say: their own setting applies.
//!
//! # A screen that reports nothing, and a screen too small to hold it
//!
//! A screen that gives no physical size cannot be worked out, and is drawn at
//! one-to-one, which is what `worked_out_for` itself answers.
//!
//! A scale is also **held to what still fits**: the screen is laid out in
//! logical pixels (`crate::sign_in_layout`), and it refuses a screen smaller
//! than `THE_SMALLEST_SCREEN_IS`. A small, very dense panel worked out at 300
//! would leave less room than that and draw nothing at all, so the scale is
//! lowered until the screen fits — never below one-to-one, where the layout's
//! own refusal stands as it always did.

use crate::display_scale::DisplayScale;
use crate::sign_in_layout::THE_SMALLEST_SCREEN_IS;

/// The scale the sign-in screen is drawn at on a screen of `pixels`, named
/// `connector`, whose glass is `millimetres` across and down — `(0, 0)` for a
/// screen that did not say.
#[must_use]
pub(crate) fn the_sign_in_screens_scale(
    connector: &str,
    pixels: (i32, i32),
    millimetres: (i32, i32),
) -> DisplayScale {
    let (Ok(across), Ok(down)) = (u32::try_from(pixels.0), u32::try_from(pixels.1)) else {
        return DisplayScale::ONE;
    };
    let glass = match millimetres {
        (across, down) if across > 0 && down > 0 => {
            u32::try_from(across).ok().zip(u32::try_from(down).ok())
        }
        _ => None,
    };
    let Ok(socket) = alo_displays::Socket::named(connector) else {
        return DisplayScale::ONE;
    };
    let Ok(reported) = alo_displays::Reported::of(socket, None, (across, down), glass) else {
        return DisplayScale::ONE;
    };
    let worked_out = alo_displays::Scale::worked_out_for(reported.pixels(), reported.millimetres())
        .as_per_cent();
    DisplayScale::of_per_cent(held_to_what_fits(worked_out, (across, down)))
}

/// `per_cent`, lowered until a screen of `pixels` still holds the smallest
/// sign-in screen at it, and never below one hundred.
fn held_to_what_fits(per_cent: u16, pixels: (u32, u32)) -> u16 {
    let room = |pixels: u32, least: i32| {
        let least = u64::from(u32::try_from(least).unwrap_or(1).max(1));
        u64::from(pixels) * 100 / least
    };
    let most =
        room(pixels.0, THE_SMALLEST_SCREEN_IS.0).min(room(pixels.1, THE_SMALLEST_SCREEN_IS.1));
    let most = u16::try_from(most).unwrap_or(u16::MAX);
    per_cent.min(most).max(100)
}

#[cfg(test)]
#[path = "the_sign_in_screens_scale_tests.rs"]
mod tests;
