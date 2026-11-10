//! **One window alone on one side of an area**, with nothing else moved.
//!
//! `docs/design/the-window-controls.md`, frame `11:84`: the window panel's
//! *Left* and *Right* put **this** window on a half of its display's work area,
//! and *other windows are not touched*. That is not a division — the
//! `Super+Left` and `Super+Right` chords divide a share **with the next
//! window** ([`crate::Division::divide_with_next`]), which moves that window
//! too — but it is the same half, so it is decided here, by the same cut.
//!
//! **Where the area is cut is the division's rule**, from the one function
//! that also decides where halving a share cuts it: the first piece rounded
//! down, the odd unit on the right or the bottom. A window put on the left by
//! the panel and one put there by the chord get the same half.
//!
//! **Refused rather than squeezed**, as halving is: a window whose minimum does
//! not fit its half is told [`Refused::TooNarrow`] or [`Refused::TooShort`],
//! and the half is still answered, so whoever asked can show where it would
//! have gone.

use crate::area::Area;
use crate::refusing::Refused;
use crate::share::Share;
use crate::side::Side;
use crate::window::Window;

/// The half of an area on one side, and whether a window fits in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OnASide {
    /// The half.
    half: Area,
    /// The window's share of it, or why it does not fit.
    share: Result<Share, Refused>,
}

impl OnASide {
    /// The half of the area on this side, whether or not the window fits it.
    #[must_use]
    pub const fn half(self) -> Area {
        self.half
    }

    /// The window's share — the whole half — or why it cannot have one.
    ///
    /// # Errors
    /// [`Refused::TooNarrow`] or [`Refused::TooShort`] where the window says it
    /// cannot be made as small as the half.
    pub const fn share(self) -> Result<Share, Refused> {
        self.share
    }
}

/// `window` alone on `side` of `area`: the half a division's cut gives, and
/// whether the window fits in it.
///
/// [`None`] where the area is too small to cut — one unit or less along the
/// axis the side is on — because a half of nothing is not a place.
#[must_use]
pub fn alone_on(area: Area, window: Window, side: Side) -> Option<OnASide> {
    let axis = side.axis();
    let first = crate::node::the_first_half_of(area.size().along(axis));
    if first == 0 {
        return None;
    }
    let (before, after) = area.cut(axis, first);
    let half = if side.is_first() { before } else { after };
    let share = if window.minimum_along(axis) > half.size().along(axis) {
        Err(Refused::squeezed(window.id(), axis))
    } else if window.minimum_along(axis.across()) > half.size().along(axis.across()) {
        Err(Refused::squeezed(window.id(), axis.across()))
    } else {
        Ok(Share::of(window.id(), half))
    };
    Some(OnASide { half, share })
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::{Division, Point, Size, WindowId};

    fn an_area(width: u32, height: u32) -> Area {
        Area::of(Point::at(0, 0), Size::of(width, height)).expect("an area")
    }

    fn any(id: u64) -> Window {
        Window::any_size(WindowId::from_compositor(id))
    }

    /// Left and right meet, cover the area, and the odd unit is on the right —
    /// the division's rule, not one of this file's.
    #[test]
    fn the_two_halves_meet_and_the_odd_unit_is_the_second() {
        let area = an_area(1329, 788);
        let left = alone_on(area, any(1), Side::Left).expect("a half");
        let right = alone_on(area, any(1), Side::Right).expect("a half");
        assert_eq!((left.half().x(), left.half().width()), (0, 664));
        assert_eq!((right.half().x(), right.half().width()), (664, 665));
        assert_eq!(left.half().height(), 788);
        assert_eq!(right.half().height(), 788);
    }

    /// **The same half as the chord gives**, on either side: the chord's
    /// `divide_with_next` puts the focused window where the panel's Left or
    /// Right puts it alone.
    #[test]
    fn a_window_alone_on_a_side_has_the_half_the_chord_would_give_it() {
        let area = an_area(1329, 788);
        for side in [Side::Left, Side::Right] {
            let mut division = Division::of(area);
            division
                .divide_with_next(any(1), Some(any(2)), side)
                .expect("divided with the next");
            let divided = division
                .share_of(WindowId::from_compositor(1))
                .expect("a share");
            let alone = alone_on(area, any(1), side).expect("a half");
            assert_eq!(alone.half(), divided, "{side:?}");
            assert_eq!(alone.share().map(Share::area), Ok(divided));
        }
    }

    /// Refused rather than squeezed, and the half still answered so it can be
    /// shown.
    #[test]
    fn a_window_that_cannot_fit_is_refused_and_the_half_is_still_given() {
        let area = an_area(1000, 800);
        let wide = Window::at_least(WindowId::from_compositor(3), Size::of(501, 1));
        let refused = alone_on(area, wide, Side::Left).expect("a half");
        assert_eq!(
            refused.share(),
            Err(Refused::TooNarrow(WindowId::from_compositor(3)))
        );
        assert_eq!(refused.half().width(), 500);
        let tall = Window::at_least(WindowId::from_compositor(4), Size::of(1, 801));
        let refused = alone_on(area, tall, Side::Right).expect("a half");
        assert_eq!(
            refused.share(),
            Err(Refused::TooShort(WindowId::from_compositor(4)))
        );
        let exactly = Window::at_least(WindowId::from_compositor(5), Size::of(500, 800));
        assert!(
            alone_on(area, exactly, Side::Right)
                .expect("a half")
                .share()
                .is_ok()
        );
    }

    /// An area one unit across has no left half to give, so nothing is.
    #[test]
    fn an_area_too_small_to_cut_has_no_half() {
        assert_eq!(alone_on(an_area(1, 800), any(1), Side::Left), None);
        assert_eq!(alone_on(an_area(1, 800), any(1), Side::Right), None);
        assert!(alone_on(an_area(2, 800), any(1), Side::Right).is_some());
    }
}
