//! Borrow modern libinput scrolling; never process duplicate legacy axis events.

use crate::InputError;
use smithay::{
    backend::input::{Axis, AxisRelativeDirection, AxisSource},
    input::pointer::AxisFrame,
    reexports::input::event::{
        EventTrait, PointerEvent,
        pointer::{PointerEventTrait, PointerScrollEvent},
    },
};

/// Extract modern wheel/finger/continuous axes without retaining their devices.
pub(crate) fn scroll(event: &PointerEvent) -> Result<Option<AxisFrame>, InputError> {
    let frame = match event {
        PointerEvent::ScrollWheel(event) => frame(event, AxisSource::Wheel, |axis| {
            Some(event.scroll_value_v120(axis))
        })?,
        PointerEvent::ScrollFinger(event) => frame(event, AxisSource::Finger, |_| None)?,
        PointerEvent::ScrollContinuous(event) => frame(event, AxisSource::Continuous, |_| None)?,
        _ => return Ok(None),
    };
    Ok(Some(frame))
}

/// Preserve upstream signed values; natural scrolling is metadata, not inversion.
fn frame(
    event: &(impl PointerScrollEvent + PointerEventTrait + EventTrait),
    source: AxisSource,
    v120: impl Fn(smithay::reexports::input::event::pointer::Axis) -> Option<f64>,
) -> Result<AxisFrame, InputError> {
    let direction = if event.device().config_scroll_natural_scroll_enabled() {
        AxisRelativeDirection::Inverted
    } else {
        AxisRelativeDirection::Identical
    };
    let mut frame = AxisFrame::new(event.time()).source(source);
    for axis in [Axis::Horizontal, Axis::Vertical] {
        if event.has_axis(axis.into()) {
            frame = add_axis(
                frame,
                source,
                axis,
                event.scroll_value(axis.into()),
                v120(axis.into()),
                direction,
            )?;
        }
    }
    Ok(frame)
}

/// Wheel v120 defines the same 15-pixel step as nested input, independent of angle.
fn add_axis(
    frame: AxisFrame,
    source: AxisSource,
    axis: Axis,
    amount: f64,
    v120: Option<f64>,
    direction: AxisRelativeDirection,
) -> Result<AxisFrame, InputError> {
    if !amount.is_finite()
        || v120.is_some_and(|v| !v.is_finite() || v < i32::MIN as f64 || v > i32::MAX as f64)
    {
        return Err(InputError::InvalidPointer);
    }
    let amount = v120.map_or(amount, |v| v / 8.0);
    if !(-8_388_608.0..8_388_608.0).contains(&amount) {
        return Err(InputError::InvalidPointer);
    }
    let mut frame = frame
        .value(axis, amount)
        .relative_direction(axis, direction);
    if let Some(value) = v120 {
        frame = frame.v120(axis, value.round() as i32);
    }
    if amount == 0.0 && matches!(source, AxisSource::Finger | AxisSource::Continuous) {
        frame = frame.stop(axis);
    }
    Ok(frame)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn libinput_scroll_wheel_uses_v120_and_preserves_natural_direction() -> Result<(), InputError> {
        let frame = add_axis(
            AxisFrame::new(7),
            AxisSource::Wheel,
            Axis::Vertical,
            -20.0,
            Some(-60.0),
            AxisRelativeDirection::Inverted,
        )?;
        assert_eq!(frame.axis, (0.0, -7.5));
        assert_eq!(frame.v120, Some((0, -60)));
        assert_eq!(frame.relative_direction.1, AxisRelativeDirection::Inverted);
        assert_eq!(frame.time, 7);
        Ok(())
    }

    #[test]
    fn libinput_scroll_finger_and_continuous_stop_only_the_reported_axis() -> Result<(), InputError>
    {
        for source in [AxisSource::Finger, AxisSource::Continuous] {
            let frame = add_axis(
                AxisFrame::new(1),
                source,
                Axis::Horizontal,
                2.5,
                None,
                AxisRelativeDirection::Identical,
            )?;
            assert_eq!(frame.axis, (2.5, 0.0));
            assert_eq!(frame.stop, (false, false));
            let frame = add_axis(
                frame,
                source,
                Axis::Horizontal,
                0.0,
                None,
                AxisRelativeDirection::Identical,
            )?;
            assert_eq!(frame.stop, (true, false));
        }
        Ok(())
    }

    #[test]
    fn libinput_scroll_refuses_nonfinite_and_unrepresentable_values() {
        for value in [
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::MAX,
            8_388_608.0,
        ] {
            assert!(
                add_axis(
                    AxisFrame::new(0),
                    AxisSource::Finger,
                    Axis::Vertical,
                    value,
                    None,
                    AxisRelativeDirection::Identical
                )
                .is_err()
            );
        }
        for value in [
            f64::NAN,
            f64::INFINITY,
            i32::MAX as f64 + 1.0,
            i32::MIN as f64 - 1.0,
        ] {
            assert!(
                add_axis(
                    AxisFrame::new(0),
                    AxisSource::Wheel,
                    Axis::Vertical,
                    1.0,
                    Some(value),
                    AxisRelativeDirection::Identical
                )
                .is_err()
            );
        }
    }
}
