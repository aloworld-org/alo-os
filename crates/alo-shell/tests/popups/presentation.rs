//! Popup placement, pointer lifetime and atomic output submission over real sockets.
use super::{Fixture, mapped};
use alo_shell::{Cursor, FrameTarget, Popup, RenderError};
use smithay::{
    backend::input::ButtonState::{Pressed, Released},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Size},
};
use wayland_client::{Proxy, protocol::wl_pointer};

pub(super) fn fixture() -> Fixture {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_popup_protocol());
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    f
}

fn motion(f: &Fixture, x: f64, y: f64) {
    assert!(f.backend(move |s| s.pointer_motion(x, y, 12)).is_ok());
}

#[test]
fn newest_popup_and_subsurface_offsets_share_stacking_and_cancel_on_child_loss() {
    let f = fixture();
    let mut app = mapped(&f);
    let (older, xdg, _role) = app.popup(true, 1);
    older.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&older);
    app.sync();
    let (newer, xdg, role) = app.popup(true, 1);
    newer.commit();
    app.sync();
    app.ack_popup(&xdg);
    let (child, child_role) = app.child_on(&newer, (2, 3));
    app.attach_popup(&newer);
    app.sync();
    motion(&f, 10.0, 15.0);
    app.sync();
    assert_eq!(
        app.events.pointer.enters.last(),
        Some(&(child.id().protocol_id(), 1.0, 2.0))
    );
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Pressed, 13)).ok(),
        Some(true)
    );
    child.attach(None, 0, 0);
    child.commit();
    newer.commit();
    app.sync();
    assert_eq!(
        app.events.pointer.buttons.last(),
        Some(&(0x110, wl_pointer::ButtonState::Released))
    );
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Released, 14)).ok(),
        Some(false)
    );
    child_role.destroy();
    child.destroy();
    app.sync();
    motion(&f, 10.0, 15.0);
    app.sync();
    assert_eq!(
        app.events.pointer.enters.last(),
        Some(&(newer.id().protocol_id(), 3.0, 5.0))
    );
    role.destroy();
    xdg.destroy();
    newer.destroy();
    app.sync();
    motion(&f, 10.0, 15.0);
    app.sync();
    assert_eq!(
        app.events.pointer.enters.last(),
        Some(&(older.id().protocol_id(), 3.0, 5.0))
    );
}

#[test]
fn popup_geometry_stacking_input_regions_and_client_isolation() {
    let f = fixture();
    let mut front = mapped(&f);
    let mut back = mapped(&f);
    // A background window's popup must not jump above the foreground window.
    let (background, xdg, _role) = back.popup(true, 0);
    background.commit();
    back.sync();
    back.ack_popup(&xdg);
    back.attach_popup(&background);
    back.sync();
    motion(&f, 8.0, 12.0);
    front.sync();
    back.sync();
    assert_eq!(
        front.events.pointer.enters.last(),
        Some(&(front.surface.id().protocol_id(), 8.0, 12.0))
    );
    assert!(back.events.pointer.enters.is_empty());

    front.xdg.set_window_geometry(2, 3, 12, 12);
    front.surface.commit();
    let (popup, xdg, _role) = front.popup(true, 1);
    popup.commit();
    front.sync();
    front.ack_popup(&xdg);
    xdg.set_window_geometry(1, 2, 12, 12);
    front.attach_popup(&popup);
    front.sync();
    // (2,3) + configured (7,10) - popup geometry (1,2) = buffer (8,11).
    assert_eq!(
        f.backend(|s| s.popup_surfaces().last().map(Popup::location)),
        Some((8.0, 11.0).into())
    );
    motion(&f, 10.0, 14.0);
    front.sync();
    assert_eq!(
        front.events.pointer.enters.last(),
        Some(&(popup.id().protocol_id(), 2.0, 3.0))
    );
    // Cached geometry changes do not apply before commit.
    xdg.set_window_geometry(3, 4, 10, 10);
    front.sync();
    assert_eq!(
        f.backend(|s| s.popup_surfaces().last().map(Popup::location)),
        Some((8.0, 11.0).into())
    );
    popup.commit();
    front.sync();
    motion(&f, 10.0, 14.0);
    front.sync();
    assert_eq!(front.events.pointer.motion.last(), Some(&(4.0, 5.0)));
    // Empty popup input falls through to its parent's tree, not another client.
    front.empty_input_on(&popup);
    front.sync();
    motion(&f, 10.0, 14.0);
    front.sync();
    back.sync();
    assert_eq!(
        front.events.pointer.enters.last(),
        Some(&(front.surface.id().protocol_id(), 10.0, 14.0))
    );
    assert!(back.events.pointer.enters.is_empty());
    // Full-range geometry cannot overflow placement arithmetic.
    xdg.set_window_geometry(i32::MAX, i32::MIN, i32::MAX, i32::MAX);
    popup.commit();
    front.sync();
    assert_eq!(
        f.backend(|s| s.popup_surfaces().last().map(Popup::location)),
        Some((9.0, 13.0).into())
    );
    let (_far, _far_role) = front.child_on(&popup, (i32::MAX, i32::MAX));
    popup.commit();
    front.sync();
    assert_eq!(
        f.backend(|s| s.popup_surfaces().last().map(Popup::location)),
        Some((9.0, 13.0).into())
    );
}

#[test]
fn popup_drag_dismissal_parent_loss_and_disconnect_cancel_without_redirecting() {
    for action in 0..5 {
        let f = fixture();
        let mut app = mapped(&f);
        let mut other = mapped(&f);
        let (popup, xdg, role) = app.popup(true, 1);
        popup.commit();
        app.sync();
        app.ack_popup(&xdg);
        app.attach_popup(&popup);
        app.sync();
        motion(&f, 20.0, 20.0);
        assert_eq!(
            f.backend(|s| s.pointer_button(0x110, Pressed, 13)).ok(),
            Some(true)
        );
        motion(&f, 1.0, 1.0);
        app.sync();
        assert_eq!(app.events.pointer.motion.last(), Some(&(-6.0, -9.0)));
        match action {
            0 => {
                popup.attach(None, 0, 0);
                popup.commit();
            }
            1 => {
                app.surface.attach(None, 0, 0);
                app.surface.commit();
            }
            2 => role.destroy(),
            3 => app.refuse_popup_position(&role),
            _ => {
                drop(app);
                f.wait_for((1, 1));
                assert_eq!(
                    f.backend(|s| s.pointer_button(0x110, Released, 14)).ok(),
                    Some(false)
                );
                other.sync();
                assert!(other.events.pointer.buttons.is_empty());
                continue;
            }
        }
        app.sync();
        assert_eq!(
            app.events.pointer.buttons,
            [
                (0x110, wl_pointer::ButtonState::Pressed),
                (0x110, wl_pointer::ButtonState::Released)
            ]
        );
        assert_eq!(app.events.pointer.leaves, 1);
        assert_eq!(
            f.backend(|s| s.pointer_button(0x110, Released, 14)).ok(),
            Some(false)
        );
        other.sync();
        assert!(other.events.pointer.buttons.is_empty());
    }
}

/// Controlled target tests the coordinator; GLES evidence is in nested_check.
pub(super) struct Target(pub(super) bool);
impl FrameTarget for Target {
    fn size(&self) -> Size<i32, Physical> {
        (320, 200).into()
    }
    fn submit(&mut self, roots: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        Ok(roots.to_vec())
    }
    fn submit_popups(
        &mut self,
        roots: &[WlSurface],
        popups: &[Popup],
        _: &Cursor,
    ) -> Result<Vec<WlSurface>, RenderError> {
        if self.0 {
            return Err(RenderError::Submission(
                "injected popup swap failure".into(),
            ));
        }
        Ok(roots
            .iter()
            .cloned()
            .chain(popups.iter().map(|p| p.surface.clone()))
            .collect())
    }
}

#[test]
fn popup_callbacks_and_output_membership_wait_for_supported_successful_submission() {
    let f = fixture();
    assert_eq!(f.render((320, 200), false, 0).ok(), Some(0));
    let mut app = mapped(&f);
    let (popup, xdg, _role) = app.popup(true, 1);
    popup.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&popup);
    app.sync();
    assert!(f.render((320, 200), false, 1).is_err());
    assert!(f.backend(|s| s.render(&mut Target(true), 2)).is_err());
    app.sync();
    assert!(app.events.frames.is_empty());
    assert_eq!(app.events.membership, (0, 0));
    assert_eq!(f.backend(|s| s.render(&mut Target(false), 3)).ok(), Some(2));
    app.sync();
    assert_eq!(app.events.frames, [3]);
    assert_eq!(app.events.membership, (2, 0));
    popup.attach(None, 0, 0);
    popup.commit();
    app.sync();
    assert!(f.backend(|s| s.render(&mut Target(true), 4)).is_err());
    app.sync();
    assert_eq!(app.events.membership, (2, 0));
    assert_eq!(f.backend(|s| s.render(&mut Target(false), 5)).ok(), Some(1));
    app.sync();
    assert_eq!(app.events.membership, (2, 1));
    assert_eq!(app.events.frames, [3]);
}
