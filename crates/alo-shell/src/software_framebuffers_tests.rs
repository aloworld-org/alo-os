//! Owned scratch storage rotates without aliasing and respects both budgets.

use super::*;

/// A real CPU image; these checks need no display or device descriptor.
fn image(
    width: usize,
    height: usize,
) -> Result<Image<'static, 'static>, Box<dyn std::error::Error>> {
    let format = pixman::FormatCode::try_from(drm::buffer::DrmFourcc::Xrgb8888)
        .map_err(|_| "no XRGB test format")?;
    Image::new(format, width, height, false).map_err(|_| "no test image".into())
}

#[test]
fn display_and_window_images_are_owned_and_reused_by_exact_dimensions()
-> Result<(), Box<dyn std::error::Error>> {
    let mut buffers = Buffers::default();
    let desktop = image(32, 24)?;
    let identity = desktop.as_ptr();
    buffers.put(desktop);
    buffers.put(image(12, 10)?);
    assert!(buffers.take((24, 32)).is_none());
    let borrowed = buffers.take((32, 24)).ok_or("missing display image")?;
    assert_eq!(borrowed.as_ptr(), identity);
    assert!(buffers.take((32, 24)).is_none());
    assert!(buffers.take((12, 10)).is_some());
    buffers.put(borrowed);
    assert_eq!(buffers.images.len(), 1);
    Ok(())
}

#[test]
fn returning_a_recent_size_evicts_the_oldest_third_size() -> Result<(), Box<dyn std::error::Error>>
{
    let mut buffers = Buffers::default();
    buffers.put(image(8, 8)?);
    buffers.put(image(12, 12)?);
    let first = buffers.take((8, 8)).ok_or("missing first image")?;
    buffers.put(first);
    buffers.put(image(16, 16)?);
    assert_eq!(buffers.images.len(), MAX_IMAGES);
    assert!(buffers.take((12, 12)).is_none());
    assert!(buffers.take((8, 8)).is_some());
    assert!(buffers.take((16, 16)).is_some());
    assert_eq!(buffers.bytes, 0);
    Ok(())
}

#[test]
fn stride_bytes_are_bounded_and_oversized_images_do_not_displace_the_pool()
-> Result<(), Box<dyn std::error::Error>> {
    let mut buffers = Buffers {
        limit: 4096,
        ..Buffers::default()
    };
    buffers.put(image(16, 16)?);
    buffers.put(image(32, 32)?);
    assert_eq!(buffers.bytes, 4096);
    assert!(buffers.take((16, 16)).is_none());
    buffers.put(image(33, 33)?);
    assert_eq!(buffers.bytes, 4096);
    assert!(buffers.take((33, 33)).is_none());
    assert!(buffers.take((32, 32)).is_some());
    Ok(())
}
