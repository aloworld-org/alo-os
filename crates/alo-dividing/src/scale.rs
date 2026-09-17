//! Logical units into pixels, so a division means the same on a scaled screen.
//!
//! **Nothing here draws.** This is the arithmetic the shell does when it puts a
//! share on a screen, kept beside the division because the promise that shares
//! neither overlap nor leave a gap has to survive it: a division that tiles in
//! logical units and leaves a one-pixel seam at 150 % has not kept the promise
//! anybody can see.
//!
//! # Edges are scaled, never sizes
//!
//! Scaling each share's width on its own and rounding it would round two
//! neighbours independently, and at a fractional scale their pixels would then
//! overlap or part by one. So [`Scale::to_pixels`] rounds each **edge** — a
//! boundary two shares meet at is one logical number, it becomes one pixel
//! number, and both shares take it. A share's pixel size is the difference of
//! its two edges.
//!
//! The scale is held in 120ths, the unit Wayland's fractional scaling protocol
//! uses, so 150 % is 180 and 125 % is 150 with no floating point anywhere.

use crate::area::Area;

/// How many 120ths of a pixel one logical unit is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Scale(u32);

/// A rectangle in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pixels {
    /// Left edge.
    pub x: u64,
    /// Top edge.
    pub y: u64,
    /// Across.
    pub width: u64,
    /// Down.
    pub height: u64,
}

impl Scale {
    /// One pixel to a logical unit.
    pub const WHOLE: Self = Self(120);

    /// The scale that is this many 120ths, or `None` for less than a quarter of
    /// a pixel to a unit or more than eight pixels: no display is either, and a
    /// scale of zero would put every share on one pixel.
    #[must_use]
    pub const fn in_120ths(n: u32) -> Option<Self> {
        if n >= 30 && n <= 960 {
            Some(Self(n))
        } else {
            None
        }
    }

    /// Where this area is in pixels.
    #[must_use]
    pub const fn to_pixels(self, area: Area) -> Pixels {
        let x = self.edge(area.x());
        let y = self.edge(area.y());
        Pixels {
            x,
            y,
            width: self.edge(area.right()) - x,
            height: self.edge(area.bottom()) - y,
        }
    }

    /// One logical coordinate as a pixel coordinate, rounded to nearest.
    const fn edge(self, logical: u32) -> u64 {
        (logical as u64 * self.0 as u64 + 60) / 120
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::area::{Point, Size};
    use crate::side::Axis;

    /// Two pieces of a cut still meet exactly in pixels at a fractional scale,
    /// even where their widths round differently.
    #[test]
    fn neighbours_meet_in_pixels_at_a_fractional_scale() {
        let display = Area::of(Point::at(0, 0), Size::of(1281, 721)).unwrap();
        for n in [120, 150, 180, 210, 240] {
            let scale = Scale::in_120ths(n).unwrap();
            for cut in [1, 333, 640, 641, 1280] {
                let (left, right) = display.cut(Axis::SideBySide, cut);
                let (l, r) = (scale.to_pixels(left), scale.to_pixels(right));
                assert_eq!(l.x + l.width, r.x, "{n}/120 at {cut}");
                assert_eq!(l.width + r.width, scale.to_pixels(display).width);
            }
        }
    }

    /// A scale that is not a display's is refused.
    #[test]
    fn a_scale_no_display_has_is_refused() {
        assert!(Scale::in_120ths(0).is_none());
        assert!(Scale::in_120ths(961).is_none());
        assert_eq!(Scale::in_120ths(120), Some(Scale::WHOLE));
    }
}
