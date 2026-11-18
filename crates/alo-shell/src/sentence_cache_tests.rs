//! Cached native text keeps exact pixels, font ownership and retention limits.

use super::*;
use crate::painted_text::{a_wrapping_buffer, measured_and_inked, sentence};
use cosmic_text::{Shaping, fontdb};

/// Load the same bundled primary and fallback faces as native text.
fn fonts(locale: &str) -> FontSystem {
    let mut fonts = FontSystem::new_with_locale_and_db(locale.to_owned(), fontdb::Database::new());
    fonts
        .db_mut()
        .load_font_data(include_bytes!("../fonts/Inter.ttf").to_vec());
    fonts
        .db_mut()
        .load_font_data(include_bytes!("../fonts/Manrope-SemiBold.ttf").to_vec());
    fonts
}

/// A fixed native label request, varied explicitly in pixel comparisons.
fn key(text: &str) -> Key {
    Key {
        layout: Layout::Wrapped,
        text: text.to_owned(),
        width: 120,
        metrics: [16f32.to_bits(), 24f32.to_bits()],
        ground: [20; 3],
        ink: [240; 3],
    }
}

#[test]
fn centred_labels_keep_exact_pixels_and_cannot_reuse_a_wrapped_paragraph() {
    let mut fonts = fonts("en-US");
    for text in ["M", "Áyg", "Screen recording", "עברית", ""] {
        for width in [24, 120] {
            let metrics = Metrics::new(16.0, 24.0);
            let (ground, ink) = ([20, 30, 40], [240, 230, 220]);
            let expected = crate::painted_text::centred_uncached(
                &mut fonts, text, width, metrics, ground, ink,
            );
            let wrapped = sentence(&mut fonts, text, width, metrics, ground, ink);
            for _ in 0..2 {
                let actual =
                    crate::painted_text::centred(&mut fonts, text, width, metrics, ground, ink);
                assert_eq!(
                    (actual.width_of_box, actual.width, actual.height),
                    (expected.width_of_box, expected.width, expected.height)
                );
                assert_eq!(actual.pixels, expected.pixels);
                let still_wrapped = sentence(&mut fonts, text, width, metrics, ground, ink);
                assert_eq!(
                    (
                        still_wrapped.width_of_box,
                        still_wrapped.width,
                        still_wrapped.height
                    ),
                    (wrapped.width_of_box, wrapped.width, wrapped.height)
                );
                assert_eq!(still_wrapped.pixels, wrapped.pixels);
            }
        }
    }
}

#[test]
fn repeated_centred_labels_shape_once_within_the_shared_cache_budget() {
    let mut fonts = fonts("en-US");
    let mut rendered = 0;
    for _ in 0..3 {
        centred(
            &mut fonts,
            "M",
            24,
            Metrics::new(16.0, 24.0),
            [0; 3],
            [255; 3],
            |_| {
                rendered += 1;
                pixels(10)
            },
        );
    }
    assert_eq!(rendered, 1);
    SENTENCES.with_borrow(|cache| {
        assert!(cache.bytes <= MAX_BYTES);
        assert!(cache.entries.len() <= MAX_ENTRIES);
    });
}

/// Owned initialized storage for retention and independent-copy checks.
fn pixels(count: usize) -> Shaped {
    Shaped {
        width_of_box: count as i32,
        width: count as i32,
        height: 1,
        pixels: vec![[1, 2, 3]; count],
    }
}

#[test]
fn a_cached_sentence_matches_the_real_shaper_across_wrap_colours_and_scripts() {
    let mut fonts = fonts("en-US");
    for text in [
        "Screen and microphone",
        "A very long line that must wrap",
        "עברית English العربية",
    ] {
        for width in [80, 230] {
            for (ground, ink) in [([20, 30, 40], [240, 230, 220]), ([0; 3], [255; 3])] {
                for metrics in [Metrics::new(16.0, 24.0), Metrics::new(20.0, 32.0)] {
                    let mut buffer = a_wrapping_buffer(&mut fonts, width, metrics);
                    buffer.set_text(&mut fonts, text, &alos_own_text(), Shaping::Advanced);
                    let expected = measured_and_inked(&mut fonts, &mut buffer, width, ground, ink);
                    assert!(expected.pixels.iter().any(|pixel| *pixel != ground));
                    for _ in 0..2 {
                        let actual = sentence(&mut fonts, text, width, metrics, ground, ink);
                        assert_eq!(
                            (actual.width_of_box, actual.width, actual.height),
                            (expected.width_of_box, expected.width, expected.height)
                        );
                        assert_eq!(actual.pixels, expected.pixels);
                    }
                }
            }
        }
    }
}

#[test]
fn large_text_uses_the_normal_renderer_on_every_call() {
    let mut fonts = fonts("en-US");
    let text = "x".repeat(MAX_TEXT + 1);
    let mut called = 0;
    for _ in 0..2 {
        with(
            &mut fonts,
            &text,
            100,
            Metrics::new(16.0, 24.0),
            [0; 3],
            [255; 3],
            |_| {
                called += 1;
                pixels(10)
            },
        );
    }
    assert_eq!(called, 2);
}

#[test]
fn another_font_system_database_mutation_and_locale_cannot_reuse_old_pixels() {
    let mut first = fonts("en-US");
    let mut cache = Cache::default();
    cache.prepare(&mut first);
    cache.insert(key("Screen"), &pixels(10));
    assert!(cache.get(&key("Screen")).is_some());
    let _database = first.db_mut();
    cache.prepare(&mut first);
    assert!(cache.entries.is_empty());
    cache.insert(key("Screen"), &pixels(10));
    cache.prepare(&mut fonts("en-US"));
    assert!(cache.entries.is_empty());
    cache.insert(key("Screen"), &pixels(10));
    cache.prepare(&mut fonts("ar"));
    assert!(cache.entries.is_empty());
}

#[test]
fn returned_pixels_are_independent_and_recent_labels_survive_eviction()
-> Result<(), Box<dyn std::error::Error>> {
    let mut cache = Cache::default();
    for index in 0..MAX_ENTRIES {
        cache.insert(key(&index.to_string()), &pixels(10));
    }
    let mut returned = cache
        .get(&key("0"))
        .ok_or("the first label was not cached")?;
    returned.pixels.clear();
    assert_eq!(
        cache
            .get(&key("0"))
            .ok_or("cached pixels disappeared")?
            .pixels
            .len(),
        10
    );
    cache.insert(key("new"), &pixels(10));
    assert!(cache.get(&key("1")).is_none());
    assert!(cache.get(&key("0")).is_some());
    assert_eq!(cache.entries.len(), MAX_ENTRIES);
    Ok(())
}

#[test]
fn pixel_capacity_is_bounded_and_large_paragraphs_are_not_retained() {
    let mut cache = Cache::default();
    for index in 0..MAX_ENTRIES * 2 {
        cache.insert(key(&index.to_string()), &pixels(MAX_ITEM / 4));
        assert!(cache.bytes <= MAX_BYTES);
        assert!(cache.entries.len() <= MAX_ENTRIES);
    }
    let before = cache.bytes;
    cache.insert(key("oversized"), &pixels(MAX_ITEM));
    assert_eq!(cache.bytes, before);
    assert!(cache.get(&key("oversized")).is_none());
}
