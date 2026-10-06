//! Which display a point, or a window, is on —
//! `docs/autonomy/more-than-one-display-plan.md`, the ground tasks 5b and 6
//! both stand on.
//!
//! # The desk is one space and the main screen is its origin
//!
//! `alo_displays::Position` says it outright: *how far across, which is
//! negative to the left of the main screen*. So a display's corner is already
//! expressed relative to the main one, and a window's own coordinates — which
//! this compositor has always kept in one space, because it has always had one
//! display — **are desk coordinates already**, with the main screen at the
//! origin.
//!
//! That is the whole reason this file is short. No new coordinate system is
//! introduced and none is needed; what was missing was somebody asking the
//! arrangement which of its rectangles a point falls in.
//!
//! # A window belongs to the display showing most of it
//!
//! A window may straddle two displays, so *which display is it on* needs a
//! rule rather than a lookup. The rule is **greatest overlap**, because it is
//! the one a person would give if asked while looking at the screen: the
//! window is on the display you can see most of it on.
//!
//! Two tie-breaks, in order, so the answer never depends on iteration order:
//! the **main screen** first — it is where a new window opens, so it is the
//! reasonable home for one split exactly down the middle — and then the
//! earliest display in the arrangement's own order.
//!
//! **A window on no display at all is `None`**, not the main screen. A window
//! dragged entirely off the desk is somewhere the person put it, and answering
//! *the main screen* would move it in the only sense this question has.

use crate::{ScreenPlace, Server};
use smithay::utils::{Physical, Rectangle};

/// How much of `window` falls on the display `place` occupies.
///
/// Zero where they do not meet, including where they only touch along an
/// edge: a window sharing a boundary with a display shows nothing on it.
fn how_much_is_on(window: Rectangle<i32, Physical>, place: &ScreenPlace) -> i64 {
    let (across, along) = place.room().across_and_along();
    let corner = place.at();
    let left = corner.across().max(window.loc.x);
    let top = corner.down().max(window.loc.y);
    let right = corner
        .across()
        .saturating_add(across)
        .min(window.loc.x.saturating_add(window.size.w));
    let bottom = corner
        .down()
        .saturating_add(along)
        .min(window.loc.y.saturating_add(window.size.h));
    if right <= left || bottom <= top {
        return 0;
    }
    i64::from(right - left) * i64::from(bottom - top)
}

impl Server {
    /// The display showing most of this window, in desk coordinates.
    ///
    /// [`None`] for a session with no arrangement, and for a window that falls
    /// on no display at all. Both are *this question has no answer* rather
    /// than a default, because every caller of this does something to a window
    /// on behalf of a display, and doing it on behalf of the wrong one is
    /// worse than not doing it.
    ///
    /// # Errors
    /// None. A refusal here is an absence, not a fault.
    #[must_use]
    pub fn the_display_a_window_is_on(
        &self,
        window: Rectangle<i32, Physical>,
    ) -> Option<&ScreenPlace> {
        let screens = self.the_screens()?;
        screens
            .each()
            .map(|place| (how_much_is_on(window, place), place))
            .filter(|(shown, _)| *shown > 0)
            // `max_by_key` keeps the **last** maximum, so the ordering is
            // written to put the preferred answer last: more shown wins, and
            // at equal area the main screen beats a later one. Relying on
            // which end `max_by_key` keeps is the kind of detail that breaks
            // silently, so the tie-break is tested rather than trusted.
            .max_by_key(|(shown, place)| (*shown, i32::from(place.is_main())))
            .map(|(_, place)| place)
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "which_display_a_window_is_on_tests.rs"]
mod tests;
