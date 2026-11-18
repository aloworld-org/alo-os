//! A covered captured client still receives real Wayland frame completions.

use super::support::{Application, Fixture};
use alo_shell::{Cursor, FrameTarget, RenderError};
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Size},
};
use std::sync::Arc;

/// A backend whose private capture is independent of desktop visibility.
struct CaptureTarget {
    /// Inject a failed private render before callbacks may complete.
    fail: bool,
}

impl FrameTarget for CaptureTarget {
    fn size(&self) -> Size<i32, Physical> {
        (320, 200).into()
    }
    fn submit(&mut self, _: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        // The captured application is covered: desktop submission contains none
        // of its surfaces and therefore cannot complete its frame callbacks.
        Ok(Vec::new())
    }
    fn capture_window(
        &mut self,
        _: Size<i32, Physical>,
        _: &WlSurface,
        _: &[alo_shell::Popup],
        _: &Cursor,
        _: alo_canvas::Camera,
    ) -> Result<Arc<[u8]>, RenderError> {
        if self.fail {
            Err(RenderError::Submission("capture failed".into()))
        } else {
            Ok(Arc::from(vec![0; 16 * 16 * 4]))
        }
    }
}

/// Capture the named application without submitting it to the desktop.
fn capture(fixture: &Fixture, fail: bool, time: u32) -> Result<(), RenderError> {
    fixture.backend(move |server| {
        let root = server
            .mapped_surfaces()
            .find(|root| server.the_name_of(root).its_own_words() == Some("Captured application"))
            .cloned()
            .ok_or_else(|| RenderError::Submission("selected application is not mapped".into()))?;
        CaptureTarget { fail }
            .shared_capture_window_at(
                (16, 16).into(),
                &root,
                &server.popup_surfaces(),
                &Cursor::Default,
                alo_canvas::Camera::new(),
                time,
            )
            .map(|_| ())
    })
}

#[test]
/// Failed, repeated and unrelated captures cannot claim a new client frame.
fn covered_capture_completes_only_its_mapped_tree_after_success() -> Result<(), RenderError> {
    let fixture = Fixture::new();
    let mut selected = Application::new(&fixture);
    selected.configure();
    selected.toplevel.set_title("Captured application".into());
    selected.surface.frame(&selected.queue.handle(), ());
    let (child, role) = selected.child((0, 0));
    selected.attach();
    selected.sync();
    let mut other = Application::new(&fixture);
    other.configure();
    other.toplevel.set_title("Other application".into());
    other.surface.frame(&other.queue.handle(), ());
    other.attach();
    other.sync();

    assert!(capture(&fixture, true, 10).is_err());
    selected.sync();
    other.sync();
    assert!(selected.events.frames.is_empty());
    assert!(other.events.frames.is_empty());
    capture(&fixture, false, 20)?;
    selected.sync();
    other.sync();
    assert_eq!(selected.events.frames, [20, 20]);
    assert!(other.events.frames.is_empty());
    capture(&fixture, false, 21)?;
    selected.sync();
    assert_eq!(selected.events.frames, [20, 20]);

    child.frame(&selected.queue.handle(), ());
    child.attach(None, 0, 0);
    child.commit();
    selected.surface.frame(&selected.queue.handle(), ());
    selected.surface.commit();
    selected.sync();
    capture(&fixture, false, 30)?;
    selected.sync();
    other.sync();
    assert_eq!(selected.events.frames, [20, 20, 30]);
    assert!(other.events.frames.is_empty());
    role.destroy();
    child.destroy();
    Ok(())
}
