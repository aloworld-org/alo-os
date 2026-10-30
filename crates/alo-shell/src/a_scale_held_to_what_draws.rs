//! A picture drawn at its display's scale, or at the largest smaller one at
//! which it still draws.
//!
//! The first-start sheets and the lock screen's PIN form are laid out in
//! logical pixels and **refuse** a room too small to hold them rather than
//! draw over themselves. A dense screen worked out at 175 per cent can leave
//! less logical room than a sheet needs — 1920 × 1080 at 175 is 1097 × 617,
//! and a sheet needs 700 down — and a refusal there would hand a person a
//! blank screen on the one machine this scale exists for.
//!
//! So a refusal at a scale above one is answered by **trying a step smaller**,
//! in quarters, down to one-to-one, where the drawing's own refusal stands as
//! it always did. How large a sheet's words are is then whatever fits, which
//! is the rule `crate::the_sign_in_screens_scale` already holds for the
//! sign-in screen by arithmetic. The sheets cannot be held by arithmetic: how
//! much room one needs depends on how its words wrap, in a language this code
//! has never seen, so the drawing itself is the only thing that knows.
//!
//! **Nothing is retried at one.** A picture that fails at one-to-one fails as
//! it did before this file, and an error at a larger scale that is not about
//! room recurs at one and is returned from there.

use crate::display_scale::DisplayScale;

/// How far one step down goes, in per cent.
const ONE_STEP_IS: u16 = 25;

/// `draw` at `display`, or at the largest smaller scale it accepts, and never
/// below one-to-one.
///
/// # Errors
/// Whatever `draw` answers at one-to-one, when every larger scale was refused.
pub(crate) fn at_the_largest_that_draws<T, E>(
    display: DisplayScale,
    mut draw: impl FnMut(DisplayScale) -> Result<T, E>,
) -> Result<T, E> {
    let mut per_cent = display.per_cent();
    loop {
        match draw(DisplayScale::of_per_cent(per_cent)) {
            Err(_) if per_cent > 100 => {
                per_cent = per_cent.saturating_sub(ONE_STEP_IS).max(100);
            }
            answered => return answered,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **What draws at its own scale is drawn there**, and nothing smaller is
    /// tried.
    #[test]
    fn a_scale_that_draws_is_kept() {
        let mut tried = Vec::new();
        let drawn: Result<u16, ()> =
            at_the_largest_that_draws(DisplayScale::of_per_cent(200), |at| {
                tried.push(at.per_cent());
                Ok(at.per_cent())
            });
        assert_eq!(drawn, Ok(200));
        assert_eq!(tried, [200]);
    }

    /// **A refusal steps down a quarter at a time**, to the largest that draws.
    #[test]
    fn a_refusal_steps_down_to_the_largest_that_draws() {
        let mut tried = Vec::new();
        let drawn: Result<u16, ()> =
            at_the_largest_that_draws(DisplayScale::of_per_cent(175), |at| {
                tried.push(at.per_cent());
                if at.per_cent() > 150 {
                    Err(())
                } else {
                    Ok(at.per_cent())
                }
            });
        assert_eq!(drawn, Ok(150));
        assert_eq!(tried, [175, 150]);
    }

    /// **At one-to-one the drawing's own refusal stands**, and an odd scale
    /// still ends at exactly one hundred.
    #[test]
    fn a_refusal_at_one_is_returned() {
        let mut tried = Vec::new();
        let drawn: Result<(), &str> =
            at_the_largest_that_draws(DisplayScale::of_per_cent(130), |at| {
                tried.push(at.per_cent());
                Err("too small")
            });
        assert_eq!(drawn, Err("too small"));
        assert_eq!(tried, [130, 105, 100]);
        let mut once = 0;
        let _: Result<(), ()> = at_the_largest_that_draws(DisplayScale::ONE, |_| {
            once += 1;
            Err(())
        });
        assert_eq!(once, 1, "nothing is retried at one");
    }
}
