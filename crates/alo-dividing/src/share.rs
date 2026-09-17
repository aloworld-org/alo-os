//! One share of a display, as a division answers it: which window, and where.

use crate::area::Area;
use crate::window::WindowId;

/// A window's share of a display, laid out.
///
/// What the shell draws a window into. It is an answer, not a handle: changing
/// a division is done through [`crate::Division`], and a share read before the
/// change describes the division as it was.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Share {
    /// The window in it.
    window: WindowId,
    /// Where it is, in logical units.
    area: Area,
}

impl Share {
    /// This window, in this area.
    pub(crate) const fn of(window: WindowId, area: Area) -> Self {
        Self { window, area }
    }

    /// The window in it.
    #[must_use]
    pub const fn window(self) -> WindowId {
        self.window
    }

    /// Where it is, in logical units.
    #[must_use]
    pub const fn area(self) -> Area {
        self.area
    }
}
