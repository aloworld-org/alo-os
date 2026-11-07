//! Real initialized targets remain immutable while a published reader owns them.

use super::*;
use smithay::backend::renderer::pixman::PixmanRenderer;

/// Allocate a real initialized CPU target through the rented engine.
fn image(size: (i32, i32)) -> Result<PixmanMapping, Box<dyn std::error::Error>> {
    let mut extent = smithay::utils::Size::from((1, 1));
    extent.w = size.0;
    extent.h = size.1;
    Ok(PixmanRenderer::new()?.create_owned_mapping(drm::buffer::DrmFourcc::Xrgb8888, extent)?)
}

#[test]
fn live_readers_prevent_reuse_and_released_storage_keeps_its_identity()
-> Result<(), Box<dyn std::error::Error>> {
    let mut pool = Buffers::default();
    let mut frame = image((3, 2))?;
    assert!(frame.as_ref().iter().all(|byte| *byte == 0));
    frame.as_mut().fill(9);
    let frame = Arc::new(frame);
    let pointer = frame.as_ref().as_ref().as_ptr();
    let weak = Arc::downgrade(&frame);
    pool.put(frame.clone());
    assert!(pool.take((3, 2)).is_none());
    assert!(frame.as_ref().as_ref().iter().all(|byte| *byte == 9));
    drop(frame);
    let mut released = pool.take((3, 2)).ok_or("released image was not reusable")?;
    assert_eq!(released.as_ref().as_ptr(), pointer);
    released.as_mut().fill(4);
    assert!(weak.upgrade().is_none());
    assert_eq!(pool.bytes, 0);
    Ok(())
}

#[test]
fn retention_is_bounded_by_count_bytes_and_complete_dimensions()
-> Result<(), Box<dyn std::error::Error>> {
    let mut pool = Buffers {
        limit: 32,
        ..Buffers::default()
    };
    pool.put(Arc::new(image((2, 2))?));
    pool.put(Arc::new(image((4, 1))?));
    assert!(pool.take((1, 4)).is_none());
    let oversized = Arc::new(image((4, 4))?);
    pool.put(oversized);
    assert_eq!(pool.bytes, 32);
    pool.put(Arc::new(image((2, 1))?));
    assert!(pool.frames.len() <= MAX_IMAGES);
    assert!(pool.bytes <= pool.limit);
    assert!(pool.take((2, 2)).is_none());
    assert!(pool.take((4, 1)).is_some());
    assert!(pool.take((2, 1)).is_some());
    Ok(())
}

#[test]
fn malformed_mapping_sizes_are_refused_before_allocation() -> Result<(), Box<dyn std::error::Error>>
{
    for size in [(0, 1), (1, 0), (-1, 2), (i32::MAX, 2)] {
        assert!(image(size).is_err());
    }
    Ok(())
}

#[test]
fn small_images_cannot_accumulate_unbounded_owner_metadata()
-> Result<(), Box<dyn std::error::Error>> {
    let mut pool = Buffers::default();
    let first = Arc::new(image((1, 1))?);
    let weak = Arc::downgrade(&first);
    pool.put(first);
    for _ in 0..MAX_IMAGES {
        pool.put(Arc::new(image((1, 1))?));
    }
    assert!(weak.upgrade().is_none());
    assert_eq!(pool.frames.len(), MAX_IMAGES);
    assert_eq!(pool.bytes, MAX_IMAGES * 4);
    Ok(())
}
