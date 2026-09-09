//! Live protocol checks for read-only native control availability.
use super::{Application, Fixture};
use alo_shell::{WindowControlSnapshot, WindowControlSnapshotError, WindowMaximizeError};
use smithay::{
    backend::input::KeyState, reexports::wayland_server::protocol::wl_surface::WlSurface,
};
use wayland_client::protocol::wl_keyboard;

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
) -> Result<WindowControlSnapshot, WindowControlSnapshotError> {
    let root = root.clone();
    f.backend(move |s| s.window_control_snapshot(&root, (120, 48), (3, 4)))
}

fn maximize(f: &Fixture, root: &WlSurface, value: bool) -> Result<(), WindowMaximizeError> {
    let root = root.clone();
    f.backend(move |s| s.set_window_maximized(&root, value).map(|_| ()))
}

fn available(view: &WindowControlSnapshot, maximize: bool, restoring: bool) {
    assert_eq!(
        view.layout().controls().map(|c| c.enabled()),
        [true, maximize, true]
    );
    assert_eq!(view.layout().restoring(), restoring);
    assert_eq!(view.maximize_refusal().is_none(), maximize);
}

#[test]
fn window_controls_capture_has_no_wire_geometry_focus_or_typing_effects()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let root = f.root();
    let mut other = mapped(&f);
    f.focus_surface(root.clone())?;
    app.sync();
    let count = app.events.sizes.len();
    for _ in 0..4 {
        let view = snapshot(&f, &root)?;
        assert_eq!(view.surface(), &root);
        available(&view, false, false);
        assert_eq!(
            view.maximize_refusal(),
            Some(&WindowMaximizeError::OutputUnavailable)
        );
        assert!(
            !view
                .layout()
                .hit(40.0, 5.0)
                .ok_or("missing disabled hit")?
                .enabled()
        );
    }
    let target = root.clone();
    assert!(matches!(
        f.backend(move |s| s.window_control_snapshot(&target, (0, 48), (3, 4))),
        Err(WindowControlSnapshotError::Layout(_))
    ));
    assert!(f.key(30, KeyState::Pressed)?);
    snapshot(&f, &root)?;
    assert!(f.key(30, KeyState::Released)?);
    app.sync();
    other.sync();
    assert_eq!(app.events.sizes.len(), count);
    assert_eq!(app.events.close_requests, 0);
    assert_eq!(app.events.keyboard.leaves, 0);
    assert_eq!(
        app.events.keyboard.keys,
        [
            (30, wl_keyboard::KeyState::Pressed),
            (30, wl_keyboard::KeyState::Released)
        ]
    );
    assert!(other.events.keyboard.keys.is_empty());
    assert_eq!(other.events.close_requests, 0);
    assert_eq!(f.backend(|s| s.mapped_surfaces().count()), 2);
    assert_eq!(
        f.backend(move |_| alo_shell::window_buffer_origin(&root)),
        (0.0, 0.0).into()
    );
    Ok(())
}

/// Private output coordinator sink, without a kernel/display claim.
struct Retire;
impl alo_shell::FrameTarget for Retire {
    fn size(&self) -> smithay::utils::Size<i32, smithay::utils::Physical> {
        (120, 48).into()
    }
    fn submit(&mut self, roots: &[WlSurface]) -> Result<Vec<WlSurface>, alo_shell::RenderError> {
        Ok(roots.to_vec())
    }
    fn retire(&mut self) -> Result<(), alo_shell::RenderError> {
        Ok(())
    }
}

#[test]
fn window_controls_output_availability_and_pending_toggles_follow_execution()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new(); // No input seat required.
    let mut app = mapped(&f);
    let root = f.root();
    assert!(f.render((120, 48), true, 1).is_err());
    available(&snapshot(&f, &root)?, false, false);
    assert_eq!(
        maximize(&f, &root, true),
        Err(WindowMaximizeError::OutputUnavailable)
    );
    f.render((120, 48), false, 2)?;
    available(&snapshot(&f, &root)?, true, false);
    for value in [true, false, true, false] {
        maximize(&f, &root, value)?;
        available(&snapshot(&f, &root)?, true, value);
        app.sync(); // Deliberately no ack/commit: glyph follows requested intent.
        assert_eq!(app.events.maximized.last(), Some(&value));
    }
    maximize(&f, &root, true)?;
    f.backend(|s| s.retire_output(&mut Retire))?;
    available(&snapshot(&f, &root)?, true, true); // Restore needs no output.
    maximize(&f, &root, false)?;
    available(&snapshot(&f, &root)?, false, false);
    f.render((1_000_001, 48), false, 3)?;
    available(&snapshot(&f, &root)?, false, false);
    f.render((121, 49), false, 4)?;
    available(&snapshot(&f, &root)?, true, false);
    let target = root.clone();
    f.backend(move |s| s.set_window_tiled(&target, Some(alo_shell::TileSide::Right)))?;
    available(&snapshot(&f, &root)?, true, false);
    maximize(&f, &root, true)?;
    available(&snapshot(&f, &root)?, true, true);
    app.sync();
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}

#[test]
fn window_controls_restore_uses_committed_limits_and_keeps_refusal_state()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    f.render((120, 48), false, 1)?;
    maximize(&f, &root, true)?;
    app.sync();
    app.xdg
        .ack_configure(app.events.serial.ok_or("missing maximize")?);
    app.surface.commit();
    app.sync();
    app.toplevel.set_min_size(1_000_001, 1);
    app.sync();
    available(&snapshot(&f, &root)?, true, true); // Pending hints cannot disable.
    app.surface.commit();
    app.sync();
    let count = app.events.sizes.len();
    let view = snapshot(&f, &root)?;
    available(&view, false, true);
    let refusal = WindowMaximizeError::Geometry(alo_shell::ResizeGeometryError::ClientLimits);
    assert_eq!(view.maximize_refusal(), Some(&refusal));
    assert_eq!(maximize(&f, &root, false), Err(refusal));
    app.sync();
    assert_eq!(app.events.sizes.len(), count);
    app.toplevel.set_min_size(0, 0);
    app.surface.commit();
    app.sync();
    available(&snapshot(&f, &root)?, true, true);
    maximize(&f, &root, false)?;
    app.sync();
    assert_eq!(app.events.sizes.last(), Some(&(16, 16)));
    Ok(())
}

#[test]
fn window_controls_hidden_unmapped_remapped_dead_and_foreign_targets_never_fall_back()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    let mut other = mapped(&f);
    let foreign = Fixture::new();
    let _foreign_app = mapped(&foreign);
    assert!(matches!(
        snapshot(&f, &foreign.root()),
        Err(WindowControlSnapshotError::Unmapped)
    ));
    f.render((120, 48), false, 1)?;
    maximize(&f, &root, true)?;
    let old = snapshot(&f, &root)?;
    let target = root.clone();
    f.backend(move |s| s.set_window_minimized(&target, true))?;
    assert!(matches!(
        snapshot(&f, &root),
        Err(WindowControlSnapshotError::Unmapped)
    ));
    let target = root.clone();
    f.backend(move |s| s.set_window_minimized(&target, false))?;
    available(&snapshot(&f, &root)?, true, true);
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    assert!(matches!(
        snapshot(&f, &root),
        Err(WindowControlSnapshotError::Unmapped)
    ));
    app.configure();
    assert!(matches!(
        snapshot(&f, &root),
        Err(WindowControlSnapshotError::Unmapped)
    ));
    app.attach();
    app.sync();
    available(&snapshot(&f, &root)?, true, false);
    available(&old, true, true); // Frozen presentation is explicitly NOT live authority.
    drop(app);
    f.wait_for((1, 1));
    assert!(matches!(
        snapshot(&f, &root),
        Err(WindowControlSnapshotError::Unmapped)
    ));
    other.sync();
    assert_eq!(other.events.close_requests, 0);
    assert_eq!(other.events.sizes, [(0, 0)]);
    Ok(())
}

#[test]
fn window_controls_popup_busy_only_disables_maximize_and_capture_preserves_grab()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_popup_protocol());
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    f.render((120, 48), false, 1)?;
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
    let view = snapshot(&f, &root)?;
    available(&view, false, false);
    assert_eq!(view.maximize_refusal(), Some(&WindowMaximizeError::Busy));
    assert_eq!(maximize(&f, &root, true), Err(WindowMaximizeError::Busy));
    let popup = f
        .backend(|s| s.popup_surfaces().first().map(|p| p.surface.clone()))
        .ok_or("missing popup")?;
    assert!(matches!(
        snapshot(&f, &popup),
        Err(WindowControlSnapshotError::Unmapped)
    ));
    app.sync();
    assert_eq!(app.events.popups.done, 0);
    assert_eq!(app.events.sizes.len(), count);
    let target = root.clone();
    f.backend(move |s| s.request_window_close(&target))?;
    let target = root.clone();
    f.backend(move |s| s.set_window_minimized(&target, true))?;
    app.sync();
    assert_eq!(app.events.close_requests, 1);
    assert_eq!(app.events.popups.done, 1);
    Ok(())
}

#[test]
fn window_controls_excessive_geometry_disables_maximize_without_changing_client()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    f.render((120, 48), false, 1)?;
    let (_child, _role) = app.child((i32::MAX, i32::MAX));
    app.surface.commit();
    app.sync();
    let target = root.clone();
    let child = f
        .backend(move |_| {
            smithay::wayland::compositor::get_children(&target)
                .into_iter()
                .find(|child| child != &target)
        })
        .ok_or("missing child")?;
    assert!(matches!(
        snapshot(&f, &child),
        Err(WindowControlSnapshotError::Unmapped)
    ));
    let view = snapshot(&f, &root)?;
    available(&view, false, false);
    let refusal = WindowMaximizeError::Geometry(alo_shell::ResizeGeometryError::Geometry);
    assert_eq!(view.maximize_refusal(), Some(&refusal));
    assert_eq!(maximize(&f, &root, true), Err(refusal));
    app.sync();
    assert_eq!(app.events.sizes, [(0, 0)]);
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}

mod input;

mod feedback;
mod labels;
mod motion;
mod nested_input;
mod presentation;
mod routing;
