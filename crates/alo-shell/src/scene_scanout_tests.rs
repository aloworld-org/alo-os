//! Prepared pixels through the production activation transaction with injected DRM.
use super::*;
use crate::{RowOrder, scene_scanout::activate};

fn pixels(size: (u32, u32)) -> Result<crate::ScanoutPixels, crate::ReadbackError> {
    let rgba: Vec<_> = (0..size.0 * size.1)
        .flat_map(|pixel| [19, (pixel / size.0) as u8, pixel as u8, 255])
        .collect();
    crate::readback::convert(size, RowOrder::TopToBottom, &rgba)
}

#[test]
fn prepared_scene_activation_preserves_pixels_until_ordered_retirement()
-> Result<(), Box<dyn std::error::Error>> {
    let (device, log) = fixture(&[], 0);
    let mut scene = activate(
        (pixels((1280, 720))?, vec![]),
        device,
        &scanout_tests::output(),
    )?;
    assert!(scene.surfaces.is_empty());
    assert_eq!(log.borrow().pixels.len(), 5184 * 720 + 37);
    for (y, row) in log.borrow().pixels.as_chunks::<5184>().0.iter().enumerate() {
        for (x, pixel) in row
            .get(..5120)
            .ok_or("missing row")?
            .as_chunks::<4>()
            .0
            .iter()
            .enumerate()
        {
            assert_eq!(pixel, &[(y * 1280 + x) as u8, y as u8, 19, 0]);
        }
        assert!(
            row.get(5120..)
                .ok_or("missing padding")?
                .iter()
                .all(|byte| *byte == 0)
        );
    }
    assert!(
        log.borrow()
            .pixels
            .as_chunks::<5184>()
            .1
            .iter()
            .all(|byte| *byte == 0)
    );
    assert!(!log.borrow().calls.contains(&"destroy buffer"));
    scene.active.retire()?;
    drop(scene);
    assert_eq!(
        log.borrow().calls,
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
            "disable",
            "destroy blob",
            "destroy framebuffer",
            "destroy buffer"
        ]
    );
    Ok(())
}

#[test]
fn prepared_scene_size_mismatch_refuses_before_any_device_call()
-> Result<(), Box<dyn std::error::Error>> {
    let (device, log) = fixture(&[], 0);
    let error = activate(
        (pixels((33, 32))?, vec![]),
        device,
        &scanout_tests::output(),
    )
    .err()
    .ok_or("mismatched scene enabled")?;
    assert_eq!(error.failure.stage, "validate prepared scene");
    assert_eq!(error.failure.source.kind(), io::ErrorKind::InvalidInput);
    assert!(log.borrow().calls.is_empty());
    Ok(())
}

#[test]
fn prepared_scene_refusals_never_return_ownership_and_retain_cleanup()
-> Result<(), Box<dyn std::error::Error>> {
    for stage in ["buffer", "framebuffer", "blob", "upload", "test", "enable"] {
        let (mut device, log) = fixture(
            &["destroy blob", "destroy framebuffer", "destroy buffer"],
            0,
        );
        // Only the second mapping is an upload: fail it separately from initialization.
        device.failures.push(if stage == "upload" {
            "upload map"
        } else {
            stage
        });
        let error = activate(
            (pixels((1280, 720))?, vec![]),
            device,
            &scanout_tests::output(),
        )
        .err()
        .ok_or("failed transaction returned active scene")?;
        assert_eq!(error.failure.source.raw_os_error(), Some(5));
        assert_eq!(
            error.cleanup.len(),
            match stage {
                "buffer" => 0,
                "framebuffer" => 1,
                "blob" => 2,
                _ => 3,
            }
        );
        assert!(!log.borrow().calls.contains(&"disable"));
        if stage != "enable" {
            assert!(!log.borrow().calls.contains(&"enable"));
        }
        for name in ["destroy blob", "destroy framebuffer", "destroy buffer"] {
            assert!(
                log.borrow()
                    .calls
                    .iter()
                    .filter(|call| **call == name)
                    .count()
                    <= 1
            );
        }
    }
    Ok(())
}

#[test]
fn prepared_scene_disable_refusal_quarantines_without_drop_retry()
-> Result<(), Box<dyn std::error::Error>> {
    let (device, log) = fixture(&["disable"], 0);
    let mut scene = activate(
        (pixels((1280, 720))?, vec![]),
        device,
        &scanout_tests::output(),
    )?;
    let error = scene.active.retire().err().ok_or("disable accepted")?;
    assert_eq!(error.failure.source.raw_os_error(), Some(5));
    drop(scene);
    assert!(!log.borrow().calls.contains(&"destroy buffer"));
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
