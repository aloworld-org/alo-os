//! Scale-one direct pointer translation, independent of device acquisition.

use crate::{InputError, Server};
use smithay::{backend::input::ButtonState, input::pointer::AxisFrame};

/// Trusted direct-seat events; never accepted from agent IPC.
pub enum DirectPointerEvent {
    /// Accelerated mouse delta in logical pixels.
    Relative {
        /// Horizontal delta.
        dx: f64,
        /// Vertical delta.
        dy: f64,
        /// Monotonic milliseconds.
        time: u32,
    },
    /// Absolute device coordinates normalized to the inclusive interval 0..=1.
    Absolute {
        /// Horizontal fraction of the output.
        x: f64,
        /// Vertical fraction of the output.
        y: f64,
        /// Monotonic milliseconds.
        time: u32,
    },
    /// Linux mouse button transition.
    Button {
        /// BTN_LEFT through BTN_TASK.
        code: u32,
        /// Press or release.
        state: ButtonState,
        /// Monotonic milliseconds.
        time: u32,
    },
    /// Scroll converted to compositor units by the backend.
    Axis(AxisFrame),
}

impl Server {
    /// Route a direct pointer against the current output's scale-one extent.
    ///
    /// Relative motion starts at the last accepted seat position (initially zero).
    /// Motion clamps to the last pixel origin, preserving fractional movement
    /// inside the output. Absolute endpoints map to first/last pixel origins.
    /// Malformed active input leaves position, focus and held buttons unchanged.
    /// Inactive calls ignore the event and extent and cancel focus/held buttons;
    /// call with `None` on pause, device removal and shutdown. Reactivation alone
    /// does not restore focus: a fresh motion is required before clicks/scroll.
    ///
    /// The trusted backend must check seat activity and supply the actual extent.
    /// This does not open input devices, drive libinput, or implement keyboard
    /// pause handling. Do not mix nested and direct drivers on one seat.
    pub fn direct_pointer(
        &mut self,
        active: bool,
        extent: (i32, i32),
        event: Option<DirectPointerEvent>,
    ) -> Result<(), InputError> {
        if !active {
            return self.pointer_leave();
        }
        let bounds = bounds(extent)?;
        let location = self
            .surfaces
            .pointer
            .as_ref()
            .ok_or(InputError::PointerUnavailable)?
            .location;
        match event {
            Some(DirectPointerEvent::Relative { dx, dy, time }) => {
                let (x, y) = relative((location.x, location.y), (dx, dy), bounds)?;
                self.pointer_motion(x, y, time)
            }
            Some(DirectPointerEvent::Absolute { x, y, time }) => {
                let (x, y) = absolute((x, y), bounds)?;
                self.pointer_motion(x, y, time)
            }
            Some(DirectPointerEvent::Button { code, state, time }) => {
                self.pointer_button(code, state, time).map(|_| ())
            }
            Some(DirectPointerEvent::Axis(frame)) => self.pointer_axis(frame).map(|_| ()),
            None => Ok(()),
        }
    }
}

/// Keep coordinates representable by the existing signed Wayland fixed boundary.
fn bounds((w, h): (i32, i32)) -> Result<(f64, f64), InputError> {
    if !(1..=8_388_608).contains(&w) || !(1..=8_388_608).contains(&h) {
        return Err(InputError::InvalidPointer);
    }
    Ok((f64::from(w - 1), f64::from(h - 1)))
}

/// Validate both axes before calculating a new seat position.
fn relative(
    (x, y): (f64, f64),
    (dx, dy): (f64, f64),
    (max_x, max_y): (f64, f64),
) -> Result<(f64, f64), InputError> {
    if !dx.is_finite() || !dy.is_finite() || !(x + dx).is_finite() || !(y + dy).is_finite() {
        return Err(InputError::InvalidPointer);
    }
    Ok(((x + dx).clamp(0.0, max_x), (y + dy).clamp(0.0, max_y)))
}

/// Refuse malformed device normalization instead of turning it into an edge hit.
fn absolute((x, y): (f64, f64), (max_x, max_y): (f64, f64)) -> Result<(f64, f64), InputError> {
    if !(0.0..=1.0).contains(&x) || !(0.0..=1.0).contains(&y) {
        return Err(InputError::InvalidPointer);
    }
    Ok((x * max_x, y * max_y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_pointer_coordinates_preserve_fractions_and_clamp_edges() -> Result<(), InputError> {
        let limits = bounds((101, 51))?;
        assert_eq!(absolute((0.25, 0.5), limits)?, (25.0, 25.0));
        assert_eq!(absolute((1.0, 0.0), limits)?, (100.0, 0.0));
        assert_eq!(relative((1.0, 2.0), (0.25, -0.5), limits)?, (1.25, 1.5));
        assert_eq!(relative((1.0, 2.0), (-10.0, 500.0), limits)?, (0.0, 50.0));
        assert_eq!(absolute((1.0, 1.0), bounds((1, 1))?)?, (0.0, 0.0));
        assert_eq!(bounds((8_388_608, 1))?, (8_388_607.0, 0.0));
        Ok(())
    }

    #[test]
    fn direct_pointer_coordinates_refuse_invalid_extents_and_nonfinite_motion() {
        for size in [(0, 1), (1, -1), (8_388_609, 1), (1, i32::MAX)] {
            assert!(bounds(size).is_err());
        }
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(relative((0.0, 0.0), (bad, 0.0), (10.0, 10.0)).is_err());
            assert!(relative((0.0, 0.0), (0.0, bad), (10.0, 10.0)).is_err());
            assert!(absolute((bad, 0.0), (10.0, 10.0)).is_err());
        }
        for bad in [-0.01, 1.01, f64::MAX] {
            assert!(absolute((0.0, bad), (10.0, 10.0)).is_err());
        }
        assert!(relative((f64::MAX, 0.0), (f64::MAX, 0.0), (10.0, 10.0)).is_err());
    }
}
