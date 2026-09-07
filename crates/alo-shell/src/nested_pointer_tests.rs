//! Backend scroll translation with the same trait boundary as Winit.
use super::*;
use smithay::backend::{
    input::{AxisRelativeDirection, Event},
    winit::{WinitInput, WinitVirtualDevice},
};

struct Scroll {
    source: AxisSource,
    pixels: Option<f64>,
    wheel: Option<f64>,
}
impl Event<WinitInput> for Scroll {
    fn time(&self) -> u64 {
        42000
    }
    fn device(&self) -> WinitVirtualDevice {
        WinitVirtualDevice
    }
}
impl PointerAxisEvent<WinitInput> for Scroll {
    fn source(&self) -> AxisSource {
        self.source
    }
    fn amount(&self, _: Axis) -> Option<f64> {
        self.pixels
    }
    fn amount_v120(&self, _: Axis) -> Option<f64> {
        self.wheel
    }
    fn relative_direction(&self, _: Axis) -> AxisRelativeDirection {
        AxisRelativeDirection::Identical
    }
}

#[test]
fn wheel_and_pixels_keep_backend_sign_and_units() -> Result<(), InputError> {
    let frame = axis(&Scroll {
        source: AxisSource::Wheel,
        pixels: None,
        wheel: Some(-240.0),
    })?;
    assert_eq!(frame.time, 42);
    assert_eq!(frame.axis, (-30.0, -30.0));
    assert_eq!(frame.v120, Some((-240, -240)));
    let frame = axis(&Scroll {
        source: AxisSource::Continuous,
        pixels: Some(2.5),
        wheel: None,
    })?;
    assert_eq!(frame.axis, (2.5, 2.5));
    assert_eq!(frame.v120, None);
    Ok(())
}

#[test]
fn scroll_refuses_nonfinite_and_out_of_range_before_casting() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, f64::MAX] {
        assert!(
            axis(&Scroll {
                source: AxisSource::Wheel,
                pixels: None,
                wheel: Some(value)
            })
            .is_err()
        );
        assert!(
            axis(&Scroll {
                source: AxisSource::Continuous,
                pixels: Some(value),
                wheel: None
            })
            .is_err()
        );
    }
}
