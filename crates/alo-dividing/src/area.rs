//! Where things are on a display, in logical units.
//!
//! **Nothing here is a pixel.** A logical unit is what the display's scale turns
//! into pixels ([`crate::scale`]), so a left half is the same left half on a
//! laptop at 150 % as on an external screen at 100 %, and a window's minimum
//! size — which an application states in the same units — means the same thing
//! on both.
//!
//! Every coordinate is a whole number of units. Two shares either side of a
//! boundary meet at one integer, so there is no rounding anywhere in the
//! division itself that could open a gap or make two shares overlap by a
//! fraction; the only rounding is the scale's, and that module is where the
//! argument for it is.

/// The largest a display may be along either axis, in logical units.
///
/// A million units is far beyond any screen made, and it keeps every sum of two
/// coordinates inside a `u32` so the arithmetic in this crate cannot overflow.
pub const LARGEST: u32 = 1_000_000;

/// One point on a display, in logical units from its top-left corner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    /// Units from the left.
    pub x: u32,
    /// Units from the top.
    pub y: u32,
}

impl Point {
    /// The point this many units from the left and from the top.
    #[must_use]
    pub const fn at(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

/// How large something is, in logical units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Size {
    /// Across.
    pub width: u32,
    /// Down.
    pub height: u32,
}

impl Size {
    /// This wide and this high.
    #[must_use]
    pub const fn of(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// The length along one axis.
    #[must_use]
    pub const fn along(self, axis: crate::Axis) -> u32 {
        match axis {
            crate::Axis::SideBySide => self.width,
            crate::Axis::OneAboveTheOther => self.height,
        }
    }
}

/// A rectangle on a display, in logical units.
///
/// Its right and bottom edges are exclusive: an area at `x` 0 and 960 wide ends
/// where the next one, at `x` 960, begins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Area {
    /// Left edge.
    x: u32,
    /// Top edge.
    y: u32,
    /// Across.
    width: u32,
    /// Down.
    height: u32,
}

/// Why a rectangle cannot be a display's area.
///
/// Said to whoever hands this crate a display — the compositor — and never to a
/// person, so it keeps its English: the thing wrong is a number the compositor
/// passed, and the person in front of the machine can do nothing with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AreaError {
    /// Nothing across or nothing down.
    #[error("a display area of {width} by {height} has nothing to divide")]
    Empty {
        /// What was passed across.
        width: u32,
        /// What was passed down.
        height: u32,
    },
    /// Further from the origin than this crate's arithmetic is held to.
    #[error("a display area reaching past {LARGEST} logical units is not a display")]
    TooLarge,
}

impl Area {
    /// The rectangle with this top-left corner and this size.
    ///
    /// # Errors
    /// [`AreaError`] when it is empty along either axis, or reaches past
    /// [`LARGEST`] units from the origin.
    pub fn of(corner: Point, size: Size) -> Result<Self, AreaError> {
        if size.width == 0 || size.height == 0 {
            return Err(AreaError::Empty {
                width: size.width,
                height: size.height,
            });
        }
        let reaches_across = u64::from(corner.x) + u64::from(size.width);
        let reaches_down = u64::from(corner.y) + u64::from(size.height);
        if reaches_across > u64::from(LARGEST) || reaches_down > u64::from(LARGEST) {
            return Err(AreaError::TooLarge);
        }
        Ok(Self {
            x: corner.x,
            y: corner.y,
            width: size.width,
            height: size.height,
        })
    }

    /// A rectangle already known to be inside a valid one.
    ///
    /// Only the division builds these, from pieces of an area it was handed.
    pub(crate) const fn inside(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// The left edge.
    #[must_use]
    pub const fn x(self) -> u32 {
        self.x
    }

    /// The top edge.
    #[must_use]
    pub const fn y(self) -> u32 {
        self.y
    }

    /// Across.
    #[must_use]
    pub const fn width(self) -> u32 {
        self.width
    }

    /// Down.
    #[must_use]
    pub const fn height(self) -> u32 {
        self.height
    }

    /// Where it ends across, exclusive.
    #[must_use]
    pub const fn right(self) -> u32 {
        self.x + self.width
    }

    /// Where it ends down, exclusive.
    #[must_use]
    pub const fn bottom(self) -> u32 {
        self.y + self.height
    }

    /// How large it is.
    #[must_use]
    pub const fn size(self) -> Size {
        Size::of(self.width, self.height)
    }

    /// How many square units it covers.
    #[must_use]
    pub const fn covers(self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// Whether this point is inside it.
    #[must_use]
    pub const fn holds(self, point: Point) -> bool {
        point.x >= self.x && point.x < self.right() && point.y >= self.y && point.y < self.bottom()
    }

    /// Whether the two share any square unit at all.
    #[must_use]
    pub const fn overlaps(self, other: Self) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }

    /// Whether the whole of `other` is inside this.
    #[must_use]
    pub const fn encloses(self, other: Self) -> bool {
        other.x >= self.x
            && other.y >= self.y
            && other.right() <= self.right()
            && other.bottom() <= self.bottom()
    }

    /// Where along one axis it starts.
    pub(crate) const fn start_along(self, axis: crate::Axis) -> u32 {
        match axis {
            crate::Axis::SideBySide => self.x,
            crate::Axis::OneAboveTheOther => self.y,
        }
    }

    /// Cut in two along an axis, the first piece `first` units long.
    ///
    /// The caller holds `first` to at most the length along that axis; the two
    /// pieces meet exactly, which is the whole of how a division has no gaps.
    pub(crate) const fn cut(self, axis: crate::Axis, first: u32) -> (Self, Self) {
        match axis {
            crate::Axis::SideBySide => (
                Self::inside(self.x, self.y, first, self.height),
                Self::inside(self.x + first, self.y, self.width - first, self.height),
            ),
            crate::Axis::OneAboveTheOther => (
                Self::inside(self.x, self.y, self.width, first),
                Self::inside(self.x, self.y + first, self.width, self.height - first),
            ),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::Axis;

    /// A display with nothing across or nothing down is refused, and so is one
    /// far enough out that a sum of two coordinates could overflow.
    #[test]
    fn an_area_that_is_not_a_display_is_refused() {
        assert_eq!(
            Area::of(Point::at(0, 0), Size::of(0, 1080)),
            Err(AreaError::Empty {
                width: 0,
                height: 1080
            })
        );
        assert_eq!(
            Area::of(Point::at(LARGEST, 0), Size::of(1, 1)),
            Err(AreaError::TooLarge)
        );
        assert!(Area::of(Point::at(1920, 0), Size::of(2560, 1440)).is_ok());
    }

    /// Two pieces of a cut meet at one whole unit: nothing between them and
    /// nothing shared.
    #[test]
    fn a_cut_leaves_two_pieces_that_meet_exactly() {
        let display = Area::of(Point::at(1920, 0), Size::of(1921, 1080)).unwrap();
        let (left, right) = display.cut(Axis::SideBySide, 960);
        assert_eq!(left.right(), right.x());
        assert!(!left.overlaps(right));
        assert_eq!(left.covers() + right.covers(), display.covers());
        let (top, bottom) = display.cut(Axis::OneAboveTheOther, 540);
        assert_eq!(top.bottom(), bottom.y());
        assert!(!top.overlaps(bottom));
    }
}
