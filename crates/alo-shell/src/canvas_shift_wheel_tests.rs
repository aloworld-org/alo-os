//! That Shift turns a mouse wheel's scroll sideways, and turns nothing else.

use smithay::{
    backend::input::{Axis, AxisSource},
    input::pointer::AxisFrame,
};

use super::as_the_canvas_pans;

/// One notch of a mouse wheel, downwards.
fn a_notch_down(source: AxisSource) -> AxisFrame {
    AxisFrame::new(7)
        .source(source)
        .value(Axis::Vertical, 15.0)
        .v120(Axis::Vertical, 120)
}

/// **With Shift held, a wheel's turn pans across instead of down.**
#[test]
fn shift_turns_a_wheel_sideways() {
    let turned = as_the_canvas_pans(a_notch_down(AxisSource::Wheel), true);
    assert_eq!(turned.axis, (15.0, 0.0));
    assert_eq!(turned.v120, Some((120, 0)));
    assert_eq!(turned.time, 7, "the moment of the scroll was lost");
}

/// **Without Shift, the wheel is left as it is.** The refusal of the turn is
/// as much the rule as the turn.
#[test]
fn without_shift_a_wheel_is_unchanged() {
    let kept = as_the_canvas_pans(a_notch_down(AxisSource::Wheel), false);
    assert_eq!(kept.axis, (0.0, 15.0));
    assert_eq!(kept.v120, Some((0, 120)));
}

/// **A touchpad is never turned**, Shift or not: two fingers already pan both
/// ways, and the design's row says *with a mouse*.
#[test]
fn a_touchpad_is_not_turned_by_shift() {
    let kept = as_the_canvas_pans(a_notch_down(AxisSource::Finger), true);
    assert_eq!(kept.axis, (0.0, 15.0));
    let kept = as_the_canvas_pans(a_notch_down(AxisSource::Continuous), true);
    assert_eq!(kept.axis, (0.0, 15.0));
}
