//! Real protocol evidence for native placement and its trust boundary.
use super::{Application, Fixture};
use alo_shell::{WindowPlacementError, window_buffer_origin};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use wayland_client::Proxy;

/// Inspect renderer state on its owning display thread.
fn origin(f: &Fixture, root: &WlSurface) -> smithay::utils::Point<f64, smithay::utils::Logical> {
    let root = root.clone();
    f.backend(move |_| window_buffer_origin(&root))
}

/// Complete the real XDG handshake and attach a buffer.
fn mapped(f: &Fixture) -> Application {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    app
}

#[test]
fn window_placement_reactive_popup_waits_for_commit_and_coalesces() {
    use wayland_protocols::xdg::shell::client::xdg_positioner::ConstraintAdjustment as Adjust;
    let f = Fixture::new();
    f.backend(|s| s.enable_popup_protocol());
    assert!(f.render((40, 40), false, 0).is_ok());
    let mut app = mapped(&f);
    let root = f.root();
    let (popup, xdg, _role) =
        app.popup_reactive(Some(&app.xdg), 40, Adjust::SlideX | Adjust::SlideY, true);
    popup.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&popup);
    app.sync();
    assert_eq!(app.events.popups.geometry.last(), Some(&(24, 10, 16, 16)));
    assert!(place(&f, &root, (20, 20)).is_ok());
    app.sync();
    assert_eq!(app.events.popups.geometry.last(), Some(&(4, 4, 16, 16)));
    let committed = || f.backend(|s| s.popup_surfaces().first().map(|popup| popup.geometry.loc));
    assert_eq!(committed(), Some((24, 10).into()));
    assert!(place(&f, &root, (20, 20)).is_ok());
    app.sync();
    assert_eq!(app.events.popups.geometry.len(), 2);
    app.ack_popup(&xdg);
    app.sync();
    assert_eq!(committed(), Some((24, 10).into()));
    popup.commit();
    app.sync();
    assert_eq!(committed(), Some((4, 4).into()));
    assert_eq!(app.events.popups.done, 0);
}

/// Serialize a trusted placement with protocol dispatch.
fn place(f: &Fixture, root: &WlSurface, point: (i32, i32)) -> Result<(), WindowPlacementError> {
    let root = root.clone();
    f.backend(move |s| s.place_window(&root, point))
}

#[test]
fn window_placement_moves_input_without_configuring_raising_or_activating() {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    let mut app = mapped(&f);
    let root = f.root();
    let mut other = mapped(&f);
    assert!(f.focus_surface(root.clone()).is_ok());
    app.sync();
    let sizes = app.events.sizes.clone();
    let activation = app.events.activation.clone();
    let roots = f.backend(|s| s.mapped_surfaces().cloned().collect::<Vec<_>>());
    assert!(f.backend(|s| s.pointer_motion(1.0, 1.0, 1)).is_ok());
    app.sync();
    assert_eq!(app.events.pointer.enters.len(), 1);
    assert!(place(&f, &root, (30, 20)).is_ok());
    app.sync();
    other.sync();
    assert_eq!(app.events.pointer.leaves, 1);
    assert_eq!(other.events.pointer.enters.len(), 1);
    assert!(f.backend(|s| s.pointer_motion(31.0, 22.0, 2)).is_ok());
    app.sync();
    assert_eq!(
        app.events.pointer.enters.last(),
        Some(&(app.surface.id().protocol_id(), 1.0, 2.0))
    );
    assert!(place(&f, &root, (30, 20)).is_ok());
    app.sync();
    assert_eq!(app.events.sizes, sizes);
    assert_eq!(app.events.activation, activation);
    assert_eq!(app.events.keyboard.leaves, 0);
    assert_eq!(
        f.backend(|s| s.mapped_surfaces().cloned().collect::<Vec<_>>()),
        roots
    );
}

#[test]
fn window_placement_anchors_committed_geometry_and_resets_on_unmap() {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    let mut app = mapped(&f);
    let root = f.root();
    assert!(place(&f, &root, (30, 20)).is_ok());
    app.xdg.set_window_geometry(2, 3, 12, 12);
    app.sync();
    assert_eq!(origin(&f, &root), (30.0, 20.0).into());
    app.surface.commit();
    app.sync();
    assert_eq!(origin(&f, &root), (28.0, 17.0).into());
    assert!(f.backend(|s| s.pointer_motion(31.0, 22.0, 1)).is_ok());
    app.sync();
    assert_eq!(
        app.events.pointer.enters.last(),
        Some(&(app.surface.id().protocol_id(), 3.0, 5.0))
    );
    for point in [(i32::MIN, 0), (0, i32::MAX), (1_000_001, 0)] {
        assert!(matches!(
            place(&f, &root, point),
            Err(WindowPlacementError::OutOfRange)
        ));
        assert_eq!(origin(&f, &root), (28.0, 17.0).into());
    }
    assert!(place(&f, &root, (-1_000_000, 1_000_000)).is_ok());
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    assert!(matches!(
        place(&f, &root, (0, 0)),
        Err(WindowPlacementError::Unmapped)
    ));
    assert_eq!(origin(&f, &root), (0.0, 0.0).into());
    app.configure();
    assert!(matches!(
        place(&f, &root, (0, 0)),
        Err(WindowPlacementError::Unmapped)
    ));
    app.attach();
    app.sync();
    assert_eq!(origin(&f, &root), (0.0, 0.0).into());
    drop(app);
    f.wait_for((0, 0));
    assert!(matches!(
        place(&f, &root, (0, 0)),
        Err(WindowPlacementError::Unmapped)
    ));
}

#[test]
fn window_placement_refuses_foreign_child_and_popup_targets() {
    let f = Fixture::new();
    f.backend(|s| s.enable_popup_protocol());
    let mut app = mapped(&f);
    let root = f.root();
    let foreign = Fixture::new();
    let _other = mapped(&foreign);
    assert!(matches!(
        place(&f, &foreign.root(), (4, 5)),
        Err(WindowPlacementError::Unmapped)
    ));
    let (_child, _role) = app.child((2, 2));
    app.surface.commit();
    app.sync();
    let parent = root.clone();
    assert!(f.backend(move |s| {
        smithay::wayland::compositor::get_children(&parent)
            .into_iter()
            .filter(|child| child != &parent)
            .any(|child| {
                matches!(
                    s.place_window(&child, (4, 5)),
                    Err(WindowPlacementError::Unmapped)
                )
            })
    }));
    let (popup, xdg, _role) = app.popup(true, 1);
    popup.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&popup);
    app.sync();
    assert!(f.backend(|s| {
        let Some(target) = s
            .popup_surfaces()
            .first()
            .map(|popup| popup.surface.clone())
        else {
            return false;
        };
        matches!(
            s.place_window(&target, (4, 5)),
            Err(WindowPlacementError::Unmapped)
        )
    }));
    assert_eq!(origin(&f, &root), (0.0, 0.0).into());
    assert_eq!(app.events.sizes, [(0, 0)]);
}

#[test]
fn window_placement_translates_popup_constraints_and_hit_testing() {
    use wayland_protocols::xdg::shell::client::xdg_positioner::ConstraintAdjustment as Adjust;
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_popup_protocol());
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    assert!(f.render((40, 40), false, 0).is_ok());
    let mut app = mapped(&f);
    let root = f.root();
    assert!(place(&f, &root, (20, 20)).is_ok());
    let (popup, xdg, _role) =
        app.popup_adjusted(Some(&app.xdg), 1, Adjust::SlideX | Adjust::SlideY);
    popup.commit();
    app.sync();
    assert_eq!(app.events.popups.geometry.last(), Some(&(4, 4, 16, 16)));
    app.ack_popup(&xdg);
    app.attach_popup(&popup);
    app.sync();
    assert!(f.backend(|s| s.pointer_motion(25.0, 26.0, 1)).is_ok());
    app.sync();
    assert_eq!(
        app.events.pointer.enters.last(),
        Some(&(popup.id().protocol_id(), 1.0, 2.0))
    );
    // A nonreactive popup keeps its parent-relative geometry and follows the root.
    assert!(place(&f, &root, (10, 10)).is_ok());
    assert!(f.backend(|s| s.pointer_motion(15.0, 16.0, 2)).is_ok());
    app.sync();
    assert_eq!(app.events.pointer.motion.last(), Some(&(1.0, 2.0)));
    assert_eq!(app.events.popups.geometry, [(4, 4, 16, 16)]);
}
