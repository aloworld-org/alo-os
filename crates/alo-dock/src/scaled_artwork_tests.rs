//! Cache identity, actual alpha-aware pixels and both retention limits.

use super::*;
use std::cell::Cell;

/// A valid icon with known dimensions, channels and opacity.
fn source(width: u32, colour: [u8; 4]) -> Result<APicture, Box<dyn std::error::Error>> {
    APicture::of(width, width, colour.repeat(width as usize * width as usize))
        .map_err(|_| "invalid test artwork".into())
}

#[test]
fn clones_reuse_exact_scaled_pixels_but_size_and_replacement_invalidate()
-> Result<(), Box<dyn std::error::Error>> {
    let mut pictures = Pictures::default();
    let original = source(4, [203, 71, 19, 128])?;
    let calls = Cell::new(0);
    let mut draw = |image: &APicture, width| {
        pictures.draw(image, width, width, || {
            calls.set(calls.get() + 1);
            image.scale_uncached(width, width)
        })
    };
    let first = draw(&original, 2);
    let again = draw(&original.clone(), 2);
    assert_eq!(calls.get(), 1);
    assert_eq!(first, original.scale_uncached(2, 2));
    assert!(Arc::ptr_eq(&first.pixels, &again.pixels));
    assert_eq!(draw(&original, 3), original.scale_uncached(3, 3));
    let replacement = source(4, [17, 41, 223, 255])?;
    assert_eq!(draw(&replacement, 2), replacement.scale_uncached(2, 2));
    assert_eq!(calls.get(), 3);
    assert_ne!(first.pixels(), replacement.scale_uncached(2, 2).pixels());
    Ok(())
}

#[test]
fn byte_budget_evicts_oldest_and_oversized_results_do_not_displace_entries()
-> Result<(), Box<dyn std::error::Error>> {
    let mut pictures = Pictures {
        limit: 80,
        slots: 2,
        ..Pictures::default()
    };
    let first = source(4, [1, 2, 3, 255])?;
    pictures.draw(&first, 2, 2, || first.scale_uncached(2, 2));
    assert_eq!(pictures.bytes, 80);
    let smaller = source(2, [4, 5, 6, 255])?;
    pictures.draw(&smaller, 1, 1, || smaller.scale_uncached(1, 1));
    assert_eq!(pictures.entries.len(), 1);
    assert_eq!(pictures.bytes, 20);
    let large = source(5, [7, 8, 9, 255])?;
    let result = pictures.draw(&large, 2, 2, || large.scale_uncached(2, 2));
    assert_eq!(result, large.scale_uncached(2, 2));
    assert_eq!(pictures.entries.len(), 1);
    assert_eq!(pictures.bytes, 20);
    Ok(())
}

#[test]
fn entry_budget_is_bounded_and_recently_used_artwork_survives()
-> Result<(), Box<dyn std::error::Error>> {
    let mut pictures = Pictures {
        slots: 2,
        ..Pictures::default()
    };
    let first = source(2, [1, 1, 1, 255])?;
    let second = source(2, [2, 2, 2, 255])?;
    let third = source(2, [3, 3, 3, 255])?;
    for image in [&first, &second, &first, &third] {
        pictures.draw(image, 1, 1, || image.scale_uncached(1, 1));
    }
    assert_eq!(pictures.entries.len(), 2);
    assert_eq!(pictures.bytes, 40);
    assert!(
        pictures
            .entries
            .iter()
            .any(|e| Arc::ptr_eq(&e.source.pixels, &first.pixels))
    );
    assert!(
        !pictures
            .entries
            .iter()
            .any(|e| Arc::ptr_eq(&e.source.pixels, &second.pixels))
    );
    Ok(())
}
