//! KMS allocation/destruction ioctls through the caller's borrowed descriptor.

use crate::retained_dumb_buffer::RetainedBuffer;
use crate::{display_resources::ResourceDevice, drm_inventory::Inventory};
use drm::{
    buffer::DrmFourcc,
    control::{Device, Mode, framebuffer},
};
use std::io;

impl ResourceDevice for Inventory<'_> {
    type Buffer = RetainedBuffer;
    fn create_buffer(&self, size: (u32, u32)) -> io::Result<RetainedBuffer> {
        self.create_dumb_buffer(size, DrmFourcc::Xrgb8888, 32)
            .map(RetainedBuffer::new)
    }
    fn with_mapping(
        &self,
        buffer: &mut RetainedBuffer,
        initialize: impl FnOnce(&mut [u8]) -> io::Result<()>,
    ) -> io::Result<()> {
        buffer.with_bytes(self, initialize)
    }
    fn create_framebuffer(&self, buffer: &RetainedBuffer) -> io::Result<framebuffer::Handle> {
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
    fn destroy_buffer(&self, buffer: RetainedBuffer) -> io::Result<()> {
        self.destroy_dumb_buffer(buffer.into_buffer())
    }
}
