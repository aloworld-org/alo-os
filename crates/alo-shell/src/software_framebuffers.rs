//! Bounded owned scratch images, with one image checked out while painting.

use pixman::Image;
use std::collections::VecDeque;

/// Retain the display size and one selected-window size without unbounded growth.
const MAX_IMAGES: usize = 2;
/// Larger images still paint normally but are not retained after readback.
const MAX_BYTES: usize = 64 * 1024 * 1024;

/// One detached CPU image, with its actual stride allocation counted.
struct Retained {
    /// Pixman owns this storage; it is never a KMS framebuffer.
    image: Image<'static, 'static>,
    /// Stride times height, including row padding.
    bytes: usize,
}

/// Two retained images at most, in addition to one currently being painted.
pub(super) struct Buffers {
    /// Oldest returned image first.
    images: VecDeque<Retained>,
    /// Sum of retained pixel allocations.
    bytes: usize,
    /// Fixed production budget, reduced only by internal resource tests.
    limit: usize,
}

impl Default for Buffers {
    fn default() -> Self {
        Self {
            images: VecDeque::new(),
            bytes: 0,
            limit: MAX_BYTES,
        }
    }
}

impl Buffers {
    /// Transfer exclusive ownership out of the pool, never a shared mutable alias.
    pub(super) fn take(&mut self, size: (i32, i32)) -> Option<Image<'static, 'static>> {
        let index = self.images.iter().position(|entry| {
            (entry.image.width(), entry.image.height()) == (size.0 as usize, size.1 as usize)
        })?;
        let entry = self.images.remove(index)?;
        self.bytes -= entry.bytes;
        Some(entry.image)
    }

    /// Keep only affordable images after the renderer's target borrow has ended.
    pub(super) fn put(&mut self, image: Image<'static, 'static>) {
        let Some(bytes) = image.stride().checked_mul(image.height()) else {
            return;
        };
        if bytes > self.limit {
            return;
        }
        while self.images.len() >= MAX_IMAGES || self.bytes.saturating_add(bytes) > self.limit {
            let Some(old) = self.images.pop_front() else {
                break;
            };
            self.bytes -= old.bytes;
        }
        self.images.push_back(Retained { image, bytes });
        self.bytes += bytes;
    }
}

#[cfg(test)]
#[path = "software_framebuffers_tests.rs"]
mod tests;
