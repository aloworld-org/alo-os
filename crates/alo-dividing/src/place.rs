//! What a piece of the screen is called: a half, a quarter, or a part.
//!
//! Read off the geometry rather than carried along with it. A proposal says
//! *Top-left quarter* because the area it would give the window **is** the
//! display's top-left quarter, and says *Part of the screen* when it is not —
//! so the label can never describe a place other than the outline drawn beside
//! it.

use alo_strings::{Filling, Said, Strings};

use crate::area::Area;
use crate::words::{self, Word};

/// A named piece of a display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Place {
    /// The left half.
    LeftHalf,
    /// The right half.
    RightHalf,
    /// The top half.
    TopHalf,
    /// The bottom half.
    BottomHalf,
    /// The top-left quarter.
    TopLeftQuarter,
    /// The top-right quarter.
    TopRightQuarter,
    /// The bottom-left quarter.
    BottomLeftQuarter,
    /// The bottom-right quarter.
    BottomRightQuarter,
    /// Some other piece — half of a quarter, say.
    Part,
}

impl Place {
    /// What `area` is on `display`.
    ///
    /// A half is the display cut once at its middle, the first piece rounded
    /// down, which is exactly how a division cuts one; a quarter is a half cut
    /// again the same way. An odd display is therefore still divided into named
    /// halves and quarters.
    #[must_use]
    pub fn of(area: Area, display: Area) -> Self {
        let left = display.width() / 2;
        let top = display.height() / 2;
        let (x, y) = (display.x(), display.y());
        let (w, h) = (display.width(), display.height());
        let is = |ax: u32, ay: u32, aw: u32, ah: u32| {
            area.x() == x + ax && area.y() == y + ay && area.width() == aw && area.height() == ah
        };
        let halves = [
            (Self::LeftHalf, is(0, 0, left, h)),
            (Self::RightHalf, is(left, 0, w - left, h)),
            (Self::TopHalf, is(0, 0, w, top)),
            (Self::BottomHalf, is(0, top, w, h - top)),
            (Self::TopLeftQuarter, is(0, 0, left, top)),
            (Self::TopRightQuarter, is(left, 0, w - left, top)),
            (Self::BottomLeftQuarter, is(0, top, left, h - top)),
            (Self::BottomRightQuarter, is(left, top, w - left, h - top)),
        ];
        halves
            .into_iter()
            .find_map(|(place, matches)| matches.then_some(place))
            .unwrap_or(Self::Part)
    }

    /// The string this crate declares for it.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::LeftHalf => words::LEFT_HALF,
            Self::RightHalf => words::RIGHT_HALF,
            Self::TopHalf => words::TOP_HALF,
            Self::BottomHalf => words::BOTTOM_HALF,
            Self::TopLeftQuarter => words::TOP_LEFT_QUARTER,
            Self::TopRightQuarter => words::TOP_RIGHT_QUARTER,
            Self::BottomLeftQuarter => words::BOTTOM_LEFT_QUARTER,
            Self::BottomRightQuarter => words::BOTTOM_RIGHT_QUARTER,
            Self::Part => words::PART,
        }
    }

    /// What it is called, in the language the person reads.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
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
    use crate::testing::in_english;

    /// An area on a display.
    fn area(x: u32, y: u32, width: u32, height: u32) -> Area {
        Area::of(Point::at(x, y), Size::of(width, height)).unwrap()
    }

    /// Halves and quarters are recognised on a display that does not start at
    /// the origin and is an odd number of units wide, and anything else is a
    /// part.
    #[test]
    fn halves_and_quarters_are_read_off_the_geometry() {
        let display = area(1920, 0, 1281, 1025);
        assert_eq!(
            Place::of(area(1920, 0, 640, 1025), display),
            Place::LeftHalf
        );
        assert_eq!(
            Place::of(area(2560, 0, 641, 1025), display),
            Place::RightHalf
        );
        assert_eq!(Place::of(area(1920, 0, 1281, 512), display), Place::TopHalf);
        assert_eq!(
            Place::of(area(1920, 512, 1281, 513), display),
            Place::BottomHalf
        );
        assert_eq!(
            Place::of(area(1920, 0, 640, 512), display),
            Place::TopLeftQuarter
        );
        assert_eq!(
            Place::of(area(2560, 512, 641, 513), display),
            Place::BottomRightQuarter
        );
        assert_eq!(Place::of(area(1920, 0, 320, 512), display), Place::Part);
        assert_eq!(Place::of(display, display), Place::Part);
    }

    /// Every place is called something of its own.
    #[test]
    fn every_place_says_what_it_is() {
        let strings = in_english();
        assert_eq!(
            Place::TopLeftQuarter.said(&strings).text(),
            "Top-left quarter"
        );
        assert!(!Place::Part.said(&strings).is_a_bug());
    }
}
