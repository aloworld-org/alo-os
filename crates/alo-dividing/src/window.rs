//! A window as a division knows it: which one, and how small it may be made.
//!
//! **Nothing else.** Not its title, not the document in it, not the application's
//! name: a division is shares and the windows in them, and the one fact about a
//! window it needs in order to keep a promise is the smallest size the window
//! says it can be drawn at. What a window is called is the shell's to draw next to
//! a refusal, and task 2 of this crate's plan is explicit that what is remembered
//! is never a title.

use crate::area::Size;
use crate::side::Axis;

/// Which window, as the compositor names it.
///
/// Opaque here: a division compares two of them and never looks inside.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(u64);

impl WindowId {
    /// The window the compositor calls this.
    #[must_use]
    pub const fn from_compositor(id: u64) -> Self {
        Self(id)
    }

    /// What the compositor called it.
    #[must_use]
    pub const fn to_compositor(self) -> u64 {
        self.0
    }
}

/// A window about to be given a share, with the smallest it can be.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Window {
    /// Which one.
    id: WindowId,
    /// The smallest size it says it can be drawn at, in logical units.
    minimum: Size,
}

impl Window {
    /// A window that says it cannot be smaller than `minimum`.
    ///
    /// An application that states no minimum is still at least one unit each
    /// way: a share of nothing is not a share.
    #[must_use]
    pub const fn at_least(id: WindowId, minimum: Size) -> Self {
        Self { id, minimum }
    }

    /// A window that states no minimum size.
    #[must_use]
    pub const fn any_size(id: WindowId) -> Self {
        Self::at_least(id, Size::of(0, 0))
    }

    /// Which one.
    #[must_use]
    pub const fn id(self) -> WindowId {
        self.id
    }

    /// The smallest size it says it can be drawn at.
    #[must_use]
    pub const fn minimum(self) -> Size {
        self.minimum
    }

    /// The smallest it can be along one axis — never less than one unit.
    pub(crate) fn minimum_along(self, axis: Axis) -> u32 {
        self.minimum.along(axis).max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A window with no minimum still takes a unit each way, so no share in a
    /// division is ever empty.
    #[test]
    fn a_window_is_never_smaller_than_one_unit() {
        let window = Window::any_size(WindowId::from_compositor(7));
        assert_eq!(window.minimum_along(Axis::SideBySide), 1);
        assert_eq!(window.minimum_along(Axis::OneAboveTheOther), 1);
        assert_eq!(window.id().to_compositor(), 7);
        let wide = Window::at_least(WindowId::from_compositor(8), Size::of(800, 600));
        assert_eq!(wide.minimum_along(Axis::SideBySide), 800);
        assert_eq!(wide.minimum_along(Axis::OneAboveTheOther), 600);
    }
}
