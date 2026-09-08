//! Committed real-client geometry feeds resize calculations without mutation.
use super::{Application, Fixture};
use alo_shell::{ResizeEdge, ResizeGeometry, ResizeGeometryError};
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

fn snapshot(
    f: &Fixture,
    root: &WlSurface,
    edge: ResizeEdge,
) -> Result<ResizeGeometry, ResizeGeometryError> {
    let root = root.clone();
    f.backend(move |s| s.window_resize_geometry(&root, edge))
}

#[test]
fn window_resize_snapshot_observes_only_committed_geometry_and_limits()
-> Result<(), ResizeGeometryError> {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    let mut app = mapped(&f);
    let root = f.root();
    let mut other = mapped(&f);
    assert!(f.focus_surface(root.clone()).is_ok());
    let placed = root.clone();
    assert!(
        f.backend(move |s| s.place_window(&placed, (100, 200)))
            .is_ok()
    );
    app.sync();
    let initial = snapshot(&f, &root, ResizeEdge::TopLeft)?;
    assert_eq!(initial.requested_size((0.0, 0.0)), Ok((16, 16)));
    app.xdg.set_window_geometry(2, 3, 12, 10);
    app.toplevel.set_min_size(8, 6);
    app.toplevel.set_max_size(20, 18);
    app.sync();
    assert_eq!(
        snapshot(&f, &root, ResizeEdge::TopLeft)?.requested_size((-20.0, -20.0)),
        Ok((36, 36))
    );
    app.surface.commit();
    app.sync();
    let current = snapshot(&f, &root, ResizeEdge::TopLeft)?;
    assert_eq!(current.requested_size((0.0, 0.0)), Ok((12, 10)));
    assert_eq!(current.requested_size((-20.0, -20.0)), Ok((20, 18)));
    assert_eq!(current.requested_size((20.0, 20.0)), Ok((8, 6)));
    assert_eq!(current.committed_origin((20, 18)), Ok((92, 192)));
    assert_eq!(initial.requested_size((0.0, 0.0)), Ok((16, 16)));
    let sizes = app.events.sizes.clone();
    let order = f.backend(|s| s.mapped_surfaces().cloned().collect::<Vec<_>>());
    assert_eq!(f.key(30, KeyState::Pressed).ok(), Some(true));
    assert_eq!(f.key(30, KeyState::Released).ok(), Some(true));
    assert!(f.backend(|s| s.pointer_motion(101.0, 201.0, 1)).is_ok());
    assert_eq!(f.render((320, 240), false, 1).ok(), Some(2));
    app.sync();
    other.sync();
    assert_eq!(app.events.sizes, sizes);
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert!(other.events.keyboard.keys.is_empty());
    assert_eq!(app.events.pointer.enters.len(), 1);
    assert_eq!(
        f.backend(|s| s.mapped_surfaces().cloned().collect::<Vec<_>>()),
        order
    );
    Ok(())
}

#[test]
fn window_resize_snapshot_tracks_acknowledged_buffer_commit_and_effective_tree()
-> Result<(), ResizeGeometryError> {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    let target = root.clone();
    assert!(
        f.backend(move |s| s.request_window_size(&target, (32, 24)))
            .is_ok()
    );
    app.sync();
    assert_eq!(
        snapshot(&f, &root, ResizeEdge::TopLeft)?.requested_size((0.0, 0.0)),
        Ok((16, 16))
    );
    assert!(app.events.serial.is_some());
    if let Some(serial) = app.events.serial {
        app.xdg.ack_configure(serial);
    }
    app.sync();
    assert_eq!(
        snapshot(&f, &root, ResizeEdge::TopLeft)?.requested_size((0.0, 0.0)),
        Ok((16, 16))
    );
    app.attach_resized();
    app.sync();
    assert_eq!(
        snapshot(&f, &root, ResizeEdge::TopLeft)?.requested_size((0.0, 0.0)),
        Ok((32, 24))
    );
    // Explicit XDG geometry is intersected with the effective buffer tree.
    app.xdg.set_window_geometry(2, 3, 100, 100);
    app.surface.commit();
    app.sync();
    let g = snapshot(&f, &root, ResizeEdge::TopLeft)?;
    assert_eq!(g.requested_size((0.0, 0.0)), Ok((30, 21)));
    assert_eq!(g.committed_origin((16, 16)), Ok((16, 8)));
    Ok(())
}

#[test]
fn window_resize_snapshot_refuses_foreign_child_popup_unmapped_and_dead_targets()
-> Result<(), ResizeGeometryError> {
    let f = Fixture::new();
    f.backend(|s| s.enable_popup_protocol());
    let mut app = mapped(&f);
    let root = f.root();
    let other = Fixture::new();
    let _outsider = mapped(&other);
    assert_eq!(
        snapshot(&f, &other.root(), ResizeEdge::Right).err(),
        Some(ResizeGeometryError::Unmapped)
    );
    let (_child, _role) = app.child((24, 32));
    app.surface.commit();
    app.sync();
    let parent = root.clone();
    assert_eq!(
        f.backend(move |s| {
            smithay::wayland::compositor::get_children(&parent)
                .into_iter()
                .find(|child| child != &parent)
                .map(|child| s.window_resize_geometry(&child, ResizeEdge::Right).err())
        }),
        Some(Some(ResizeGeometryError::Unmapped))
    );
    let (popup, xdg, _popup_role) = app.popup(true, 1);
    popup.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&popup);
    app.sync();
    assert_eq!(
        f.backend(|s| {
            s.popup_surfaces().first().map(|p| {
                s.window_resize_geometry(&p.surface, ResizeEdge::Right)
                    .err()
            })
        }),
        Some(Some(ResizeGeometryError::Unmapped))
    );
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    assert_eq!(
        snapshot(&f, &root, ResizeEdge::Right).err(),
        Some(ResizeGeometryError::Unmapped)
    );
    app.configure();
    assert_eq!(
        snapshot(&f, &root, ResizeEdge::Right).err(),
        Some(ResizeGeometryError::Unmapped)
    );
    app.attach();
    app.sync();
    assert!(snapshot(&f, &root, ResizeEdge::Right).is_ok());
    drop(app);
    f.wait_for((0, 0));
    assert_eq!(
        snapshot(&f, &root, ResizeEdge::Right).err(),
        Some(ResizeGeometryError::Unmapped)
    );
    Ok(())
}

#[test]
fn window_resize_snapshot_refuses_excessive_tree_bounds_without_poisoning_the_client()
-> Result<(), ResizeGeometryError> {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    let (_child, _role) = app.child((i32::MAX, i32::MAX));
    app.surface.commit();
    app.sync();
    assert_eq!(
        snapshot(&f, &root, ResizeEdge::BottomRight).err(),
        Some(ResizeGeometryError::Geometry)
    );
    // The same live client can supply a bounded effective window geometry.
    app.xdg.set_window_geometry(0, 0, 16, 16);
    app.surface.commit();
    app.sync();
    assert_eq!(
        snapshot(&f, &root, ResizeEdge::BottomRight)?.requested_size((1.0, 2.0)),
        Ok((17, 18))
    );
    assert_eq!(app.events.sizes, [(0, 0)]);
    Ok(())
}
