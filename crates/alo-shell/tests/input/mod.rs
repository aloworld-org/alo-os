//! Real keyboard wire routing and refusal, through the production server.
use super::support::{Application, Fixture};
use smithay::backend::input::KeyState::{Pressed, Released};
use wayland_client::protocol::{wl_keyboard::KeyState, wl_seat::Capability};

#[test]
fn keyboard_keymap_focus_and_keys_are_isolated_between_clients() {
    let fixture = Fixture::keyboard();
    let mut first = Application::new(&fixture);
    let mut second = Application::new(&fixture);
    first.sync();
    assert_eq!(
        first.events.keyboard.capabilities,
        Some(Capability::Keyboard)
    );
    assert!(first.events.keyboard.keymap.starts_with("xkb_keymap"));
    assert_eq!(first.events.keyboard.repeat, Some((25, 600)));
    assert_eq!(fixture.key(30, Pressed).ok(), Some(false));
    first.configure();
    first.attach();
    first.sync();
    second.configure();
    second.attach();
    second.sync();
    fixture.wait_for((2, 2));
    assert!(fixture.focus(Some(0)).is_ok());
    assert_eq!(fixture.key(42, Pressed).ok(), Some(true));
    assert_eq!(fixture.key(30, Pressed).ok(), Some(true));
    assert_eq!(fixture.key(30, Pressed).ok(), Some(false));
    first.sync();
    second.sync();
    assert_eq!(first.events.keyboard.enters, [Vec::<u8>::new()]);
    assert_eq!(
        first.events.keyboard.keys,
        [(42, KeyState::Pressed), (30, KeyState::Pressed)]
    );
    assert!(
        first
            .events
            .keyboard
            .modifiers
            .iter()
            .any(|mask| *mask != 0)
    );
    assert!(second.events.keyboard.enters.is_empty());
    assert!(second.events.keyboard.keys.is_empty());
    assert!(fixture.focus(Some(1)).is_ok());
    first.sync();
    second.sync();
    assert_eq!(first.events.keyboard.leaves, 1);
    assert_eq!(first.events.keyboard.keys.len(), 4);
    assert!(
        first
            .events
            .keyboard
            .keys
            .contains(&(30, KeyState::Released))
    );
    assert!(
        first
            .events
            .keyboard
            .keys
            .contains(&(42, KeyState::Released))
    );
    assert_eq!(second.events.keyboard.enters, [Vec::<u8>::new()]);
    assert_eq!(second.events.keyboard.modifiers.last(), Some(&0));
    assert_eq!(fixture.key(30, Released).ok(), Some(false));
    assert_eq!(fixture.key(30, Pressed).ok(), Some(true));
    assert!(fixture.focus(None).is_ok());
    assert_eq!(fixture.key(31, Pressed).ok(), Some(false));
    second.sync();
    assert_eq!(
        second.events.keyboard.keys,
        [(30, KeyState::Pressed), (30, KeyState::Released)]
    );
    assert_eq!(second.events.keyboard.leaves, 1);
    assert!(fixture.focus(Some(1)).is_ok());
    second.sync();
    assert_eq!(second.events.keyboard.enters.last(), Some(&vec![]));
}

#[test]
fn unmap_and_disconnect_clear_focus_without_redirecting_keys() {
    let fixture = Fixture::keyboard();
    let mut first = Application::new(&fixture);
    let mut second = Application::new(&fixture);
    first.configure();
    first.attach();
    first.sync();
    second.configure();
    second.attach();
    second.sync();
    assert!(fixture.focus(Some(0)).is_ok());
    assert_eq!(fixture.key(42, Pressed).ok(), Some(true));
    first.surface.attach(None, 0, 0);
    first.surface.commit();
    first.sync();
    fixture.wait_for((2, 1));
    assert_eq!(fixture.key(30, Pressed).ok(), Some(false));
    first.sync();
    second.sync();
    assert_eq!(first.events.keyboard.leaves, 1);
    assert!(second.events.keyboard.keys.is_empty());
    assert!(fixture.focus(Some(0)).is_ok());
    assert_eq!(fixture.key(42, Pressed).ok(), Some(true));
    drop(second);
    fixture.wait_for((1, 0));
    assert_eq!(fixture.key(30, Pressed).ok(), Some(false));
    first.configure();
    first.attach();
    first.sync();
    assert!(fixture.focus(Some(0)).is_ok());
    first.sync();
    assert_eq!(first.events.keyboard.enters.last(), Some(&vec![]));
    assert_eq!(first.events.keyboard.modifiers.last(), Some(&0));
    assert_eq!(fixture.key(30, Pressed).ok(), Some(true));
    assert_eq!(fixture.key(30, Released).ok(), Some(true));
}

#[test]
fn absent_keyboard_and_invalid_codes_refuse() {
    let fixture = Fixture::new();
    assert!(matches!(
        fixture.key(30, Pressed),
        Err(alo_shell::InputError::Unavailable)
    ));
    assert!(matches!(
        fixture.focus(None),
        Err(alo_shell::InputError::Unavailable)
    ));
    let fixture = Fixture::keyboard();
    for code in [0, 0x300, u32::MAX] {
        assert!(matches!(
            fixture.key(code, Pressed),
            Err(alo_shell::InputError::InvalidKey)
        ));
    }
}

#[test]
fn stale_foreign_and_destroyed_focus_targets_refuse() {
    let fixture = Fixture::keyboard();
    let foreign = Fixture::keyboard();
    let mut app = Application::new(&fixture);
    app.configure();
    app.attach();
    app.sync();
    let root = fixture.root();
    assert!(matches!(
        foreign.focus_surface(root.clone()),
        Err(alo_shell::InputError::Unmapped)
    ));
    assert!(fixture.focus_surface(root.clone()).is_ok());
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    assert!(matches!(
        fixture.focus_surface(root.clone()),
        Err(alo_shell::InputError::Unmapped)
    ));
    app.configure();
    app.attach();
    app.sync();
    assert!(fixture.focus_surface(root.clone()).is_ok());
    assert_eq!(fixture.key(30, Pressed).ok(), Some(true));
    app.toplevel.destroy();
    app.xdg.destroy();
    app.surface.destroy();
    app.sync();
    fixture.wait_for((0, 0));
    assert!(matches!(
        fixture.focus_surface(root),
        Err(alo_shell::InputError::Unmapped)
    ));
    assert_eq!(fixture.key(30, Pressed).ok(), Some(false));
}

#[test]
fn invalid_keymap_refuses_and_removes_the_owned_socket() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;
    let runtime = tempfile::tempdir()?;
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))?;
    let result = alo_shell::Server::bind_keyboard(
        runtime.path(),
        "invalid",
        smithay::input::keyboard::XkbConfig {
            layout: "alo-nonexistent-layout",
            ..Default::default()
        },
    );
    assert!(matches!(result, Err(alo_shell::InputError::Keymap(_))));
    assert!(!runtime.path().join("invalid").exists());
    assert!(alo_shell::Server::bind(runtime.path(), "invalid").is_ok());
    Ok(())
}
