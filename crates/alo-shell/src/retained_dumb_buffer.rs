//! One private CPU mapping per owned dumb buffer, retired before its GEM handle.

use drm::{
    buffer::{Buffer, DrmFourcc},
    control::{
        Device,
        dumbbuffer::{DumbBuffer, DumbMapping},
    },
};
use self_cell::{MutBorrow, self_cell};
use std::io;

/// The driver's borrowed mapping; its lifetime never escapes the owning cell.
type BorrowedPixels<'a> = DumbMapping<'a>;

self_cell!(
    /// Rent self-referential ownership instead of extending a mapping's lifetime.
    struct Mapped {
        owner: MutBorrow<DumbBuffer>,
        #[not_covariant]
        dependent: BorrowedPixels,
    }
);

/// The metadata is a copy of drm-rs's handle value, never a second GEM owner.
pub(crate) struct RetainedBuffer {
    /// Buffer metadata remains readable while the cell owns its mutable borrow.
    metadata: DumbBuffer,
    /// At most one successful mapping, even across detached scanout reuse.
    mapping: Option<Mapped>,
}

impl RetainedBuffer {
    /// Adopt the sole cleanup responsibility for an existing GEM allocation.
    pub(crate) fn new(buffer: DumbBuffer) -> Self {
        Self {
            metadata: buffer,
            mapping: None,
        }
    }

    /// Borrow pixels only for this upload; active scanout has no caller here.
    pub(crate) fn with_bytes<D: Device, T>(
        &mut self,
        device: &D,
        write: impl FnOnce(&mut [u8]) -> io::Result<T>,
    ) -> io::Result<T> {
        cached(
            &mut self.mapping,
            || {
                Mapped::try_new(MutBorrow::new(self.metadata), |owner| {
                    device.map_dumb_buffer(owner.borrow_mut())
                })
            },
            |mapping| mapping.with_dependent_mut(|_, pixels| write(pixels.as_mut())),
        )
    }

    /// Unmap before the caller destroys the sole GEM handle, including refusal.
    pub(crate) fn into_buffer(self) -> DumbBuffer {
        drop(self.mapping);
        self.metadata
    }
}

impl Buffer for RetainedBuffer {
    fn size(&self) -> (u32, u32) {
        self.metadata.size()
    }
    fn format(&self) -> DrmFourcc {
        self.metadata.format()
    }
    fn pitch(&self) -> u32 {
        self.metadata.pitch()
    }
    fn handle(&self) -> drm::buffer::Handle {
        self.metadata.handle()
    }
}

/// Cache only a successful mapping; an upload refusal leaves cleanup to its owner.
fn cached<M, T>(
    slot: &mut Option<M>,
    map: impl FnOnce() -> io::Result<M>,
    write: impl FnOnce(&mut M) -> io::Result<T>,
) -> io::Result<T> {
    if slot.is_none() {
        *slot = Some(map()?);
    }
    match slot.as_mut() {
        Some(mapping) => write(mapping),
        None => Err(io::Error::other("missing owned scanout mapping")),
    }
}

#[cfg(test)]
#[path = "retained_dumb_buffer_tests.rs"]
mod tests;
