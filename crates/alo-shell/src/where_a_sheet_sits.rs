//! **Where a sheet sits on a display: clear of the alo Bar.**
//!
//! The owner, 2026-11-17, on seeing the file chooser drawn over the Bar: *"Yes,
//! fix the overlap"*. A sheet the desktop asks the person something on is
//! centred in **the room above the Bar**, not on the whole display, so its
//! foot never covers the Bar at any display size.
//!
//! # The same rule Settings fits by
//!
//! The room is `crate::the_room_a_window_fills::the_work_area` with only the
//! Dock's band counted — the rule `crate::settings_fits_the_room` sizes
//! Settings by, and the one a window alo places opens clear of. **No figure is
//! written here**: the Bar's band is the Dock's as drawn on this display, at
//! its scale, and the room is whatever it leaves. A Dock that has given way
//! leaves the whole display, because there is nothing to cover.

use smithay::utils::{Physical, Rectangle};

use crate::the_room_a_window_fills::{WhatIsReserved, the_work_area};

/// A display, and the room on it a sheet is centred in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OnTheDisplay {
    /// The display's size, in its own pixels.
    pub(crate) size: (i32, i32),
    /// The room clear of the Bar.
    pub(crate) room: Rectangle<i32, Physical>,
}

impl OnTheDisplay {
    /// A display of `size` pixels with the Bar's band at `bar` as drawn on it,
    /// or [`None`] where no Bar is drawn.
    #[must_use]
    pub(crate) fn with_the_bar_at(size: (i32, i32), bar: Option<Rectangle<i32, Physical>>) -> Self {
        Self {
            size,
            room: the_room_a_sheet_sits_in(size, bar),
        }
    }
}

/// **The room a sheet is centred in** on a display of `size` pixels, with the
/// Bar's band at `bar` as drawn on it, or [`None`] where no Bar is drawn.
#[must_use]
pub(crate) fn the_room_a_sheet_sits_in(
    size: (i32, i32),
    bar: Option<Rectangle<i32, Physical>>,
) -> Rectangle<i32, Physical> {
    the_work_area(
        size,
        WhatIsReserved {
            dock: bar,
            panel: None,
            top: None,
            leaving: None,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The room ends where the Bar begins**, and with no Bar it is the whole
    /// display.
    #[test]
    fn the_room_ends_above_the_bar() {
        let bar = Rectangle::new((384, 636).into(), (512, 76).into());
        let room = the_room_a_sheet_sits_in((1280, 720), Some(bar));
        assert!(room.loc.y + room.size.h <= bar.loc.y, "{room:?}");
        assert_eq!(room.size.w, 1280);
        assert_eq!(
            the_room_a_sheet_sits_in((1280, 720), None),
            Rectangle::new((0, 0).into(), (1280, 720).into())
        );
    }
}
