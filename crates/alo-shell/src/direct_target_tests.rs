//! Direct FrameTarget lifecycle using the production transaction and injected DRM.
use super::*;
use crate::{
    Cursor, FrameTarget, Popup, RenderError,
    direct_target::{ScenePainter, Target},
};
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Size},
};

#[path = "direct_target_protocol_tests.rs"]
mod protocol;

#[test]
fn replacement_connector_aliases_are_refused_before_drm_io()
-> Result<(), Box<dyn std::error::Error>> {
    for alias in [2, 3] {
        let (device, log) = fixture(&[], 0);
        let mut output = scanout_tests::output();
        output.output.connector = NonZeroU32::MIN.saturating_add(alias - 1).into();
        let mut target = Target::new(Painter::default(), device, output);
        let Err(RenderError::Scanout(error)) = target.submit(&[]) else {
            return Err("aliased display objects were not refused".into());
        };
        assert_eq!(error.failure.stage, "validate prepared scene");
        assert_eq!(error.failure.source.kind(), io::ErrorKind::InvalidData);
        drop(target);
        assert!(log.borrow().calls.is_empty());
    }
    Ok(())
}

#[test]
fn unused_direct_output_retirement_is_terminal_without_graphics_or_drm() {
    let (device, log) = fixture(&[], 0);
    let painter = Painter::default();
    let paints = painter.calls.clone();
    let mut target = Target::new(painter, device, scanout_tests::output());
    assert!(target.retire().is_ok());
    assert!(matches!(target.submit(&[]), Err(RenderError::DirectHalted)));
    assert!(matches!(target.retire(), Err(RenderError::DirectHalted)));
    drop(target);
    assert_eq!(paints.get(), 0);
    assert!(log.borrow().calls.is_empty());
}

/// Synthetic pixels; only the graphics boundary is replaced in these tests.
#[derive(Default)]
struct Painter {
    /// Number of attempts, including graphics refusal.
    calls: Rc<std::cell::Cell<usize>>,
    /// Refuse before any DRM I/O.
    refuse: bool,
}
impl ScenePainter for Painter {
    fn paint(
        &mut self,
        size: Size<i32, Physical>,
        roots: &[WlSurface],
        _: &[Popup],
        _: &Cursor,
    ) -> Result<(crate::ScanoutPixels, Vec<WlSurface>), RenderError> {
        self.calls.set(self.calls.get() + 1);
        if self.refuse {
            return Err(RenderError::Submission("injected painter refusal".into()));
        }
        Ok((
            scene_scanout_tests::pixels((size.w as u32, size.h as u32))?,
            roots.to_vec(),
        ))
    }
}

#[test]
fn direct_target_clean_refusals_allow_retry_and_drop_disables_once()
-> Result<(), Box<dyn std::error::Error>> {
    for failure in ["buffer", "upload map", "test", "enable"] {
        let (device, log) = fixture(&[], 0);
        let mut target = Target::new(Painter::default(), device, scanout_tests::output());
        assert_eq!(target.size(), (1280, 720).into());
        log.borrow_mut().failures = vec![failure];
        assert!(matches!(target.submit(&[]), Err(RenderError::Scanout(_))));
        log.borrow_mut().failures.clear();
        assert!(target.submit_scene(&[], &Cursor::Hidden)?.is_empty());
        log.borrow_mut().failures = vec![failure];
        assert!(matches!(target.submit(&[]), Err(RenderError::Scanout(_))));
        assert!(!log.borrow().calls.contains(&"disable"));
        log.borrow_mut().failures.clear();
        target.submit(&[])?;
        drop(target);
        assert_eq!(
            log.borrow()
                .calls
                .iter()
                .filter(|c| **c == "disable")
                .count(),
            1
        );
    }
    Ok(())
}

#[test]
fn direct_target_cleanup_refusal_halts_initial_and_replacement_submission()
-> Result<(), Box<dyn std::error::Error>> {
    for active in [false, true] {
        let (device, log) = fixture(&[], 0);
        let painter = Painter::default();
        let paints = painter.calls.clone();
        let mut target = Target::new(painter, device, scanout_tests::output());
        if active {
            target.submit(&[])?;
        }
        log.borrow_mut().failures = vec!["enable", "destroy framebuffer"];
        let Err(RenderError::Scanout(error)) = target.submit(&[]) else {
            return Err("expected refusal".into());
        };
        assert!(!error.cleanup.is_empty());
        let before = log.borrow().calls.clone();
        let painted = paints.get();
        log.borrow_mut().failures.clear();
        assert!(matches!(target.submit(&[]), Err(RenderError::DirectHalted)));
        assert_eq!(log.borrow().calls, before);
        assert_eq!(paints.get(), painted);
        target.disable()?;
        assert_eq!(log.borrow().calls.contains(&"disable"), active);
    }
    Ok(())
}

#[test]
fn direct_target_committed_cleanup_failure_preserves_both_shutdown_errors()
-> Result<(), Box<dyn std::error::Error>> {
    let (device, log) = fixture(&[], 0);
    let painter = Painter::default();
    let paints = painter.calls.clone();
    let mut target = Target::new(painter, device, scanout_tests::output());
    target.submit(&[])?;
    log.borrow_mut().failures = vec!["destroy framebuffer"];
    assert!(target.submit(&[])?.is_empty());
    let before = log.borrow().calls.clone();
    assert!(matches!(target.submit(&[]), Err(RenderError::DirectHalted)));
    assert_eq!(log.borrow().calls, before);
    assert_eq!(paints.get(), 2);
    log.borrow_mut().failures = vec!["disable"];
    let error = target.disable().err().ok_or("expected shutdown failure")?;
    assert_eq!(error.errors.len(), 2);
    assert_eq!(
        error.errors.first().ok_or("cleanup missing")?.failure.stage,
        "destroy framebuffer"
    );
    assert_eq!(
        error.errors.last().ok_or("disable missing")?.failure.stage,
        "atomic disable; retire session device"
    );
    assert_eq!(log.borrow().calls.last(), Some(&"disable"));
    assert_eq!(
        log.borrow()
            .calls
            .iter()
            .filter(|c| **c == "disable")
            .count(),
        1
    );
    Ok(())
}

#[test]
fn direct_target_graphics_failure_never_touches_drm() {
    let (device, log) = fixture(&[], 0);
    let mut target = Target::new(
        Painter {
            refuse: true,
            ..Default::default()
        },
        device,
        scanout_tests::output(),
    );
    assert!(matches!(
        target.submit(&[]),
        Err(RenderError::Submission(_))
    ));
    assert!(log.borrow().calls.is_empty());
}
