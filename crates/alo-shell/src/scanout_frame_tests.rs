//! Pixel integrity, refusal and allocation-to-scanout integration checks.

use super::*;
use crate::{XrgbFrame, scanout::Scanout, scanout_frame::upload};

#[test]
fn converted_readback_uploads_with_padding_and_scanout_lifetime()
-> Result<(), Box<dyn std::error::Error>> {
    let (device, log) = fixture(&[], 0);
    let (mut owned, fb, blob) = allocate(device)?;
    // Bottom row first, distinct row/column markers and R/B channels.
    let rgba: Vec<u8> = (0..720)
        .flat_map(|row| (0..1280).flat_map(move |column| [row as u8, column as u8, 83, 128]))
        .collect();
    let converted = crate::readback::convert((1280, 720), crate::RowOrder::BottomToTop, &rgba)?;
    owned.write_frame(&converted.frame()?)?;
    for (row, bytes) in log
        .borrow()
        .pixels
        .get(..5184 * 720)
        .ok_or("missing rows")?
        .as_chunks::<5184>()
        .0
        .iter()
        .enumerate()
    {
        for (column, pixel) in bytes[..5120].as_chunks::<4>().0.iter().enumerate() {
            assert_eq!(*pixel, [83, column as u8, (719 - row) as u8, 0]);
        }
        assert!(bytes[5120..].iter().all(|byte| *byte == 0));
    }
    assert!(
        log.borrow()
            .pixels
            .get(5184 * 720..)
            .ok_or("missing tail")?
            .iter()
            .all(|byte| *byte == 0)
    );
    let mut active = Scanout::activate(owned, scanout_tests::plan()?, fb, blob)?;
    active.retire()?;
    let calls = &log.borrow().calls;
    assert_eq!(
        calls
            .get(calls.len().checked_sub(6).ok_or("missing calls")?..)
            .ok_or("missing calls")?,
        [
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
fn malformed_source_layouts_refuse_without_io() {
    for (size, stride, length) in [
        ((0, 1), 4, 4),
        ((1, 0), 4, 0),
        ((2, 1), 4, 4),
        ((1, 1), 5, 5),
        ((1, 2), 4, 7),
        ((1, 2), 4, 9),
        ((1, 2), usize::MAX - 3, 0),
        ((u32::MAX, u32::MAX), 4, 0),
    ] {
        let bytes = vec![0; length];
        let error = XrgbFrame::new(size, stride, &bytes).err();
        assert_eq!(
            error.map(|error| error.kind()),
            Some(io::ErrorKind::InvalidInput)
        );
    }
}

#[test]
fn independent_strides_preserve_pixels_and_clear_padding_and_tail() -> io::Result<()> {
    let bytes = [
        1, 2, 3, 0, 4, 5, 6, 0, 99, 99, 99, 99, 7, 8, 9, 0, 10, 11, 12, 0, 99, 99, 99, 99,
    ];
    let frame = XrgbFrame::new((2, 2), 12, &bytes)?;
    for pitch in [8, 16] {
        let (device, log) = fixture(&[], 0);
        let mut buffer = FakeBuffer {
            size: (2, 2),
            pitch,
            format: DrmFourcc::Xrgb8888,
        };
        upload(&device, &mut buffer, &frame)?;
        let mut expected = vec![0; pitch as usize * 2 + 37];
        expected
            .get_mut(..8)
            .ok_or(io::ErrorKind::Other)?
            .copy_from_slice(&[1, 2, 3, 0, 4, 5, 6, 0]);
        expected
            .get_mut(pitch as usize..pitch as usize + 8)
            .ok_or(io::ErrorKind::Other)?
            .copy_from_slice(&[7, 8, 9, 0, 10, 11, 12, 0]);
        assert_eq!(log.borrow().pixels, expected);
        assert_eq!(log.borrow().calls, ["map", "unmap"]);
    }
    Ok(())
}

#[test]
fn destination_layout_refuses_before_mapping() -> io::Result<()> {
    let frame = XrgbFrame::new((2, 2), 8, &[1; 16])?;
    for (size, pitch, format) in [
        ((1, 2), 8, DrmFourcc::Xrgb8888),
        ((2, 1), 8, DrmFourcc::Xrgb8888),
        ((2, 2), 4, DrmFourcc::Xrgb8888),
        ((2, 2), 9, DrmFourcc::Xrgb8888),
        ((2, 2), 8, DrmFourcc::Argb8888),
    ] {
        let (device, log) = fixture(&[], 0);
        let mut buffer = FakeBuffer {
            size,
            pitch,
            format,
        };
        assert_eq!(
            upload(&device, &mut buffer, &frame).err().map(|e| e.kind()),
            Some(io::ErrorKind::InvalidData)
        );
        assert!(log.borrow().calls.is_empty());
    }
    Ok(())
}

#[test]
fn short_mapping_refuses_without_partial_pixel_write() -> io::Result<()> {
    let (device, log) = fixture(&[], 7);
    let mut buffer = FakeBuffer {
        size: (2, 2),
        pitch: 12,
        format: DrmFourcc::Xrgb8888,
    };
    let frame = XrgbFrame::new((2, 2), 8, &[1; 16])?;
    assert_eq!(
        upload(&device, &mut buffer, &frame).err().map(|e| e.kind()),
        Some(io::ErrorKind::InvalidData)
    );
    assert_eq!(log.borrow().pixels, vec![0xa5; 23]);
    assert_eq!(log.borrow().calls, ["map", "unmap"]);
    Ok(())
}

#[test]
fn upload_failure_releases_candidate_once_preserving_errno_and_cleanup()
-> Result<(), Box<dyn std::error::Error>> {
    for failure in ["map", "unmap", "size"] {
        let (device, log) = fixture(&[], 0);
        let (mut owned, _, _) = allocate(device)?;
        owned.device.failures = vec![
            failure,
            "destroy blob",
            "destroy framebuffer",
            "destroy buffer",
        ];
        let size = if failure == "size" {
            (1, 1)
        } else {
            (1280, 720)
        };
        let pixels = vec![1; size.0 as usize * size.1 as usize * 4];
        let frame = XrgbFrame::new(size, size.0 as usize * 4, &pixels)?;
        let error = owned.write_frame(&frame).err().ok_or("upload accepted")?;
        assert_eq!(error.failure.stage, "upload scanout frame");
        if failure == "size" {
            assert_eq!(error.failure.source.kind(), io::ErrorKind::InvalidData);
        } else {
            assert_eq!(error.failure.source.raw_os_error(), Some(5));
        }
        assert_eq!(error.cleanup.len(), 3);
        assert!(
            error
                .cleanup
                .iter()
                .all(|failure| failure.source.raw_os_error() == Some(5))
        );
        drop(owned);
        for call in ["destroy blob", "destroy framebuffer", "destroy buffer"] {
            assert_eq!(
                log.borrow()
                    .calls
                    .iter()
                    .filter(|name| **name == call)
                    .count(),
                1
            );
        }
        assert!(!log.borrow().calls.contains(&"test"));
    }
    Ok(())
}

#[test]
fn uploaded_frame_survives_test_enable_and_ordered_disable()
-> Result<(), Box<dyn std::error::Error>> {
    let (device, log) = fixture(&[], 0);
    let (mut owned, fb, blob) = allocate(device)?;
    let pixels: Vec<u8> = (0..1280 * 720)
        .flat_map(|pixel| [pixel as u8, 27, 83, 0])
        .collect();
    let frame = XrgbFrame::new((1280, 720), 5120, &pixels)?;
    owned.write_frame(&frame)?;
    let before_scanout = log.borrow().pixels.clone();
    for (source, destination) in pixels
        .as_chunks::<5120>()
        .0
        .iter()
        .zip(before_scanout.as_chunks::<5184>().0)
    {
        assert_eq!(destination.get(..5120), Some(source.as_slice()));
        assert!(
            destination
                .get(5120..)
                .ok_or("missing padding")?
                .iter()
                .all(|byte| *byte == 0)
        );
    }
    let mut active = Scanout::activate(owned, scanout_tests::plan()?, fb, blob)?;
    assert_eq!(log.borrow().pixels, before_scanout);
    active.retire()?;
    drop(active);
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
