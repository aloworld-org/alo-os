//! Real-client tiling plans: committed limits, output lifetime and input isolation.
use super::{Application, Fixture};
use alo_shell::{TileGeometry, TileGeometryError, TileSide};
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

fn plan(f: &Fixture, root: &WlSurface, side: TileSide) -> Result<TileGeometry, TileGeometryError> {
    let root = root.clone();
    f.backend(move |s| s.window_tile_geometry(&root, side))
}

#[test]
fn window_tiling_committed_limits_and_plans_preserve_scene_and_ordinary_input()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    let mut other = mapped(&f);
    f.focus_surface(root.clone())?;
    let target = root.clone();
    f.backend(move |s| s.place_window(&target, (8, 9)))?;
    f.render((81, 60), false, 1)?;
    app.sync();
    let sizes = app.events.sizes.clone();
    let order = f.backend(|s| s.mapped_surfaces().cloned().collect::<Vec<_>>());
    let left = plan(&f, &root, TileSide::Left)?;
    let right = plan(&f, &root, TileSide::Right)?;
    assert_eq!(left.requested_size(), (40, 60));
    assert_eq!(right.requested_size(), (41, 60));
    assert_eq!(right.committed_origin((32, 24)), Ok((49, 0)));
    app.toplevel.set_min_size(41, 60);
    app.toplevel.set_max_size(41, 60);
    app.sync();
    assert_eq!(plan(&f, &root, TileSide::Left)?, left);
    app.surface.commit();
    app.sync();
    assert_eq!(
        plan(&f, &root, TileSide::Left),
        Err(TileGeometryError::ClientLimits)
    );
    assert_eq!(plan(&f, &root, TileSide::Right)?, right);
    // Snapshots remain immutable data, never live authority.
    assert_eq!(left.requested_size(), (40, 60));
    assert!(f.key(30, KeyState::Pressed)?);
    assert!(f.key(30, KeyState::Released)?);
    f.backend(|s| s.pointer_motion(9.0, 10.0, 2))?;
    app.sync();
    other.sync();
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert!(other.events.keyboard.keys.is_empty());
    assert_eq!(app.events.pointer.enters.len(), 1);
    assert_eq!(app.events.sizes, sizes);
    assert_eq!(
        f.backend(|s| s.mapped_surfaces().cloned().collect::<Vec<_>>()),
        order
    );
    let target = root.clone();
    assert_eq!(
        f.backend(move |_| alo_shell::window_buffer_origin(&target)),
        (8.0, 9.0).into()
    );
    Ok(())
}

struct RetireTarget(bool);
impl alo_shell::FrameTarget for RetireTarget {
    fn size(&self) -> smithay::utils::Size<i32, smithay::utils::Physical> {
        (80, 60).into()
    }
    fn submit(&mut self, roots: &[WlSurface]) -> Result<Vec<WlSurface>, alo_shell::RenderError> {
        Ok(roots.to_vec())
    }
    fn retire(&mut self) -> Result<(), alo_shell::RenderError> {
        if self.0 {
            Err(alo_shell::RenderError::RetirementUnsupported)
        } else {
            Ok(())
        }
    }
}

#[test]
fn window_tiling_output_requires_successful_submission_and_tracks_retirement()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    assert_eq!(
        plan(&f, &root, TileSide::Left),
        Err(TileGeometryError::OutputUnavailable)
    );
    assert!(f.render((80, 60), true, 1).is_err());
    assert_eq!(
        plan(&f, &root, TileSide::Left),
        Err(TileGeometryError::OutputUnavailable)
    );
    f.render((80, 60), false, 2)?;
    let old = plan(&f, &root, TileSide::Right)?;
    assert!(f.render((100, 70), true, 3).is_err());
    assert_eq!(plan(&f, &root, TileSide::Right)?, old);
    assert!(
        f.backend(|s| s.retire_output(&mut RetireTarget(true)))
            .is_err()
    );
    assert_eq!(plan(&f, &root, TileSide::Right)?, old);
    f.backend(|s| s.retire_output(&mut RetireTarget(false)))?;
    assert_eq!(
        plan(&f, &root, TileSide::Right),
        Err(TileGeometryError::OutputUnavailable)
    );
    f.render((101, 70), false, 4)?;
    assert_eq!(plan(&f, &root, TileSide::Right)?.requested_size(), (51, 70));
    assert_eq!(old.requested_size(), (40, 60));
    for size in [(1, 60), (1_000_001, 60), (80, 1_000_001)] {
        f.render(size, false, 5)?;
        assert_eq!(
            plan(&f, &root, TileSide::Left),
            Err(TileGeometryError::OutputUnavailable)
        );
    }
    app.sync();
    assert_eq!(app.events.sizes, [(0, 0)]);
    Ok(())
}

#[test]
fn window_tiling_refuses_foreign_child_popup_hidden_unmapped_and_dead_roots()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new();
    f.backend(|s| s.enable_popup_protocol());
    let mut app = mapped(&f);
    let root = f.root();
    f.render((80, 60), false, 1)?;
    let foreign = Fixture::new();
    let _other = mapped(&foreign);
    assert_eq!(
        plan(&f, &foreign.root(), TileSide::Left),
        Err(TileGeometryError::Unmapped)
    );
    let (_child, _role) = app.child((2, 2));
    app.surface.commit();
    app.sync();
    let target = root.clone();
    assert_eq!(
        f.backend(move |s| {
            smithay::wayland::compositor::get_children(&target)
                .into_iter()
                .find(|child| child != &target)
                .map(|child| s.window_tile_geometry(&child, TileSide::Left))
        }),
        Some(Err(TileGeometryError::Unmapped))
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
            .map(|p| s.window_tile_geometry(&p.surface, TileSide::Left))),
        Some(Err(TileGeometryError::Unmapped))
    );
    let target = root.clone();
    f.backend(move |s| s.set_window_minimized(&target, true))?;
    assert_eq!(
        plan(&f, &root, TileSide::Left),
        Err(TileGeometryError::Unmapped)
    );
    let target = root.clone();
    f.backend(move |s| s.set_window_minimized(&target, false))?;
    assert!(plan(&f, &root, TileSide::Left).is_ok());
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    assert_eq!(
        plan(&f, &root, TileSide::Left),
        Err(TileGeometryError::Unmapped)
    );
    app.configure();
    assert_eq!(
        plan(&f, &root, TileSide::Left),
        Err(TileGeometryError::Unmapped)
    );
    app.attach();
    app.sync();
    assert!(plan(&f, &root, TileSide::Left).is_ok());
    drop(app);
    f.wait_for((0, 0));
    assert_eq!(
        plan(&f, &root, TileSide::Left),
        Err(TileGeometryError::Unmapped)
    );
    Ok(())
}

#[test]
fn window_tiling_refuses_excessive_effective_geometry_without_poisoning_client()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    f.render((80, 60), false, 1)?;
    let (_child, _role) = app.child((i32::MAX, i32::MAX));
    app.surface.commit();
    app.sync();
    assert_eq!(
        plan(&f, &root, TileSide::Left),
        Err(TileGeometryError::Geometry(
            alo_shell::ResizeGeometryError::Geometry
        ))
    );
    app.xdg.set_window_geometry(2, 3, 12, 10);
    app.surface.commit();
    app.sync();
    assert_eq!(plan(&f, &root, TileSide::Left)?.requested_size(), (40, 60));
    assert_eq!(app.events.sizes, [(0, 0)]);
    Ok(())
}
