//! KMS allocation/destruction ioctls through the caller's borrowed descriptor.

use crate::{display_resources::ResourceDevice, drm_inventory::Inventory};
use drm::{
    buffer::DrmFourcc,
    control::{Device, Mode, dumbbuffer::DumbBuffer, framebuffer},
};
use std::io;

impl ResourceDevice for Inventory<'_> {
    type Buffer = DumbBuffer;
    fn create_buffer(&self, size: (u32, u32)) -> io::Result<DumbBuffer> {
        self.create_dumb_buffer(size, DrmFourcc::Xrgb8888, 32)
    }
    fn with_mapping(
        &self,
        buffer: &mut DumbBuffer,
        initialize: impl FnOnce(&mut [u8]) -> io::Result<()>,
    ) -> io::Result<()> {
        let mut mapping = self.map_dumb_buffer(buffer)?;
        initialize(mapping.as_mut())
        // drm-rs unmaps on drop, before registration or allocation unwind.
    }
    fn create_framebuffer(&self, buffer: &DumbBuffer) -> io::Result<framebuffer::Handle> {
        // ADDFB registers the fixed depth-24/bpp-32 XRGB format. It is not a
        // legacy modeset and does not provide a fallback from atomic KMS.
        self.add_framebuffer(buffer, 24, 32)
    }
    fn create_blob(&self, mode: &Mode) -> io::Result<u64> {
        // drm-rs Mode is repr(transparent) over the kernel's exact timing struct.
        self.create_property_blob(mode).map(Into::into)
    }
    fn destroy_blob(&self, blob: u64) -> io::Result<()> {
        self.destroy_property_blob(blob)
    }
    fn destroy_framebuffer(&self, framebuffer: framebuffer::Handle) -> io::Result<()> {
        Device::destroy_framebuffer(self, framebuffer)
    }
    fn destroy_buffer(&self, buffer: DumbBuffer) -> io::Result<()> {
        self.destroy_dumb_buffer(buffer)
    }
}
