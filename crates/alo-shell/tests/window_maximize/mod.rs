//! Real-client maximize/restore ordering, isolation and lifetime evidence.
use super::{Application, Fixture};
mod client_requests;
use alo_shell::WindowMaximizeError;
use smithay::{
    backend::input::KeyState, reexports::wayland_server::protocol::wl_surface::WlSurface,
};

/// Map a real protocol client.
fn mapped(f: &Fixture) -> Application {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    app
}

/// Serialize the trusted native action and expose its wire serial.
fn maximize(
    f: &Fixture,
    root: &WlSurface,
    value: bool,
) -> Result<Option<u32>, WindowMaximizeError> {
    let root = root.clone();
    f.backend(move |s| {
        s.set_window_maximized(&root, value)
            .map(|serial| serial.map(u32::from))
    })
}

/// Read scene geometry on its owning display thread.
fn origin(f: &Fixture, root: &WlSurface) -> smithay::utils::Point<f64, smithay::utils::Logical> {
    let root = root.clone();
    f.backend(move |_| alo_shell::window_buffer_origin(&root))
}

/// Acknowledgement and commit are explicitly separate in every ordering test.
fn ack(app: &mut Application) -> Result<(), Box<dyn std::error::Error>> {
    app.sync();
    app.xdg
        .ack_configure(app.events.serial.ok_or("missing configure")?);
    app.sync();
    Ok(())
}

/// Explicit retirement fixture: no physical display or graphics submission claim.
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
fn window_maximize_captures_effective_geometry_and_refuses_excessive_tree_without_mutation()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    f.render((80, 60), false, 1)?;
    let (_child, _role) = app.child((i32::MAX, i32::MAX));
    app.surface.commit();
    app.sync();
    assert_eq!(
        maximize(&f, &root, true),
        Err(WindowMaximizeError::Geometry(
            alo_shell::ResizeGeometryError::Geometry
        ))
    );
    app.sync();
    assert_eq!(app.events.sizes, [(0, 0)]);
    app.xdg.set_window_geometry(2, 3, 12, 10);
    app.surface.commit();
    app.sync();
    maximize(&f, &root, true)?;
    ack(&mut app)?;
    // A geometry origin is distinct from the buffer origin (including shadows).
    app.xdg.set_window_geometry(4, 5, 10, 9);
    app.surface.commit();
    app.sync();
    assert_eq!(origin(&f, &root), (-4.0, -5.0).into());
    maximize(&f, &root, false)?;
    ack(&mut app)?;
    assert_eq!(app.events.sizes.last(), Some(&(12, 10)));
    app.surface.commit();
    app.sync();
    assert_eq!(origin(&f, &root), (-2.0, -2.0).into());
    Ok(())
}

#[test]
fn window_maximize_retirement_invalidates_pending_anchor_and_new_output_reconfigures()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    let target = root.clone();
    f.backend(move |s| s.place_window(&target, (8, 9)))?;
    f.render((80, 60), false, 1)?;
    maximize(&f, &root, true)?;
    assert!(
        f.backend(|s| s.retire_output(&mut RetireTarget(true)))
            .is_err()
    );
    assert_eq!(maximize(&f, &root, true), Ok(None));
    f.backend(|s| s.retire_output(&mut RetireTarget(false)))?;
    assert_eq!(
        maximize(&f, &root, true),
        Err(WindowMaximizeError::OutputUnavailable)
    );
    ack(&mut app)?;
    app.attach_resized();
    app.sync();
    assert_eq!(origin(&f, &root), (8.0, 9.0).into());
    f.render((100, 70), false, 2)?;
    ack(&mut app)?;
    assert_eq!(app.events.sizes.last(), Some(&(100, 70)));
    app.surface.commit();
    app.sync();
    assert_eq!(origin(&f, &root), (0.0, 0.0).into());
    f.backend(|s| s.retire_output(&mut RetireTarget(false)))?;
    maximize(&f, &root, false)?;
    ack(&mut app)?;
    app.attach();
    app.sync();
    assert_eq!(origin(&f, &root), (8.0, 9.0).into());
    // Malformed or excessive output dimensions never authorize a new maximize.
    assert!(f.render((0, 60), false, 3).is_err());
    f.render((1_000_001, 60), false, 4)?;
    assert_eq!(
        maximize(&f, &root, true),
        Err(WindowMaximizeError::OutputUnavailable)
    );
    Ok(())
}

#[test]
fn window_maximize_and_pointer_operations_refuse_competing_ownership()
-> Result<(), Box<dyn std::error::Error>> {
    use smithay::backend::input::ButtonState::{Pressed, Released};
    use wayland_protocols::xdg::shell::client::xdg_toplevel::ResizeEdge;
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    // Test output does not support a live cursor, so establish output before motion.
    f.render((80, 60), false, 1)?;
    f.backend(|s| s.pointer_motion(4.0, 5.0, 1))?;
    f.backend(|s| s.pointer_button(0x110, Pressed, 2))?;
    app.sync();
    let seat = app.events.keyboard.seat.clone().ok_or("missing seat")?;
    app.toplevel.resize(
        &seat,
        app.events.pointer.button_serial,
        ResizeEdge::BottomRight,
    );
    app.sync();
    assert_eq!(maximize(&f, &root, true), Err(WindowMaximizeError::Busy));
    f.backend(|s| s.pointer_button(0x110, Released, 3))?;
    assert_eq!(maximize(&f, &root, true), Err(WindowMaximizeError::Busy));
    ack(&mut app)?;
    app.surface.commit();
    app.sync();
    maximize(&f, &root, true)?;
    ack(&mut app)?;
    app.surface.commit();
    app.sync();
    f.backend(|s| s.pointer_motion(4.0, 5.0, 4))?;
    f.backend(|s| s.pointer_button(0x110, Pressed, 5))?;
    app.sync();
    let count = app.events.sizes.len();
    app.toplevel
        .resize(&seat, app.events.pointer.button_serial, ResizeEdge::TopLeft);
    app.toplevel._move(&seat, app.events.pointer.button_serial);
    app.sync();
    f.backend(|s| s.pointer_motion(9.0, 10.0, 6))?;
    app.sync();
    assert_eq!(app.events.sizes.len(), count);
    assert_eq!(origin(&f, &root), (0.0, 0.0).into());
    f.backend(|s| s.pointer_button(0x110, Released, 7))?;
    maximize(&f, &root, false)?;
    ack(&mut app)?;
    app.surface.commit();
    app.sync();
    f.backend(|s| s.pointer_motion(4.0, 5.0, 8))?;
    f.backend(|s| s.pointer_button(0x110, Pressed, 9))?;
    app.sync();
    app.toplevel._move(&seat, app.events.pointer.button_serial);
    app.sync();
    assert_eq!(maximize(&f, &root, true), Err(WindowMaximizeError::Busy));
    f.backend(|s| s.pointer_button(0x110, Released, 10))?;
    assert!(maximize(&f, &root, true)?.is_some());
    Ok(())
}

#[test]
fn window_maximize_preserves_pixels_and_keyboard_until_commit_and_restores_normal_geometry()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let root = f.root();
    let mut other = mapped(&f);
    let placed = root.clone();
    f.backend(move |s| s.place_window(&placed, (23, -7)))?;
    f.focus_surface(root.clone())?;
    f.render((33, 32), false, 1)?;
    app.sync();
    other.sync();
    let others = other.events.sizes.len();
    assert!(matches!(maximize(&f, &root, true), Ok(Some(_))));
    app.sync();
    assert_eq!(app.events.sizes.last(), Some(&(33, 32)));
    assert_eq!(app.events.maximized.last(), Some(&true));
    assert_eq!(app.events.activation.last(), Some(&true));
    assert_eq!(maximize(&f, &root, true), Ok(None));
    assert_eq!(origin(&f, &root), (23.0, -7.0).into());
    assert!(f.key(30, KeyState::Pressed)?);
    assert!(f.key(30, KeyState::Released)?);
    app.sync();
    other.sync();
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert!(other.events.keyboard.keys.is_empty());
    assert_eq!(other.events.sizes.len(), others);
    ack(&mut app)?;
    assert_eq!(origin(&f, &root), (23.0, -7.0).into());
    // A conforming application commits the complete output-sized buffer.
    app.attach_maximized();
    app.sync();
    assert_eq!(origin(&f, &root), (0.0, 0.0).into());
    let target = root.clone();
    assert_eq!(
        f.backend(move |s| s.request_window_size(&target, (20, 20))),
        Err(alo_shell::WindowSizeError::Maximized)
    );
    let target = root.clone();
    assert!(matches!(
        f.backend(move |s| s.place_window(&target, (9, 9))),
        Err(alo_shell::WindowPlacementError::Maximized)
    ));
    assert!(matches!(maximize(&f, &root, false), Ok(Some(_))));
    app.sync();
    assert_eq!(app.events.sizes.last(), Some(&(16, 16)));
    assert_eq!(app.events.maximized.last(), Some(&false));
    assert_eq!(maximize(&f, &root, false), Ok(None));
    ack(&mut app)?;
    assert_eq!(origin(&f, &root), (0.0, 0.0).into());
    app.attach();
    app.sync();
    assert_eq!(origin(&f, &root), (23.0, -7.0).into());
    let target = root.clone();
    f.backend(move |s| s.place_window(&target, (5, 6)))?;
    assert_eq!(maximize(&f, &root, false), Ok(None));
    Ok(())
}

#[test]
fn window_maximize_output_success_supersedes_old_responses_and_toggles_keep_original_geometry()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    let target = root.clone();
    f.backend(move |s| s.place_window(&target, (11, 12)))?;
    assert_eq!(
        maximize(&f, &root, true),
        Err(WindowMaximizeError::OutputUnavailable)
    );
    assert!(f.render((80, 60), true, 1).is_err());
    assert_eq!(
        maximize(&f, &root, true),
        Err(WindowMaximizeError::OutputUnavailable)
    );
    f.render((80, 60), false, 2)?;
    let old = maximize(&f, &root, true)?.ok_or("missing maximize")?;
    app.sync();
    let count = app.events.sizes.len();
    assert!(f.render((90, 70), true, 3).is_err());
    app.sync();
    assert_eq!(app.events.sizes.len(), count);
    f.render((90, 70), false, 4)?;
    app.sync();
    assert_eq!(app.events.sizes.last(), Some(&(90, 70)));
    app.xdg.ack_configure(old);
    app.attach_resized();
    app.sync();
    assert_eq!(origin(&f, &root), (11.0, 12.0).into());
    ack(&mut app)?;
    app.surface.commit();
    app.sync();
    assert_eq!(origin(&f, &root), (0.0, 0.0).into());
    let restore = maximize(&f, &root, false)?.ok_or("missing restore")?;
    maximize(&f, &root, true)?;
    app.sync();
    app.xdg.ack_configure(restore);
    app.attach();
    app.sync();
    assert_eq!(origin(&f, &root), (0.0, 0.0).into());
    maximize(&f, &root, false)?;
    ack(&mut app)?;
    assert_eq!(app.events.sizes.last(), Some(&(16, 16)));
    app.surface.commit();
    app.sync();
    assert_eq!(origin(&f, &root), (11.0, 12.0).into());
    Ok(())
}

#[test]
fn window_maximize_ignores_normal_limits_and_restore_uses_only_committed_limits()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    f.render((80, 60), false, 1)?;
    app.toplevel.set_max_size(20, 20);
    app.surface.commit();
    app.sync();
    maximize(&f, &root, true)?;
    ack(&mut app)?;
    assert_eq!(app.events.sizes.last(), Some(&(80, 60)));
    app.attach_resized();
    app.sync();
    app.toplevel.set_min_size(18, 19);
    app.sync();
    maximize(&f, &root, false)?;
    app.sync();
    assert_eq!(app.events.sizes.last(), Some(&(16, 16)));
    maximize(&f, &root, true)?;
    app.surface.commit();
    app.sync();
    maximize(&f, &root, false)?;
    ack(&mut app)?;
    assert_eq!(app.events.sizes.last(), Some(&(18, 19)));
    app.attach();
    app.sync();
    // Unrepresentable limits must not clear maximized or overwrite saved geometry.
    maximize(&f, &root, true)?;
    ack(&mut app)?;
    app.surface.commit();
    app.sync();
    app.toplevel.set_max_size(0, 0);
    app.toplevel.set_min_size(1_000_001, 1);
    app.surface.commit();
    app.sync();
    let count = app.events.sizes.len();
    assert_eq!(
        maximize(&f, &root, false),
        Err(WindowMaximizeError::Geometry(
            alo_shell::ResizeGeometryError::ClientLimits
        ))
    );
    app.sync();
    assert_eq!(app.events.sizes.len(), count);
    Ok(())
}

#[test]
fn window_maximize_refuses_foreign_child_popup_unmapped_and_dead_targets_and_forgets_remapping()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new();
    f.backend(|s| s.enable_popup_protocol());
    let mut app = mapped(&f);
    let root = f.root();
    f.render((80, 60), false, 1)?;
    let foreign = Fixture::new();
    let _other = mapped(&foreign);
    assert_eq!(
        maximize(&f, &foreign.root(), true),
        Err(WindowMaximizeError::Unmapped)
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
                .map(|child| s.set_window_maximized(&child, true))
        }),
        Some(Err(WindowMaximizeError::Unmapped))
    );
    let (popup, xdg, _role) = app.popup(true, 1);
    popup.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&popup);
    app.sync();
    assert_eq!(
        f.backend(|s| {
            s.popup_surfaces()
                .first()
                .map(|popup| popup.surface.clone())
                .map(|popup| s.set_window_maximized(&popup, true))
        }),
        Some(Err(WindowMaximizeError::Unmapped))
    );
    maximize(&f, &root, true)?;
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    assert_eq!(
        maximize(&f, &root, false),
        Err(WindowMaximizeError::Unmapped)
    );
    app.configure();
    assert_eq!(
        maximize(&f, &root, true),
        Err(WindowMaximizeError::Unmapped)
    );
    app.attach();
    app.sync();
    assert_eq!(maximize(&f, &root, false), Ok(None));
    assert_eq!(app.events.maximized.last(), Some(&false));
    maximize(&f, &root, true)?;
    ack(&mut app)?;
    app.attach_resized();
    app.sync();
    maximize(&f, &root, false)?;
    ack(&mut app)?;
    app.attach();
    app.sync();
    assert_eq!(origin(&f, &root), (0.0, 0.0).into());
    maximize(&f, &root, true)?;
    drop(app);
    f.wait_for((0, 0));
    assert_eq!(
        maximize(&f, &root, true),
        Err(WindowMaximizeError::Unmapped)
    );
    Ok(())
}
