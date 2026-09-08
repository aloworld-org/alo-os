//! Trusted cycling exercised through real XDG clients and keyboard events.
use super::{Application, Fixture};
use alo_shell::{
    InputError, WindowActivationError, WindowSwitchDirection as Direction, WindowSwitchError,
};
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

fn switch(f: &Fixture, direction: Direction) -> Result<WlSurface, WindowSwitchError> {
    f.backend(move |server| server.switch_window(direction))
}

#[test]
fn window_switch_visits_three_roots_in_both_directions_despite_raising()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let mut first = mapped(&f);
    let mut second = mapped(&f);
    let mut third = mapped(&f);
    let roots: Vec<_> = f.backend(|s| s.mapped_surfaces().cloned().collect());
    let [a, b, c] = roots.as_slice() else {
        return Err("three roots required".into());
    };
    for expected in [a, b, c, a] {
        assert_eq!(switch(&f, Direction::Forward)?, *expected);
        assert_eq!(f.root(), *expected);
    }
    for expected in [c, b, a, c] {
        assert_eq!(switch(&f, Direction::Backward)?, *expected);
        assert_eq!(f.root(), *expected);
    }
    first.sync();
    second.sync();
    third.sync();
    assert_eq!(first.events.activation.last(), Some(&false));
    assert_eq!(second.events.activation.last(), Some(&false));
    assert_eq!(third.events.activation.last(), Some(&true));
    // Explicit focus, not the last cycling selection or presentation order, anchors selection.
    let root = a.clone();
    f.backend(move |s| s.keyboard_focus(Some(&root)))?;
    assert_eq!(switch(&f, Direction::Forward)?, *b);
    f.backend(|s| s.keyboard_focus(None))?;
    assert_eq!(switch(&f, Direction::Backward)?, *c);
    Ok(())
}

#[test]
fn window_switch_refuses_empty_and_missing_keyboard_without_activation()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    for direction in [Direction::Forward, Direction::Backward] {
        assert!(matches!(
            switch(&f, direction),
            Err(WindowSwitchError::Empty)
        ));
    }
    let no_keyboard = Fixture::new();
    let mut app = mapped(&no_keyboard);
    let original = no_keyboard.root();
    assert!(matches!(
        switch(&no_keyboard, Direction::Forward),
        Err(WindowSwitchError::Activation(WindowActivationError::Input(
            InputError::Unavailable
        )))
    ));
    app.sync();
    assert_eq!(app.events.activation, [false]);
    assert_eq!(no_keyboard.root(), original);
    Ok(())
}

#[test]
fn window_switch_discards_unmapped_and_disconnected_roots_and_appends_remaps()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let mut first = mapped(&f);
    let mut second = mapped(&f);
    let third = mapped(&f);
    let roots: Vec<_> = f.backend(|s| s.mapped_surfaces().cloned().collect());
    let [a, b, c] = roots.as_slice() else {
        return Err("three roots required".into());
    };
    assert_eq!(switch(&f, Direction::Forward)?, *a);
    second.surface.attach(None, 0, 0);
    second.surface.commit();
    second.sync();
    assert_eq!(switch(&f, Direction::Forward)?, *c);
    second.configure();
    second.attach();
    second.sync();
    assert_eq!(switch(&f, Direction::Forward)?, *b);
    drop(third);
    f.wait_for((2, 2));
    assert_eq!(switch(&f, Direction::Backward)?, *a);
    drop(second);
    f.wait_for((1, 1));
    first.sync();
    let count = first.events.activation.len();
    assert_eq!(switch(&f, Direction::Forward)?, *a);
    first.sync();
    assert_eq!(first.events.activation.len(), count);
    drop(first);
    f.wait_for((0, 0));
    assert!(matches!(
        switch(&f, Direction::Forward),
        Err(WindowSwitchError::Empty)
    ));
    Ok(())
}

#[test]
fn window_switch_anchors_popup_ownership_and_isolates_held_keys()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_popup_protocol());
    let mut first = mapped(&f);
    let a = switch(&f, Direction::Forward)?;
    assert_eq!(f.key(30, KeyState::Pressed).ok(), Some(true));
    first.sync();
    let (popup, xdg, role) = first.popup(true, 20);
    first.grab_popup(&role, first.events.keyboard.key_serial);
    popup.commit();
    first.sync();
    first.ack_popup(&xdg);
    first.attach_popup(&popup);
    first.sync();
    assert_eq!(switch(&f, Direction::Forward)?, a);
    first.sync();
    assert_eq!(first.events.popups.done, 0);
    assert_eq!(f.key(31, KeyState::Pressed).ok(), Some(true));
    let mut second = mapped(&f);
    let b = f
        .backend(move |s| s.mapped_surfaces().find(|root| **root != a).cloned())
        .ok_or("second root required")?;
    assert_eq!(switch(&f, Direction::Forward)?, b);
    first.sync();
    second.sync();
    assert_eq!(first.events.popups.done, 1);
    assert_eq!(first.events.activation.last(), Some(&false));
    assert_eq!(second.events.activation.last(), Some(&true));
    assert!(second.events.keyboard.keys.is_empty());
    assert!(second.events.keyboard.enters.iter().all(Vec::is_empty));
    assert!(first.events.keyboard.keys.contains(&(
        31,
        wayland_client::protocol::wl_keyboard::KeyState::Released,
    )));
    assert_eq!(f.key(31, KeyState::Released).ok(), Some(false));
    assert_eq!(f.key(32, KeyState::Pressed).ok(), Some(true));
    assert_eq!(f.key(32, KeyState::Released).ok(), Some(true));
    second.sync();
    assert_eq!(second.events.keyboard.keys.len(), 2);
    Ok(())
}
