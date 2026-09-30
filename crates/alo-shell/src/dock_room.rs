//! Whether a window needs the room the dock is sitting in.
//!
//! `alo-dock` has held the whole of the person's side of this since #247:
//! [`alo_dock::Hiding`] is their two choices, [`alo_dock::TheRoom`] is the
//! input, and `Dock::showing` is the decision between them, exhaustive and
//! tested. What was missing until now is the one thing that looks at real
//! windows — measured 2026-09-30, `TheRoom::` appeared nowhere outside
//! `alo-dock`, so a person could choose the behaviour, the dock knew what to do
//! when told, and **nothing ever told it.**
//!
//! This is the telling. `docs/design/when-the-dock-gives-way.md` settles what
//! the sentence *a window needs the room* has to mean, and this file is that
//! decision as arithmetic.
//!
//! # Which windows count
//!
//! **Any mapped window on this display whose area overlaps the dock's band** —
//! not only the focused one. A person who set the dock to give way and is
//! watching something play behind it did not ask for the dock back when they
//! clicked elsewhere, and reading *focused* would make the dock appear and
//! disappear on focus changes that have nothing to do with the room.
//!
//! Not `HowItSits::FillingTheScreen` either: a window filling a display already
//! bypasses the dock entirely, so reading fullscreen as the trigger would leave
//! the ordinary case — a window dragged over the band — doing nothing.
//!
//! # Why this cannot oscillate
//!
//! The obvious implementation flickers: the dock hides, the work area grows,
//! the window re-lays out, it no longer overlaps, the dock returns, the work
//! area shrinks. A predicate computed from a geometry its own answer moves
//! cannot distinguish *a window needs the room* from *a window needs the room
//! because the dock hid* — the two inputs are the same reading, which is the
//! family
//! [ADR 0080](../../../docs/decisions/0080-a-signal-that-cannot-be-wrong-tells-you-nothing.md)
//! names.
//!
//! **So the work area does not depend on the dock's setting.** The dock is laid
//! out and its band computed whatever the person chose; only whether it is
//! *painted* depends on the windows. `crate::desktop_raster` keeps handing
//! `room_beside` the drawn dock for that reason — the two desktop panels get
//! the same room whether the dock is showing or not. The band is therefore a
//! constant with respect to this predicate, the cycle has no edge, and there is
//! nothing for hysteresis or a timer to damp. Following `alo_dock::revealing`,
//! which has no timers on purpose.
//!
//! # Told, never fetched
//!
//! The rectangles are handed in. This file does not reach into the compositor
//! for them, because only the caller knows which display's windows these are
//! and what scale converts them into the band's own space.

use smithay::utils::{Physical, Rectangle};

use alo_dock::TheRoom;

/// Whether any of these windows needs the room the dock's band occupies.
///
/// `band` and `windows` must be in the same space — this display's physical
/// pixels. The caller converts, because the caller is the only one that knows
/// this display's scale and origin.
///
/// An empty list is [`TheRoom::Free`], which is the true answer: no window
/// needs the room when there are no windows.
pub(crate) fn the_room(
    band: Rectangle<i32, Physical>,
    windows: &[Rectangle<i32, Physical>],
) -> TheRoom {
    if windows.iter().any(|window| overlaps(*window, band)) {
        TheRoom::AWindowNeedsIt
    } else {
        TheRoom::Free
    }
}

/// Whether two rectangles share any area at all.
///
/// **Touching is not overlapping.** A window whose bottom edge is exactly the
/// band's top edge shares an edge and no area, and a dock that gave way to a
/// window merely resting against it would give way to a maximised window on
/// every machine — which is the common case, not the exception.
///
/// **A rectangle with no area overlaps nothing, and that is checked rather
/// than assumed.** The first version of this function said the strict
/// comparisons gave it for free and they do not: a zero-sized window *inside*
/// the band satisfies all four, because each edge comparison is against the
/// other rectangle and none of them asks whether this one encloses anything.
/// Its own test caught it before it ran anywhere. A client can commit a
/// zero-sized geometry, so this is a real case and not a tidiness one.
fn overlaps(one: Rectangle<i32, Physical>, other: Rectangle<i32, Physical>) -> bool {
    if one.size.w <= 0 || one.size.h <= 0 || other.size.w <= 0 || other.size.h <= 0 {
        return false;
    }
    let one_right = one.loc.x.saturating_add(one.size.w);
    let one_bottom = one.loc.y.saturating_add(one.size.h);
    let other_right = other.loc.x.saturating_add(other.size.w);
    let other_bottom = other.loc.y.saturating_add(other.size.h);
    one.loc.x < other_right
        && other.loc.x < one_right
        && one.loc.y < other_bottom
        && other.loc.y < one_bottom
}

#[cfg(test)]
mod tests {
    use super::*;
    use smithay::utils::{Point, Size};

    /// A display 1000 wide and 800 tall, with the dock's band along the bottom
    /// 60 pixels of it.
    fn band() -> Rectangle<i32, Physical> {
        Rectangle::new(Point::from((0, 740)), Size::from((1000, 60)))
    }

    /// A window at these coordinates.
    fn window(x: i32, y: i32, w: i32, h: i32) -> Rectangle<i32, Physical> {
        Rectangle::new(Point::from((x, y)), Size::from((w, h)))
    }

    /// **No windows is not a window needing the room.** The arm that every
    /// machine answers on a fresh desktop, and the one a zero would have got
    /// wrong if this returned a count.
    #[test]
    fn an_empty_desktop_leaves_the_room_free() {
        assert_eq!(the_room(band(), &[]), TheRoom::Free);
    }

    /// **A window nowhere near the band leaves it free.**
    #[test]
    fn a_window_above_the_band_leaves_the_room_free() {
        let windows = [window(100, 100, 400, 300)];
        assert_eq!(the_room(band(), &windows), TheRoom::Free);
    }

    /// **A window over the band needs the room** — the case the whole setting
    /// exists for.
    #[test]
    fn a_window_over_the_band_needs_the_room() {
        let windows = [window(100, 700, 400, 300)];
        assert_eq!(the_room(band(), &windows), TheRoom::AWindowNeedsIt);
    }

    /// **Touching is not overlapping.** A window whose bottom edge is exactly
    /// the band's top edge shares an edge and no area. A maximised window on a
    /// machine whose work area stops at the dock is exactly this, and a dock
    /// that gave way to it would give way permanently on the common case.
    #[test]
    fn a_window_resting_on_the_band_does_not_need_the_room() {
        let windows = [window(0, 0, 1000, 740)];
        assert_eq!(the_room(band(), &windows), TheRoom::Free);

        let one_more = [window(0, 0, 1000, 741)];
        assert_eq!(
            the_room(band(), &one_more),
            TheRoom::AWindowNeedsIt,
            "one pixel into the band is into the band"
        );
    }

    /// **Any window counts, not the first or the focused one.** The list is
    /// unordered as far as this is concerned, and one window over the band is
    /// enough however many are not.
    #[test]
    fn one_window_over_the_band_is_enough_however_many_are_not() {
        let windows = [
            window(0, 0, 200, 200),
            window(300, 100, 200, 200),
            window(600, 760, 200, 100),
        ];
        assert_eq!(the_room(band(), &windows), TheRoom::AWindowNeedsIt);

        let none_of_them = [window(0, 0, 200, 200), window(300, 100, 200, 200)];
        assert_eq!(the_room(band(), &none_of_them), TheRoom::Free);
    }

    /// **A window off the side of the band does not need it**, even at the
    /// same height. The dock is a floating bar since #274 and does not span
    /// the display, so *beside it* is a real position a window can be in.
    #[test]
    fn a_window_beside_a_floating_band_does_not_need_the_room() {
        let floating = Rectangle::new(Point::from((300, 740)), Size::from((400, 60)));
        let beside = [window(0, 700, 250, 100)];
        assert_eq!(the_room(floating, &beside), TheRoom::Free);

        let over = [window(0, 700, 400, 100)];
        assert_eq!(the_room(floating, &over), TheRoom::AWindowNeedsIt);
    }

    /// **A window of no size needs nothing**, and this had to be written
    /// rather than inherited.
    ///
    /// The first version of `overlaps` claimed the strict comparisons gave
    /// this for free. They do not: a zero-sized window *inside* the band
    /// satisfies all four, because every comparison is against the other
    /// rectangle's edges and none asks whether this one encloses anything.
    /// This test failed on its first run and the comment claiming otherwise
    /// was wrong in the same breath as the code.
    #[test]
    fn a_window_with_no_area_needs_no_room() {
        let inside = [window(500, 760, 0, 0)];
        assert_eq!(the_room(band(), &inside), TheRoom::Free);

        let no_height = [window(100, 760, 400, 0)];
        assert_eq!(the_room(band(), &no_height), TheRoom::Free);

        let no_width = [window(500, 700, 0, 200)];
        assert_eq!(the_room(band(), &no_width), TheRoom::Free);
    }
}
