//! Real release serials authorize one parent menu, never historical or synthetic input.
use super::{Application, Fixture, mapped};
use smithay::backend::input::ButtonState::{self, Pressed, Released};
use wayland_client::Proxy;

fn fixture() -> Fixture {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_popup_protocol());
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    f
}

fn button(f: &Fixture, code: u32, state: ButtonState, accepted: bool) {
    assert_eq!(
        f.backend(move |s| s.pointer_button(code, state, 10)).ok(),
        Some(accepted)
    );
}

fn release(f: &Fixture, app: &mut Application) -> u32 {
    assert!(f.backend(|s| s.pointer_motion(1.0, 1.0, 1)).is_ok());
    button(f, 0x110, Pressed, true);
    button(f, 0x110, Released, true);
    app.sync();
    app.events.pointer.button_serial
}

#[test]
fn pointer_release_popup_maps_submenu_from_subsurface_and_consumes_serial() {
    let f = fixture();
    let mut app = mapped(&f);
    let (child, _child_role) = app.child((0, 0));
    app.surface.commit();
    app.sync();
    let serial = release(&f, &mut app);
    assert_eq!(
        app.events.pointer.enters.last().map(|e| e.0),
        Some(child.id().protocol_id())
    );
    button(&f, 0x110, Released, false);
    assert!(f.backend(|s| s.pointer_button(0, Pressed, 11)).is_err());
    assert!(f.backend(|s| s.pointer_motion(f64::NAN, 1.0, 12)).is_err());
    // Motion within the same exact recipient preserves the release authority.
    assert!(f.backend(|s| s.pointer_motion(2.0, 2.0, 13)).is_ok());
    let (surface, xdg, role) = app.popup(true, 1);
    app.grab_popup(&role, serial);
    surface.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&surface);
    app.sync();
    assert_eq!(app.events.popups.done, 0);
    let (submenu, submenu_xdg, submenu_role) = app.popup_on(Some(&xdg), 1);
    app.grab_popup(&submenu_role, serial);
    submenu.commit();
    app.sync();
    app.ack_popup(&submenu_xdg);
    app.attach_popup(&submenu);
    app.sync();
    assert_eq!(
        app.events.keyboard.surfaces.last(),
        Some(&submenu.id().protocol_id())
    );
    submenu_role.destroy();
    role.destroy();
    app.sync();
    let (again, _, role) = app.popup(true, 1);
    app.grab_popup(&role, serial);
    again.commit();
    app.sync();
    assert_eq!(app.events.popups.done, 1);
}

#[test]
fn pointer_release_pending_popup_cannot_replay_after_destruction() {
    let f = fixture();
    let mut app = mapped(&f);
    let serial = release(&f, &mut app);
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

#[test]
fn pointer_release_popup_refuses_supersession_foreign_focus_loss_and_synthetic_serials() {
    for refusal in 0..9 {
        let f = fixture();
        let mut app = mapped(&f);
        let mut other = mapped(&f);
        let mut serial = release(&f, &mut app);
        match refusal {
            0 => button(&f, 0x111, Pressed, true),
            1 => {
                button(&f, 0x111, Pressed, true);
                button(&f, 0x111, Released, true);
            }
            2 => {} // The recipient's serial cannot authorize another client's parent.
            3 => {
                assert!(f.backend(|s| s.pointer_motion(300.0, 300.0, 11)).is_ok());
                assert!(f.backend(|s| s.pointer_motion(1.0, 1.0, 12)).is_ok());
            }
            4 => {
                app.surface.attach(None, 0, 0);
                app.surface.commit();
                app.sync();
                app.configure();
                app.attach();
                app.sync();
                assert!(f.backend(|s| s.pointer_motion(1.0, 1.0, 12)).is_ok());
            }
            5 => {
                button(&f, 0x111, Pressed, true);
                assert!(f.backend(|s| s.pointer_leave()).is_ok());
                app.sync();
                assert_ne!(serial, app.events.pointer.button_serial);
                serial = app.events.pointer.button_serial;
                assert!(f.backend(|s| s.pointer_motion(1.0, 1.0, 12)).is_ok());
            }
            6 => serial = serial.wrapping_add(1000),
            7 => {
                // Releasing outside must lose authority despite Smithay retaining focus.
                button(&f, 0x110, Pressed, true);
                assert!(f.backend(|s| s.pointer_motion(300.0, 300.0, 11)).is_ok());
                button(&f, 0x110, Released, true);
                app.sync();
                serial = app.events.pointer.button_serial;
                assert!(f.backend(|s| s.pointer_motion(1.0, 1.0, 12)).is_ok());
            }
            _ => {
                let (child, role) = app.child((0, 0));
                app.surface.commit();
                app.sync();
                serial = release(&f, &mut app);
                role.destroy();
                child.destroy();
                app.sync();
                assert!(f.backend(|s| s.pointer_motion(1.0, 1.0, 12)).is_ok());
            }
        }
        let target = if refusal == 2 { &mut other } else { &mut app };
        let (surface, _, role) = target.popup(true, 1);
        target.grab_popup(&role, serial);
        surface.commit();
        target.sync();
        assert_eq!(target.events.popups.done, 1, "release refusal {refusal}");
        other.sync();
        assert!(other.events.pointer.buttons.is_empty());
        assert_eq!(f.backend(|s| s.mapped_surfaces().count()), 2);
    }
}
