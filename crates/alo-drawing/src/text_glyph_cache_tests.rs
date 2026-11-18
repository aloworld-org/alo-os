//! Exact output, database isolation and retained-storage checks for native text.

#![expect(
    clippy::unwrap_used,
    reason = "missing test fonts or glyphs fail the regression"
)]

use super::*;
use cosmic_text::{Attrs, Color, Family, Metrics, Shaping};

/// A private database with one deliberately selected bundled face.
fn fonts(bytes: &[u8]) -> FontSystem {
    let mut db = fontdb::Database::new();
    db.load_font_data(bytes.to_vec());
    FontSystem::new_with_locale_and_db("en-US".into(), db)
}

/// Shape the same visible word using the database's own face.
fn buffer(fonts: &mut FontSystem) -> Buffer {
    let family = fonts
        .db()
        .faces()
        .next()
        .unwrap()
        .families
        .first()
        .unwrap()
        .0
        .clone();
    let mut buffer = Buffer::new(fonts, Metrics::new(24.0, 30.0));
    buffer.set_size(fonts, Some(240.0), Some(60.0));
    buffer.set_text(
        fonts,
        "Recording",
        &Attrs::new().family(Family::Name(&family)),
        Shaping::Advanced,
    );
    buffer.shape_until_scroll(fonts, false);
    buffer
}

/// Preserve every emitted pixel and its position for an exact comparison.
fn drawn(
    fonts: &mut FontSystem,
    buffer: &Buffer,
    swash: &mut SwashCache,
) -> Vec<(i32, i32, u32, u32, u32)> {
    let mut pixels = Vec::new();
    buffer.draw(fonts, swash, Color::rgb(15, 30, 45), |x, y, w, h, color| {
        pixels.push((x, y, w, h, color.0))
    });
    pixels
}

/// Cache hits preserve pixels while keeping the original image allocation.
#[test]
fn repeated_draws_reuse_images_and_match_a_fresh_raster() {
    let mut fonts = fonts(include_bytes!("../fonts/Manrope-Regular.ttf"));
    let buffer = buffer(&mut fonts);
    let mut cache = Glyphs::new();
    assert!(cache.prepare(&mut fonts, &buffer));
    let first = drawn(&mut fonts, &buffer, &mut cache.swash);
    let key = *cache
        .swash
        .image_cache
        .iter()
        .find(|(_, image)| image.is_some())
        .unwrap()
        .0;
    let pointer = cache
        .swash
        .image_cache
        .get(&key)
        .unwrap()
        .as_ref()
        .unwrap()
        .data
        .as_ptr();
    assert!(!first.is_empty());
    assert!(cache.prepare(&mut fonts, &buffer));
    assert_eq!(first, drawn(&mut fonts, &buffer, &mut cache.swash));
    assert_eq!(
        pointer,
        cache
            .swash
            .image_cache
            .get(&key)
            .unwrap()
            .as_ref()
            .unwrap()
            .data
            .as_ptr()
    );
    assert_eq!(first, drawn(&mut fonts, &buffer, &mut SwashCache::new()));
}

/// Database-local IDs cannot authorize reuse across different loaded faces.
#[test]
fn identical_database_ids_cannot_reuse_a_different_faces_pixels() {
    let mut first = fonts(include_bytes!("../fonts/Manrope-Regular.ttf"));
    let mut second = fonts(include_bytes!("../fonts/GeistMono-Bold.ttf"));
    assert_eq!(
        first.db().faces().next().unwrap().id,
        second.db().faces().next().unwrap().id
    );
    let a = buffer(&mut first);
    let b = buffer(&mut second);
    let mut cache = Glyphs::new();
    assert!(cache.prepare(&mut first, &a));
    let old = drawn(&mut first, &a, &mut cache.swash);
    assert!(cache.prepare(&mut second, &b));
    assert!(cache.swash.image_cache.is_empty());
    let actual = drawn(&mut second, &b, &mut cache.swash);
    assert_ne!(old, actual);
    assert_eq!(actual, drawn(&mut second, &b, &mut SwashCache::new()));
}

/// A large image cannot remain retained after the buffer has been drawn.
#[test]
fn oversized_pixel_storage_is_released_after_the_draw() {
    let mut fonts = fonts(include_bytes!("../fonts/Manrope-Regular.ttf"));
    let buffer = buffer(&mut fonts);
    let mut cache = Glyphs::new();
    assert!(cache.prepare(&mut fonts, &buffer));
    drawn(&mut fonts, &buffer, &mut cache.swash);
    cache
        .swash
        .image_cache
        .values_mut()
        .flatten()
        .next()
        .unwrap()
        .data
        .resize(MAX_BYTES + 1, 0);
    cache.trim();
    assert!(cache.faces.is_empty());
    assert!(cache.swash.image_cache.is_empty());
}

/// Failed glyph lookups consume bounded entry storage even without pixels.
#[test]
fn missing_glyph_entries_are_bounded_too() {
    let mut fonts = fonts(include_bytes!("../fonts/Manrope-Regular.ttf"));
    let buffer = buffer(&mut fonts);
    let mut cache = Glyphs::new();
    assert!(cache.prepare(&mut fonts, &buffer));
    drawn(&mut fonts, &buffer, &mut cache.swash);
    let key = *cache.swash.image_cache.keys().next().unwrap();
    for glyph in 0..=MAX_IMAGES {
        let mut missing = key;
        missing.glyph_id = glyph as u16;
        cache.swash.image_cache.insert(missing, None);
    }
    cache.trim();
    assert!(cache.faces.is_empty());
    assert!(cache.swash.image_cache.is_empty());
}
