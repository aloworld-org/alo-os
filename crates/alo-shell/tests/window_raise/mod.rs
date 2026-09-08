//! Real socket stacking, stationary routing, grab lifetime and refusal checks.
use super::{Application, Fixture};
use alo_shell::{FrameTarget, RenderError, WindowRaiseError};
use smithay::{
    backend::input::{ButtonState, KeyState},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Size},
};

fn mapped(f: &Fixture) -> Application {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    app
}

fn roots(f: &Fixture) -> Vec<WlSurface> {
    f.backend(|s| s.mapped_surfaces().cloned().collect())
}

fn raise(f: &Fixture, root: &WlSurface) -> Result<(), WindowRaiseError> {
    let root = root.clone();
    f.backend(move |s| s.raise_window(&root))
}

/// Observe order at the production renderer boundary, not just enumeration.
struct Order(Vec<WlSurface>);
impl FrameTarget for Order {
    fn size(&self) -> Size<i32, Physical> {
        (320, 200).into()
    }
    fn submit(&mut self, roots: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        assert_eq!(roots, self.0);
        Ok(roots.to_vec())
    }
    fn submit_scene(
        &mut self,
        roots: &[WlSurface],
        _: &alo_shell::Cursor,
    ) -> Result<Vec<WlSurface>, RenderError> {
        self.submit(roots)
    }
}

#[test]
fn window_raise_updates_render_order_and_stationary_pointer_without_stealing_keyboard()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    let mut first = mapped(&f);
    let mut second = mapped(&f);
    let _third = mapped(&f);
    let original = roots(&f);
    let [_first_root, second_root, _tail @ ..] = original.as_slice() else {
        return Err("two mapped roots required".into());
    };
    assert!(f.focus_surface(_first_root.clone()).is_ok());
    assert!(f.backend(|s| s.pointer_motion(2.0, 3.0, 1)).is_ok());
    first.sync();
    assert_eq!(first.events.pointer.enters.len(), 1);
    assert!(raise(&f, second_root).is_ok());
    let expected = vec![
        second_root.clone(),
        _first_root.clone(),
        _tail.first().cloned().ok_or("third root required")?,
    ];
    assert_eq!(roots(&f), expected);
    let order = expected.clone();
    assert_eq!(
        f.backend(move |s| s.render(&mut Order(order), 2)).ok(),
        Some(3)
    );
    second.sync();
    first.sync();
    assert_eq!(first.events.pointer.leaves, 1);
    assert_eq!(second.events.pointer.enters.len(), 1);
    assert!(f.key(30, KeyState::Pressed).is_ok());
    assert!(f.key(30, KeyState::Released).is_ok());
    first.sync();
    second.sync();
    assert_eq!(first.events.keyboard.keys.len(), 2);
    assert!(second.events.keyboard.keys.is_empty());
    assert!(raise(&f, second_root).is_ok());
    assert_eq!(roots(&f), expected);
    second.sync();
    assert_eq!(second.events.pointer.enters.len(), 1);
    Ok(())
}

#[test]
fn window_raise_preserves_held_button_recipient_until_release()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    let mut first = mapped(&f);
    let mut second = mapped(&f);
    let original = roots(&f);
    let [_first_root, second_root, _tail @ ..] = original.as_slice() else {
        return Err("two mapped roots required".into());
    };
    assert!(f.backend(|s| s.pointer_motion(2.0, 3.0, 1)).is_ok());
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, ButtonState::Pressed, 2))
            .ok(),
        Some(true)
    );
    assert!(raise(&f, second_root).is_ok());
    second.sync();
    assert!(second.events.pointer.enters.is_empty());
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, ButtonState::Released, 3))
            .ok(),
        Some(true)
    );
    first.sync();
    second.sync();
    assert_eq!(first.events.pointer.buttons.len(), 2);
    assert!(second.events.pointer.buttons.is_empty());
    assert!(f.backend(|s| s.pointer_motion(2.0, 3.0, 4)).is_ok());
    second.sync();
    assert_eq!(second.events.pointer.enters.len(), 1);
    Ok(())
}

#[test]
fn window_raise_refuses_foreign_unmapped_popup_and_dead_targets_without_reordering()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new();
    f.backend(|s| s.enable_popup_protocol());
    let mut first = mapped(&f);
    let second = mapped(&f);
    let original = roots(&f);
    let [_first_root, second_root, _tail @ ..] = original.as_slice() else {
        return Err("two mapped roots required".into());
    };
    let foreign = Fixture::new();
    let _foreign_app = mapped(&foreign);
    assert!(matches!(
        raise(&f, &foreign.root()),
        Err(WindowRaiseError::Unmapped)
    ));
    let (popup, xdg, _role) = first.popup(true, 1);
    popup.commit();
    first.sync();
    first.ack_popup(&xdg);
    first.attach_popup(&popup);
    first.sync();
    let Some(popup_root) = f.backend(|s| s.popup_surfaces().into_iter().next().map(|p| p.surface))
    else {
        return Err("mapped popup required".into());
    };
    assert!(matches!(
        raise(&f, &popup_root),
        Err(WindowRaiseError::Unmapped)
    ));
    assert_eq!(roots(&f), original);
    first.surface.attach(None, 0, 0);
    first.surface.commit();
    first.sync();
    assert!(matches!(
        raise(&f, _first_root),
        Err(WindowRaiseError::Unmapped)
    ));
    first.configure();
    assert!(matches!(
        raise(&f, _first_root),
        Err(WindowRaiseError::Unmapped)
    ));
    first.attach();
    first.sync();
    assert!(raise(&f, _first_root).is_ok());
    drop(second);
    f.wait_for((1, 1));
    assert!(matches!(
        raise(&f, second_root),
        Err(WindowRaiseError::Unmapped)
    ));
    assert_eq!(roots(&f), vec![_first_root.clone()]);
    Ok(())
}

#[test]
fn window_raise_preserves_popup_grab_and_does_not_reenter_after_pointer_leave()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    assert!(
        f.backend(|s| {
            s.enable_popup_protocol();
            s.enable_pointer()
        })
        .is_ok()
    );
    let mut first = mapped(&f);
    let mut second = mapped(&f);
    let original = roots(&f);
    let [_first_root, second_root, _tail @ ..] = original.as_slice() else {
        return Err("two mapped roots required".into());
    };
    assert!(f.focus(Some(0)).is_ok());
    assert!(f.backend(|s| s.pointer_motion(1.0, 1.0, 1)).is_ok());
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, ButtonState::Pressed, 2))
            .ok(),
        Some(true)
    );
    first.sync();
    let (popup, xdg, role) = first.popup(true, 20);
    first.grab_popup(&role, first.events.pointer.button_serial);
    popup.commit();
    first.sync();
    first.ack_popup(&xdg);
    first.attach_popup(&popup);
    first.sync();
    assert!(raise(&f, second_root).is_ok());
    assert_eq!(f.key(30, KeyState::Pressed).ok(), Some(true));
    assert_eq!(f.key(30, KeyState::Released).ok(), Some(true));
    first.sync();
    second.sync();
    assert_eq!(first.events.popups.done, 0);
    assert_eq!(first.events.keyboard.keys.len(), 2);
    assert!(second.events.keyboard.keys.is_empty());
    assert!(second.events.pointer.enters.is_empty());
    role.destroy();
    xdg.destroy();
    popup.destroy();
    first.sync();
    assert!(f.backend(|s| s.pointer_leave()).is_ok());
    first.sync();
    let enters = first.events.pointer.enters.len();
    assert!(raise(&f, _first_root).is_ok());
    first.sync();
    assert_eq!(first.events.pointer.enters.len(), enters);
    Ok(())
}
