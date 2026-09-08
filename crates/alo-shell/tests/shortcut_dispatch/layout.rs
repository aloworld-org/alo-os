//! Focus ownership, layout transactions and refusal through configured commands.
use super::{Action, Changes, Chord, Fixture, Key, Modifier, Modifiers, Shortcuts, mapped};
use alo_shell::{
    InputError, TileGeometryError, WindowCommandError, WindowMaximizeError, WindowModeError,
};
use smithay::backend::input::KeyState;
use wayland_client::protocol::wl_keyboard;

fn dispatch(
    f: &Fixture,
    settings: &Shortcuts,
    chord: Chord,
) -> Result<Option<Action>, WindowCommandError> {
    let settings = settings.clone();
    f.backend(move |s| s.dispatch_window_command(&settings, chord))
}

fn command(f: &Fixture, action: Action) -> Result<Option<Action>, WindowCommandError> {
    let settings = Shortcuts::shipped();
    // Use one custom binding for every action to exercise current settings resolution.
    let chord = Chord::checked(
        Modifiers::just(Modifier::Ctrl).and(Modifier::Alt),
        Key::Space,
    )
    .map_err(|_| {
        WindowCommandError::Shortcut(alo_shell::ShortcutDispatchError::Unsupported(action))
    })?;
    let mut changes = Changes::none();
    changes.set(action, Some(chord));
    dispatch(f, &settings.with(changes), chord)
}

const ACTIONS: [Action; 4] = [
    Action::MinimiseWindow,
    Action::MaximiseWindow,
    Action::SnapLeft,
    Action::SnapRight,
];

#[test]
fn shortcut_dispatch_command_preserves_legacy_errors_and_delegates_close_and_cycle()
-> Result<(), Box<dyn std::error::Error>> {
    use alo_shell::ShortcutDispatchError::{Close, Input, NoFocusedWindow, Switch, Unsupported};
    // This exhaustive match is an external caller's source-compatibility check.
    let code = |error| match error {
        Close(_) => 0,
        Input(_) => 1,
        NoFocusedWindow => 2,
        Switch(_) => 3,
        Unsupported(_) => 4,
    };
    assert_eq!(code(NoFocusedWindow), 2);
    let f = Fixture::keyboard();
    assert!(matches!(
        command(&f, Action::NextWindow),
        Err(WindowCommandError::Shortcut(Switch(
            alo_shell::WindowSwitchError::Empty
        )))
    ));
    let mut app = mapped(&f);
    command(&f, Action::NextWindow)?;
    command(&f, Action::CloseWindow)?;
    app.sync();
    assert_eq!(app.events.close_requests, 1);
    for action in [
        Action::TheAgent,
        Action::Launcher,
        Action::NextApplication,
        Action::PreviousApplication,
    ] {
        assert!(
            matches!(command(&f, action), Err(WindowCommandError::Shortcut(Unsupported(found))) if found == action)
        );
    }
    Ok(())
}

#[test]
fn shortcut_dispatch_layout_conflicts_and_cleared_bindings_do_not_change_clients()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    f.focus(Some(0))?;
    f.render((33, 32), false, 1)?;
    app.sync();
    let count = app.events.sizes.len();
    for action in ACTIONS {
        let settings = Shortcuts::shipped();
        let chord = settings.chord_for(action).ok_or("missing layout binding")?;
        let mut changes = Changes::none();
        changes.set(action, Some(chord));
        changes.set(Action::CloseWindow, Some(chord));
        assert_eq!(dispatch(&f, &settings.clone().with(changes), chord)?, None);
        let mut cleared = settings;
        cleared.unbind(action);
        assert_eq!(dispatch(&f, &cleared, chord)?, None);
    }
    app.sync();
    assert_eq!(app.events.sizes.len(), count);
    assert_eq!(app.events.close_requests, 0);
    assert_eq!(f.backend(|s| s.minimized_surfaces().count()), 0);
    assert!(f.key(30, KeyState::Pressed)?);
    assert!(f.key(30, KeyState::Released)?);
    Ok(())
}

#[test]
fn shortcut_dispatch_layout_refuses_absent_focus_and_seat_without_stack_fallback()
-> Result<(), Box<dyn std::error::Error>> {
    let absent = Fixture::new();
    let mut app = mapped(&absent);
    let f = Fixture::keyboard();
    let mut unfocused = mapped(&f);
    for action in ACTIONS {
        assert!(matches!(
            command(&absent, action),
            Err(WindowCommandError::Input(InputError::Unavailable))
        ));
        assert!(matches!(
            command(&f, action),
            Err(WindowCommandError::NoFocusedWindow)
        ));
    }
    app.sync();
    unfocused.sync();
    assert_eq!(app.events.sizes, [(0, 0)]);
    assert_eq!(unfocused.events.sizes, [(0, 0)]);
    assert_eq!(f.backend(|s| s.mapped_surfaces().count()), 1);
    Ok(())
}

#[test]
fn shortcut_dispatch_layout_toggles_pending_intent_and_preserves_normal_geometry()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let root = f.root();
    let target = root.clone();
    f.backend(move |s| s.place_window(&target, (8, 9)))?;
    f.focus(Some(0))?;
    f.render((33, 32), false, 1)?;
    app.sync();
    for maximized in [true, false, true, false] {
        assert_eq!(
            command(&f, Action::MaximiseWindow)?,
            Some(Action::MaximiseWindow)
        );
        app.sync();
        assert_eq!(app.events.maximized.last(), Some(&maximized));
        assert_eq!(
            app.events.sizes.last(),
            Some(&if maximized { (33, 32) } else { (16, 16) })
        );
    }
    // No acknowledgements above: a toggle must follow requested, not committed state.
    app.xdg
        .ack_configure(app.events.serial.ok_or("missing restore")?);
    app.surface.commit();
    app.sync();
    let target = root.clone();
    assert_eq!(
        f.backend(move |_| alo_shell::window_buffer_origin(&target)),
        (8.0, 9.0).into()
    );
    for (action, side, origin) in [
        (Action::SnapRight, alo_shell::TileSide::Right, (16.0, 0.0)),
        (Action::SnapLeft, alo_shell::TileSide::Left, (0.0, 0.0)),
    ] {
        assert_eq!(command(&f, action)?, Some(action));
        app.sync();
        app.xdg
            .ack_configure(app.events.serial.ok_or("missing tile")?);
        app.attach_tiled(side);
        app.sync();
        let target = root.clone();
        assert_eq!(
            f.backend(move |_| alo_shell::window_buffer_origin(&target)),
            origin.into()
        );
        assert!(
            !app.events
                .tiled
                .last()
                .ok_or("missing tile flags")?
                .is_empty()
        );
    }
    command(&f, Action::MaximiseWindow)?; // Tiled -> maximized, not normal.
    app.sync();
    assert_eq!(app.events.maximized.last(), Some(&true));
    command(&f, Action::MaximiseWindow)?;
    app.sync();
    assert_eq!(app.events.sizes.last(), Some(&(16, 16)));
    app.xdg
        .ack_configure(app.events.serial.ok_or("missing normal")?);
    app.attach();
    app.sync();
    assert_eq!(
        f.backend(move |_| alo_shell::window_buffer_origin(&root)),
        (8.0, 9.0).into()
    );
    assert!(f.key(30, KeyState::Pressed)?);
    assert!(f.key(30, KeyState::Released)?);
    app.sync();
    assert_eq!(
        app.events.keyboard.keys,
        [
            (30, wl_keyboard::KeyState::Pressed),
            (30, wl_keyboard::KeyState::Released)
        ]
    );
    Ok(())
}

#[test]
fn shortcut_dispatch_layout_output_and_limit_refusals_keep_wire_state()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    f.focus(Some(0))?;
    app.sync();
    let count = app.events.sizes.len();
    assert!(matches!(
        command(&f, Action::MaximiseWindow),
        Err(WindowCommandError::Maximize(
            WindowMaximizeError::OutputUnavailable
        ))
    ));
    for action in [Action::SnapLeft, Action::SnapRight] {
        assert!(matches!(
            command(&f, action),
            Err(WindowCommandError::Tile(WindowModeError::Tile(
                TileGeometryError::OutputUnavailable
            )))
        ));
    }
    app.sync();
    assert_eq!(app.events.sizes.len(), count);
    f.render((33, 32), false, 1)?;
    app.toplevel.set_min_size(20, 1);
    app.surface.commit();
    app.sync();
    let count = app.events.sizes.len();
    for action in [Action::SnapLeft, Action::SnapRight] {
        assert!(matches!(
            command(&f, action),
            Err(WindowCommandError::Tile(WindowModeError::Tile(
                TileGeometryError::ClientLimits
            )))
        ));
    }
    app.sync();
    assert_eq!(app.events.sizes.len(), count);
    assert!(f.key(30, KeyState::Pressed)?);
    assert!(f.key(30, KeyState::Released)?);
    Ok(())
}

#[test]
fn shortcut_dispatch_minimize_uses_focus_and_does_not_retarget_hidden_or_dead_windows()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let mut first = mapped(&f);
    let root = f.root();
    let mut second = mapped(&f);
    f.focus(Some(0))?; // Focus differs from the last/upper root.
    assert!(f.key(30, KeyState::Pressed)?);
    command(&f, Action::MinimiseWindow)?;
    first.sync();
    second.sync();
    assert_eq!(
        first.events.keyboard.keys,
        [
            (30, wl_keyboard::KeyState::Pressed),
            (30, wl_keyboard::KeyState::Released)
        ]
    );
    assert!(second.events.keyboard.keys.is_empty());
    assert_eq!(f.backend(|s| s.minimized_surfaces().count()), 1);
    for action in ACTIONS {
        assert!(matches!(
            command(&f, action),
            Err(WindowCommandError::NoFocusedWindow)
        ));
    }
    assert!(!f.key(30, KeyState::Released)?);
    let target = root.clone();
    f.backend(move |s| s.set_window_minimized(&target, false))?;
    f.focus(Some(0))?;
    first.surface.attach(None, 0, 0);
    first.surface.commit();
    first.sync();
    assert!(matches!(
        command(&f, Action::MaximiseWindow),
        Err(WindowCommandError::NoFocusedWindow)
    ));
    first.configure();
    first.attach();
    first.sync();
    f.focus_surface(root)?;
    f.render((33, 32), false, 1)?;
    command(&f, Action::MaximiseWindow)?;
    first.sync();
    assert_eq!(first.events.maximized.last(), Some(&true));
    drop(first);
    second.sync();
    assert!(matches!(
        command(&f, Action::SnapLeft),
        Err(WindowCommandError::NoFocusedWindow)
    ));
    Ok(())
}

#[test]
fn shortcut_dispatch_layout_popup_grab_refuses_modes_but_minimize_retires_it()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_popup_protocol());
    let mut app = mapped(&f);
    f.focus(Some(0))?;
    f.render((33, 32), false, 1)?;
    assert!(f.key(30, KeyState::Pressed)?);
    app.sync();
    let (surface, xdg, role) = app.popup(true, 1);
    app.grab_popup(&role, app.events.keyboard.key_serial);
    surface.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&surface);
    app.sync();
    let count = app.events.sizes.len();
    assert!(matches!(
        command(&f, Action::MaximiseWindow),
        Err(WindowCommandError::Maximize(WindowMaximizeError::Busy))
    ));
    for action in [Action::SnapLeft, Action::SnapRight] {
        assert!(matches!(
            command(&f, action),
            Err(WindowCommandError::Tile(WindowModeError::Busy))
        ));
    }
    app.sync();
    assert_eq!(app.events.sizes.len(), count);
    assert_eq!(app.events.popups.done, 0);
    command(&f, Action::MinimiseWindow)?;
    app.sync();
    assert_eq!(app.events.popups.done, 1);
    assert_eq!(f.backend(|s| s.mapped_surfaces().count()), 0);
    Ok(())
}
