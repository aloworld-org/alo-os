//! Extracted libinput data reaches real Wayland clients; no physical device claim.
use super::support::{Application, Fixture};
use alo_shell::{DirectKeyEvent, DirectPointerEvent, DirectSeatEvent, InputError, InputUpdate};
use smithay::backend::input::{ButtonState, KeyState};
use wayland_client::protocol::{wl_keyboard, wl_pointer};

fn key(state: KeyState, count: u32) -> DirectSeatEvent {
    DirectSeatEvent::Key(
        DirectKeyEvent {
            code: 42,
            state,
            time: 17,
        },
        count,
    )
}
fn button(state: ButtonState, count: u32) -> DirectSeatEvent {
    DirectSeatEvent::Button {
        code: 0x110,
        state,
        time: 18,
        count,
    }
}
fn send(f: &Fixture, event: DirectSeatEvent) -> Result<(), InputError> {
    f.backend(move |s| s.direct_seat(true, (32, 32), event))
}
fn mapped(f: &Fixture) -> Application {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    app
}
fn motion(f: &Fixture) -> Result<(), InputError> {
    send(
        f,
        DirectSeatEvent::Pointer(DirectPointerEvent::Absolute {
            x: 0.25,
            y: 0.5,
            time: 16,
        }),
    )
}

#[test]
fn libinput_seat_two_devices_deliver_first_press_and_last_release() -> Result<(), InputError> {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    f.focus(Some(0))?;
    motion(&f)?;
    for count in [1, 2] {
        send(&f, key(KeyState::Pressed, count))?;
        send(&f, button(ButtonState::Pressed, count))?;
    }
    send(&f, key(KeyState::Released, 1))?;
    send(&f, button(ButtonState::Released, 1))?;
    app.sync();
    assert_eq!(
        app.events.keyboard.keys,
        [(42, wl_keyboard::KeyState::Pressed)]
    );
    assert_ne!(app.events.keyboard.modifiers.last(), Some(&0));
    assert_eq!(
        app.events.pointer.buttons,
        [(0x110, wl_pointer::ButtonState::Pressed)]
    );
    send(&f, key(KeyState::Released, 0))?;
    send(&f, button(ButtonState::Released, 0))?;
    // Unmatched releases cannot acquire fresh serials.
    send(&f, key(KeyState::Released, 0))?;
    send(&f, button(ButtonState::Released, 0))?;
    app.sync();
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert_eq!(
        app.events.keyboard.keys.get(1),
        Some(&(42, wl_keyboard::KeyState::Released))
    );
    assert_eq!(app.events.keyboard.modifiers.last(), Some(&0));
    assert_eq!(app.events.pointer.buttons.len(), 2);
    assert_eq!(
        app.events.pointer.buttons.get(1),
        Some(&(0x110, wl_pointer::ButtonState::Released))
    );
    assert!(
        app.events
            .pointer
            .enters
            .iter()
            .any(|&(_, x, y)| (x, y) == (7.75, 15.5))
    );
    Ok(())
}

#[test]
fn libinput_seat_removal_cancels_held_input_and_requires_fresh_focus() -> Result<(), InputError> {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    f.focus(Some(0))?;
    motion(&f)?;
    send(&f, key(KeyState::Pressed, 1))?;
    send(&f, button(ButtonState::Pressed, 1))?;
    send(&f, DirectSeatEvent::Removed)?;
    send(&f, DirectSeatEvent::Removed)?;
    // Another device's held key/button cannot restore focus after removal.
    send(&f, key(KeyState::Pressed, 2))?;
    send(&f, button(ButtonState::Pressed, 2))?;
    send(&f, key(KeyState::Released, 0))?;
    send(&f, button(ButtonState::Released, 0))?;
    app.sync();
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert_eq!(app.events.pointer.buttons.len(), 2);
    assert_eq!(app.events.keyboard.leaves, 1);
    assert_eq!(app.events.pointer.leaves, 1);
    assert_eq!(app.events.keyboard.modifiers.last(), Some(&0));
    f.focus(Some(0))?;
    motion(&f)?;
    send(&f, key(KeyState::Pressed, 1))?;
    send(&f, button(ButtonState::Pressed, 1))?;
    app.sync();
    assert_eq!(app.events.keyboard.keys.len(), 3);
    assert_eq!(app.events.pointer.buttons.len(), 3);
    Ok(())
}

#[test]
fn libinput_seat_malformed_events_preserve_state_and_inactivity_wins() -> Result<(), InputError> {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    f.focus(Some(0))?;
    motion(&f)?;
    send(&f, key(KeyState::Pressed, 1))?;
    send(&f, button(ButtonState::Pressed, 1))?;
    assert!(matches!(
        send(&f, key(KeyState::Pressed, 0)),
        Err(InputError::InvalidKey)
    ));
    assert!(matches!(
        send(&f, button(ButtonState::Pressed, 0)),
        Err(InputError::InvalidPointer)
    ));
    assert!(
        send(
            &f,
            DirectSeatEvent::Pointer(DirectPointerEvent::Relative {
                dx: f64::NAN,
                dy: 0.0,
                time: 0
            })
        )
        .is_err()
    );
    assert!(
        f.backend(|s| s.direct_seat(true, (0, 0), button(ButtonState::Released, 0)))
            .is_err()
    );
    app.sync();
    assert_eq!(app.events.keyboard.keys.len(), 1);
    assert_eq!(app.events.pointer.buttons.len(), 1);
    f.backend(|s| s.direct_seat(false, (0, 0), key(KeyState::Pressed, 0)))?;
    f.backend(|s| s.libinput_update(InputUpdate::Reset, (0, 0)))?;
    app.sync();
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert_eq!(app.events.pointer.buttons.len(), 2);
    let absent = Fixture::new();
    assert!(matches!(
        send(&absent, key(KeyState::Pressed, 1)),
        Err(InputError::Unavailable)
    ));
    absent.backend(|s| s.libinput_update(InputUpdate::Reset, (0, 0)))?;
    Ok(())
}

#[test]
fn libinput_seat_relative_motion_and_scroll_stop_reach_client() -> Result<(), InputError> {
    use smithay::{
        backend::input::{Axis, AxisSource},
        input::pointer::AxisFrame,
    };
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    motion(&f)?;
    send(
        &f,
        DirectSeatEvent::Pointer(DirectPointerEvent::Relative {
            dx: 0.25,
            dy: -0.5,
            time: 18,
        }),
    )?;
    send(
        &f,
        DirectSeatEvent::Pointer(DirectPointerEvent::Axis(
            AxisFrame::new(19)
                .source(AxisSource::Finger)
                .value(Axis::Vertical, -2.5),
        )),
    )?;
    send(
        &f,
        DirectSeatEvent::Pointer(DirectPointerEvent::Axis(
            AxisFrame::new(20)
                .source(AxisSource::Finger)
                .stop(Axis::Vertical),
        )),
    )?;
    app.sync();
    assert!(app.events.pointer.motion.contains(&(8.0, 15.0)));
    assert!(
        app.events
            .pointer
            .axes
            .contains(&(wl_pointer::Axis::VerticalScroll, -2.5))
    );
    assert_eq!(app.events.pointer.stops, 1);
    drop(app);
    f.wait_for((0, 0));
    send(&f, DirectSeatEvent::Removed)?;
    let mut replacement = mapped(&f);
    send(&f, button(ButtonState::Released, 0))?;
    replacement.sync();
    assert!(replacement.events.pointer.buttons.is_empty());
    Ok(())
}

#[test]
fn libinput_seat_removal_dismisses_popup_and_invalidates_key_serial() -> Result<(), InputError> {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_popup_protocol());
    let mut app = mapped(&f);
    f.focus(Some(0))?;
    send(&f, key(KeyState::Pressed, 1))?;
    app.sync();
    let serial = app.events.keyboard.key_serial;
    let (surface, xdg, role) = app.popup(true, 1);
    app.grab_popup(&role, serial);
    surface.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&surface);
    app.sync();
    assert_eq!(app.events.popups.done, 0);
    send(&f, DirectSeatEvent::Removed)?;
    app.sync();
    assert_eq!(app.events.popups.done, 1);
    f.focus(Some(0))?;
    send(&f, key(KeyState::Pressed, 1))?;
    app.sync();
    let unused_serial = app.events.keyboard.key_serial;
    send(&f, DirectSeatEvent::Removed)?;
    f.focus(Some(0))?;
    let (surface, _, role) = app.popup(true, 1);
    app.grab_popup(&role, unused_serial);
    surface.commit();
    app.sync();
    assert_eq!(app.events.popups.done, 2);
    Ok(())
}

#[test]
fn libinput_seat_real_empty_context_pause_flushes_reset_to_client()
-> Result<(), Box<dyn std::error::Error>> {
    use smithay::backend::session::Session;
    use std::{io, os::fd::OwnedFd, path::Path};
    struct EmptySeat;
    impl Session for EmptySeat {
        type Error = ();
        fn open(&mut self, _: &Path, _: rustix::fs::OFlags) -> Result<OwnedFd, ()> {
            Err(())
        }
        fn close(&mut self, _: OwnedFd) -> Result<(), ()> {
            Ok(())
        }
        fn change_vt(&mut self, _: i32) -> Result<(), ()> {
            Err(())
        }
        fn is_active(&self) -> bool {
            true
        }
        fn seat(&self) -> String {
            format!("alo-routing-empty-{}", std::process::id())
        }
    }
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    f.focus(Some(0))?;
    send(&f, key(KeyState::Pressed, 1))?;
    f.backend(|s| -> Result<(), alo_shell::InputDispatchError> {
        let mut input = alo_shell::SeatInput::new(EmptySeat)?;
        input.dispatch(
            || Ok(()),
            |update| {
                s.libinput_update(update, (32, 32))
                    .map_err(io::Error::other)
            },
        )?;
        let result = input.dispatch(
            || Err(io::Error::other("seat paused")),
            |update| s.libinput_update(update, (0, 0)).map_err(io::Error::other),
        );
        let Err(error) = result else {
            return Err(alo_shell::InputDispatchError {
                source: io::Error::other("pause unexpectedly succeeded"),
                cleanup: None,
            });
        };
        assert_eq!(error.source.to_string(), "seat paused");
        assert!(error.cleanup.is_none());
        Ok(())
    })?;
    app.sync();
    assert_eq!(
        app.events.keyboard.keys,
        [
            (42, wl_keyboard::KeyState::Pressed),
            (42, wl_keyboard::KeyState::Released)
        ]
    );
    assert_eq!(app.events.keyboard.leaves, 1);
    assert_eq!(app.events.keyboard.modifiers.last(), Some(&0));
    Ok(())
}
