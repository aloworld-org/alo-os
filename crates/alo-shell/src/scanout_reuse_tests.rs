//! Reuse is bounded, never maps active storage, and preserves refusal cleanup.

use super::*;
use crate::scene_scanout::{Submitted, activate};
use scene_scanout_tests::pixels;

#[test]
fn the_software_desktop_target_uses_reuse_and_retires_both_allocations()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::{Cursor, FrameTarget, Popup, RenderError, ScanoutPixels};
    use smithay::{
        reexports::wayland_server::protocol::wl_surface::WlSurface,
        utils::{Physical, Size},
    };
    struct Painter;
    impl crate::direct_target::ScenePainter for Painter {
        fn paint(
            &mut self,
            size: Size<i32, Physical>,
            _: &[WlSurface],
            _: &[Popup],
            _: &Cursor,
            _: crate::scene_native::NativeLayers<'_>,
            _: alo_canvas::Camera,
        ) -> Result<(ScanoutPixels, Vec<WlSurface>), RenderError> {
            Ok((pixels((size.w as u32, size.h as u32))?, vec![]))
        }
    }
    let (device, log) = fixture(&[], 0);
    let mut target = crate::direct_target::Target::new(Painter, device, scanout_tests::output())
        .with_reusable_scanout();
    for _ in 0..5 {
        target.submit(&[])?;
    }
    assert_eq!(
        log.borrow()
            .calls
            .iter()
            .filter(|op| **op == "buffer")
            .count(),
        2
    );
    target.disable()?;
    assert_eq!(
        log.borrow()
            .calls
            .iter()
            .filter(|op| **op == "destroy buffer")
            .count(),
        2
    );
    Ok(())
}

fn reusable(device: Device) -> Result<Submitted<Device>, Box<dyn std::error::Error>> {
    let mut scene = activate(
        (pixels((1280, 720))?, vec![]),
        device,
        &scanout_tests::output(),
    )?;
    scene.reuse_buffers = true;
    Ok(scene)
}

#[test]
fn software_frames_rotate_only_two_initialized_buffers() -> Result<(), Box<dyn std::error::Error>> {
    let (device, log) = fixture(&[], 0);
    let mut scene = reusable(device)?;
    for index in 0..6 {
        let next = crate::readback::convert(
            (1280, 720),
            crate::RowOrder::TopToBottom,
            &vec![index + 30; 1280 * 720 * 4],
        )?;
        assert!(scene.replace((next, vec![]))?.retirement_error.is_none());
        assert_eq!(
            scene.active.owned.bindings(),
            Some(if index % 2 == 0 {
                (NonZeroU32::MIN.saturating_add(21).into(), 73)
            } else {
                (NonZeroU32::MIN.saturating_add(20).into(), 72)
            })
        );
        assert!(!scene.spare.as_ref().ok_or("missing spare")?.enabled);
        assert_eq!(
            log.borrow()
                .pixels
                .get(..4)
                .ok_or("missing uploaded pixel")?,
            &[index + 30, index + 30, index + 30, 0]
        );
    }
    assert_eq!(
        log.borrow()
            .calls
            .iter()
            .filter(|op| **op == "buffer")
            .count(),
        2
    );
    assert!(log.borrow().released.is_empty());
    scene.retire()?;
    assert_eq!(log.borrow().released.len(), 4);
    assert_eq!(
        log.borrow()
            .calls
            .iter()
            .filter(|op| **op == "destroy buffer")
            .count(),
        2
    );
    assert_eq!(
        log.borrow()
            .calls
            .iter()
            .filter(|op| **op == "disable")
            .count(),
        1
    );
    let before = log.borrow().calls.clone();
    drop(scene);
    assert_eq!(log.borrow().calls, before);
    Ok(())
}

#[test]
fn reused_frame_refusals_keep_the_previous_scanout_and_allow_clean_retry()
-> Result<(), Box<dyn std::error::Error>> {
    for failure in ["map", "unmap", "test", "enable"] {
        let (device, log) = fixture(&[], 0);
        let mut scene = reusable(device)?;
        assert!(
            scene
                .replace((pixels((1280, 720))?, vec![]))?
                .retirement_error
                .is_none()
        );
        let active = scene.active.owned.bindings();
        log.borrow_mut().failures = vec![failure];
        let error = scene
            .replace((pixels((1280, 720))?, vec![]))
            .err()
            .ok_or("refusal lost")?;
        assert_eq!(error.failure.source.raw_os_error(), Some(5));
        assert_eq!(scene.active.owned.bindings(), active);
        assert!(scene.active.enabled);
        assert!(!log.borrow().calls.contains(&"disable"));
        assert!(!log.borrow().released.contains(&73));
        assert!(!log.borrow().released.contains(&22));
        log.borrow_mut().failures.clear();
        assert!(
            scene
                .replace((pixels((1280, 720))?, vec![]))?
                .retirement_error
                .is_none()
        );
        scene.retire()?;
    }
    Ok(())
}

#[test]
fn reused_frame_cleanup_failure_latches_the_output_until_retirement()
-> Result<(), Box<dyn std::error::Error>> {
    let (device, log) = fixture(&[], 0);
    let mut scene = reusable(device)?;
    assert!(
        scene
            .replace((pixels((1280, 720))?, vec![]))?
            .retirement_error
            .is_none()
    );
    log.borrow_mut().failures = vec![
        "test",
        "destroy blob",
        "destroy framebuffer",
        "destroy buffer",
    ];
    let error = scene
        .replace((pixels((1280, 720))?, vec![]))
        .err()
        .ok_or("refusal lost")?;
    assert_eq!(error.cleanup.len(), 3);
    let before = log.borrow().calls.clone();
    assert!(scene.replace((pixels((1280, 720))?, vec![])).is_err());
    assert_eq!(log.borrow().calls, before);
    log.borrow_mut().failures.clear();
    scene.retire()?;
    Ok(())
}

#[test]
fn failed_disable_quarantines_active_storage_and_releases_only_the_detached_spare()
-> Result<(), Box<dyn std::error::Error>> {
    let (device, log) = fixture(&[], 0);
    let mut scene = reusable(device)?;
    assert!(
        scene
            .replace((pixels((1280, 720))?, vec![]))?
            .retirement_error
            .is_none()
    );
    log.borrow_mut().failures = vec!["disable"];
    let error = scene.retire().err().ok_or("disable accepted")?;
    assert_eq!(error.failure.stage, "atomic disable; retire session device");
    assert_eq!(log.borrow().released, [72, 21]);
    let before = log.borrow().calls.clone();
    drop(scene);
    assert_eq!(log.borrow().calls, before);
    Ok(())
}

#[test]
fn active_storage_cannot_be_overwritten_by_the_reuse_path() -> Result<(), Box<dyn std::error::Error>>
{
    let (device, log) = fixture(&[], 0);
    let scene = reusable(device)?;
    let before = log.borrow().calls.iter().filter(|op| **op == "map").count();
    let next = pixels((1280, 720))?;
    let error = scene
        .active
        .reuse(&next.frame()?)
        .err()
        .ok_or("active upload accepted")?;
    assert_eq!(error.failure.stage, "cannot overwrite active scanout");
    assert_eq!(
        log.borrow().calls.iter().filter(|op| **op == "map").count(),
        before
    );
    Ok(())
}

#[test]
fn retirement_reports_active_and_spare_cleanup_failures_without_retry()
-> Result<(), Box<dyn std::error::Error>> {
    let (device, log) = fixture(&[], 0);
    let mut scene = reusable(device)?;
    assert!(
        scene
            .replace((pixels((1280, 720))?, vec![]))?
            .retirement_error
            .is_none()
    );
    log.borrow_mut().failures = vec!["destroy blob", "destroy framebuffer", "destroy buffer"];
    let error = scene.retire().err().ok_or("cleanup accepted")?;
    assert_eq!(error.failure.stage, "destroy mode blob");
    assert_eq!(error.cleanup.len(), 5);
    let before = log.borrow().calls.clone();
    drop(scene);
    assert_eq!(log.borrow().calls, before);
    Ok(())
}
