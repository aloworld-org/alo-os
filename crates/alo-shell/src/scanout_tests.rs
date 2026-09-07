//! Exercise the real allocation owner through blocking commit and cleanup paths.

use super::*;
use crate::{
    DirectOutput,
    scanout::{Scanout, ScanoutDevice},
};
use drm::control::{AtomicCommitFlags, atomic::AtomicModeReq};

impl ScanoutDevice for Device {
    fn commit(&self, flags: AtomicCommitFlags, request: AtomicModeReq) -> io::Result<()> {
        let disabling = self.log.borrow().calls.contains(&"enable");
        // drm-rs exposes Debug, but not a request iterator. Compare independently
        // constructed wire requests, including allocation-owned framebuffer/blob.
        assert_eq!(
            format!("{request:?}"),
            format!("{:?}", expected_request(disabling))
        );
        assert!(
            !flags.intersects(AtomicCommitFlags::NONBLOCK | AtomicCommitFlags::PAGE_FLIP_EVENT)
        );
        if flags.contains(AtomicCommitFlags::TEST_ONLY) {
            assert_eq!(
                flags,
                AtomicCommitFlags::TEST_ONLY | AtomicCommitFlags::ALLOW_MODESET
            );
            self.call("test")
        } else {
            assert_eq!(flags, AtomicCommitFlags::ALLOW_MODESET);
            if self.log.borrow().calls.contains(&"enable") {
                self.call("disable")
            } else {
                self.call("enable")
            }
        }
    }
}

/// Expected kernel request from the fixture's documented routing and resources.
fn expected_request(disabling: bool) -> AtomicModeReq {
    let writes = if disabling {
        vec![(1, 11, 0), (2, 21, 0), (2, 22, 0), (3, 31, 0), (3, 32, 0)]
    } else {
        vec![
            (1, 11, 2),
            (2, 21, 1),
            (2, 22, 72),
            (3, 31, 2),
            (3, 32, 21),
            (3, 33, 0),
            (3, 34, 0),
            (3, 35, 1280),
            (3, 36, 720),
            (3, 37, 0),
            (3, 38, 0),
            (3, 39, 1280 << 16),
            (3, 40, 720 << 16),
        ]
    };
    let mut request = AtomicModeReq::new();
    for (object, property, value) in writes {
        request.add_raw_property(
            NonZeroU32::MIN.saturating_add(object - 1),
            NonZeroU32::MIN.saturating_add(property - 1).into(),
            value,
        );
    }
    request
}

/// Frozen standard routing for the same mode used by the allocation fixture.
pub(super) fn plan() -> io::Result<AtomicPlan> {
    let properties = |names: &[&'static str], start: u32| {
        names
            .iter()
            .zip(start..)
            .map(|(name, id)| (*name, NonZeroU32::MIN.saturating_add(id).into()))
            .collect()
    };
    AtomicPlan::new(&AtomicOutput {
        output: DirectOutput {
            connector: NonZeroU32::MIN.into(),
            crtc: NonZeroU32::MIN.saturating_add(1).into(),
            mode: mode(),
        },
        plane: NonZeroU32::MIN.saturating_add(2).into(),
        formats: vec![DrmFourcc::Xrgb8888 as u32],
        connector_properties: properties(&["CRTC_ID"], 10),
        crtc_properties: properties(&["ACTIVE", "MODE_ID"], 20),
        plane_properties: properties(
            &[
                "CRTC_ID", "FB_ID", "CRTC_X", "CRTC_Y", "CRTC_W", "CRTC_H", "SRC_X", "SRC_Y",
                "SRC_W", "SRC_H",
            ],
            30,
        ),
    })
}

#[test]
fn active_resources_survive_until_blocking_disable_then_release_once()
-> Result<(), Box<dyn std::error::Error>> {
    for explicit in [false, true] {
        let (device, log) = fixture(&[], 0);
        let (owned, fb, blob) = allocate(device)?;
        let mut active = Scanout::activate(owned, plan()?, fb, blob)?;
        assert_eq!(
            log.borrow().calls,
            [
                "buffer",
                "map",
                "unmap",
                "framebuffer",
                "blob",
                "test",
                "enable"
            ]
        );
        if explicit {
            active.retire()?;
        }
        drop(active);
        assert_eq!(
            log.borrow().calls,
            [
                "buffer",
                "map",
                "unmap",
                "framebuffer",
                "blob",
                "test",
                "enable",
                "disable",
                "destroy blob",
                "destroy framebuffer",
                "destroy buffer"
            ]
        );
    }
    Ok(())
}

#[test]
fn test_and_enable_refusals_preserve_errno_and_all_cleanup_errors()
-> Result<(), Box<dyn std::error::Error>> {
    for stage in ["test", "enable"] {
        for cleanup in [false, true] {
            let mut failures = vec![stage];
            if cleanup {
                failures.extend(["destroy blob", "destroy framebuffer", "destroy buffer"]);
            }
            let (device, log) = fixture(&failures, 0);
            let (owned, fb, blob) = allocate(device)?;
            let error = Scanout::activate(owned, plan()?, fb, blob)
                .err()
                .ok_or("commit accepted")?;
            assert_eq!(
                error.failure.stage,
                if stage == "test" {
                    "atomic TEST_ONLY"
                } else {
                    "atomic enable"
                }
            );
            assert_eq!(error.failure.source.raw_os_error(), Some(5));
            assert_eq!(error.cleanup.len(), if cleanup { 3 } else { 0 });
            assert!(!log.borrow().calls.contains(&"disable"));
            assert_eq!(
                log.borrow()
                    .calls
                    .iter()
                    .filter(|call| **call == "enable")
                    .count(),
                usize::from(stage == "enable")
            );
            for name in ["destroy blob", "destroy framebuffer", "destroy buffer"] {
                assert_eq!(
                    log.borrow()
                        .calls
                        .iter()
                        .filter(|call| **call == name)
                        .count(),
                    1
                );
            }
        }
    }
    Ok(())
}

#[test]
fn disable_refusal_quarantines_resources_and_never_retries_on_drop()
-> Result<(), Box<dyn std::error::Error>> {
    for explicit in [false, true] {
        let (device, log) = fixture(&["disable"], 0);
        let (owned, fb, blob) = allocate(device)?;
        let mut active = Scanout::activate(owned, plan()?, fb, blob)?;
        if explicit {
            let error = active.retire().err().ok_or("disable accepted")?;
            assert_eq!(error.failure.source.raw_os_error(), Some(5));
            assert_eq!(error.failure.stage, "atomic disable; retire session device");
            assert!(error.cleanup.is_empty());
        }
        drop(active);
        assert_eq!(
            log.borrow().calls,
            [
                "buffer",
                "map",
                "unmap",
                "framebuffer",
                "blob",
                "test",
                "enable",
                "disable"
            ]
        );
    }
    Ok(())
}

#[test]
fn successful_disable_preserves_all_release_failures_without_second_disable()
-> Result<(), Box<dyn std::error::Error>> {
    let (device, log) = fixture(
        &["destroy blob", "destroy framebuffer", "destroy buffer"],
        0,
    );
    let (owned, fb, blob) = allocate(device)?;
    let mut active = Scanout::activate(owned, plan()?, fb, blob)?;
    let error = active.retire().err().ok_or("cleanup accepted")?;
    assert_eq!(error.failure.stage, "destroy mode blob");
    assert_eq!(
        error.cleanup.iter().map(|e| e.stage).collect::<Vec<_>>(),
        ["destroy framebuffer", "destroy dumb buffer"]
    );
    drop(active);
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
