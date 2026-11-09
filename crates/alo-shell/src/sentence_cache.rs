//! Reuse native sentences within one font database, with bounded retention.

use super::{Shaped, alos_own_text};
use cosmic_text::{FontMatchKey, FontSystem, Metrics};
use std::{cell::RefCell, collections::VecDeque, sync::Arc};

/// Keep short native labels rather than arbitrarily large application text.
const MAX_TEXT: usize = 4096;
/// Bound entries independently of their pixel dimensions.
const MAX_ENTRIES: usize = 128;
/// Bound the pixel and key allocations retained by this cache.
const MAX_BYTES: usize = 4 * 1024 * 1024;
/// Large paragraphs render normally without displacing the label cache.
const MAX_ITEM: usize = 256 * 1024;

thread_local! {
    /// Text remains on its rendering thread and never crosses font domains.
    static SENTENCES: RefCell<Cache> = RefCell::new(Cache::default());
}

/// Every value that changes the pixels or dimensions of a native sentence.
#[derive(PartialEq, Eq)]
struct Key {
    /// Wrapped paragraphs and trimmed icon letters have different geometry.
    layout: Layout,
    /// Full text includes paragraph direction and line breaks.
    text: String,
    /// Wrapping width in physical pixels.
    width: i32,
    /// Font size and line height, compared exactly.
    metrics: [u32; 2],
    /// Opaque background used while rasterizing glyphs.
    ground: [u8; 3],
    /// Glyph colour.
    ink: [u8; 3],
}

/// Both paths use the same font-domain and memory bounds.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Layout {
    /// Shape paragraphs within the requested wrapping width.
    Wrapped,
    /// Trim and centre a native label within the requested width.
    Centred,
}

/// Owned output, never a mutable alias of a returned sentence.
struct Entry {
    /// Exact request that produced this output.
    key: Key,
    /// Fully shaped and rasterized pixels and measurements.
    shaped: Shaped,
    /// Allocations counted against the retention budget.
    bytes: usize,
}

/// A font-match allocation is replaced when cosmic-text's database is mutated.
#[derive(Default)]
struct Cache {
    /// Retaining the actual owner prevents address reuse from impersonating it.
    domain: Option<Arc<Vec<FontMatchKey>>>,
    /// Locale controls fallback selection even for the same database.
    locale: String,
    /// Least recently used first.
    entries: VecDeque<Entry>,
    /// Total retained key and pixel capacity.
    bytes: usize,
}

impl Cache {
    /// Discard layouts when a different font system or database is supplied.
    fn prepare(&mut self, fonts: &mut FontSystem) {
        let domain = fonts.get_font_matches(&alos_own_text());
        if self
            .domain
            .as_ref()
            .is_none_or(|old| !Arc::ptr_eq(old, &domain))
            || self.locale != fonts.locale()
        {
            self.entries.clear();
            self.bytes = 0;
            self.locale = fonts.locale().to_owned();
            self.domain = Some(domain);
        }
    }

    /// Promote a hit without sharing mutable pixel storage with the caller.
    fn get(&mut self, key: &Key) -> Option<Shaped> {
        let index = self.entries.iter().position(|entry| entry.key == *key)?;
        let entry = self.entries.remove(index)?;
        let result = entry.shaped.clone();
        self.entries.push_back(entry);
        Some(result)
    }

    /// Retain only affordable outputs, evicting before inserting a new entry.
    fn insert(&mut self, key: Key, shaped: &Shaped) {
        let minimum = key.text.capacity().saturating_add(
            shaped
                .pixels
                .len()
                .saturating_mul(std::mem::size_of::<[u8; 3]>()),
        );
        if minimum > MAX_ITEM {
            return;
        }
        let shaped = shaped.clone();
        let bytes = key.text.capacity().saturating_add(
            shaped
                .pixels
                .capacity()
                .saturating_mul(std::mem::size_of::<[u8; 3]>()),
        );
        if bytes > MAX_ITEM {
            return;
        }
        while self.entries.len() >= MAX_ENTRIES || self.bytes.saturating_add(bytes) > MAX_BYTES {
            let Some(old) = self.entries.pop_front() else {
                break;
            };
            self.bytes -= old.bytes;
        }
        self.entries.push_back(Entry { key, shaped, bytes });
        self.bytes += bytes;
    }
}

/// Share the existing shaper on misses; rich text and terminal runs stay separate.
pub(super) fn with(
    fonts: &mut FontSystem,
    text: &str,
    width: i32,
    metrics: Metrics,
    ground: [u8; 3],
    ink: [u8; 3],
    render: impl FnOnce(&mut FontSystem) -> Shaped,
) -> Shaped {
    cached(
        fonts,
        request(text, width, metrics, ground, ink, Layout::Wrapped),
        render,
    )
}

/// Reuse the same exact-pixel cache for trimmed, unwrapped native icon text.
pub(super) fn centred(
    fonts: &mut FontSystem,
    text: &str,
    width: i32,
    metrics: Metrics,
    ground: [u8; 3],
    ink: [u8; 3],
    render: impl FnOnce(&mut FontSystem) -> Shaped,
) -> Shaped {
    cached(
        fonts,
        request(text, width, metrics, ground, ink, Layout::Centred),
        render,
    )
}

/// Refuse large keys before allocating their text.
fn request(
    text: &str,
    width: i32,
    metrics: Metrics,
    ground: [u8; 3],
    ink: [u8; 3],
    layout: Layout,
) -> Option<Key> {
    (text.len() <= MAX_TEXT).then(|| Key {
        layout,
        text: text.to_owned(),
        width,
        metrics: [metrics.font_size.to_bits(), metrics.line_height.to_bits()],
        ground,
        ink,
    })
}

/// Both layout kinds share one bounded cache and font invalidation policy.
fn cached(
    fonts: &mut FontSystem,
    key: Option<Key>,
    render: impl FnOnce(&mut FontSystem) -> Shaped,
) -> Shaped {
    let Some(key) = key else { return render(fonts) };
    SENTENCES.with_borrow_mut(|cache| {
        cache.prepare(fonts);
        if let Some(shaped) = cache.get(&key) {
            return shaped;
        }
        let shaped = render(fonts);
        cache.insert(key, &shaped);
        shaped
    })
}

#[cfg(test)]
#[path = "sentence_cache_tests.rs"]
mod tests;
