//! A raster made premultiplied: what a renderer blends.

use super::Washed;
use crate::area::Area;

/// A raster of these pixels, as many as the bytes hold.
fn of(pixels: Vec<u8>) -> Washed {
    Washed {
        area: Area::from_size((1, 1).into()),
        pixels,
    }
}

/// **Opaque pixels are unchanged**, so a window's line and a strip's ground
/// come out the colour they were made.
#[test]
fn an_opaque_pixel_is_unchanged() {
    assert_eq!(
        of(vec![0x07, 0x13, 0x1F, 255]).premultiplied().pixels,
        vec![0x07, 0x13, 0x1F, 255]
    );
}

/// **A transparent pixel is nothing**: black at no alpha, which a
/// premultiplied blend adds nothing for. Handed over straight, the shadow's
/// navy here was added to the canvas and drew it white.
#[test]
fn a_transparent_pixel_carries_no_colour() {
    assert_eq!(
        of(vec![0x07, 0x13, 0x1F, 0]).premultiplied().pixels,
        vec![0, 0, 0, 0]
    );
}

/// **Partly there, partly coloured**: each channel times its alpha, rounded
/// to nearest, the alpha itself kept.
#[test]
fn a_partial_pixel_is_its_colour_times_its_alpha() {
    assert_eq!(
        of(vec![255, 100, 1, 128]).premultiplied().pixels,
        vec![128, 50, 1, 128]
    );
}

/// **Bytes that are not a whole pixel are left alone** rather than read past:
/// a raster whose length is wrong is refused by whoever imports it, not
/// panicked over here.
#[test]
fn a_trailing_part_pixel_is_left_as_it_was() {
    assert_eq!(
        of(vec![255, 255, 255, 0, 9, 9]).premultiplied().pixels,
        vec![0, 0, 0, 0, 9, 9]
    );
}
