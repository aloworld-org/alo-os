//! The sign-in screen's layout, converted to the pixels it is drawn in.
//!
//! `crate::sign_in_layout` places everything in logical pixels, because the
//! design's numbers are logical. A screen drawn at 200 per cent is laid out at
//! half its pixels, and every rectangle the picture draws text into is then
//! converted here, once, through the same smithay rounding
//! `crate::sign_in_surface` uses for the rounded shapes — so a line of type and
//! the shape it sits in cannot land a pixel apart. At a scale of one every
//! conversion is the identity.
//!
//! A press is answered the other way round, against the logical layout
//! (`crate::sign_in_pointer`), so nothing here is hit-tested.

use smithay::utils::{Logical, Physical, Rectangle, Scale};

use crate::sign_in_layout::SignInLayout;

/// The rectangles the picture draws into, in its own pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SignInInPixels {
    /// The monogram's square.
    pub(crate) monogram: Rectangle<i32, Physical>,
    /// The line that greets the person by name.
    pub(crate) account: Rectangle<i32, Physical>,
    /// The entry field.
    pub(crate) entry: Rectangle<i32, Physical>,
    /// The submit target, inside the entry.
    pub(crate) submit: Rectangle<i32, Physical>,
    /// The arrow drawn inside the submit target.
    pub(crate) arrow: Rectangle<i32, Physical>,
    /// Where a refusal is said.
    pub(crate) feedback: Rectangle<i32, Physical>,
    /// *Other ways to sign in*.
    pub(crate) other_ways: Rectangle<i32, Physical>,
    /// *Accessibility*.
    pub(crate) accessibility: Rectangle<i32, Physical>,
    /// *Switch person*.
    pub(crate) switch_person: Rectangle<i32, Physical>,
    /// *Back to lock*.
    pub(crate) back: Rectangle<i32, Physical>,
}

impl SignInInPixels {
    /// `at`, drawn at `scale`.
    pub(crate) fn of(at: &SignInLayout, scale: Scale<f64>) -> Self {
        let px = |logical: Rectangle<i32, Logical>| -> Rectangle<i32, Physical> {
            logical.to_physical_precise_round(scale)
        };
        Self {
            monogram: px(at.monogram),
            account: px(at.account),
            entry: px(at.entry),
            submit: px(at.submit),
            arrow: px(at.arrow),
            feedback: px(at.feedback),
            other_ways: px(at.other_ways),
            accessibility: px(at.accessibility),
            switch_person: px(at.switch_person),
            back: px(at.back),
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::expect_used,
        reason = "in a test, a panic on an unexpected None is the failure being reported"
    )]

    use super::*;
    use smithay::utils::Size;

    /// **At one, every rectangle is the layout's own**, which is what keeps the
    /// screen most people meet the picture it was.
    #[test]
    fn at_one_nothing_moves() {
        let at = crate::sign_in_layout::sign_in_laid_out(Size::from((1440, 960)))
            .expect("the reference size lays out");
        let drawn = SignInInPixels::of(&at, Scale::from(1.0));
        for (logical, pixels) in [
            (at.entry, drawn.entry),
            (at.submit, drawn.submit),
            (at.account, drawn.account),
            (at.back, drawn.back),
            (at.switch_person, drawn.switch_person),
        ] {
            assert_eq!(
                (logical.loc.x, logical.loc.y, logical.size.w, logical.size.h),
                (pixels.loc.x, pixels.loc.y, pixels.size.w, pixels.size.h)
            );
        }
    }

    /// **At two, every rectangle is exactly twice itself**, place and size.
    #[test]
    fn at_two_everything_doubles() {
        let at = crate::sign_in_layout::sign_in_laid_out(Size::from((960, 540)))
            .expect("half of 1920 × 1080 lays out");
        let drawn = SignInInPixels::of(&at, Scale::from(2.0));
        for (logical, pixels) in [
            (at.entry, drawn.entry),
            (at.submit, drawn.submit),
            (at.monogram, drawn.monogram),
            (at.other_ways, drawn.other_ways),
            (at.accessibility, drawn.accessibility),
        ] {
            assert_eq!(
                (
                    2 * logical.loc.x,
                    2 * logical.loc.y,
                    2 * logical.size.w,
                    2 * logical.size.h
                ),
                (pixels.loc.x, pixels.loc.y, pixels.size.w, pixels.size.h)
            );
        }
    }
}
