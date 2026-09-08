//! Real-client visibility, input isolation, refusal and mapping lifetime.
use super::{Application, Fixture};
mod interaction;
use alo_shell::{WindowMinimizeError, WindowSwitchDirection as Direction};
use smithay::{
    backend::input::{ButtonState, KeyState},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
};

fn mapped(f: &Fixture) -> Application {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    app
}

fn minimize(f: &Fixture, root: &WlSurface, value: bool) -> Result<bool, WindowMinimizeError> {
    let root = root.clone();
    f.backend(move |s| s.set_window_minimized(&root, value))
}

#[test]
fn window_minimize_retires_held_input_without_replaying_into_other_window()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    let mut other = mapped(&f);
    f.focus(Some(0))?;
    assert!(f.key(30, KeyState::Pressed)?);
    f.backend(|s| s.pointer_motion(2.0, 2.0, 1))?;
    assert!(f.backend(|s| s.pointer_button(0x110, ButtonState::Pressed, 2))?);
    assert!(minimize(&f, &root, true)?);
    assert!(!minimize(&f, &root, true)?);
    app.sync();
    assert_eq!(app.events.activation.last(), Some(&false));
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert_eq!(app.events.keyboard.leaves, 1);
    assert_eq!(app.events.pointer.buttons.len(), 2);
    assert_eq!(app.events.pointer.leaves, 1);
    assert!(matches!(
        f.focus_surface(root.clone()),
        Err(alo_shell::InputError::Unmapped)
    ));
    assert!(!f.key(31, KeyState::Pressed)?);
    f.focus(Some(0))?;
    f.backend(|s| s.pointer_motion(2.0, 2.0, 3))?;
    assert!(!f.key(30, KeyState::Released)?);
    assert!(!f.backend(|s| s.pointer_button(0x110, ButtonState::Released, 4))?);
    assert!(f.key(32, KeyState::Pressed)?);
    assert!(f.key(32, KeyState::Released)?);
    other.sync();
    assert_eq!(other.events.keyboard.keys.len(), 2);
    assert!(other.events.pointer.buttons.is_empty());
    assert!(minimize(&f, &root, false)?);
    app.sync();
    assert_eq!(app.events.activation.last(), Some(&false));
    assert!(!minimize(&f, &root, false)?);
    Ok(())
}

#[test]
fn window_minimize_preserves_buffers_callbacks_and_maximize_memory()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    let target = root.clone();
    f.backend(move |s| s.place_window(&target, (20, 10)))?;
    f.render((80, 60), false, 1)?;
    let target = root.clone();
    f.backend(move |s| s.set_window_maximized(&target, true))?;
    app.sync();
    app.xdg
        .ack_configure(app.events.serial.ok_or("configure missing")?);
    app.surface.commit();
    app.sync();
    assert!(minimize(&f, &root, true)?);
    app.surface.frame(&app.queue.handle(), ());
    app.attach_resized();
    app.sync();
    assert_eq!(f.backend(|s| s.minimized_surfaces().count()), 1);
    assert_eq!(f.render((80, 60), false, 2)?, 0);
    app.sync();
    assert!(app.events.frames.is_empty());
    assert_eq!(app.events.membership, (1, 1));
    assert!(minimize(&f, &root, false)?);
    assert_eq!(f.render((80, 60), false, 3)?, 1);
    app.sync();
    assert_eq!(app.events.frames, [3]);
    let target = root.clone();
    f.backend(move |s| s.set_window_maximized(&target, false))?;
    app.sync();
    app.xdg
        .ack_configure(app.events.serial.ok_or("restore configure missing")?);
    app.surface.commit();
    app.sync();
    assert_eq!(
        f.backend(move |_| alo_shell::window_buffer_origin(&root)),
        (20.0, 10.0).into()
    );
    Ok(())
}

#[test]
fn window_minimize_cycling_skips_hidden_roots_and_preserves_original_ring()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let _a = mapped(&f);
    let a = f.root();
    let _b = mapped(&f);
    let b = f
        .backend(|s| s.mapped_surfaces().nth(1).cloned())
        .ok_or("second missing")?;
    let _c = mapped(&f);
    let c = f
        .backend(|s| s.mapped_surfaces().nth(2).cloned())
        .ok_or("third missing")?;
    f.focus_surface(a.clone())?;
    minimize(&f, &b, true)?;
    assert_eq!(f.backend(|s| s.switch_window(Direction::Forward))?, c);
    minimize(&f, &b, false)?;
    assert_eq!(f.backend(|s| s.switch_window(Direction::Backward))?, b);
    for root in [&a, &b, &c] {
        minimize(&f, root, true)?;
    }
    assert!(matches!(
        f.backend(|s| s.switch_window(Direction::Forward)),
        Err(alo_shell::WindowSwitchError::Empty)
    ));
    minimize(&f, &b, false)?;
    assert_eq!(f.backend(|s| s.switch_window(Direction::Forward))?, b);
    Ok(())
}

#[test]
fn window_minimize_refuses_foreign_child_popup_unmapped_and_dead_targets()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new();
    f.backend(|s| s.enable_popup_protocol());
    let mut app = mapped(&f);
    let root = f.root();
    let foreign = Fixture::new();
    let _other = mapped(&foreign);
    for value in [true, false] {
        assert_eq!(
            minimize(&f, &foreign.root(), value),
            Err(WindowMinimizeError::Unmapped)
        );
    }
    let (_child, _role) = app.child((2, 2));
    app.surface.commit();
    app.sync();
    let target = root.clone();
    let child = f
        .backend(move |_| {
            smithay::wayland::compositor::get_children(&target)
                .into_iter()
                .find(|child| child != &target)
        })
        .ok_or("child missing")?;
    assert_eq!(
        minimize(&f, &child, true),
        Err(WindowMinimizeError::Unmapped)
    );
    let (popup, xdg, _role) = app.popup(true, 1);
    popup.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&popup);
    app.sync();
    let popup = f
        .backend(|s| s.popup_surfaces().first().map(|p| p.surface.clone()))
        .ok_or("popup missing")?;
    assert_eq!(
        minimize(&f, &popup, false),
        Err(WindowMinimizeError::Unmapped)
    );
    assert!(minimize(&f, &root, true)?);
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    assert_eq!(
        minimize(&f, &root, false),
        Err(WindowMinimizeError::Unmapped)
    );
    assert_eq!(f.backend(|s| s.minimized_surfaces().count()), 0);
    app.configure();
    app.attach();
    app.sync();
    assert_eq!(f.root(), root);
    assert!(!minimize(&f, &root, false)?);
    minimize(&f, &root, true)?;
    drop(app);
    f.wait_for((0, 0));
    assert_eq!(
        minimize(&f, &root, true),
        Err(WindowMinimizeError::Unmapped)
    );
    assert_eq!(f.backend(|s| s.minimized_surfaces().count()), 0);
    Ok(())
}

#[test]
fn window_minimize_dismisses_grabbed_popup_and_its_keyboard_ownership()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_popup_protocol());
    let mut app = mapped(&f);
    let root = f.root();
    f.focus(Some(0))?;
    f.key(30, KeyState::Pressed)?;
    app.sync();
    let (popup, xdg, role) = app.popup(true, 1);
    app.grab_popup(&role, app.events.keyboard.key_serial);
    popup.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&popup);
    app.sync();
    assert!(f.key(31, KeyState::Pressed)?);
    minimize(&f, &root, true)?;
    app.sync();
    assert_eq!(app.events.popups.done, 1);
    assert_eq!(f.backend(|s| s.popup_surfaces().len()), 0);
    assert!(!f.key(31, KeyState::Released)?);
    minimize(&f, &root, false)?;
    assert!(!f.key(32, KeyState::Pressed)?);
    assert_eq!(f.backend(|s| s.popup_surfaces().len()), 0);
    Ok(())
}
