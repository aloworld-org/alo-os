//! **What a window takes on the plane: itself and the title strip alo draws
//! above it** — the rectangle every placer must keep clear, rather than the
//! window alone.
//!
//! # Why
//!
//! Since 2026-11-01 every window — an application's, alo's terminal, Settings,
//! the record, a task window — has a title strip **above** its own rectangle
//! (`crate::window_edge::edge_of`), and the placers were handed the rectangles
//! alone. So a window opened below another was given its gap from the other's
//! bottom edge to its **own** top, and its strip, 32 tall, was drawn over the
//! gap and 16 into the window above. Seen on 2026-11-08 with Dolphin opened
//! under alo's terminal: the terminal's frame ends at 234 and Dolphin's strip
//! starts at 220. `docs/design/where-a-new-window-opens.md` promises the new
//! window opens *without covering any window*, with the gap.
//!
//! # The strip is not zoomed, so it is a different size on the plane at every
//! zoom
//!
//! `crate::an_edge_in_its_displays_units`: *the edge's own furniture is not
//! zoomed* — a strip is 32 of the design's units at the display's scale however
//! far the canvas is zoomed. On the plane that is `32 × scale ÷ zoom`: 32 at
//! life size and 100 %, 64 zoomed out to half. So [`ItsStrip::at`] takes the
//! zoom the window opens at and the display's scale, and is worked out for each
//! opening.
//!
//! **What this cannot promise, and says so.** A window placed at one zoom and
//! then looked at further out has a strip that has grown on the plane while the
//! gap has not, so far enough out a strip reaches over the gap again. That is
//! what an unzoomed label on a zoomable canvas does, and whether the gap should
//! grow with it is a question for the owner, not a number to pick here.
//!
//! # And at least as wide as the narrowest strip
//!
//! A window narrower than `THE_COMPACT_EDGE_IS` has a strip that wide, centred
//! over it (`window_edge::where_the_edge_goes`), so it takes that much room
//! across. The line round the window (one unit) is not counted: it is drawn in
//! the gap, which is what the gap is between.

use smithay::utils::{Logical, Point, Rectangle, Size};

use crate::window_edge::{THE_COMPACT_EDGE_IS, THE_STRIP_IS_TALL};

/// The ordinary scale, in per cent.
const ORDINARY: u32 = 100;

/// The title strip above every window, on the plane at one zoom and one scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ItsStrip {
    /// How tall the strip is on the plane.
    tall: i32,
    /// How wide the strip is at least, on the plane.
    at_least_wide: i32,
}

impl ItsStrip {
    /// The strip at this `zoom` on a display at `scale` per cent.
    ///
    /// [`None`] for a zoom that cannot be divided by — none can arrive, since
    /// `alo_canvas::Zoom` is built from thousandths above zero, and it is
    /// refused rather than reasoned about. A scale of nought is the ordinary
    /// one, as `crate::an_edge_in_its_displays_units` reads it.
    ///
    /// **Rounded up**, so a strip is never given less room than it is drawn in.
    #[must_use]
    pub(crate) fn at(zoom: f64, scale: u32) -> Option<Self> {
        if !zoom.is_finite() || zoom <= 0.0 {
            return None;
        }
        let scale = if scale == 0 { ORDINARY } else { scale };
        let on_the_plane = |units: i32| -> Option<i32> {
            let it = (f64::from(units) * f64::from(scale) / f64::from(ORDINARY) / zoom).ceil();
            if !it.is_finite() || it > f64::from(i32::MAX) || it < 0.0 {
                return None;
            }
            #[expect(
                clippy::cast_possible_truncation,
                reason = "checked to be a whole number inside an i32 on the line above"
            )]
            let it = it as i32;
            Some(it)
        };
        Some(Self {
            tall: on_the_plane(THE_STRIP_IS_TALL)?,
            at_least_wide: on_the_plane(THE_COMPACT_EDGE_IS)?,
        })
    }

    /// What a window at `window` takes: itself and its strip.
    #[must_use]
    pub(crate) fn round(self, window: Rectangle<i32, Logical>) -> Rectangle<i32, Logical> {
        let taken = self.round_a_size(window.size);
        Rectangle::new(
            Point::from((
                window
                    .loc
                    .x
                    .saturating_sub(self.wider_on_the_left(window.size.w)),
                window.loc.y.saturating_sub(self.tall),
            )),
            taken,
        )
    }

    /// How big a window of `window` is with its strip.
    #[must_use]
    pub(crate) fn round_a_size(self, window: Size<i32, Logical>) -> Size<i32, Logical> {
        Size::from((
            window.w.max(self.at_least_wide),
            window.h.saturating_add(self.tall),
        ))
    }

    /// Where a window of `window` goes when what it takes has its corner at
    /// `taken`: the other way from [`Self::round`].
    #[must_use]
    pub(crate) fn the_window_in(
        self,
        taken: Point<i32, Logical>,
        window: Size<i32, Logical>,
    ) -> Point<i32, Logical> {
        Point::from((
            taken.x.saturating_add(self.wider_on_the_left(window.w)),
            taken.y.saturating_add(self.tall),
        ))
    }

    /// How far a narrow window's strip reaches past its left side: half what
    /// it is wider, the odd unit on the right, as `window_edge` centres it.
    fn wider_on_the_left(self, wide: i32) -> i32 {
        (self.at_least_wide.saturating_sub(wide)).max(0) / 2
    }

    /// Rules 3 to 7 for a window and its strip, handed and answering windows
    /// alone: every open window and the active one are taken with their strips,
    /// and the answer is where the new window itself goes.
    ///
    /// **The handle the placer protects is the strip's corner**, which is where
    /// a person grabs a window, so it is checked against the fixed controls
    /// where it is really drawn.
    #[must_use]
    pub(crate) fn where_a_window_opens(
        self,
        view: Rectangle<i32, Logical>,
        open: &[Rectangle<i32, Logical>],
        active: Option<Rectangle<i32, Logical>>,
        wanted: Size<i32, Logical>,
        reading: alo_strings::Direction,
        fixed: &[Rectangle<i32, Logical>],
    ) -> Point<i32, Logical> {
        let open = self.round_each(open);
        let taken = crate::where_a_window_opens(
            view,
            &open,
            active.map(|window| self.round(window)),
            self.round_a_size(wanted),
            reading,
            fixed,
        );
        self.the_window_in(taken, wanted)
    }

    /// Every one of `windows` with its strip.
    #[must_use]
    pub(crate) fn round_each(
        self,
        windows: &[Rectangle<i32, Logical>],
    ) -> Vec<Rectangle<i32, Logical>> {
        windows.iter().map(|window| self.round(*window)).collect()
    }
}

#[cfg(test)]
#[path = "a_window_and_its_strip_tests.rs"]
mod tests;
