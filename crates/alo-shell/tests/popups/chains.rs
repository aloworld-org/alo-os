//! Nested popup geometry, input and child-first terminal dismissal.
use super::{fixture, mapped};
use smithay::backend::input::ButtonState::Pressed;
use wayland_client::{Proxy, protocol::wl_pointer};

#[test]
fn three_popup_levels_stack_and_submit_atomically() {
    let f = super::Fixture::keyboard();
    f.backend(|s| s.enable_popup_protocol());
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    let mut app = mapped(&f);
    let mut chain = Vec::new();
    for _ in 0..3 {
        let parent = chain.last().map(|(_, xdg, _)| xdg).unwrap_or(&app.xdg);
        let (surface, xdg, role) = app.popup_on(Some(parent), 1);
        surface.commit();
        app.sync();
        app.ack_popup(&xdg);
        app.attach_popup(&surface);
        app.sync();
        chain.push((surface, xdg, role));
    }
    assert_eq!(chain.len(), 3);
    if let [first, middle, last] = chain.as_slice() {
        assert!(f.backend(|s| s.pointer_motion(22.0, 31.0, 1)).is_ok());
        app.sync();
        assert_eq!(
            app.events.pointer.enters.last(),
            Some(&(last.0.id().protocol_id(), 1.0, 1.0))
        );
        assert!(
            f.backend(|s| s.render(&mut super::presentation::Target(true), 2))
                .is_err()
        );
        app.sync();
        assert!(app.events.frames.is_empty());
        assert_eq!(app.events.membership, (0, 0));
        assert_eq!(
            f.backend(|s| s.render(&mut super::presentation::Target(false), 3))
                .ok(),
            Some(4)
        );
        app.sync();
        assert_eq!(app.events.frames, [3, 3, 3]);
        assert_eq!(app.events.membership, (4, 0));
        // Dismissing the middle removes its child, preserving the ancestor.
        app.refuse_popup_position(&middle.2);
        app.sync();
        assert_eq!(
            app.events.popups.done_order,
            [last.2.id().protocol_id(), middle.2.id().protocol_id()]
        );
        assert_eq!(f.backend(|s| s.popup_surfaces().len()), 1);
        assert_eq!(
            f.backend(|s| s.render(&mut super::presentation::Target(false), 4))
                .ok(),
            Some(2)
        );
        app.sync();
        assert_eq!(app.events.membership, (4, 2));
        assert!(f.backend(|s| s.pointer_motion(8.0, 11.0, 5)).is_ok());
        app.sync();
        assert_eq!(
            app.events.pointer.enters.last(),
            Some(&(first.0.id().protocol_id(), 1.0, 1.0))
        );
    }
}

#[test]
fn nested_popup_coordinates_input_and_ancestor_cleanup() {
    for action in 0..5 {
        let f = super::Fixture::keyboard();
        f.backend(|s| s.enable_popup_protocol());
        assert!(f.backend(|s| s.enable_pointer()).is_ok());
        let mut app = mapped(&f);
        let mut healthy = mapped(&f);
        let (parent, parent_xdg, parent_role) = app.popup(true, 1);
        parent.commit();
        app.sync();
        app.ack_popup(&parent_xdg);
        parent_xdg.set_window_geometry(1, 2, 12, 12);
        app.attach_popup(&parent);
        app.sync();
        let (child, child_xdg, child_role) = app.popup_on(Some(&parent_xdg), 1);
        child.commit();
        app.sync();
        app.ack_popup(&child_xdg);
        app.attach_popup(&child);
        app.sync();
        assert_eq!(f.backend(|s| s.popup_surfaces().len()), 2);
        // Parent buffer (6,8), child (6,8)+(1,2)+(7,10)=(14,20).
        assert!(f.backend(|s| s.pointer_motion(15.0, 22.0, 10)).is_ok());
        app.sync();
        assert_eq!(
            app.events.pointer.enters.last(),
            Some(&(child.id().protocol_id(), 1.0, 2.0))
        );
        assert_eq!(
            f.backend(|s| s.pointer_button(0x110, Pressed, 11)).ok(),
            Some(true)
        );
        match action {
            0 => {
                parent.attach(None, 0, 0);
                parent.commit();
            }
            1 => {
                app.surface.attach(None, 0, 0);
                app.surface.commit();
            }
            2 => app.refuse_popup_position(&parent_role),
            3 => parent_role.destroy(),
            _ => {
                drop(app);
                f.wait_for((1, 1));
                assert!(f.backend(|s| s.popup_surfaces().is_empty()));
                healthy.sync();
                assert!(healthy.events.pointer.buttons.is_empty());
                continue;
            }
        }
        app.sync();
        assert!(f.backend(|s| s.popup_surfaces().is_empty()));
        let expected = if action == 3 {
            vec![child_role.id().protocol_id()]
        } else {
            vec![
                child_role.id().protocol_id(),
                parent_role.id().protocol_id(),
            ]
        };
        assert_eq!(app.events.popups.done_order, expected);
        assert_eq!(
            app.events.pointer.buttons.last(),
            Some(&(0x110, wl_pointer::ButtonState::Released))
        );
        assert_eq!(app.events.pointer.leaves, 1);
        app.attach_popup(&child);
        app.sync();
        assert!(f.backend(|s| s.popup_surfaces().is_empty()));
        assert_eq!(app.events.popups.done_order, expected);
        healthy.sync();
        assert!(healthy.events.pointer.buttons.is_empty());
    }
}

#[test]
fn nested_popup_requires_live_mapped_parent_and_dismissal_is_terminal() {
    for dismissed in [false, true] {
        let f = fixture();
        let mut app = mapped(&f);
        let (parent, xdg, role) = app.popup(true, 1);
        parent.commit();
        app.sync();
        app.ack_popup(&xdg);
        if dismissed {
            app.attach_popup(&parent);
            app.sync();
            app.refuse_popup_position(&role);
            app.sync();
        }
        let (child, _, child_role) = app.popup_on(Some(&xdg), 1);
        child.commit();
        app.sync();
        assert_eq!(
            app.events.popups.done_order.last(),
            Some(&child_role.id().protocol_id())
        );
        assert!(f.backend(|s| s.popup_surfaces().is_empty()));
        app.attach_popup(&child);
        app.sync();
        assert!(f.backend(|s| s.popup_surfaces().is_empty()));
    }
}
