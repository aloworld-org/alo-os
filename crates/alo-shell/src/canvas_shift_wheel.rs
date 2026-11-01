//! **Shift and a mouse wheel pan the canvas sideways.**
//!
//! `docs/design/the-shortcuts-and-the-edges.md`, *Moving around the canvas*:
//! *Pan sideways with a mouse — Shift + wheel on empty canvas*. A mouse wheel
//! turns one way only, so without this a person with an ordinary mouse could
//! pan the canvas up and down and never left or right except by Space and a
//! drag. **Found missing on 2026-11-01** while reading every canvas shortcut
//! against the code, after a booted machine showed two others broken. It makes
//! an existing promise true rather than adding one.
//!
//! # A mouse wheel only
//!
//! The design's row says *with a mouse*, and that is the rule here: only a
//! scroll whose source is a wheel is turned. A touchpad already pans in both
//! directions with two fingers, and turning its gesture through ninety degrees
//! because Shift happened to be held would move the canvas a way the person's
//! fingers did not.
//!
//! # Only over empty canvas
//!
//! The caller asks this only where a scroll is about to pan the plane, so a
//! Shift+wheel over an application still reaches that application unchanged —
//! *scrolling inside an application scrolls that application*, and what Shift
//! means there is the application's business.

use smithay::{backend::input::AxisSource, input::pointer::AxisFrame};

impl crate::Server {
    /// Whether Shift is held right now, as the keyboard sees it.
    ///
    /// [`false`] with no keyboard, for the reason `Server::ctrl_is_held` gives:
    /// a machine with no keyboard is holding no modifier.
    pub(crate) fn shift_is_held(&self) -> bool {
        self.surfaces
            .keyboard
            .as_ref()
            .is_some_and(|keyboard| keyboard.handle.modifier_state().shift)
    }
}

/// This scroll as the canvas pans by it: a mouse wheel's turn made sideways when
/// Shift is held, and every other scroll unchanged.
///
/// The two axes are swapped rather than the vertical one moved across, so a
/// wheel that tilts as well as turns keeps both of what it sent.
#[must_use]
pub(crate) fn as_the_canvas_pans(frame: AxisFrame, shift: bool) -> AxisFrame {
    if !shift || frame.source != Some(AxisSource::Wheel) {
        return frame;
    }
    AxisFrame {
        axis: (frame.axis.1, frame.axis.0),
        v120: frame.v120.map(|(across, down)| (down, across)),
        relative_direction: (frame.relative_direction.1, frame.relative_direction.0),
        stop: (frame.stop.1, frame.stop.0),
        ..frame
    }
}

#[cfg(test)]
#[path = "canvas_shift_wheel_tests.rs"]
mod tests;
