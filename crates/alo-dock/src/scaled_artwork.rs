//! Retain bounded, immutable icon sizes instead of averaging them every frame.

use super::APicture;
use std::{cell::RefCell, collections::VecDeque, sync::Arc};

/// Bound both small-entry metadata and retained source/output pixel allocations.
const MAX_ENTRIES: usize = 16;
/// Count each source again per entry, conservatively, even when clones share it.
const MAX_BYTES: usize = 4 * 1024 * 1024;

thread_local! {
    /// A drawing thread owns its cache without a shared compositor lock.
    static PICTURES: RefCell<Pictures> = RefCell::new(Pictures::default());
}

/// Replacement bytes always have another allocation, even for the same app ID.
pub(super) fn picture(source: &APicture, across: u32, down: u32) -> APicture {
    PICTURES.with(|pictures| {
        pictures
            .borrow_mut()
            .draw(source, across, down, || source.scale_uncached(across, down))
    })
}

/// Keeping the source alive prevents an allocation address from being reused.
struct Entry {
    /// Source dimensions and exact immutable allocation identity.
    source: APicture,
    /// Exact requested size, with only an immutable pixel view exposed.
    scaled: APicture,
    /// Source plus output allocation lengths, counted within the fixed budget.
    bytes: usize,
}

/// Oldest used entry first; resource tests supply smaller budgets.
struct Pictures {
    /// Sources are public application icons, never application window pixels.
    entries: VecDeque<Entry>,
    /// Conservative sum of retained source/output lengths.
    bytes: usize,
    /// Retained-pixel budget.
    limit: usize,
    /// Entry-metadata budget.
    slots: usize,
}

impl Default for Pictures {
    fn default() -> Self {
        Self {
            entries: VecDeque::new(),
            bytes: 0,
            limit: MAX_BYTES,
            slots: MAX_ENTRIES,
        }
    }
}

impl Pictures {
    /// A miss preserves the existing scaler; an oversized result is not retained.
    fn draw(
        &mut self,
        source: &APicture,
        across: u32,
        down: u32,
        draw: impl FnOnce() -> APicture,
    ) -> APicture {
        if let Some(index) = self.entries.iter().position(|entry| {
            entry.source.across == source.across
                && entry.source.down == source.down
                && Arc::ptr_eq(&entry.source.pixels, &source.pixels)
                && (entry.scaled.across, entry.scaled.down) == (across, down)
        }) && let Some(entry) = self.entries.remove(index)
        {
            let result = entry.scaled.clone();
            self.entries.push_back(entry);
            return result;
        }
        let scaled = draw();
        let Some(bytes) = source.pixels.len().checked_add(scaled.pixels.len()) else {
            return scaled;
        };
        if bytes > self.limit || self.slots == 0 {
            return scaled;
        }
        while self.entries.len() >= self.slots || self.bytes > self.limit - bytes {
            let Some(oldest) = self.entries.pop_front() else {
                break;
            };
            self.bytes -= oldest.bytes;
        }
        self.entries.push_back(Entry {
            source: source.clone(),
            scaled: scaled.clone(),
            bytes,
        });
        self.bytes += bytes;
        scaled
    }
}

#[cfg(test)]
#[path = "scaled_artwork_tests.rs"]
mod tests;
