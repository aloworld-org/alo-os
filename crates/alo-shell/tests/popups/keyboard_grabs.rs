//! Keyboard-only popup initiation uses actual delivered serials, never guessed ones.
use super::{Fixture, mapped};
use smithay::backend::input::KeyState::{Pressed, Released};
use wayland_client::Proxy;

fn fixture() -> Fixture {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_popup_protocol());
    f
}

#[test]
fn keyboard_popup_grab_maps_submenu_and_consumes_initiating_key() {
    let f = fixture();
    let mut app = mapped(&f);
    assert!(f.focus(Some(0)).is_ok());
    assert_eq!(f.key(30, Pressed).ok(), Some(true));
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
    assert_eq!(
        app.events.keyboard.surfaces.last(),
        Some(&surface.id().protocol_id())
    );
    assert_eq!(f.key(30, Released).ok(), Some(false));
    let (child, child_xdg, child_role) = app.popup_on(Some(&xdg), 1);
    app.grab_popup(&child_role, serial);
    child.commit();
    app.sync();
    app.ack_popup(&child_xdg);
    app.attach_popup(&child);
    app.sync();
    assert_eq!(
        app.events.keyboard.surfaces.last(),
        Some(&child.id().protocol_id())
    );
    assert_eq!(f.key(48, Pressed).ok(), Some(true));
    child_role.destroy();
    app.sync();
    assert_eq!(f.key(48, Released).ok(), Some(false));
    role.destroy();
    app.sync();
    assert_eq!(
        app.events.keyboard.surfaces.last(),
        Some(&app.surface.id().protocol_id())
    );
    let (again, _, role) = app.popup(true, 1);
    app.grab_popup(&role, serial);
    again.commit();
    app.sync();
    assert_eq!(
        app.events.popups.done, 1,
        "consumed serial reopened a root grab"
    );
}

#[test]
fn keyboard_popup_grab_refuses_released_superseded_foreign_and_lost_focus_serials() {
    for refusal in 0..6 {
        let f = fixture();
        let mut app = mapped(&f);
        let mut other = mapped(&f);
        assert!(f.focus(Some(0)).is_ok());
        assert_eq!(f.key(30, Pressed).ok(), Some(true));
        app.sync();
        let serial = app.events.keyboard.key_serial;
        match refusal {
            0 => {
                assert_eq!(f.key(30, Released).ok(), Some(true));
            }
            1 => {
                assert_eq!(f.key(48, Pressed).ok(), Some(true));
            }
            2 => {}
            3 => {
                assert!(f.focus(None).is_ok());
                assert!(f.focus(Some(0)).is_ok());
            }
            4 => {
                app.surface.attach(None, 0, 0);
                app.surface.commit();
                app.sync();
                app.configure();
                app.attach();
                app.sync();
                assert!(f.focus(Some(0)).is_ok());
            }
            _ => {}
        }
        let target = if refusal == 2 { &mut other } else { &mut app };
        let (surface, _, role) = target.popup(true, 1);
        target.grab_popup(
            &role,
            if refusal == 5 {
                serial.wrapping_add(1000)
            } else {
                serial
            },
        );
        surface.commit();
        target.sync();
        assert_eq!(target.events.popups.done, 1, "refusal case {refusal}");
        other.sync();
        assert!(other.events.keyboard.keys.is_empty());
    }
}

#[test]
fn keyboard_popup_pending_grab_cannot_reuse_serial_after_destruction() {
    let f = fixture();
    let mut app = mapped(&f);
    assert!(f.focus(Some(0)).is_ok());
    assert_eq!(f.key(30, Pressed).ok(), Some(true));
    // Duplicate input does not invent or invalidate an actual delivered serial.
    assert_eq!(f.key(30, Pressed).ok(), Some(false));
    app.sync();
    let serial = app.events.keyboard.key_serial;
    let (_, _, role) = app.popup(true, 1);
    app.grab_popup(&role, serial);
    app.sync();
    assert_eq!(app.events.popups.done, 0);
    role.destroy();
    app.sync();
    let (surface, _, role) = app.popup(true, 1);
    app.grab_popup(&role, serial);
    surface.commit();
    app.sync();
    assert_eq!(app.events.popups.done, 1);
}
