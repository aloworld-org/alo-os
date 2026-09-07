//! Initialize unbound scanout memory before any framebuffer can expose it.

use crate::display_resources::ResourceDevice;
use drm::buffer::{Buffer, DrmFourcc};
use std::io;

/// Clear visible pixels, row padding and the allocation tail, then unmap.
pub(crate) fn initialize<D: ResourceDevice>(device: &D, buffer: &mut D::Buffer) -> io::Result<()> {
    let (width, height) = buffer.size();
    let pitch = buffer.pitch();
    if width == 0
        || height == 0
        || buffer.format() != DrmFourcc::Xrgb8888
        || width.checked_mul(4).is_none_or(|row| row > pitch)
        || !pitch.is_multiple_of(4)
    {
        return Err(io::ErrorKind::InvalidData.into());
    }
    let required = u64::from(pitch)
        .checked_mul(u64::from(height))
        .and_then(|length| usize::try_from(length).ok())
        .ok_or(io::ErrorKind::InvalidData)?;
    device.with_mapping(buffer, |bytes| {
        if bytes.len() < required {
            return Err(io::ErrorKind::InvalidData.into());
        }
        // XRGB8888 zero is black; this is memory initialization, not the shell's
        // visual palette. Clear padding too, never assuming the driver did so.
        bytes.fill(0);
        Ok(())
    })
}
