//! Real-client checks of the adapter installed in the Winit seat pump.
use super::{mapped, routing::CLOSE};
use crate::Fixture;
use alo_shell::{NestedControlInput, NestedPointerEvent as Event, PaintedWindowControls};
use smithay::{
    backend::input::{Axis, ButtonState, KeyState},
    input::pointer::AxisFrame,
};
use std::sync::{Arc, Mutex};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

struct Input(Arc<Mutex<NestedControlInput>>);
impl Input {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(NestedControlInput::default())))
    }
    fn route(&self, f: &Fixture, active: bool, event: Option<Event>) -> Result {
        let input = self.0.clone();
        Ok(f.backend(move |s| {
            input
                .lock()
                .map_err(|_| "poisoned input".to_owned())
                .and_then(|mut input| {
                    input
                        .route(s, active, event)
                        .map_err(|error| error.to_string())
                })
        })?)
    }
    fn motion(&self, f: &Fixture, position: (f64, f64)) -> Result {
        self.route(
            f,
            true,
            Some(Event::Motion {
                x: position.0,
                y: position.1,
                time: 1,
            }),
        )
    }
    fn button(&self, f: &Fixture, code: u32, state: ButtonState) -> Result {
        self.route(
            f,
            true,
            Some(Event::Button {
                code,
                state,
                time: 2,
            }),
        )
    }
}

fn publish(f: &Fixture) -> Result {
    let root = f.root();
    Ok(f.backend(move |s| {
        s.present_window_controls(Some(PaintedWindowControls {
            surface: &root,
            viewport: (120, 48),
            origin: (3, 4),
        }))
    })?)
}

#[test]
fn window_controls_nested_input_executes_once_and_keeps_typing_buttons_scroll() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    f.focus_surface(f.root())?;
    let input = Input::new();
    input.motion(&f, (2.0, 2.0))?;
    for state in [ButtonState::Pressed, ButtonState::Released] {
        input.button(&f, 0x110, state)?;
    }
    input.route(
        &f,
        true,
        Some(Event::Axis(AxisFrame::new(3).value(Axis::Vertical, 7.0))),
    )?;
    publish(&f)?;
    input.motion(&f, CLOSE)?;
    input.button(&f, 0x110, ButtonState::Pressed)?;
    input.button(&f, 0x110, ButtonState::Pressed)?;
    assert!(f.key(30, KeyState::Pressed)?);
    assert!(f.key(30, KeyState::Released)?);
    input.button(&f, 0x110, ButtonState::Released)?;
    input.button(&f, 0x110, ButtonState::Released)?;
    input.motion(&f, (2.0, 2.0))?;
    for state in [ButtonState::Pressed, ButtonState::Released] {
        input.button(&f, 0x111, state)?;
    }
    app.sync();
    assert_eq!(app.events.close_requests, 1);
    assert_eq!(app.events.pointer.buttons.len(), 4);
    assert_eq!(app.events.pointer.axes.len(), 1);
    assert_eq!(
        app.events.pointer.axes.first().ok_or("missing scroll")?.1,
        7.0
    );
    assert_eq!(app.events.keyboard.keys.len(), 2);
    Ok(())
}

#[test]
fn window_controls_nested_input_tracks_owned_motion_and_cancels_out_and_back() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let input = Input::new();
    publish(&f)?;
    input.motion(&f, CLOSE)?;
    app.sync();
    let motions = app.events.pointer.motion.len();
    input.button(&f, 0x110, ButtonState::Pressed)?;
    input.motion(&f, (2.0, 2.0))?;
    assert_eq!(
        input.0.lock().map_err(|_| "poisoned input")?.position(),
        Some((2.0, 2.0))
    );
    input.motion(&f, CLOSE)?;
    input.button(&f, 0x110, ButtonState::Released)?;
    app.sync();
    assert_eq!(app.events.close_requests, 0);
    assert!(app.events.pointer.buttons.is_empty());
    assert_eq!(app.events.pointer.motion.len(), motions);
    Ok(())
}

#[test]
fn window_controls_nested_input_deactivation_drains_release_and_requires_motion() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let input = Input::new();
    for release_active in [false, true] {
        publish(&f)?;
        input.motion(&f, CLOSE)?;
        input.button(&f, 0x110, ButtonState::Pressed)?;
        input.route(&f, false, None)?;
        assert_eq!(
            input.0.lock().map_err(|_| "poisoned input")?.position(),
            None
        );
        assert!(f.backend(|s| s.presented_window_controls(None)).is_none());
        input.route(
            &f,
            release_active,
            Some(Event::Button {
                code: 0x110,
                state: ButtonState::Released,
                time: 3,
            }),
        )?;
        publish(&f)?;
        for state in [ButtonState::Pressed, ButtonState::Released] {
            input.button(&f, 0x110, state)?;
        }
        app.sync();
        assert_eq!(app.events.close_requests, 0);
        assert!(app.events.pointer.buttons.is_empty());
    }
    // The cancelled release was drained; a genuinely new gesture works.
    input.motion(&f, CLOSE)?;
    input.button(&f, 0x110, ButtonState::Pressed)?;
    input.button(&f, 0x110, ButtonState::Released)?;
    app.sync();
    assert_eq!(app.events.close_requests, 1);
    Ok(())
}

#[test]
fn window_controls_nested_input_invalid_motion_retires_and_never_rearms() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let input = Input::new();
    for value in [f64::NAN, f64::INFINITY, 8_388_608.0] {
        publish(&f)?;
        input.motion(&f, CLOSE)?;
        input.button(&f, 0x110, ButtonState::Pressed)?;
        assert!(input.motion(&f, (value, 5.0)).is_err());
        assert_eq!(
            input.0.lock().map_err(|_| "poisoned input")?.position(),
            None
        );
        assert!(f.backend(|s| s.presented_window_controls(None)).is_none());
        input.button(&f, 0x110, ButtonState::Released)?;
        publish(&f)?;
        input.button(&f, 0x110, ButtonState::Pressed)?;
        input.button(&f, 0x110, ButtonState::Released)?;
    }
    app.sync();
    assert_eq!(app.events.close_requests, 0);
    assert!(app.events.pointer.buttons.is_empty());
    Ok(())
}
