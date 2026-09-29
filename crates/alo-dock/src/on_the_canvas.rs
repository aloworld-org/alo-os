//! Where a window is on the workspace plane, and what the person is looking at.
//!
//! **Windows keep their positions; the canvas moves to them.** That sentence is
//! the whole of the workspace's model and this file is the arithmetic under it:
//! a window occupies a [`Patch`] of the plane, the person is looking at a
//! [`TheView`] of the same plane, and the only question asked here is whether
//! one is inside the other.
//!
//! # Logical pixels on a plane that is larger than a screen
//!
//! The units are the same logical pixels [`crate::measures`] uses, but the
//! coordinates are the **canvas's**, not a screen's. The plane extends past
//! every edge of every display, which is why a coordinate here is signed and a
//! screen's is not: a window can sit to the left of where the person is looking,
//! and *to the left of* has to be expressible.
//!
//! # Nothing here knows about a dock, a screen or a display
//!
//! A view is handed in. It is the compositor's business what part of the plane a
//! display is showing and how zoom changes it; this file only compares two
//! rectangles, so a settings panel previewing *what would a click do* asks the
//! same question the compositor asks.

/// A point on the workspace plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Spot {
    /// How far across, from the plane's origin. Negative is to the left of it.
    x: i64,
    /// How far down, from the plane's origin. Negative is above it.
    y: i64,
}

impl Spot {
    /// The point this far across and this far down.
    #[must_use]
    pub const fn at(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    /// How far across it is.
    #[must_use]
    pub const fn x(self) -> i64 {
        self.x
    }

    /// How far down it is.
    #[must_use]
    pub const fn y(self) -> i64 {
        self.y
    }
}

/// The part of the plane something occupies: a corner and a size.
///
/// A window's place on the canvas is one of these, and so is what a person is
/// looking at ([`TheView`]), because *is this window in view* is one rectangle
/// against another and would be a different question if they were different
/// shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Patch {
    /// Its top-left corner.
    corner: Spot,
    /// How wide it is. Never zero: see [`Patch::of`].
    wide: u32,
    /// How tall it is. Never zero.
    tall: u32,
}

/// Why something is not a patch of the plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotAPatch {
    /// A width or a height of nothing: what was given, both numbers.
    ///
    /// A window with no extent could not be looked at, brought forward or
    /// travelled to, and *is it in view* would have no honest answer — a point
    /// is either inside a rectangle or outside it, and a rectangle with no area
    /// is neither.
    NoExtent(u32, u32),
}

impl Patch {
    /// The patch with this corner, this wide and this tall.
    ///
    /// # Errors
    /// [`NotAPatch::NoExtent`] when either side is zero, naming both numbers.
    pub const fn of(corner: Spot, wide: u32, tall: u32) -> Result<Self, NotAPatch> {
        if wide == 0 || tall == 0 {
            return Err(NotAPatch::NoExtent(wide, tall));
        }
        Ok(Self { corner, wide, tall })
    }

    /// Its top-left corner.
    #[must_use]
    pub const fn corner(self) -> Spot {
        self.corner
    }

    /// How wide it is.
    #[must_use]
    pub const fn wide(self) -> u32 {
        self.wide
    }

    /// How tall it is.
    #[must_use]
    pub const fn tall(self) -> u32 {
        self.tall
    }

    /// The x just past its right edge.
    #[must_use]
    const fn past_right(self) -> i64 {
        self.corner.x.saturating_add(self.wide as i64)
    }

    /// The y just past its bottom edge.
    #[must_use]
    const fn past_bottom(self) -> i64 {
        self.corner.y.saturating_add(self.tall as i64)
    }

    /// Whether this patch lies **wholly** inside `outer`.
    #[must_use]
    const fn wholly_inside(self, outer: Self) -> bool {
        self.corner.x >= outer.corner.x
            && self.corner.y >= outer.corner.y
            && self.past_right() <= outer.past_right()
            && self.past_bottom() <= outer.past_bottom()
    }
}

/// What the person is looking at: the part of the plane on screen right now.
///
/// Zoom is not here. A view at 50% shows twice as much of the plane as one at
/// 100%, and that is already expressed by the patch being twice as large — so a
/// zoom level would be a second way to say the same thing, and two ways to say
/// one thing is how they come to disagree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheView {
    /// The part of the plane it shows.
    showing: Patch,
}

impl TheView {
    /// The view showing this part of the plane.
    #[must_use]
    pub const fn showing(showing: Patch) -> Self {
        Self { showing }
    }

    /// The part of the plane it shows.
    #[must_use]
    pub const fn shown(self) -> Patch {
        self.showing
    }

    /// Whether this patch needs no travel to be seen.
    ///
    /// **Wholly inside, not merely touching.** A window with a corner on screen
    /// is a window the person cannot read, and *pan only if needed* means
    /// needed to see the thing rather than needed to see evidence of it. The
    /// cost of the strict reading is a pan for a window that is nearly there;
    /// the cost of the loose one is a click that appears to do nothing.
    #[must_use]
    pub const fn already_shows(self, patch: Patch) -> bool {
        patch.wholly_inside(self.showing)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A patch this file's tests name often enough to be worth a word.
    fn patch(x: i64, y: i64, wide: u32, tall: u32) -> Patch {
        Patch::of(Spot::at(x, y), wide, tall).unwrap()
    }

    /// The plane extends in every direction, so a window to the left of the
    /// origin is an ordinary window rather than an impossible one.
    #[test]
    fn a_spot_can_be_left_of_and_above_the_origin() {
        let corner = Spot::at(-4_000, -2_500);
        assert_eq!((corner.x(), corner.y()), (-4_000, -2_500));
        assert!(Patch::of(corner, 800, 600).is_ok());
    }

    /// **A patch with no extent is refused**, naming both numbers, because a
    /// rectangle with no area is neither inside a view nor outside it and every
    /// question this file answers would have to invent a reply.
    #[test]
    fn something_with_no_width_or_no_height_is_not_a_patch() {
        assert_eq!(
            Patch::of(Spot::at(0, 0), 0, 600),
            Err(NotAPatch::NoExtent(0, 600))
        );
        assert_eq!(
            Patch::of(Spot::at(0, 0), 800, 0),
            Err(NotAPatch::NoExtent(800, 0))
        );
        assert!(
            Patch::of(Spot::at(0, 0), 1, 1).is_ok(),
            "one pixel is a patch"
        );
    }

    /// **A window wholly inside the view needs no travel**, and one hanging over
    /// any edge does — including the case that looks closest to being fine,
    /// which is a window over by a single pixel.
    #[test]
    fn a_window_needs_no_travel_only_when_all_of_it_is_shown() {
        let view = TheView::showing(patch(0, 0, 1_000, 800));

        assert!(view.already_shows(patch(100, 100, 400, 300)), "well inside");
        assert!(
            view.already_shows(patch(0, 0, 1_000, 800)),
            "exactly the view"
        );

        assert!(
            !view.already_shows(patch(999, 100, 400, 300)),
            "over the right"
        );
        assert!(
            !view.already_shows(patch(-1, 100, 400, 300)),
            "over the left"
        );
        assert!(
            !view.already_shows(patch(100, -1, 400, 300)),
            "over the top"
        );
        assert!(
            !view.already_shows(patch(100, 799, 400, 300)),
            "over the bottom"
        );
    }

    /// **One pixel is the whole of the difference**, in both directions, which
    /// is what says the edge case is decided rather than left to whichever way
    /// a comparison happened to be written.
    #[test]
    fn the_edge_is_decided_to_the_pixel() {
        let view = TheView::showing(patch(0, 0, 1_000, 800));
        assert!(
            view.already_shows(patch(600, 500, 400, 300)),
            "flush with both"
        );
        assert!(!view.already_shows(patch(601, 500, 400, 300)), "one past");
        assert!(!view.already_shows(patch(600, 501, 400, 300)), "one below");
    }

    /// **A window far off the plane is simply not shown**, rather than being a
    /// case anything has to special-case: the same comparison answers it.
    #[test]
    fn a_window_nowhere_near_the_view_is_not_shown() {
        let view = TheView::showing(patch(0, 0, 1_000, 800));
        assert!(!view.already_shows(patch(90_000, 90_000, 400, 300)));
        assert!(!view.already_shows(patch(-90_000, -90_000, 400, 300)));
    }

    /// **A view larger than the plane's populated part shows everything in it**,
    /// which is what zooming out is: the same question, a bigger rectangle.
    #[test]
    fn zooming_out_is_a_bigger_view_and_nothing_else() {
        let close = TheView::showing(patch(0, 0, 500, 400));
        let far = TheView::showing(patch(-2_000, -2_000, 8_000, 8_000));
        let window = patch(900, 700, 400, 300);

        assert!(!close.already_shows(window));
        assert!(far.already_shows(window), "the same window, a wider view");
    }
}
