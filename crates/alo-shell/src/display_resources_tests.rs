//! Resource lifetime tests with ioctl failure injection and real fd refusal.

use super::*;
use std::{cell::RefCell, num::NonZeroU32, os::fd::AsFd, rc::Rc};

#[path = "scanout_tests.rs"]
mod scanout_tests;

#[path = "scanout_frame_tests.rs"]
mod scanout_frame_tests;

#[path = "scene_scanout_tests.rs"]
mod scene_scanout_tests;

/// A public-metadata buffer standing in for a kernel dumb allocation.
struct FakeBuffer {
    /// Advertised dimensions.
    size: (u32, u32),
    /// Advertised row stride.
    pitch: u32,
    /// Advertised format.
    format: DrmFourcc,
}
impl Buffer for FakeBuffer {
    fn size(&self) -> (u32, u32) {
        self.size
    }
    fn pitch(&self) -> u32 {
        self.pitch
    }
    fn format(&self) -> DrmFourcc {
        self.format
    }
    fn handle(&self) -> drm::buffer::Handle {
        NonZeroU32::MIN.into()
    }
}

/// Shared event log outlives the cleanup owner.
#[derive(Default)]
struct Log {
    /// Ordered allocation/destruction requests.
    calls: Vec<&'static str>,
    /// Exact mode passed to the blob transport.
    mode: Option<Mode>,
    /// Bytes observed immediately before unmapping.
    pixels: Vec<u8>,
}

/// Configurable failures, including several cleanup failures on the same path.
struct Device {
    /// Observed operations.
    log: Rc<RefCell<Log>>,
    /// Operations refused with a stable errno.
    failures: Vec<&'static str>,
    /// Malformed buffer metadata variant.
    malformed: u8,
}
impl Device {
    /// Record even refused operations.
    fn call(&self, name: &'static str) -> io::Result<()> {
        self.log.borrow_mut().calls.push(name);
        if self.failures.contains(&name) {
            Err(io::Error::from_raw_os_error(5))
        } else {
            Ok(())
        }
    }
}
impl ResourceDevice for Device {
    type Buffer = FakeBuffer;
    fn create_buffer(&self, size: (u32, u32)) -> io::Result<FakeBuffer> {
        self.call("buffer")?;
        Ok(FakeBuffer {
            size: if self.malformed == 1 { (1, 1) } else { size },
            pitch: match self.malformed {
                2 => 4,
                3 => size.0 * 4 + 1,
                _ => size.0 * 4 + 64,
            },
            format: if self.malformed == 4 {
                DrmFourcc::Argb8888
            } else {
                DrmFourcc::Xrgb8888
            },
        })
    }
    fn with_mapping(
        &self,
        buffer: &mut FakeBuffer,
        initialize: impl FnOnce(&mut [u8]) -> io::Result<()>,
    ) -> io::Result<()> {
        self.call("map")?;
        if self.failures.contains(&"upload map") && self.log.borrow().calls.contains(&"blob") {
            return Err(io::Error::from_raw_os_error(5));
        }
        let required = buffer.pitch as usize * buffer.size.1 as usize;
        let length = if self.malformed == 7 {
            required - 1
        } else {
            required + 37
        };
        let mut bytes = vec![0xa5; length];
        let result = initialize(&mut bytes);
        self.log.borrow_mut().pixels = bytes;
        self.call("unmap")?;
        result
    }
    fn create_framebuffer(&self, _: &FakeBuffer) -> io::Result<framebuffer::Handle> {
        self.call("framebuffer")?;
        Ok(NonZeroU32::MIN.saturating_add(20).into())
    }
    fn create_blob(&self, mode: &Mode) -> io::Result<u64> {
        self.call("blob")?;
        self.log.borrow_mut().mode = Some(*mode);
        Ok(match self.malformed {
            5 => 0,
            6 => u64::from(u32::MAX) + 1,
            _ => 72,
        })
    }
    fn destroy_blob(&self, blob: u64) -> io::Result<()> {
        assert_eq!(blob, 72);
        self.call("destroy blob")
    }
    fn destroy_framebuffer(&self, fb: framebuffer::Handle) -> io::Result<()> {
        assert_eq!(u32::from(fb), 21);
        self.call("destroy framebuffer")
    }
    fn destroy_buffer(&self, _: FakeBuffer) -> io::Result<()> {
        self.call("destroy buffer")
    }
}

/// A realistic mode with nontrivial timing bytes to preserve.
fn mode() -> Mode {
    drm_ffi::drm_mode_modeinfo {
        clock: 74250,
        hdisplay: 1280,
        hsync_start: 1390,
        hsync_end: 1430,
        htotal: 1650,
        vdisplay: 720,
        vsync_start: 725,
        vsync_end: 730,
        vtotal: 750,
        vrefresh: 60,
        ..Default::default()
    }
    .into()
}

/// Fresh log and independently configurable transport.
fn fixture(failures: &[&'static str], malformed: u8) -> (Device, Rc<RefCell<Log>>) {
    let log = Rc::new(RefCell::new(Log::default()));
    (
        Device {
            log: log.clone(),
            failures: failures.to_vec(),
            malformed,
        },
        log,
    )
}

/// Allocate the standard fixture candidate.
fn allocate(
    device: Device,
) -> Result<(Allocation<Device>, framebuffer::Handle, u64), ResourceError> {
    Allocation::allocate(device, &mode(), &[DrmFourcc::Xrgb8888 as u32])
}

#[test]
fn owns_exact_mode_and_releases_in_reverse_order_once() -> Result<(), ResourceError> {
    let (device, log) = fixture(&[], 0);
    let (mut owner, fb, blob) = allocate(device)?;
    assert_eq!(u32::from(fb), 21);
    assert_eq!(blob, 72);
    assert_eq!(log.borrow().mode, Some(mode()));
    assert_eq!(
        log.borrow().calls,
        ["buffer", "map", "unmap", "framebuffer", "blob"]
    );
    owner.release()?;
    owner.release()?;
    drop(owner);
    assert_eq!(
        log.borrow().calls,
        [
            "buffer",
            "map",
            "unmap",
            "framebuffer",
            "blob",
            "destroy blob",
            "destroy framebuffer",
            "destroy buffer"
        ]
    );
    Ok(())
}

#[test]
fn drop_retires_all_resources_without_explicit_release() -> Result<(), ResourceError> {
    let (device, log) = fixture(&[], 0);
    drop(allocate(device)?);
    assert_eq!(
        log.borrow().calls,
        [
            "buffer",
            "map",
            "unmap",
            "framebuffer",
            "blob",
            "destroy blob",
            "destroy framebuffer",
            "destroy buffer"
        ]
    );
    Ok(())
}

#[test]
fn each_allocation_failure_unwinds_only_acquired_resources()
-> Result<(), Box<dyn std::error::Error>> {
    for (stage, expected) in [
        ("buffer", vec!["buffer"]),
        (
            "framebuffer",
            vec!["buffer", "map", "unmap", "framebuffer", "destroy buffer"],
        ),
        (
            "blob",
            vec![
                "buffer",
                "map",
                "unmap",
                "framebuffer",
                "blob",
                "destroy framebuffer",
                "destroy buffer",
            ],
        ),
    ] {
        let (device, log) = fixture(&[stage], 0);
        let error = allocate(device).err().ok_or("allocation should refuse")?;
        assert_eq!(error.failure.source.raw_os_error(), Some(5));
        assert!(error.cleanup.is_empty());
        assert_eq!(log.borrow().calls, expected);
    }
    Ok(())
}

#[test]
fn invalid_format_or_empty_dimensions_never_allocate() -> Result<(), Box<dyn std::error::Error>> {
    let empty: Mode = drm_ffi::drm_mode_modeinfo::default().into();
    for (mode, formats) in [
        (mode(), vec![]),
        (mode(), vec![DrmFourcc::Argb8888 as u32]),
        (empty, vec![DrmFourcc::Xrgb8888 as u32]),
    ] {
        let (device, log) = fixture(&[], 0);
        let error = Allocation::allocate(device, &mode, &formats)
            .err()
            .ok_or("invalid candidate accepted")?;
        assert_eq!(error.failure.source.kind(), io::ErrorKind::InvalidData);
        assert!(log.borrow().calls.is_empty());
    }
    Ok(())
}

#[test]
fn malformed_buffer_layout_is_retired_before_registration() -> Result<(), Box<dyn std::error::Error>>
{
    for variant in 1..=4 {
        let (device, log) = fixture(&[], variant);
        let error = allocate(device).err().ok_or("malformed layout accepted")?;
        assert_eq!(error.failure.stage, "dumb buffer layout");
        assert_eq!(log.borrow().calls, ["buffer", "destroy buffer"]);
    }
    Ok(())
}

#[test]
fn cleanup_errors_preserve_original_failure_and_attempt_every_release()
-> Result<(), Box<dyn std::error::Error>> {
    let (device, log) = fixture(&["blob", "destroy framebuffer", "destroy buffer"], 0);
    let error = allocate(device)
        .err()
        .ok_or("blob allocation should refuse")?;
    assert_eq!(error.failure.stage, "create mode blob");
    assert_eq!(
        error.cleanup.iter().map(|e| e.stage).collect::<Vec<_>>(),
        ["destroy framebuffer", "destroy dumb buffer"]
    );
    assert_eq!(
        log.borrow().calls,
        [
            "buffer",
            "map",
            "unmap",
            "framebuffer",
            "blob",
            "destroy framebuffer",
            "destroy buffer"
        ]
    );
    for failure in [
        vec!["destroy blob"],
        vec!["destroy framebuffer"],
        vec!["destroy buffer"],
        vec!["destroy blob", "destroy framebuffer", "destroy buffer"],
    ] {
        let (device, log) = fixture(&failure, 0);
        let (mut owner, _, _) = allocate(device)?;
        let error = owner.release().err().ok_or("cleanup should refuse")?;
        assert_eq!(1 + error.cleanup.len(), failure.len());
        assert_eq!(error.failure.source.raw_os_error(), Some(5));
        drop(owner);
        assert_eq!(log.borrow().calls.len(), 8);
    }
    Ok(())
}

#[test]
fn invalid_blob_ids_never_become_cleanup_targets() -> Result<(), Box<dyn std::error::Error>> {
    for variant in [5, 6] {
        let (device, log) = fixture(&[], variant);
        let error = allocate(device).err().ok_or("invalid blob accepted")?;
        assert_eq!(error.failure.stage, "mode blob ID");
        assert_eq!(
            log.borrow().calls,
            [
                "buffer",
                "map",
                "unmap",
                "framebuffer",
                "blob",
                "destroy framebuffer",
                "destroy buffer"
            ]
        );
    }
    Ok(())
}

#[test]
fn real_non_drm_allocation_refuses_without_closing_callers_descriptor()
-> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::open("/dev/null")?;
    let error = Allocation::allocate(
        Inventory(file.as_fd()),
        &mode(),
        &[DrmFourcc::Xrgb8888 as u32],
    )
    .err()
    .ok_or("non-DRM allocation accepted")?;
    assert_eq!(error.failure.stage, "create dumb buffer");
    assert_eq!(error.failure.source.raw_os_error(), Some(25));
    assert!(error.cleanup.is_empty());
    assert!(file.metadata().is_ok());
    eprintln!("real CREATE_DUMB refused ENOTTY (25); borrowed descriptor survived");
    Ok(())
}

#[test]
fn atomic_validation_always_retires_resources_and_keeps_all_errors()
-> Result<(), Box<dyn std::error::Error>> {
    for refuse in [false, true] {
        for cleanup in [false, true] {
            let failures = if cleanup {
                vec!["destroy blob", "destroy framebuffer", "destroy buffer"]
            } else {
                vec![]
            };
            let (device, log) = fixture(&failures, 0);
            let (mut owner, _, _) = allocate(device)?;
            let result = owner.test_and_release(|device| {
                device.call("test")?;
                if refuse {
                    Err(io::Error::from_raw_os_error(22))
                } else {
                    Ok(())
                }
            });
            if refuse || cleanup {
                let error = result.err().ok_or("failure lost")?;
                assert_eq!(
                    error.failure.stage,
                    if refuse {
                        "atomic TEST_ONLY"
                    } else {
                        "destroy mode blob"
                    }
                );
                assert_eq!(
                    error.failure.source.raw_os_error(),
                    Some(if refuse { 22 } else { 5 })
                );
                assert_eq!(
                    error.cleanup.len(),
                    if cleanup {
                        if refuse { 3 } else { 2 }
                    } else {
                        0
                    }
                );
            } else {
                result?;
            }
            drop(owner);
            assert_eq!(
                log.borrow().calls,
                [
                    "buffer",
                    "map",
                    "unmap",
                    "framebuffer",
                    "blob",
                    "test",
                    "destroy blob",
                    "destroy framebuffer",
                    "destroy buffer"
                ]
            );
        }
    }
    Ok(())
}

#[test]
fn initialization_clears_pixels_stride_padding_and_allocation_tail() -> Result<(), ResourceError> {
    let (device, log) = fixture(&[], 0);
    let (mut owner, _, _) = allocate(device)?;
    let observed = log.borrow();
    assert_eq!(observed.pixels.len(), (1280 * 4 + 64) * 720 + 37);
    assert!(observed.pixels.iter().all(|byte| *byte == 0));
    assert_eq!(
        observed.calls,
        ["buffer", "map", "unmap", "framebuffer", "blob"]
    );
    drop(observed);
    owner.release()
}

#[test]
fn short_mapping_is_unmapped_without_writing_or_registering()
-> Result<(), Box<dyn std::error::Error>> {
    let (device, log) = fixture(&[], 7);
    let error = allocate(device).err().ok_or("short mapping accepted")?;
    assert_eq!(error.failure.stage, "initialize dumb buffer");
    assert_eq!(error.failure.source.kind(), io::ErrorKind::InvalidData);
    assert!(error.cleanup.is_empty());
    assert!(log.borrow().pixels.iter().all(|byte| *byte == 0xa5));
    assert_eq!(
        log.borrow().calls,
        ["buffer", "map", "unmap", "destroy buffer"]
    );
    Ok(())
}

#[test]
fn map_refusal_preserves_errno_and_buffer_cleanup_failure() -> Result<(), Box<dyn std::error::Error>>
{
    for cleanup_fails in [false, true] {
        let failures = if cleanup_fails {
            vec!["map", "destroy buffer"]
        } else {
            vec!["map"]
        };
        let (device, log) = fixture(&failures, 0);
        let error = allocate(device).err().ok_or("failed mapping accepted")?;
        assert_eq!(error.failure.stage, "initialize dumb buffer");
        assert_eq!(error.failure.source.raw_os_error(), Some(5));
        assert_eq!(error.cleanup.len(), usize::from(cleanup_fails));
        if cleanup_fails {
            let cleanup = error.cleanup.first().ok_or("cleanup error lost")?;
            assert_eq!(cleanup.stage, "destroy dumb buffer");
            assert_eq!(cleanup.source.raw_os_error(), Some(5));
        }
        assert_eq!(log.borrow().calls, ["buffer", "map", "destroy buffer"]);
        assert!(log.borrow().pixels.is_empty());
    }
    Ok(())
}

#[test]
fn invalid_initialization_layout_never_maps() -> Result<(), Box<dyn std::error::Error>> {
    for (size, pitch, format) in [
        ((0, 1), 4, DrmFourcc::Xrgb8888),
        ((1, 0), 4, DrmFourcc::Xrgb8888),
        ((2, 1), 4, DrmFourcc::Xrgb8888),
        ((1, 1), 5, DrmFourcc::Xrgb8888),
        ((u32::MAX, 1), u32::MAX - 3, DrmFourcc::Xrgb8888),
        ((1, 1), 4, DrmFourcc::Argb8888),
    ] {
        let (device, log) = fixture(&[], 0);
        let mut buffer = FakeBuffer {
            size,
            pitch,
            format,
        };
        let error = crate::scanout_buffer::initialize(&device, &mut buffer)
            .err()
            .ok_or("invalid layout accepted")?;
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(log.borrow().calls.is_empty());
    }
    Ok(())
}

#[test]
fn real_map_dumb_ioctl_refuses_without_closing_descriptor() -> Result<(), Box<dyn std::error::Error>>
{
    let file = std::fs::File::open("/dev/null")?;
    // Same MAP_DUMB ioctl used by pinned drm-rs before mmap. No fabricated
    // DumbBuffer or unsafe mapping is needed to exercise the kernel refusal.
    let error = drm_ffi::mode::dumbbuffer::map(file.as_fd(), 1, 0, 0)
        .err()
        .ok_or("non-DRM mapping accepted")?;
    assert_eq!(error.raw_os_error(), Some(25));
    assert!(file.metadata().is_ok());
    eprintln!("real MAP_DUMB refused ENOTTY (25); borrowed descriptor survived");
    Ok(())
}
