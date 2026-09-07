//! Real-client explicit grabs, input isolation, serial refusal and terminal cleanup.
use super::{Application, Fixture, mapped};
use smithay::backend::input::{
    ButtonState::{Pressed, Released},
    KeyState,
};
use wayland_client::{Proxy, protocol::wl_keyboard};

fn fixture() -> Fixture {
    let f = Fixture::keyboard();
    assert!(
        f.backend(|s| {
            s.enable_popup_protocol();
            s.enable_pointer()
        })
        .is_ok()
    );
    f
}

fn press(f: &Fixture, app: &mut Application) -> u32 {
    assert!(f.backend(|s| s.pointer_motion(1.0, 1.0, 1)).is_ok());
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Pressed, 2)).ok(),
        Some(true)
    );
    app.sync();
    app.events.pointer.button_serial
}

#[test]
fn pointer_popup_grab_routes_keys_and_consumes_outside_click() {
    let f = fixture();
    let mut app = mapped(&f);
    let mut other = mapped(&f);
    assert!(f.focus(Some(0)).is_ok());
    let serial = press(&f, &mut app);
    let (surface, xdg, role) = app.popup(true, 20);
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
    // Repeated nested-backend root selection must preserve the popup focus.
    assert!(f.focus(Some(0)).is_ok());
    assert_eq!(f.key(30, KeyState::Pressed).ok(), Some(true));
    assert!(f.backend(|s| s.pointer_motion(27.0, 11.0, 3)).is_ok());
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Released, 4)).ok(),
        Some(false)
    );
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Pressed, 5)).ok(),
        Some(true)
    );
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Released, 6)).ok(),
        Some(true)
    );
    app.sync();
    assert_eq!(
        app.events.pointer.enters.last(),
        Some(&(surface.id().protocol_id(), 1.0, 1.0))
    );
    // Expose the unrelated window beneath the parent's empty input region.
    app.empty_input();
    app.sync();
    assert!(f.backend(|s| s.pointer_motion(1.0, 1.0, 7)).is_ok());
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Pressed, 8)).ok(),
        Some(false)
    );
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Released, 9)).ok(),
        Some(false)
    );
    app.sync();
    other.sync();
    assert_eq!(app.events.popups.done, 1);
    assert_eq!(
        app.events.keyboard.surfaces.last(),
        Some(&app.surface.id().protocol_id())
    );
    assert_eq!(
        app.events.keyboard.keys,
        [
            (30, wl_keyboard::KeyState::Pressed),
            (30, wl_keyboard::KeyState::Released)
        ]
    );
    assert!(other.events.pointer.enters.is_empty());
    assert!(other.events.pointer.buttons.is_empty());
    assert!(other.events.keyboard.keys.is_empty());
    // Only fresh motion may select the unrelated client after dismissal.
    assert!(f.backend(|s| s.pointer_motion(1.0, 1.0, 10)).is_ok());
    other.sync();
    assert_eq!(other.events.pointer.enters.len(), 1);
}

#[test]
fn pointer_popup_nested_grab_restores_parent_and_dismisses_child_first() {
    let f = fixture();
    let mut app = mapped(&f);
    let serial = press(&f, &mut app);
    let (parent, parent_xdg, parent_role) = app.popup(true, 1);
    app.grab_popup(&parent_role, serial);
    parent.commit();
    app.sync();
    app.ack_popup(&parent_xdg);
    app.attach_popup(&parent);
    app.sync();
    let (child, child_xdg, child_role) = app.popup_on(Some(&parent_xdg), 1);
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
    assert_eq!(f.key(30, KeyState::Pressed).ok(), Some(true));
    child_role.destroy();
    app.sync();
    assert_eq!(
        app.events.keyboard.surfaces.last(),
        Some(&parent.id().protocol_id())
    );
    assert_eq!(f.key(30, KeyState::Released).ok(), Some(false));
    let (child, child_xdg, child_role) = app.popup_on(Some(&parent_xdg), 1);
    app.grab_popup(&child_role, serial);
    child.commit();
    app.sync();
    app.ack_popup(&child_xdg);
    app.attach_popup(&child);
    app.sync();
    assert!(f.focus(None).is_ok());
    app.sync();
    assert_eq!(
        app.events.popups.done_order,
        [
            child_role.id().protocol_id(),
            parent_role.id().protocol_id()
        ]
    );
    assert_eq!(f.key(30, KeyState::Pressed).ok(), Some(false));
    assert!(f.backend(|s| s.popup_surfaces().is_empty()));
}

#[test]
fn pointer_popup_grab_rejects_stale_foreign_late_and_non_topmost_requests() {
    for refusal in 0..5 {
        let f = fixture();
        let mut app = mapped(&f);
        let mut other = mapped(&f);
        let serial = press(&f, &mut app);
        let target = if refusal == 1 { &mut other } else { &mut app };
        let (surface, xdg, role) = target.popup(true, 1);
        if refusal == 2 {
            surface.commit();
            target.sync();
            target.ack_popup(&xdg);
            target.attach_popup(&surface);
            target.sync();
        }
        if refusal == 3 {
            assert_eq!(
                f.backend(|s| s.pointer_button(0x110, Released, 3)).ok(),
                Some(true)
            );
        }
        target.grab_popup(
            &role,
            if refusal == 0 {
                serial.wrapping_sub(1)
            } else {
                serial
            },
        );
        if refusal == 2 {
            target.refused();
            other.sync();
            f.wait_for((1, 1));
            continue;
        }
        target.sync();
        if refusal == 4 {
            surface.commit();
            target.sync();
            target.ack_popup(&xdg);
            target.attach_popup(&surface);
            target.sync();
            let (sibling, _, sibling_role) = target.popup(true, 1);
            target.grab_popup(&sibling_role, serial);
            sibling.commit();
            target.sync();
            assert_eq!(
                target.events.popups.done_order,
                [sibling_role.id().protocol_id()]
            );
            assert_eq!(f.backend(|s| s.popup_surfaces().len()), 1);
        } else {
            assert_eq!(target.events.popups.done, 1);
            assert!(f.backend(|s| s.popup_surfaces().is_empty()));
        }
        app.sync();
        other.sync();
        assert!(other.events.keyboard.keys.is_empty());
    }
}

#[test]
fn pointer_popup_grab_clears_on_unmap_disconnect_and_backend_leave() {
    for action in 0..4 {
        let f = fixture();
        let mut app = mapped(&f);
        let mut other = mapped(&f);
        let serial = press(&f, &mut app);
        let (surface, xdg, role) = app.popup(true, 1);
        app.grab_popup(&role, serial);
        surface.commit();
        app.sync();
        app.ack_popup(&xdg);
        app.attach_popup(&surface);
        app.sync();
        assert!(f.backend(|s| s.pointer_motion(8.0, 11.0, 4)).is_ok());
        assert_eq!(
            f.backend(|s| s.pointer_button(0x110, Pressed, 5)).ok(),
            Some(true)
        );
        assert_eq!(f.key(30, KeyState::Pressed).ok(), Some(true));
        match action {
            0 => {
                app.surface.attach(None, 0, 0);
                app.surface.commit();
                app.sync();
            }
            1 => {
                surface.attach(None, 0, 0);
                surface.commit();
                app.sync();
            }
            2 => {
                drop(app);
                f.wait_for((1, 1));
            }
            _ => {
                assert!(f.backend(|s| s.pointer_leave()).is_ok());
                app.sync();
            }
        }
        assert!(f.backend(|s| s.popup_surfaces().is_empty()));
        assert_eq!(
            f.backend(|s| s.pointer_button(0x110, Released, 6)).ok(),
            Some(false)
        );
        other.sync();
        assert!(other.events.pointer.buttons.is_empty());
        assert!(other.events.keyboard.keys.is_empty());
        assert!(f.focus(Some(if action == 3 { 1 } else { 0 })).is_ok());
        assert_eq!(f.key(31, KeyState::Pressed).ok(), Some(true));
    }
}

#[test]
fn pointer_popup_owner_events_rehit_and_outside_second_button_cancels_drag() {
    let f = fixture();
    let mut app = mapped(&f);
    let mut other = mapped(&f);
    let serial = press(&f, &mut app);
    let (surface, xdg, role) = app.popup(true, 20);
    app.grab_popup(&role, serial);
    surface.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&surface);
    app.sync();
    // No motion after mapping: the stationary pointer still hits the parent.
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Pressed, 3)).ok(),
        Some(true)
    );
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Released, 4)).ok(),
        Some(true)
    );
    app.sync();
    assert_eq!(app.events.popups.done, 0);
    assert_eq!(
        app.events.pointer.enters.last().map(|e| e.0),
        Some(app.surface.id().protocol_id())
    );
    assert!(f.backend(|s| s.pointer_motion(27.0, 11.0, 5)).is_ok());
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Pressed, 6)).ok(),
        Some(true)
    );
    assert!(f.backend(|s| s.pointer_motion(300.0, 190.0, 7)).is_ok());
    // Duplicate presses cannot dismiss; another button must test the actual hit,
    // even though Smithay's implicit drag retains the popup as current focus.
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Pressed, 8)).ok(),
        Some(false)
    );
    app.sync();
    assert_eq!(app.events.popups.done, 0);
    assert_eq!(
        f.backend(|s| s.pointer_button(0x111, Pressed, 9)).ok(),
        Some(false)
    );
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Released, 10)).ok(),
        Some(false)
    );
    assert_eq!(
        f.backend(|s| s.pointer_button(0x111, Released, 11)).ok(),
        Some(false)
    );
    app.sync();
    other.sync();
    assert_eq!(app.events.popups.done, 1);
    assert!(other.events.pointer.buttons.is_empty());
    // The original serial cannot authorize a new chain after dismissal.
    let (retry, _, retry_role) = app.popup(true, 1);
    app.grab_popup(&retry_role, serial);
    retry.commit();
    app.sync();
    assert_eq!(app.events.popups.done, 2);
    assert!(f.backend(|s| s.popup_surfaces().is_empty()));
}

#[test]
fn pointer_popup_protocol_errors_are_isolated_to_the_invalid_chain() {
    for destroy_ancestor in [false, true] {
        let f = fixture();
        let mut app = mapped(&f);
        let mut other = mapped(&f);
        let serial = press(&f, &mut app);
        let (parent, parent_xdg, parent_role) = app.popup(true, 20);
        if destroy_ancestor {
            app.grab_popup(&parent_role, serial);
        }
        parent.commit();
        app.sync();
        app.ack_popup(&parent_xdg);
        app.attach_popup(&parent);
        app.sync();
        let (child, child_xdg, child_role) = app.popup_on(Some(&parent_xdg), 1);
        app.grab_popup(&child_role, serial);
        if destroy_ancestor {
            child.commit();
            app.sync();
            app.ack_popup(&child_xdg);
            app.attach_popup(&child);
            app.sync();
            parent_role.destroy();
        }
        app.refused();
        other.sync();
        f.wait_for((1, 1));
        assert!(f.backend(|s| s.popup_surfaces().is_empty()));
        assert!(other.events.keyboard.keys.is_empty());
    }
}
