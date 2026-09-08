//! Configured command dispatch over real client sockets, independent of raw keys.
use super::{Application, Fixture};
use alo_shell::{InputError, ShortcutDispatchError, WindowSwitchError};
use alo_shortcuts::{Action, Changes, Chord, Key, Modifier, Modifiers, Shortcuts};

/// Create a mapped root with a connected client to observe protocol effects.
fn mapped(f: &Fixture) -> Application {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    app
}

/// Run on the server thread with the current settings snapshot.
fn dispatch(
    f: &Fixture,
    settings: &Shortcuts,
    chord: Chord,
) -> Result<Option<Action>, ShortcutDispatchError> {
    let settings = settings.clone();
    f.backend(move |s| s.dispatch_window_shortcut(&settings, chord))
}

#[test]
fn shortcut_dispatch_rebinding_and_clearing_apply_on_the_next_request()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let mut first = mapped(&f);
    let a = f.root();
    let mut second = mapped(&f);
    let mut settings = Shortcuts::shipped();
    let old = settings
        .chord_for(Action::NextWindow)
        .ok_or("missing default")?;
    let custom = Chord::checked(
        Modifiers::just(Modifier::Ctrl).and(Modifier::Alt),
        Key::Space,
    )
    .map_err(|_| "invalid chord")?;
    settings
        .bind(Action::NextWindow, custom)
        .map_err(|_| "binding refused")?;
    assert_eq!(dispatch(&f, &settings, old)?, None);
    assert_eq!(dispatch(&f, &settings, custom)?, Some(Action::NextWindow));
    assert_eq!(f.root(), a);
    assert_eq!(dispatch(&f, &settings, custom)?, Some(Action::NextWindow));
    assert_ne!(f.root(), a);
    let reverse = settings
        .chord_for(Action::PreviousWindow)
        .ok_or("missing reverse")?;
    assert_eq!(
        dispatch(&f, &settings, reverse)?,
        Some(Action::PreviousWindow)
    );
    assert_eq!(f.root(), a);
    settings.unbind(Action::NextWindow);
    assert_eq!(dispatch(&f, &settings, custom)?, None);
    first.sync();
    second.sync();
    assert_eq!(first.events.activation.last(), Some(&true));
    assert_eq!(second.events.activation.last(), Some(&false));
    assert_eq!(
        first.events.close_requests + second.events.close_requests,
        0
    );
    Ok(())
}

#[test]
fn shortcut_dispatch_closes_actual_focus_once_without_stack_fallback()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let mut first = mapped(&f);
    let root = f.root();
    let mut second = mapped(&f);
    let settings = Shortcuts::shipped();
    let close = settings
        .chord_for(Action::CloseWindow)
        .ok_or("missing close")?;
    assert!(matches!(
        dispatch(&f, &settings, close),
        Err(ShortcutDispatchError::NoFocusedWindow)
    ));
    let other = f
        .backend(|s| s.mapped_surfaces().nth(1).cloned())
        .ok_or("missing second")?;
    f.focus_surface(root)?;
    f.backend(move |s| s.raise_window(&other))?;
    assert_eq!(dispatch(&f, &settings, close)?, Some(Action::CloseWindow));
    first.sync();
    second.sync();
    assert_eq!(first.events.close_requests, 1);
    assert_eq!(second.events.close_requests, 0);
    f.wait_for((2, 2));
    f.focus(None)?;
    assert!(matches!(
        dispatch(&f, &settings, close),
        Err(ShortcutDispatchError::NoFocusedWindow)
    ));
    first.sync();
    assert_eq!(first.events.close_requests, 1);
    Ok(())
}

#[test]
fn shortcut_dispatch_conflicts_and_unimplemented_actions_do_not_change_clients()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let settings = Shortcuts::shipped();
    let close = settings
        .chord_for(Action::CloseWindow)
        .ok_or("missing close")?;
    let mut changes = Changes::none();
    changes.set(Action::CloseWindow, Some(close));
    changes.set(Action::NextWindow, Some(close));
    assert_eq!(dispatch(&f, &settings.clone().with(changes), close)?, None);
    for action in Action::ALL.iter().copied().filter(|a| {
        !matches!(
            a,
            Action::NextWindow | Action::PreviousWindow | Action::CloseWindow
        )
    }) {
        let mut changes = Changes::none();
        changes.set(Action::CloseWindow, None);
        changes.set(action, Some(close));
        assert!(
            matches!(dispatch(&f, &settings.clone().with(changes), close), Err(ShortcutDispatchError::Unsupported(found)) if found == action)
        );
    }
    app.sync();
    assert_eq!(app.events.close_requests, 0);
    assert_eq!(app.events.activation, [false]);
    Ok(())
}

#[test]
fn shortcut_dispatch_retains_empty_and_missing_keyboard_errors()
-> Result<(), Box<dyn std::error::Error>> {
    let settings = Shortcuts::shipped();
    let next = settings
        .chord_for(Action::NextWindow)
        .ok_or("missing next")?;
    let close = settings
        .chord_for(Action::CloseWindow)
        .ok_or("missing close")?;
    let f = Fixture::keyboard();
    assert!(matches!(
        dispatch(&f, &settings, next),
        Err(ShortcutDispatchError::Switch(WindowSwitchError::Empty))
    ));
    let absent = Fixture::new();
    assert!(matches!(
        dispatch(&absent, &settings, close),
        Err(ShortcutDispatchError::Input(InputError::Unavailable))
    ));
    Ok(())
}

#[test]
fn shortcut_dispatch_close_with_popup_focus_preserves_the_grab()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_popup_protocol());
    let mut app = mapped(&f);
    f.focus(Some(0))?;
    f.key(30, smithay::backend::input::KeyState::Pressed)?;
    app.sync();
    let serial = app.events.keyboard.key_serial;
    let (surface, xdg, role) = app.popup(true, 1);
    app.grab_popup(&role, serial);
    surface.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&surface);
    app.sync();
    let settings = Shortcuts::shipped();
    let close = settings
        .chord_for(Action::CloseWindow)
        .ok_or("missing close")?;
    assert_eq!(dispatch(&f, &settings, close)?, Some(Action::CloseWindow));
    app.sync();
    assert_eq!(app.events.close_requests, 1);
    assert_eq!(app.events.popups.done, 0);
    assert_eq!(
        f.key(48, smithay::backend::input::KeyState::Pressed).ok(),
        Some(true)
    );
    role.destroy();
    app.sync();
    Ok(())
}
