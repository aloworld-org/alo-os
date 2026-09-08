//! Actual tile/restore requests through a real Wayland connection.
use super::{Application, Fixture, RetireTarget, TileSide, WlSurface, mapped};
use alo_shell::{TileGeometryError, WindowModeError};
mod lifetime;

fn tile(
    f: &Fixture,
    root: &WlSurface,
    side: Option<TileSide>,
) -> Result<Option<u32>, WindowModeError> {
    let root = root.clone();
    f.backend(move |s| s.set_window_tiled(&root, side).map(|v| v.map(u32::from)))
}
fn maximize(
    f: &Fixture,
    root: &WlSurface,
    value: bool,
) -> Result<Option<u32>, alo_shell::WindowMaximizeError> {
    let root = root.clone();
    f.backend(move |s| {
        s.set_window_maximized(&root, value)
            .map(|v| v.map(u32::from))
    })
}
fn origin(f: &Fixture, root: &WlSurface) -> (i32, i32) {
    let root = root.clone();
    f.backend(move |_| {
        let p = alo_shell::window_buffer_origin(&root);
        (p.x as i32, p.y as i32)
    })
}
fn place(f: &Fixture, root: &WlSurface, point: (i32, i32)) {
    let root = root.clone();
    assert!(f.backend(move |s| s.place_window(&root, point)).is_ok());
}
fn ack(app: &mut Application) -> Result<(), Box<dyn std::error::Error>> {
    app.sync();
    app.xdg
        .ack_configure(app.events.serial.ok_or("missing configure")?);
    app.sync();
    Ok(())
}
fn flags(app: &Application, tiled: bool) -> Result<(), Box<dyn std::error::Error>> {
    use wayland_protocols::xdg::shell::client::xdg_toplevel::State;
    let mut actual = app
        .events
        .tiled
        .last()
        .cloned()
        .ok_or("missing configure flags")?;
    actual.sort();
    let mut expected = if tiled {
        vec![
            State::TiledLeft as u32,
            State::TiledRight as u32,
            State::TiledTop as u32,
            State::TiledBottom as u32,
        ]
    } else {
        vec![]
    };
    expected.sort();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn window_tiling_transaction_missing_flags_are_a_failure() {
    let f = Fixture::new();
    let app = Application::new(&f);
    assert!(flags(&app, false).is_err());
    assert!(flags(&app, true).is_err());
}

#[test]
fn window_tiling_transaction_exact_half_buffers_and_output_limit_suspension()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    place(&f, &root, (8, 9));
    f.render((33, 32), false, 1)?;
    for (side, expected) in [(TileSide::Right, (16, 0)), (TileSide::Left, (0, 0))] {
        tile(&f, &root, Some(side))?;
        ack(&mut app)?;
        app.attach_tiled(side);
        app.sync();
        flags(&app, true)?;
        assert_eq!(origin(&f, &root), expected);
    }
    app.toplevel.set_max_size(17, 32);
    app.surface.commit();
    app.sync();
    let count = app.events.sizes.len();
    f.render((81, 60), false, 2)?;
    app.sync();
    assert_eq!(app.events.sizes.len(), count); // Output cannot authorize impossible exact size.
    app.attach_resized();
    app.sync();
    assert_eq!(origin(&f, &root), (0, 0));
    assert_eq!(
        tile(&f, &root, Some(TileSide::Right)),
        Err(WindowModeError::Tile(TileGeometryError::ClientLimits))
    );
    f.render((33, 32), false, 3)?;
    ack(&mut app)?;
    app.attach_tiled(TileSide::Left);
    app.sync();
    tile(&f, &root, None)?;
    ack(&mut app)?;
    app.attach();
    app.sync();
    assert_eq!(origin(&f, &root), (8, 9));
    Ok(())
}

#[test]
fn window_tiling_transaction_waits_for_commit_anchors_actual_geometry_and_preserves_keyboard()
-> Result<(), Box<dyn std::error::Error>> {
    use smithay::backend::input::KeyState;
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let root = f.root();
    let mut other = mapped(&f);
    place(&f, &root, (8, 9));
    f.focus_surface(root.clone())?;
    f.render((81, 60), false, 1)?;
    app.sync();
    other.sync();
    let order = f.backend(|s| s.mapped_surfaces().cloned().collect::<Vec<_>>());
    let count = other.events.sizes.len();
    tile(&f, &root, Some(TileSide::Right))?.ok_or("missing tile")?;
    app.sync();
    assert_eq!(app.events.sizes.last(), Some(&(41, 60)));
    assert_eq!(app.events.maximized.last(), Some(&false));
    assert_eq!(app.events.activation.last(), Some(&true));
    flags(&app, true)?;
    assert_eq!(tile(&f, &root, Some(TileSide::Right)), Ok(None));
    assert_eq!(origin(&f, &root), (8, 9));
    assert!(f.key(30, KeyState::Pressed)?);
    assert!(f.key(30, KeyState::Released)?);
    app.sync();
    other.sync();
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert!(other.events.keyboard.keys.is_empty());
    assert_eq!(other.events.sizes.len(), count);
    assert_eq!(
        f.backend(|s| s.mapped_surfaces().cloned().collect::<Vec<_>>()),
        order
    );
    ack(&mut app)?;
    assert_eq!(origin(&f, &root), (8, 9));
    app.attach_resized();
    app.sync(); // Actual 32x24, deliberately smaller than suggestion.
    assert_eq!(origin(&f, &root), (49, 0));
    app.xdg.set_window_geometry(4, 5, 20, 15);
    app.surface.commit();
    app.sync();
    assert_eq!(origin(&f, &root), (57, -5)); // Right edge of geometry, excluding shadow.
    tile(&f, &root, None)?;
    ack(&mut app)?;
    flags(&app, false)?;
    assert_eq!(app.events.sizes.last(), Some(&(16, 16)));
    assert_eq!(origin(&f, &root), (57, -5));
    app.xdg.set_window_geometry(0, 0, 16, 16);
    app.attach();
    app.sync();
    assert_eq!(origin(&f, &root), (8, 9));
    place(&f, &root, (1, 2));
    Ok(())
}

#[test]
fn window_tiling_transaction_rapid_modes_share_original_normal_geometry()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    place(&f, &root, (11, 12));
    f.render((81, 60), false, 1)?;
    let old = tile(&f, &root, Some(TileSide::Right))?.ok_or("tile")?;
    tile(&f, &root, Some(TileSide::Left))?;
    app.sync();
    app.xdg.ack_configure(old);
    app.attach_resized();
    app.sync();
    assert_eq!(origin(&f, &root), (11, 12));
    ack(&mut app)?;
    app.surface.commit();
    app.sync();
    assert_eq!(origin(&f, &root), (0, 0));
    maximize(&f, &root, true)?;
    ack(&mut app)?;
    flags(&app, false)?;
    assert_eq!(app.events.maximized.last(), Some(&true));
    app.surface.commit();
    app.sync();
    tile(&f, &root, Some(TileSide::Right))?;
    ack(&mut app)?;
    flags(&app, true)?;
    assert_eq!(app.events.maximized.last(), Some(&false));
    app.surface.commit();
    app.sync();
    assert_eq!(origin(&f, &root), (49, 0));
    let restore = tile(&f, &root, None)?.ok_or("restore")?;
    maximize(&f, &root, true)?;
    app.sync();
    app.xdg.ack_configure(restore);
    app.attach();
    app.sync();
    assert_eq!(origin(&f, &root), (49, 0));
    // Client unmaximize shares restoration even after tiled/maximized transitions.
    app.toplevel.unset_maximized();
    ack(&mut app)?;
    assert_eq!(app.events.sizes.last(), Some(&(16, 16)));
    flags(&app, false)?;
    app.surface.commit();
    app.sync();
    assert_eq!(origin(&f, &root), (11, 12));
    assert_eq!(tile(&f, &root, None), Ok(None));
    Ok(())
}

#[test]
fn window_tiling_transaction_limits_refuse_without_mutation_and_invalidate_old_placement()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    place(&f, &root, (8, 9));
    f.render((81, 60), false, 1)?;
    app.toplevel.set_max_size(20, 20);
    app.surface.commit();
    app.sync();
    let count = app.events.sizes.len();
    assert_eq!(
        tile(&f, &root, Some(TileSide::Left)),
        Err(WindowModeError::Tile(TileGeometryError::ClientLimits))
    );
    app.sync();
    assert_eq!(app.events.sizes.len(), count);
    place(&f, &root, (8, 9));
    app.toplevel.set_max_size(0, 0);
    app.surface.commit();
    app.sync();
    tile(&f, &root, Some(TileSide::Right))?;
    app.toplevel.set_min_size(42, 60); // Pending hints do not affect duplicate acceptance.
    assert_eq!(tile(&f, &root, Some(TileSide::Right)), Ok(None));
    ack(&mut app)?;
    app.attach_resized();
    app.sync();
    assert_eq!(origin(&f, &root), (8, 9));
    assert_eq!(
        tile(&f, &root, Some(TileSide::Right)),
        Err(WindowModeError::Tile(TileGeometryError::ClientLimits))
    );
    app.toplevel.set_min_size(0, 0);
    app.surface.commit();
    app.sync();
    assert_eq!(origin(&f, &root), (8, 9)); // Old ack cannot revive cancelled placement.
    assert!(tile(&f, &root, Some(TileSide::Right))?.is_some());
    ack(&mut app)?;
    app.surface.commit();
    app.sync();
    assert_eq!(origin(&f, &root), (49, 0));
    app.toplevel.set_min_size(18, 19);
    app.surface.commit();
    app.sync();
    tile(&f, &root, None)?;
    ack(&mut app)?;
    assert_eq!(app.events.sizes.last(), Some(&(18, 19)));
    app.surface.commit();
    app.sync();
    assert_eq!(origin(&f, &root), (8, 9));
    Ok(())
}
