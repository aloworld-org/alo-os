//! Retain software render targets only while their immutable owners permit reuse.

use smithay::backend::renderer::{Texture, pixman::PixmanMapping};
use std::{collections::VecDeque, sync::Arc};

/// Cover active display/window readers without an unbounded ownership queue.
const MAX_IMAGES: usize = 6;
/// Count complete owned pixel allocations rather than visible rectangles.
const MAX_BYTES: usize = 64 * 1024 * 1024;

/// Recent immutable owners; a live reader prevents mutation of its image.
pub(super) struct Buffers {
    /// Oldest published image first.
    frames: VecDeque<Arc<PixmanMapping>>,
    /// Retained pixel bytes, including row padding.
    bytes: usize,
    /// Fixed production budget, reduced only in resource tests.
    limit: usize,
}

impl Default for Buffers {
    fn default() -> Self {
        Self {
            frames: VecDeque::new(),
            bytes: 0,
            limit: MAX_BYTES,
        }
    }
}

impl Buffers {
    /// Unwrap only released storage of the requested complete target size.
    pub(super) fn take(&mut self, size: (i32, i32)) -> Option<PixmanMapping> {
        let index = self.frames.iter().position(|frame| {
            frame.width() == size.0 as u32
                && frame.height() == size.1 as u32
                && Arc::strong_count(frame) == 1
        })?;
        let frame = self.frames.remove(index)?;
        self.bytes -= frame.as_ref().as_ref().len();
        match Arc::try_unwrap(frame) {
            Ok(frame) => Some(frame),
            Err(frame) => {
                self.put(frame);
                None
            }
        }
    }

    /// Evict old owners by count and bytes; a reader keeps its own allocation.
    pub(super) fn put(&mut self, frame: Arc<PixmanMapping>) {
        let length = frame.as_ref().as_ref().len();
        if length > self.limit {
            return;
        }
        while self.frames.len() >= MAX_IMAGES || self.bytes.saturating_add(length) > self.limit {
            let Some(oldest) = self.frames.pop_front() else {
                break;
            };
            self.bytes -= oldest.as_ref().as_ref().len();
        }
        self.frames.push_back(frame);
        self.bytes += length;
    }
}

#[cfg(test)]
#[path = "software_mapped_pool_tests.rs"]
mod tests;
