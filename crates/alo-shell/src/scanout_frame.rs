//! Validated CPU frames copied only into unbound scanout allocations.

use crate::display_resources::ResourceDevice;
use drm::buffer::{Buffer, DrmFourcc};
use std::io;

/// Borrowed, top-to-bottom DRM XRGB8888 pixels with an explicit byte stride.
///
/// Each pixel is four bytes in DRM's little-endian B, G, R, X order. No channel,
/// alpha, orientation or colour-space conversion is performed. Source padding is
/// ignored; destination padding and allocation tail are cleared. Renderers must
/// convert their output to this format before constructing the frame.
pub struct XrgbFrame<'a> {
    /// Full-mode pixel dimensions, never an implicit crop or scaling request.
    size: (u32, u32),
    /// Source row pitch in bytes.
    stride: usize,
    /// Validated visible bytes per row.
    row_bytes: usize,
    /// Complete rows, including source stride padding.
    pixels: &'a [u8],
}

impl<'a> XrgbFrame<'a> {
    /// Validate nonempty dimensions, aligned stride and exact `stride * height`
    /// byte length, with checked arithmetic. Refuses malformed input without I/O.
    pub fn new(size: (u32, u32), stride: usize, pixels: &'a [u8]) -> io::Result<Self> {
        let row_bytes = usize::try_from(size.0)
            .ok()
            .and_then(|width| width.checked_mul(4))
            .ok_or(io::ErrorKind::InvalidInput)?;
        let length = usize::try_from(size.1)
            .ok()
            .and_then(|height| stride.checked_mul(height))
            .ok_or(io::ErrorKind::InvalidInput)?;
        if size.0 == 0
            || size.1 == 0
            || stride < row_bytes
            || !stride.is_multiple_of(4)
            || pixels.len() != length
        {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        Ok(Self {
            size,
            stride,
            row_bytes,
            pixels,
        })
    }
}

/// Validate both layouts before mapping; never expose the mapped allocation.
pub(crate) fn upload<D: ResourceDevice>(
    device: &D,
    buffer: &mut D::Buffer,
    frame: &XrgbFrame<'_>,
) -> io::Result<()> {
    let pitch = usize::try_from(buffer.pitch()).map_err(|_| io::ErrorKind::InvalidData)?;
    let height = usize::try_from(frame.size.1).map_err(|_| io::ErrorKind::InvalidData)?;
    let required = pitch
        .checked_mul(height)
        .ok_or(io::ErrorKind::InvalidData)?;
    if buffer.size() != frame.size
        || buffer.format() != DrmFourcc::Xrgb8888
        || pitch < frame.row_bytes
        || !pitch.is_multiple_of(4)
    {
        return Err(io::ErrorKind::InvalidData.into());
    }
    device.with_mapping(buffer, |bytes| {
        if bytes.len() < required {
            return Err(io::ErrorKind::InvalidData.into());
        }
        bytes.fill(0);
        for (source, destination) in frame.pixels.chunks_exact(frame.stride).zip(
            bytes
                .get_mut(..required)
                .ok_or(io::ErrorKind::InvalidData)?
                .chunks_exact_mut(pitch),
        ) {
            let source = source
                .get(..frame.row_bytes)
                .ok_or(io::ErrorKind::InvalidData)?;
            let destination = destination
                .get_mut(..frame.row_bytes)
                .ok_or(io::ErrorKind::InvalidData)?;
            destination.copy_from_slice(source);
        }
        Ok(())
    })
}
