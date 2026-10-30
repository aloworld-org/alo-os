//! The pointer the sign-in screen owns, before anybody is signed in.
//!
//! `crate::sign_in_seat` is this file's twin for the keyboard, and its header
//! gives the reason both exist: a key typed at the sign-in screen is
//! **intercepted at the seat and forwarded to nobody**, because no client has
//! any business hearing a password. A press is the same. So the position of
//! the pointer while the sign-in screen is up is kept **here**, in the
//! screen's own state, and never in `Surfaces::pointer`, which is the seat's
//! client-facing one.
//!
//! # Why not the seat's pointer
//!
//! `Server::pointer_motion` prunes client focus, offers the position to a
//! space-drag, a window move and a resize band, hit-tests the scene and then
//! tells whatever it found. Every one of those is about a session that has not
//! started. Routing a sign-in click through them would mean the screen a
//! machine boots to shares its input path with window management — and the
//! first bug in that path is somebody's password arriving at a surface.
//!
//! # No position until a device gives one
//!
//! [`ThePointerOnTheSignInScreen::where_it_is`] is [`None`] until a device has
//! moved, so **a machine with no mouse draws no arrow** rather than one parked
//! in a corner. The first relative delta is applied from the middle of the
//! screen rather than from (0, 0): a mouse reports where it went, never where
//! it was, and a cursor that appeared in the top-left corner on the first
//! nudge is a cursor a person has to find.
//!
//! An absolute device — a touchscreen, a tablet, a virtual machine's tablet —
//! says where it is outright, and that position is taken as given.

use smithay::utils::{Logical, Point};

use crate::display_scale::DisplayScale;
use crate::sign_in_layout::SignInLayout;
use crate::sign_in_pointing::{OnTheSignInScreen, what_is_under_the_pointer};
use crate::{DirectPointerEvent, InputError};

/// Where the pointer is on the sign-in screen, and what a press there means.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct ThePointerOnTheSignInScreen {
    /// The position, or [`None`] until a device has moved it.
    at: Option<Point<f64, Logical>>,
}

impl ThePointerOnTheSignInScreen {
    /// Take one pointer event, and answer whether it moved the pointer.
    ///
    /// A button, a scroll and a pinch move nothing and are not errors: a
    /// trackpad at a sign-in screen produces all three and none of them is a
    /// fault. Scroll and pinch do nothing to the pointer, deliberately: the
    /// sign-in screen has nothing to scroll and nothing to zoom, and on a
    /// first-start sheet the wheel over a list is the lane's to read
    /// (`crate::a_wheel_over_a_sheet`), never a move of the arrow.
    ///
    /// # Errors
    /// [`InputError::InvalidPointer`] for a screen with no area, a
    /// non-finite delta and an absolute coordinate outside `0..=1` — the same
    /// three refusals [`crate::Server::direct_pointer`] makes, through the same
    /// three functions, so a malformed device is refused identically whether or
    /// not anybody has signed in yet.
    pub(crate) fn moved(
        &mut self,
        event: &DirectPointerEvent,
        extent: (i32, i32),
    ) -> Result<bool, InputError> {
        let limits = crate::direct_pointer::bounds(extent)?;
        match *event {
            DirectPointerEvent::Relative { dx, dy, .. } => {
                // The middle, for a first delta with nothing to add it to.
                let from = self
                    .at
                    .unwrap_or_else(|| Point::from((limits.0 / 2.0, limits.1 / 2.0)));
                let (x, y) = crate::direct_pointer::relative((from.x, from.y), (dx, dy), limits)?;
                self.at = Some(Point::from((x, y)));
                Ok(true)
            }
            DirectPointerEvent::Absolute { x, y, .. } => {
                let (x, y) = crate::direct_pointer::absolute((x, y), limits)?;
                self.at = Some(Point::from((x, y)));
                Ok(true)
            }
            DirectPointerEvent::Button { .. }
            | DirectPointerEvent::Axis(_)
            | DirectPointerEvent::Pinch(_) => Ok(false),
        }
    }

    /// Where the arrow is drawn, or [`None`] for a pointer no device has moved.
    #[must_use]
    pub(crate) fn where_it_is(self) -> Option<Point<f64, Logical>> {
        self.at
    }

    /// **The output pixel the pointer is in**, or [`None`] before a device has
    /// moved it — for a sheet, whose controls are kept in the pixels they were
    /// drawn in, so nothing is converted.
    #[must_use]
    pub(crate) fn in_pixels(self) -> Option<(i32, i32)> {
        let point = self.at?;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a pointer inside a pixel belongs to that pixel, and the position is \
                      already held within the output's bounds"
        )]
        let pixel = (point.x as i32, point.y as i32);
        Some(pixel)
    }

    /// What a press at this position is on.
    ///
    /// [`None`] both for a pointer that has never moved and for a press on the
    /// screen itself. **A pointer with no position is not a pointer at the
    /// origin** — the rule `crate::a_press_in_the_menu` already holds, and
    /// here it is the difference between a stray button event doing nothing
    /// and one landing on *Back to lock* at (32, 28).
    ///
    /// **The pointer is kept in the output's pixels and `at` is logical**, so
    /// the pixel it is in is taken through `display`, the scale the frame was
    /// drawn at. At a whole scale every pixel of a drawn control lands inside
    /// the logical control it was drawn from; at one this is the identity.
    #[must_use]
    pub(crate) fn pressed(
        self,
        at: &SignInLayout,
        display: DisplayScale,
    ) -> Option<OnTheSignInScreen> {
        let point = self.at?;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a pointer inside a pixel belongs to that pixel: the half-open rule the \
                      rectangles use, and `crate::the_pointer_in_pixels` gives the reason"
        )]
        let (x, y) = (point.x as i32, point.y as i32);
        let logical = |pixels: i32| i32::try_from(display.logical(pixels)).unwrap_or(i32::MAX);
        what_is_under_the_pointer(at, Point::from((logical(x), logical(y))))
    }
}

#[cfg(test)]
#[path = "sign_in_pointer_tests.rs"]
mod tests;
