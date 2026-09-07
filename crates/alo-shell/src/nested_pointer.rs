//! Parent pointer translation, independent of the graphics event loop.

use crate::{InputError, Server};
use smithay::{
    backend::input::{Axis, AxisSource, ButtonState, InputBackend, PointerAxisEvent},
    input::pointer::AxisFrame,
};

#[cfg(test)]
#[path = "nested_pointer_tests.rs"]
mod tests;

/// Trusted nested-backend input, never exposed through agent IPC.
pub enum NestedPointerEvent {
    /// Physical parent pixels match the nested output's logical scale one.
    Motion {
        /// Horizontal position in parent physical pixels.
        x: f64,
        /// Vertical position in parent physical pixels.
        y: f64,
        /// Monotonic backend timestamp in milliseconds.
        time: u32,
    },
    /// Linux evdev mouse button transition.
    Button {
        /// Linux BTN_LEFT through BTN_TASK.
        code: u32,
        /// Press or release.
        state: ButtonState,
        /// Monotonic backend timestamp in milliseconds.
        time: u32,
    },
    /// Scroll after backend unit conversion.
    Axis(AxisFrame),
}

impl Server {
    /// Route a parent event only while active; deactivation cancels any drag.
    ///
    /// Call with `None` on activation changes and close. Reactivation requires
    /// fresh motion before accepting buttons or scroll. Pointer capability must
    /// already be enabled. This is a trusted backend API, not input injection IPC.
    pub fn nested_pointer(
        &mut self,
        active: bool,
        event: Option<NestedPointerEvent>,
    ) -> Result<(), InputError> {
        if !active {
            return self.pointer_leave();
        }
        match event {
            Some(NestedPointerEvent::Motion { x, y, time }) => self.pointer_motion(x, y, time),
            Some(NestedPointerEvent::Button { code, state, time }) => {
                self.pointer_button(code, state, time).map(|_| ())
            }
            Some(NestedPointerEvent::Axis(frame)) => self.pointer_axis(frame).map(|_| ()),
            None => Ok(()),
        }
    }
}

/// Initial shell wheel policy: 120 units is one 15-pixel scroll step.
/// Pixel scrolling is already signed by the upstream backend; do not invert it.
pub(crate) fn axis<B: InputBackend>(
    event: &impl PointerAxisEvent<B>,
) -> Result<AxisFrame, InputError> {
    let mut frame = AxisFrame::new(event.time_msec()).source(event.source());
    for axis in [Axis::Horizontal, Axis::Vertical] {
        let v120 = event.amount_v120(axis);
        if v120.is_some_and(|v| !v.is_finite() || v < i32::MIN as f64 || v > i32::MAX as f64) {
            return Err(InputError::InvalidPointer);
        }
        let amount = event
            .amount(axis)
            .unwrap_or_else(|| v120.unwrap_or(0.0) / 8.0);
        if !amount.is_finite() || !(-8_388_608.0..8_388_608.0).contains(&amount) {
            return Err(InputError::InvalidPointer);
        }
        frame = frame
            .value(axis, amount)
            .relative_direction(axis, event.relative_direction(axis));
        if let Some(value) = v120 {
            frame = frame.v120(axis, value.round() as i32);
        }
        if amount == 0.0 && event.source() == AxisSource::Finger {
            frame = frame.stop(axis);
        }
    }
    Ok(frame)
}
