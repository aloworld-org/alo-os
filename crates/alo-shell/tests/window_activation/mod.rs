//! Activation state and recipient isolation over actual Wayland sockets.
use super::{Application, Fixture};
use alo_shell::{InputError, WindowActivationError};
use smithay::{
    backend::input::KeyState, reexports::wayland_server::protocol::wl_surface::WlSurface,
};

fn mapped(f: &Fixture) -> Application {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    app
}

fn activate(f: &Fixture, root: &WlSurface) -> Result<(), WindowActivationError> {
    let root = root.clone();
    f.backend(move |s| s.activate_window(&root))
}

#[test]
fn window_activation_transfers_state_keys_and_order_without_repeated_configures()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let mut first = mapped(&f);
    let mut second = mapped(&f);
    let roots: Vec<_> = f.backend(|s| s.mapped_surfaces().cloned().collect());
    let [a, b] = roots.as_slice() else {
        return Err("two roots required".into());
    };
    activate(&f, a)?;
    first.sync();
    assert_eq!(first.events.activation.last(), Some(&true));
    let count = first.events.activation.len();
    activate(&f, a)?;
    first.sync();
    assert_eq!(first.events.activation.len(), count);
    assert_eq!(f.key(30, KeyState::Pressed).ok(), Some(true));
    activate(&f, b)?;
    first.sync();
    second.sync();
    assert_eq!(first.events.activation.last(), Some(&false));
    assert_eq!(second.events.activation.last(), Some(&true));
    assert_eq!(first.events.keyboard.keys.len(), 2);
    assert!(second.events.keyboard.keys.is_empty());
    assert_eq!(f.root(), *b);
    assert_eq!(f.key(30, KeyState::Released).ok(), Some(false));
    assert_eq!(f.key(31, KeyState::Pressed).ok(), Some(true));
    assert_eq!(f.key(31, KeyState::Released).ok(), Some(true));
    second.sync();
    assert_eq!(second.events.keyboard.keys.len(), 2);
    f.backend(|s| s.keyboard_focus(None))?;
    second.sync();
    assert_eq!(second.events.activation.last(), Some(&false));
    assert_eq!(f.key(32, KeyState::Pressed).ok(), Some(false));
    Ok(())
}

#[test]
fn window_activation_refuses_foreign_unmapped_dead_and_missing_keyboard()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let root = f.root();
    activate(&f, &root)?;
    app.sync();
    let count = app.events.activation.len();
    let foreign = Fixture::keyboard();
    let _foreign_app = mapped(&foreign);
    assert!(matches!(
        activate(&f, &foreign.root()),
        Err(WindowActivationError::Input(InputError::Unmapped))
    ));
    app.sync();
    assert_eq!(app.events.activation.len(), count);
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    assert!(matches!(
        activate(&f, &root),
        Err(WindowActivationError::Input(InputError::Unmapped))
    ));
    assert_eq!(f.key(30, KeyState::Pressed).ok(), Some(false));
    app.configure();
    assert_eq!(app.events.activation.last(), Some(&false));
    app.attach();
    app.sync();
    activate(&f, &root)?;
    app.sync();
    assert_eq!(app.events.activation.last(), Some(&true));
    drop(app);
    f.wait_for((0, 0));
    assert!(matches!(
        activate(&f, &root),
        Err(WindowActivationError::Input(InputError::Unmapped))
    ));
    assert_eq!(f.key(30, KeyState::Pressed).ok(), Some(false));
    let no_keyboard = Fixture::new();
    let mut app = mapped(&no_keyboard);
    let root = no_keyboard.root();
    assert!(matches!(
        activate(&no_keyboard, &root),
        Err(WindowActivationError::Input(InputError::Unavailable))
    ));
    app.sync();
    assert_eq!(app.events.activation, vec![false]);
    Ok(())
}

#[test]
fn window_activation_keeps_grabbed_root_active_and_dismisses_on_transfer()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_popup_protocol());
    let mut first = mapped(&f);
    let mut second = mapped(&f);
    let roots: Vec<_> = f.backend(|s| s.mapped_surfaces().cloned().collect());
    let [a, b] = roots.as_slice() else {
        return Err("two roots required".into());
    };
    activate(&f, a)?;
    assert_eq!(f.key(30, KeyState::Pressed).ok(), Some(true));
    first.sync();
    let before_popup = first.events.activation.len();
    let (popup, xdg, role) = first.popup(true, 20);
    first.grab_popup(&role, first.events.keyboard.key_serial);
    popup.commit();
    first.sync();
    first.ack_popup(&xdg);
    first.attach_popup(&popup);
    first.sync();
    let count = first.events.activation.len();
    assert_eq!(count, before_popup);
    let popup_root = f
        .backend(|s| s.popup_surfaces().into_iter().next().map(|p| p.surface))
        .ok_or("popup required")?;
    assert!(matches!(
        activate(&f, &popup_root),
        Err(WindowActivationError::Input(InputError::Unmapped))
    ));
    activate(&f, a)?;
    first.sync();
    assert_eq!(first.events.activation.len(), count);
    assert_eq!(first.events.activation.last(), Some(&true));
    assert_eq!(first.events.popups.done, 0);
    activate(&f, b)?;
    first.sync();
    second.sync();
    assert_eq!(first.events.popups.done, 1);
    assert_eq!(first.events.activation.last(), Some(&false));
    assert_eq!(second.events.activation.last(), Some(&true));
    Ok(())
}

#[test]
fn window_activation_popup_teardown_restores_root_without_deactivating_it()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_popup_protocol());
    let mut app = mapped(&f);
    activate(&f, &f.root())?;
    assert_eq!(f.key(30, KeyState::Pressed).ok(), Some(true));
    app.sync();
    let count = app.events.activation.len();
    let (popup, xdg, role) = app.popup(true, 20);
    app.grab_popup(&role, app.events.keyboard.key_serial);
    popup.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&popup);
    app.sync();
    role.destroy();
    xdg.destroy();
    popup.destroy();
    app.sync();
    assert_eq!(app.events.activation.len(), count);
    assert_eq!(app.events.activation.last(), Some(&true));
    assert_eq!(f.key(31, KeyState::Pressed).ok(), Some(true));
    assert_eq!(f.key(31, KeyState::Released).ok(), Some(true));
    app.sync();
    use wayland_client::Proxy;
    assert_eq!(
        app.events.keyboard.surfaces.last(),
        Some(&app.surface.id().protocol_id())
    );
    // A committed acknowledgement does not resurrect activation after focus loss.
    app.xdg
        .ack_configure(app.events.serial.ok_or("configure required")?);
    app.surface.commit();
    app.sync();
    f.backend(|s| s.keyboard_focus(None))?;
    app.sync();
    assert_eq!(app.events.activation.last(), Some(&false));
    Ok(())
}
