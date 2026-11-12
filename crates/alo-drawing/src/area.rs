//! **Where something is drawn and how large**, in one display's pixels.
//!
//! The shell's drawing borrowed Smithay's rectangle to say where its pixels go,
//! and that was the only thing tying it to a compositor. This is the drawing's
//! own, with **the same fields and the same arithmetic** as the one it replaces
//! (`smithay::utils::Rectangle<i32, Physical>` at 0.7.0), so a picture that moves
//! here lands on exactly the pixels it did; `alo-shell`'s
//! `the_desktops_pictures_pinned.rs` holds that. The shell converts at the one
//! seam where pixels are handed to the compositor.

/// A point, in pixels from the top left of what is drawn on.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Point {
    /// Across, from the left.
    pub x: i32,
    /// Down, from the top.
    pub y: i32,
}

/// How large something is, in pixels.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Size {
    /// How wide.
    pub w: i32,
    /// How tall.
    pub h: i32,
}

/// A rectangle: its top left corner and its size.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Area {
    /// The top left corner.
    pub loc: Point,
    /// How large.
    pub size: Size,
}

impl From<(i32, i32)> for Point {
    fn from((x, y): (i32, i32)) -> Self {
        Self { x, y }
    }
}

impl From<(i32, i32)> for Size {
    fn from((w, h): (i32, i32)) -> Self {
        Self { w, h }
    }
}

impl Area {
    /// The rectangle at `loc`, `size` large.
    #[must_use]
    pub const fn new(loc: Point, size: Size) -> Self {
        Self { loc, size }
    }

    /// The rectangle `size` large at the top left corner.
    #[must_use]
    pub const fn from_size(size: Size) -> Self {
        Self {
            loc: Point { x: 0, y: 0 },
            size,
        }
    }

    /// Whether `point` is inside, its right and bottom edges excluded.
    #[must_use]
    pub fn contains(self, point: impl Into<Point>) -> bool {
        let point = point.into();
        point.x >= self.loc.x
            && point.y >= self.loc.y
            && point.x < self.loc.x.saturating_add(self.size.w)
            && point.y < self.loc.y.saturating_add(self.size.h)
    }

    /// Whether `other` is wholly inside.
    #[must_use]
    pub const fn contains_rect(self, other: Self) -> bool {
        other.loc.x >= self.loc.x
            && other.loc.y >= self.loc.y
            && other.loc.x.saturating_add(other.size.w) <= self.loc.x.saturating_add(self.size.w)
            && other.loc.y.saturating_add(other.size.h) <= self.loc.y.saturating_add(self.size.h)
    }

    /// Whether the two share a pixel. Touching edges do not.
    #[must_use]
    pub const fn overlaps(self, other: Self) -> bool {
        self.loc.x < other.loc.x.saturating_add(other.size.w)
            && other.loc.x < self.loc.x.saturating_add(self.size.w)
            && self.loc.y < other.loc.y.saturating_add(other.size.h)
            && other.loc.y < self.loc.y.saturating_add(self.size.h)
    }

    /// The pixels the two share, or [`None`] where they share none.
    #[must_use]
    pub fn intersection(self, other: Self) -> Option<Self> {
        if !self.overlaps(other) {
            return None;
        }
        let left = self.loc.x.max(other.loc.x);
        let top = self.loc.y.max(other.loc.y);
        let right = self
            .loc
            .x
            .saturating_add(self.size.w)
            .min(other.loc.x.saturating_add(other.size.w));
        let bottom = self
            .loc
            .y
            .saturating_add(self.size.h)
            .min(other.loc.y.saturating_add(other.size.h));
        Some(Self::new(
            Point { x: left, y: top },
            Size {
                w: right - left,
                h: bottom - top,
            },
        ))
    }
}

#[cfg(test)]
#[path = "area_tests.rs"]
mod tests;
