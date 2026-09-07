//! Real pointer wire delivery, hit testing, cancellation and refusal.
use super::support::{Application, Fixture};
use smithay::{
    backend::input::{
        Axis, AxisSource,
        ButtonState::{Pressed, Released},
    },
    input::pointer::AxisFrame,
};
use wayland_client::{
    Proxy,
    protocol::{wl_pointer, wl_seat::Capability},
};

fn fixture() -> Fixture {
    let fixture = Fixture::keyboard();
    assert!(fixture.backend(|s| s.enable_pointer()).is_ok());
    assert!(fixture.backend(|s| s.enable_pointer()).is_ok());
    fixture
}
fn mapped(f: &Fixture) -> Application {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    app
}
fn motion(f: &Fixture, x: f64, y: f64) {
    assert!(f.backend(move |s| s.pointer_motion(x, y, 123)).is_ok());
}
fn button(f: &Fixture, state: smithay::backend::input::ButtonState, accepted: bool) {
    assert_eq!(
        f.backend(move |s| s.pointer_button(0x110, state, 124)).ok(),
        Some(accepted)
    );
}

#[test]
fn pointer_hit_regions_subsurface_coordinates_and_drag_isolation() {
    let f = fixture();
    let mut first = mapped(&f);
    let mut second = mapped(&f);
    assert_eq!(
        first.events.keyboard.capabilities,
        Some(Capability::Keyboard | Capability::Pointer)
    );
    let (child, _role) = first.child((24, 32));
    first.surface.commit();
    first.sync();
    motion(&f, 26.0, 35.0);
    first.sync();
    assert_eq!(
        first.events.pointer.enters.last(),
        Some(&(child.id().protocol_id(), 2.0, 3.0))
    );
    motion(&f, 2.0, 3.0);
    first.sync();
    assert_eq!(
        first.events.pointer.enters.last(),
        Some(&(first.surface.id().protocol_id(), 2.0, 3.0))
    );
    button(&f, Pressed, true);
    button(&f, Pressed, false);
    first.empty_input();
    first.sync();
    motion(&f, 3.0, 4.0);
    first.sync();
    second.sync();
    assert_eq!(first.events.pointer.motion.last(), Some(&(3.0, 4.0)));
    assert!(second.events.pointer.enters.is_empty());
    button(&f, Released, true);
    button(&f, Released, false);
    motion(&f, 3.0, 4.0);
    first.sync();
    second.sync();
    assert_eq!(
        second.events.pointer.enters.last(),
        Some(&(second.surface.id().protocol_id(), 3.0, 4.0))
    );
    assert!(second.events.pointer.buttons.is_empty());
    assert_eq!(
        first.events.pointer.buttons,
        [
            (0x110, wl_pointer::ButtonState::Pressed),
            (0x110, wl_pointer::ButtonState::Released)
        ]
    );
}

#[test]
fn pointer_scroll_and_invalid_input_preserve_focus() {
    let f = fixture();
    let mut app = mapped(&f);
    button(&f, Pressed, false);
    motion(&f, 1.0, 2.0);
    for (x, y) in [(f64::NAN, 0.0), (0.0, f64::INFINITY), (8_388_608.0, 0.0)] {
        assert!(matches!(
            f.backend(move |s| s.pointer_motion(x, y, 123)),
            Err(alo_shell::InputError::InvalidPointer)
        ));
    }
    assert!(f.backend(|s| s.pointer_button(30, Pressed, 123)).is_err());
    assert!(
        f.backend(|s| s.pointer_axis(AxisFrame::new(123).value(Axis::Vertical, f64::NAN)))
            .is_err()
    );
    assert_eq!(
        f.backend(|s| s.pointer_axis(
            AxisFrame::new(123)
                .source(AxisSource::Finger)
                .value(Axis::Vertical, 5.0)
                .value(Axis::Horizontal, -2.0)
        ))
        .ok(),
        Some(true)
    );
    assert_eq!(
        f.backend(|s| s.pointer_axis(
            AxisFrame::new(124)
                .source(AxisSource::Finger)
                .stop(Axis::Vertical)
                .stop(Axis::Horizontal)
        ))
        .ok(),
        Some(true)
    );
    app.sync();
    assert_eq!(app.events.pointer.enters.len(), 1);
    assert_eq!(
        app.events.pointer.axes,
        [
            (wl_pointer::Axis::HorizontalScroll, -2.0),
            (wl_pointer::Axis::VerticalScroll, 5.0)
        ]
    );
    assert_eq!(app.events.pointer.stops, 2);
    assert!(app.events.pointer.frames >= 3);
    motion(&f, 16.0, 16.0);
    button(&f, Pressed, false);
    assert_eq!(
        f.backend(|s| s.pointer_axis(AxisFrame::new(125).value(Axis::Vertical, 1.0)))
            .ok(),
        Some(false)
    );
}

#[test]
fn pointer_leave_unmap_and_disconnect_cancel_buttons() {
    let f = fixture();
    let mut first = mapped(&f);
    let mut second = mapped(&f);
    motion(&f, 1.0, 1.0);
    button(&f, Pressed, true);
    assert!(f.backend(|s| s.pointer_leave()).is_ok());
    first.sync();
    assert_eq!(
        first.events.pointer.buttons.last(),
        Some(&(0x110, wl_pointer::ButtonState::Released))
    );
    assert_eq!(first.events.pointer.leaves, 1);
    button(&f, Released, false);
    button(&f, Pressed, false);
    let (other_child, other_role) = first.child((24, 32));
    first.surface.commit();
    first.sync();
    motion(&f, 26.0, 35.0);
    button(&f, Pressed, true);
    other_role.destroy();
    other_child.destroy();
    first.sync();
    button(&f, Released, false);
    motion(&f, 1.0, 1.0);
    button(&f, Pressed, true);
    first.surface.attach(None, 0, 0);
    first.surface.commit();
    first.sync();
    f.wait_for((2, 1));
    button(&f, Released, false);
    button(&f, Pressed, false);
    second.sync();
    assert!(second.events.pointer.enters.is_empty());
    motion(&f, 1.0, 1.0);
    button(&f, Pressed, true);
    drop(second);
    f.wait_for((1, 0));
    button(&f, Released, false);
    first.configure();
    first.attach();
    first.sync();
    motion(&f, 1.0, 1.0);
    button(&f, Pressed, true);
    button(&f, Released, true);
}

#[test]
fn pointer_child_unmap_cancels_grab_and_missing_seat_refuses() {
    let missing = Fixture::new();
    assert!(missing.backend(|s| s.enable_pointer()).is_err());
    assert!(missing.backend(|s| s.pointer_motion(1.0, 1.0, 0)).is_err());
    assert!(missing.backend(|s| s.pointer_leave()).is_err());
    let f = fixture();
    let mut app = mapped(&f);
    let (child, role) = app.child((24, 32));
    app.surface.commit();
    app.sync();
    motion(&f, 26.0, 35.0);
    button(&f, Pressed, true);
    child.attach(None, 0, 0);
    child.commit();
    app.surface.commit();
    app.sync();
    button(&f, Released, false);
    button(&f, Pressed, false);
    motion(&f, 1.0, 1.0);
    button(&f, Pressed, true);
    role.destroy();
    child.destroy();
    app.sync();
    button(&f, Released, true);
}
