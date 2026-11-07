//! Bounded reuse of completed pixel allocations after every reader releases them.

use std::{collections::VecDeque, sync::Arc};

const MAX_FRAMES: usize = 6;
const MAX_BYTES: usize = 64 * 1024 * 1024;

pub(super) struct Buffers {
    frames: VecDeque<Arc<Vec<u8>>>,
    bytes: usize,
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
    pub(super) fn take(&mut self, length: usize) -> Option<Vec<u8>> {
        let index = self
            .frames
            .iter()
            .position(|frame| frame.len() == length && Arc::strong_count(frame) == 1)?;
        let frame = self.frames.remove(index)?;
        self.bytes -= frame.capacity();
        match Arc::try_unwrap(frame) {
            Ok(pixels) => Some(pixels),
            // A weak reader can acquire ownership after the count was checked.
            // Never reuse that allocation while the reader can still observe it.
            Err(frame) => {
                self.put(frame);
                None
            }
        }
    }

    pub(super) fn put(&mut self, frame: Arc<Vec<u8>>) {
        let bytes = frame.capacity();
        if bytes > self.limit {
            return;
        }
        while self.frames.len() >= MAX_FRAMES || self.bytes.saturating_add(bytes) > self.limit {
            let Some(oldest) = self.frames.pop_front() else {
                break;
            };
            self.bytes -= oldest.capacity();
        }
        self.frames.push_back(frame);
        self.bytes += bytes;
    }
}

#[cfg(test)]
#[path = "software_readback_pool_tests.rs"]
mod tests;
