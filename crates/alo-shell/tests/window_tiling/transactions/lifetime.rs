//! Output, mapping and competing-operation boundaries for tile transactions.
use super::*;

#[test]
fn window_tiling_transaction_popup_grab_and_excessive_geometry_refuse_without_configures()
-> Result<(), Box<dyn std::error::Error>> {
    use smithay::backend::input::ButtonState::Pressed;
    let f = Fixture::keyboard();
    f.backend(|s| {
        s.enable_popup_protocol();
        s.enable_pointer()
    })?;
    let mut app = mapped(&f);
    let root = f.root();
    f.render((81, 60), false, 1)?;
    f.backend(|s| s.pointer_motion(1.0, 1.0, 1))?;
    f.backend(|s| s.pointer_button(0x110, Pressed, 2))?;
    app.sync();
    let (popup, xdg, role) = app.popup(true, 20);
    app.grab_popup(&role, app.events.pointer.button_serial);
    popup.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&popup);
    app.sync();
    let count = app.events.sizes.len();
    assert_eq!(
        tile(&f, &root, Some(TileSide::Left)),
        Err(WindowModeError::Busy)
    );
    app.sync();
    assert_eq!(app.events.sizes.len(), count);
    role.destroy();
    xdg.destroy();
    popup.destroy();
    app.sync();
    let (_child, _role) = app.child((i32::MAX, i32::MAX));
    app.surface.commit();
    app.sync();
    assert_eq!(
        tile(&f, &root, Some(TileSide::Left)),
        Err(WindowModeError::Tile(TileGeometryError::Geometry(
            alo_shell::ResizeGeometryError::Geometry
        )))
    );
    app.sync();
    assert_eq!(app.events.sizes.len(), count);
    app.xdg.set_window_geometry(2, 3, 12, 10);
    app.surface.commit();
    app.sync();
    assert!(tile(&f, &root, Some(TileSide::Left))?.is_some());
    Ok(())
}

#[test]
fn window_tiling_transaction_output_failure_retirement_replacement_and_restore()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    place(&f, &root, (8, 9));
    assert_eq!(
        tile(&f, &root, Some(TileSide::Right)),
        Err(WindowModeError::Tile(TileGeometryError::OutputUnavailable))
    );
    assert!(f.render((81, 60), true, 1).is_err());
    assert_eq!(
        tile(&f, &root, Some(TileSide::Right)),
        Err(WindowModeError::Tile(TileGeometryError::OutputUnavailable))
    );
    f.render((81, 60), false, 2)?;
    let old = tile(&f, &root, Some(TileSide::Right))?.ok_or("tile")?;
    app.sync();
    let count = app.events.sizes.len();
    assert!(f.render((91, 70), true, 3).is_err());
    assert!(
        f.backend(|s| s.retire_output(&mut RetireTarget(true)))
            .is_err()
    );
    app.sync();
    assert_eq!(app.events.sizes.len(), count);
    f.render((91, 70), false, 4)?;
    app.sync();
    assert_eq!(app.events.sizes.last(), Some(&(46, 70)));
    app.xdg.ack_configure(old);
    app.attach_resized();
    app.sync();
    assert_eq!(origin(&f, &root), (8, 9));
    f.backend(|s| s.retire_output(&mut RetireTarget(false)))?;
    ack(&mut app)?;
    app.surface.commit();
    app.sync();
    assert_eq!(origin(&f, &root), (8, 9));
    assert_eq!(
        tile(&f, &root, Some(TileSide::Right)),
        Err(WindowModeError::Tile(TileGeometryError::OutputUnavailable))
    );
    f.render((101, 80), false, 5)?;
    ack(&mut app)?;
    assert_eq!(app.events.sizes.last(), Some(&(51, 80)));
    app.surface.commit();
    app.sync();
    assert_eq!(origin(&f, &root), (69, 0));
    f.render((1, 80), false, 6)?; // Valid maximize output, invalid tile output.
    app.attach();
    app.sync();
    assert_eq!(origin(&f, &root), (69, 0));
    f.backend(|s| s.retire_output(&mut RetireTarget(false)))?;
    tile(&f, &root, None)?;
    ack(&mut app)?;
    f.render((81, 60), false, 7)?; // Output changes cannot supersede normal restore.
    app.surface.commit();
    app.sync();
    assert_eq!(origin(&f, &root), (8, 9));
    Ok(())
}

#[test]
fn window_tiling_transaction_hidden_commits_preserve_memory_unmap_and_disconnect_forget_it()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    place(&f, &root, (8, 9));
    f.render((81, 60), false, 1)?;
    tile(&f, &root, Some(TileSide::Right))?;
    let target = root.clone();
    f.backend(move |s| s.set_window_minimized(&target, true))?;
    assert_eq!(tile(&f, &root, None), Err(WindowModeError::Unmapped));
    ack(&mut app)?;
    app.attach_resized();
    app.sync();
    f.wait_for((1, 0));
    assert_eq!(origin(&f, &root), (49, 0));
    let target = root.clone();
    f.backend(move |s| s.set_window_minimized(&target, false))?;
    tile(&f, &root, None)?;
    ack(&mut app)?;
    app.attach();
    app.sync();
    assert_eq!(origin(&f, &root), (8, 9));
    tile(&f, &root, Some(TileSide::Left))?;
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    assert_eq!(tile(&f, &root, None), Err(WindowModeError::Unmapped));
    app.configure();
    flags(&app, false)?;
    assert_eq!(
        tile(&f, &root, Some(TileSide::Right)),
        Err(WindowModeError::Unmapped)
    );
    app.attach();
    app.sync();
    assert_eq!(tile(&f, &root, None), Ok(None));
    tile(&f, &root, Some(TileSide::Right))?;
    ack(&mut app)?;
    app.attach_resized();
    app.sync();
    tile(&f, &root, None)?;
    ack(&mut app)?;
    assert_eq!(app.events.sizes.last(), Some(&(16, 16)));
    app.attach();
    app.sync();
    assert_eq!(origin(&f, &root), (0, 0));
    tile(&f, &root, Some(TileSide::Left))?;
    drop(app);
    f.wait_for((0, 0));
    assert_eq!(
        tile(&f, &root, Some(TileSide::Left)),
        Err(WindowModeError::Unmapped)
    );
    Ok(())
}

#[test]
fn window_tiling_transaction_refuses_foreign_children_and_popups()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new();
    f.backend(|s| s.enable_popup_protocol());
    let mut app = mapped(&f);
    let root = f.root();
    f.render((81, 60), false, 1)?;
    let foreign = Fixture::new();
    let _other = mapped(&foreign);
    assert_eq!(
        tile(&f, &foreign.root(), Some(TileSide::Left)),
        Err(WindowModeError::Unmapped)
    );
    let (_child, _role) = app.child((2, 2));
    app.surface.commit();
    app.sync();
    let target = root.clone();
    assert_eq!(
        f.backend(move |s| smithay::wayland::compositor::get_children(&target)
            .into_iter()
            .find(|child| child != &target)
            .map(|child| s.set_window_tiled(&child, Some(TileSide::Left)))),
        Some(Err(WindowModeError::Unmapped))
    );
    let (popup, xdg, _role) = app.popup(true, 1);
    popup.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&popup);
    app.sync();
    assert_eq!(
        f.backend(|s| s
            .popup_surfaces()
            .first()
            .map(|p| p.surface.clone())
            .map(|p| s.set_window_tiled(&p, Some(TileSide::Left)))),
        Some(Err(WindowModeError::Unmapped))
    );
    assert_eq!(app.events.sizes, [(0, 0)]);
    Ok(())
}

#[test]
fn window_tiling_transaction_excludes_pointer_operations_until_restore_commits()
-> Result<(), Box<dyn std::error::Error>> {
    use smithay::backend::input::ButtonState::{Pressed, Released};
    use wayland_protocols::xdg::shell::client::xdg_toplevel::ResizeEdge;
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    f.render((81, 60), false, 1)?;
    f.backend(|s| s.pointer_motion(4.0, 5.0, 1))?;
    f.backend(|s| s.pointer_button(0x110, Pressed, 2))?;
    app.sync();
    let seat = app.events.keyboard.seat.clone().ok_or("seat")?;
    app.toplevel.resize(
        &seat,
        app.events.pointer.button_serial,
        ResizeEdge::BottomRight,
    );
    app.sync();
    assert_eq!(
        tile(&f, &root, Some(TileSide::Left)),
        Err(WindowModeError::Busy)
    );
    f.backend(|s| s.pointer_button(0x110, Released, 3))?;
    assert_eq!(
        tile(&f, &root, Some(TileSide::Left)),
        Err(WindowModeError::Busy)
    );
    ack(&mut app)?;
    app.surface.commit();
    app.sync();
    tile(&f, &root, Some(TileSide::Left))?;
    ack(&mut app)?;
    app.surface.commit();
    app.sync();
    for restoring in [false, true] {
        if restoring {
            tile(&f, &root, None)?;
        }
        let target = root.clone();
        assert!(matches!(
            f.backend(move |s| s.place_window(&target, (1, 2))),
            Err(alo_shell::WindowPlacementError::Maximized)
        ));
        let target = root.clone();
        assert_eq!(
            f.backend(move |s| s.request_window_size(&target, (20, 20))),
            Err(alo_shell::WindowSizeError::Maximized)
        );
        f.backend(|s| s.pointer_motion(4.0, 5.0, 4))?;
        f.backend(|s| s.pointer_button(0x110, Pressed, 5))?;
        app.sync();
        let count = app.events.sizes.len();
        app.toplevel._move(&seat, app.events.pointer.button_serial);
        app.toplevel
            .resize(&seat, app.events.pointer.button_serial, ResizeEdge::TopLeft);
        app.sync();
        f.backend(|s| s.pointer_motion(9.0, 10.0, 6))?;
        app.sync();
        assert_eq!(origin(&f, &root), (0, 0));
        assert_eq!(app.events.sizes.len(), count);
        f.backend(|s| s.pointer_button(0x110, Released, 7))?;
    }
    ack(&mut app)?;
    app.surface.commit();
    app.sync();
    place(&f, &root, (0, 0));
    f.backend(|s| s.pointer_motion(4.0, 5.0, 8))?;
    f.backend(|s| s.pointer_button(0x110, Pressed, 9))?;
    app.sync();
    app.toplevel._move(&seat, app.events.pointer.button_serial);
    app.sync();
    assert_eq!(
        tile(&f, &root, Some(TileSide::Right)),
        Err(WindowModeError::Busy)
    );
    f.backend(|s| s.pointer_button(0x110, Released, 10))?;
    assert!(tile(&f, &root, Some(TileSide::Right))?.is_some());
    Ok(())
}
