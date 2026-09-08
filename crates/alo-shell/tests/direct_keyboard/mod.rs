//! Direct seat transitions verified on real Wayland client connections.
use super::support::{Application, Fixture};
use alo_shell::{DirectKeyEvent, InputError};
use smithay::backend::input::{ButtonState, KeyState};
use wayland_client::protocol::{wl_keyboard, wl_pointer};

fn key(f: &Fixture, active: bool, code: u32, state: KeyState) -> Result<bool, InputError> {
    f.backend(move |s| {
        s.direct_keyboard(
            active,
            Some(DirectKeyEvent {
                code,
                state,
                time: 42,
            }),
        )
    })
}

fn mapped(f: &Fixture) -> Application {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    app
}

#[test]
fn direct_keyboard_pause_releases_modifiers_and_requires_explicit_focus() -> Result<(), InputError>
{
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    f.focus(Some(0))?;
    assert!(key(&f, true, 42, KeyState::Pressed)?);
    assert!(key(&f, true, 30, KeyState::Pressed)?);
    assert!(!key(&f, true, 30, KeyState::Pressed)?);
    assert!(matches!(
        key(&f, true, 0, KeyState::Released),
        Err(InputError::InvalidKey)
    ));
    app.sync();
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert_ne!(app.events.keyboard.modifiers.last(), Some(&0));
    // Inactivity wins over malformed queued input, including its timestamp.
    assert!(!key(&f, false, u32::MAX, KeyState::Pressed)?);
    assert!(!f.backend(|s| s.direct_keyboard(false, None))?);
    app.sync();
    assert_eq!(app.events.keyboard.leaves, 1);
    assert_eq!(app.events.keyboard.keys.len(), 4);
    for code in [30, 42] {
        assert!(
            app.events
                .keyboard
                .keys
                .contains(&(code, wl_keyboard::KeyState::Released))
        );
    }
    assert_eq!(app.events.keyboard.modifiers.last(), Some(&0));
    assert!(!f.backend(|s| s.direct_keyboard(true, None))?);
    assert!(!key(&f, true, 30, KeyState::Pressed)?);
    f.focus(Some(0))?;
    assert!(!key(&f, true, 30, KeyState::Released)?);
    assert!(key(&f, true, 30, KeyState::Pressed)?);
    assert!(key(&f, true, 30, KeyState::Released)?);
    app.sync();
    assert_eq!(app.events.keyboard.enters, [vec![], vec![]]);
    assert_eq!(app.events.keyboard.keys.len(), 6);
    Ok(())
}

#[test]
fn direct_keyboard_clear_input_retires_pointer_and_keyboard_together() -> Result<(), InputError> {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    f.focus(Some(0))?;
    assert!(key(&f, true, 42, KeyState::Pressed)?);
    f.backend(|s| s.pointer_motion(4.0, 4.0, 1))?;
    assert!(f.backend(|s| s.pointer_button(0x110, ButtonState::Pressed, 2))?);
    f.backend(|s| {
        s.clear_input();
        s.clear_input();
    });
    app.sync();
    assert_eq!(
        app.events.keyboard.keys,
        [
            (42, wl_keyboard::KeyState::Pressed),
            (42, wl_keyboard::KeyState::Released)
        ]
    );
    assert_eq!(app.events.keyboard.leaves, 1);
    assert_eq!(
        app.events.pointer.buttons,
        [
            (0x110, wl_pointer::ButtonState::Pressed),
            (0x110, wl_pointer::ButtonState::Released)
        ]
    );
    assert_eq!(app.events.pointer.leaves, 1);
    assert!(!f.backend(|s| s.pointer_button(0x110, ButtonState::Pressed, 3))?);
    assert!(!key(&f, true, 42, KeyState::Released)?);
    f.backend(|s| s.pointer_motion(4.0, 4.0, 4))?;
    assert!(f.backend(|s| s.pointer_button(0x110, ButtonState::Pressed, 5))?);
    Ok(())
}

#[test]
fn direct_keyboard_absent_capability_refuses_and_disconnect_does_not_transfer_keys()
-> Result<(), InputError> {
    let absent = Fixture::new();
    for active in [false, true] {
        assert!(matches!(
            absent.backend(move |s| s.direct_keyboard(active, None)),
            Err(InputError::Unavailable)
        ));
    }
    absent.backend(|s| s.clear_input());
    let f = Fixture::keyboard();
    let app = mapped(&f);
    f.focus(Some(0))?;
    assert!(key(&f, true, 42, KeyState::Pressed)?);
    drop(app);
    f.wait_for((0, 0));
    f.backend(|s| s.clear_input());
    let mut replacement = mapped(&f);
    assert!(!key(&f, true, 42, KeyState::Released)?);
    f.focus(Some(0))?;
    replacement.sync();
    assert_eq!(replacement.events.keyboard.enters, [vec![]]);
    assert_eq!(replacement.events.keyboard.modifiers.last(), Some(&0));
    assert!(replacement.events.keyboard.keys.is_empty());
    Ok(())
}

#[test]
fn direct_keyboard_pause_dismisses_popup_and_invalidates_initiating_serial()
-> Result<(), InputError> {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_popup_protocol());
    let mut app = mapped(&f);
    f.focus(Some(0))?;
    assert!(key(&f, true, 30, KeyState::Pressed)?);
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
    assert!(key(&f, true, 42, KeyState::Pressed)?);
    assert!(!f.backend(|s| s.direct_keyboard(false, None))?);
    app.sync();
    assert_eq!(app.events.popups.done, 1);
    assert!(
        app.events
            .keyboard
            .keys
            .contains(&(42, wl_keyboard::KeyState::Released))
    );
    assert!(!key(&f, true, 30, KeyState::Pressed)?);
    f.focus(Some(0))?;
    // A fresh, unconsumed root serial would normally authorize this popup.
    assert!(key(&f, true, 31, KeyState::Pressed)?);
    app.sync();
    let serial = app.events.keyboard.key_serial;
    assert!(!f.backend(|s| s.direct_keyboard(false, None))?);
    f.focus(Some(0))?;
    let (again, _, again_role) = app.popup(true, 1);
    app.grab_popup(&again_role, serial);
    again.commit();
    app.sync();
    assert_eq!(app.events.popups.done, 2);
    Ok(())
}
