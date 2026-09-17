//! The windows, displays and strings this crate's own tests are written against.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_strings::Strings;

use crate::area::{Area, Point, Size};
use crate::division::Division;
use crate::node::Node;
use crate::side::Side;
use crate::window::{Window, WindowId};
use crate::words::dividing_words;

/// This crate's own words, with nothing translated.
pub(crate) fn in_english() -> Strings {
    Strings::of(dividing_words().unwrap())
}

/// A window that states no minimum.
pub(crate) fn any(id: u64) -> Window {
    Window::any_size(WindowId::from_compositor(id))
}

/// A window that cannot be smaller than this.
pub(crate) fn at_least(id: u64, width: u32, height: u32) -> Window {
    Window::at_least(WindowId::from_compositor(id), Size::of(width, height))
}

/// An area in logical units.
pub(crate) fn area(x: u32, y: u32, width: u32, height: u32) -> Area {
    Area::of(Point::at(x, y), Size::of(width, height)).unwrap()
}

/// A 1920 by 1080 display at the origin.
pub(crate) fn display() -> Area {
    area(0, 0, 1920, 1080)
}

/// That display, divided into a left half holding `left` and a right half
/// holding `right`.
pub(crate) fn divided_in_halves(left: u64, right: u64) -> Division {
    let mut division = Division::of(display());
    division.replace(Node::Share(any(left)));
    division
        .divide(WindowId::from_compositor(left), any(right), Side::Right)
        .unwrap();
    division
}
