//! Cursor authorization and lifetime over actual Unix-socket Wayland requests.
use super::support::{Application, Fixture};
use alo_shell::{Cursor, FrameTarget, RenderError};
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Size},
};

fn setup() -> (Fixture, Application) {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    let mut app = Application::new(&f);
    app.configure();
    app.attach();
    app.sync();
    assert!(f.backend(|s| s.pointer_motion(8.0, 9.0, 1)).is_ok());
    app.sync();
    (f, app)
}

#[test]
fn default_arrow_tracks_empty_desktop_and_refuses_invalid_motion_and_legacy_target() {
    let f = Fixture::keyboard();
    assert!(matches!(f.backend(|s| s.cursor()), Cursor::Default));
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    for location in [(0.0, 0.0), (25.75, 30.5), (-2.0, -4.0)] {
        assert!(
            f.backend(move |s| s.pointer_motion(location.0, location.1, 1))
                .is_ok()
        );
        assert!(
            matches!(f.backend(|s| s.cursor()), Cursor::Arrow { location: actual } if actual == location.into())
        );
    }
    assert!(f.backend(|s| s.pointer_motion(f64::NAN, 1.0, 2)).is_err());
    assert!(
        matches!(f.backend(|s| s.cursor()), Cursor::Arrow { location } if location == (-2.0, -4.0).into())
    );
    assert!(f.backend(|s| s.render(&mut LegacyTarget, 3)).is_err());
    assert_eq!(f.backend(|s| s.render(&mut Target(false), 4)).ok(), Some(0));
}

/// Older backend deliberately implements no cursor-aware submission method.
struct LegacyTarget;
impl FrameTarget for LegacyTarget {
    fn size(&self) -> Size<i32, Physical> {
        (320, 200).into()
    }
    fn submit(&mut self, roots: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        Ok(roots.to_vec())
    }
}

#[test]
fn cursor_hotspot_hidden_movement_and_focus_cleanup() {
    let (f, mut app) = setup();
    let cursor = app.cursor((2, 3));
    app.sync();
    assert!(
        matches!(f.backend(|s| s.cursor()), Cursor::Surface { location, .. } if location == (6.0, 6.0).into())
    );
    assert!(f.backend(|s| s.pointer_motion(10.0, 11.0, 2)).is_ok());
    assert!(
        matches!(f.backend(|s| s.cursor()), Cursor::Surface { location, .. } if location == (8.0, 8.0).into())
    );
    app.set_cursor(app.events.pointer.serial, None, (0, 0));
    app.sync();
    assert!(matches!(f.backend(|s| s.cursor()), Cursor::Hidden));
    app.set_cursor(
        app.events.pointer.serial,
        Some(&cursor),
        (i32::MIN, i32::MAX),
    );
    app.sync();
    assert!(matches!(f.backend(|s| s.cursor()), Cursor::Surface { .. }));
    cursor.destroy();
    app.sync();
    assert!(matches!(f.backend(|s| s.cursor()), Cursor::Arrow { .. }));
    let _cursor = app.cursor((0, 0));
    app.sync();
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    assert!(matches!(f.backend(|s| s.cursor()), Cursor::Arrow { .. }));
}

#[test]
fn cursor_rejects_unfocused_client_stale_serial_and_conflicting_role() {
    let (f, mut app) = setup();
    let _cursor = app.cursor((1, 2));
    app.sync();
    let mut other = Application::new(&f);
    other.configure();
    other.attach();
    other.sync();
    other.set_cursor(app.events.pointer.serial, None, (0, 0));
    other.sync();
    app.set_cursor(app.events.pointer.serial.wrapping_sub(1), None, (0, 0));
    app.sync();
    assert!(
        matches!(f.backend(|s| s.cursor()), Cursor::Surface { location, .. } if location == (7.0, 7.0).into())
    );
    app.set_cursor(app.events.pointer.serial, Some(&app.surface), (0, 0));
    app.refused();
    f.wait_for((1, 1));
    assert!(matches!(f.backend(|s| s.cursor()), Cursor::Arrow { .. }));
    other.sync();
    assert!(f.backend(|s| s.pointer_motion(5.0, 6.0, 3)).is_ok());
    other.sync();
    let _cursor = other.cursor((0, 0));
    other.sync();
    assert!(matches!(f.backend(|s| s.cursor()), Cursor::Surface { .. }));
    drop(other);
    f.wait_for((0, 0));
    assert!(matches!(f.backend(|s| s.cursor()), Cursor::Arrow { .. }));
}

/// Controlled submission accepts the cursor only when the simulated swap succeeds.
struct Target(bool);
impl FrameTarget for Target {
    fn size(&self) -> Size<i32, Physical> {
        (320, 200).into()
    }
    fn submit(&mut self, roots: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        Ok(roots.to_vec())
    }
    fn submit_scene(
        &mut self,
        roots: &[WlSurface],
        cursor: &Cursor,
    ) -> Result<Vec<WlSurface>, RenderError> {
        if self.0 {
            return Err(RenderError::Submission(
                "injected cursor swap failure".into(),
            ));
        }
        let mut result = roots.to_vec();
        if let Cursor::Surface { surface, .. } = cursor {
            result.push(surface.clone());
        }
        Ok(result)
    }
}

#[test]
fn cursor_callbacks_wait_for_submission_and_backend_support() {
    let (f, mut app) = setup();
    let _cursor = app.cursor((0, 0));
    app.sync();
    assert!(f.render((320, 200), false, 1).is_err());
    assert!(f.backend(|s| s.render(&mut Target(true), 2)).is_err());
    app.sync();
    assert!(app.events.frames.is_empty());
    assert_eq!(f.backend(|s| s.render(&mut Target(false), 3)).ok(), Some(2));
    app.sync();
    assert_eq!(app.events.frames, [3]);
    assert!(f.backend(|s| s.pointer_leave()).is_ok());
    assert!(matches!(f.backend(|s| s.cursor()), Cursor::Arrow { .. }));
}
