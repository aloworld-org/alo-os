//! Multiple distinct allocations through the production scene replacement path.
use super::*;
use crate::scene_scanout::activate;
use scene_scanout_tests::pixels;

#[test]
fn replacement_commits_before_old_cleanup_without_disabling_new_scene()
-> Result<(), Box<dyn std::error::Error>> {
    let (device, log) = fixture(&[], 0);
    let mut scene = activate(
        (pixels((1280, 720))?, vec![]),
        device,
        &scanout_tests::output(),
    )?;
    for index in 0..2 {
        let start = log.borrow().calls.len();
        let new_pixels = crate::readback::convert(
            (1280, 720),
            crate::RowOrder::TopToBottom,
            &vec![index + 30; 1280 * 720 * 4],
        )?;
        let result = scene.replace((new_pixels, vec![]))?;
        assert!(result.retirement_error.is_none());
        assert_eq!(
            log.borrow().calls.get(start..).ok_or("missing calls")?,
            [
                "buffer",
                "map",
                "unmap",
                "framebuffer",
                "blob",
                "map",
                "unmap",
                "test",
                "enable",
                "destroy blob",
                "destroy framebuffer",
                "destroy buffer"
            ]
        );
        assert_eq!(
            log.borrow().pixels.get(..4).ok_or("missing pixels")?,
            &[index + 30, index + 30, index + 30, 0]
        );
    }
    assert_eq!(log.borrow().released, [72, 21, 73, 22]);
    scene.active.retire()?;
    drop(scene);
    assert_eq!(log.borrow().released, [72, 21, 73, 22, 74, 23]);
    assert_eq!(
        log.borrow()
            .calls
            .iter()
            .filter(|call| **call == "disable")
            .count(),
        1
    );
    Ok(())
}

#[test]
fn replacement_refusals_preserve_old_allocation_and_allow_clean_retry()
-> Result<(), Box<dyn std::error::Error>> {
    for failure in [
        "buffer",
        "map",
        "framebuffer",
        "blob",
        "upload map",
        "test",
        "enable",
    ] {
        let (device, log) = fixture(&[], 0);
        let mut scene = activate(
            (pixels((1280, 720))?, vec![]),
            device,
            &scanout_tests::output(),
        )?;
        log.borrow_mut().failures.push(failure);
        let error = scene
            .replace((pixels((1280, 720))?, vec![]))
            .err()
            .ok_or("replacement accepted")?;
        assert_eq!(error.failure.source.raw_os_error(), Some(5));
        assert!(!log.borrow().released.contains(&21));
        assert!(!log.borrow().released.contains(&72));
        assert!(!log.borrow().calls.contains(&"disable"));
        log.borrow_mut().failures.clear();
        assert!(
            scene
                .replace((pixels((1280, 720))?, vec![]))?
                .retirement_error
                .is_none()
        );
        scene.active.retire()?;
    }
    Ok(())
}

#[test]
fn replacement_size_refuses_without_io_and_retired_owner_cannot_replace()
-> Result<(), Box<dyn std::error::Error>> {
    let (device, log) = fixture(&[], 0);
    let mut scene = activate(
        (pixels((1280, 720))?, vec![]),
        device,
        &scanout_tests::output(),
    )?;
    let before = log.borrow().calls.clone();
    assert_eq!(
        scene
            .replace((pixels((2, 2))?, vec![]))
            .err()
            .ok_or("size accepted")?
            .failure
            .stage,
        "validate replacement scene"
    );
    assert_eq!(log.borrow().calls, before);
    scene.active.retire()?;
    let before = log.borrow().calls.clone();
    assert!(scene.replace((pixels((1280, 720))?, vec![])).is_err());
    assert_eq!(log.borrow().calls, before);
    Ok(())
}

#[test]
fn replacement_cleanup_failure_reports_committed_state_and_requires_retirement()
-> Result<(), Box<dyn std::error::Error>> {
    for refused_commit in [false, true] {
        let (device, log) = fixture(&[], 0);
        let mut scene = activate(
            (pixels((1280, 720))?, vec![]),
            device,
            &scanout_tests::output(),
        )?;
        log.borrow_mut().failures = vec!["destroy blob", "destroy framebuffer", "destroy buffer"];
        if refused_commit {
            log.borrow_mut().failures.push("enable");
        }
        let result = scene.replace((pixels((1280, 720))?, vec![]));
        if refused_commit {
            assert_eq!(result.err().ok_or("commit accepted")?.cleanup.len(), 3);
            assert_eq!(log.borrow().released, [73, 22]);
        } else {
            let error = result?.retirement_error.ok_or("cleanup accepted")?;
            assert_eq!(error.failure.stage, "destroy mode blob");
            assert_eq!(error.cleanup.len(), 2);
            assert_eq!(log.borrow().released, [72, 21]);
        }
        let before = log.borrow().calls.clone();
        assert_eq!(
            scene
                .replace((pixels((1280, 720))?, vec![]))
                .err()
                .ok_or("continued after cleanup failure")?
                .failure
                .stage,
            "scene requires session retirement"
        );
        assert_eq!(log.borrow().calls, before);
        log.borrow_mut().failures = vec!["disable"];
        assert!(scene.active.retire().is_err());
        let before = log.borrow().calls.clone();
        drop(scene);
        assert_eq!(log.borrow().calls, before);
    }
    Ok(())
}
