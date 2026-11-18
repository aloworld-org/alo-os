//! Reuse native glyph images without carrying them between different faces.

use cosmic_text::{Buffer, Font, FontSystem, SwashCache, fontdb};
use std::{cell::RefCell, collections::HashMap, sync::Arc};

/// Bound retained images independently of the text a client puts in a title.
const MAX_IMAGES: usize = 4096;
/// Bound retained pixel allocations; oversized single draws remain uncached.
const MAX_BYTES: usize = 16 * 1024 * 1024;
/// Bound face references as well as image storage.
const MAX_FACES: usize = 32;

thread_local! {
    /// A renderer thread owns its glyphs; no application text is retained.
    static GLYPHS: RefCell<Glyphs> = RefCell::new(Glyphs::new());
}

/// Font IDs are database-local, so retain and compare the actual face owners.
struct Glyphs {
    /// Both present and missing faces participate in invalidation.
    faces: HashMap<fontdb::ID, Option<Arc<Font>>>,
    /// The scaler's images include size, subpixel position and rendering flags.
    swash: SwashCache,
}

impl Glyphs {
    /// Start with no rasterized glyphs or face references.
    fn new() -> Self {
        Self {
            faces: HashMap::new(),
            swash: SwashCache::new(),
        }
    }

    /// Release allocations, including the scaler's own bounded working cache.
    fn clear(&mut self) {
        *self = Self::new();
    }

    /// Validate every face before a cached image can be read.
    fn prepare(&mut self, fonts: &mut FontSystem, buffer: &Buffer) -> bool {
        let mut current = HashMap::new();
        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                current
                    .entry(glyph.font_id)
                    .or_insert_with(|| fonts.get_font(glyph.font_id));
                if current.len() > MAX_FACES {
                    self.clear();
                    return false;
                }
            }
        }
        let changed = current.iter().any(|(id, face)| {
            self.faces.get(id).is_some_and(|old| match (old, face) {
                (Some(old), Some(face)) => !Arc::ptr_eq(old, face),
                (None, None) => false,
                _ => true,
            })
        });
        let additional = current
            .keys()
            .filter(|id| !self.faces.contains_key(*id))
            .count();
        if changed || self.faces.len() + additional > MAX_FACES {
            self.clear();
        }
        self.faces.extend(current);
        true
    }

    /// Oversized draws cannot leave an oversized cache behind.
    fn trim(&mut self) {
        let bytes: usize = self
            .swash
            .image_cache
            .values()
            .flatten()
            .map(|image| image.data.capacity())
            .sum();
        if self.swash.image_cache.len() > MAX_IMAGES || bytes > MAX_BYTES {
            self.clear();
        }
    }
}

/// Draw with reusable glyphs after checking the shaped buffer's face identities.
pub(crate) fn with<R>(
    fonts: &mut FontSystem,
    buffer: &Buffer,
    draw: impl FnOnce(&mut FontSystem, &mut SwashCache) -> R,
) -> R {
    GLYPHS.with_borrow_mut(|cache| {
        if !cache.prepare(fonts, buffer) {
            return draw(fonts, &mut SwashCache::new());
        }
        let result = draw(fonts, &mut cache.swash);
        cache.trim();
        result
    })
}

#[cfg(test)]
#[path = "text_glyph_cache_tests.rs"]
mod tests;
