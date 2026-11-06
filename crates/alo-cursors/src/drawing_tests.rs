//! What a drawn cursor has to be: inside its square, in the palette's colours,
//! premultiplied, and refused when it cannot be drawn.

use super::*;
use crate::artwork::{Drawing, EVERY_SHAPE, artwork};
use crate::measures::{LOGICAL_SIZE, pixels_at};

/// The sizes a pointer is drawn at on real displays: 100, 125, 150, 200 and 300
/// per cent of the logical size.
const SIZES: [u32; 5] = [24, 30, 36, 48, 72];

/// A drawing that must succeed.
fn drawn(icon: CursorIcon, silhouette: Silhouette, size: u32) -> Picture {
    draw(icon, silhouette, size)
        .unwrap_or_else(|why| unreachable!("{} at {size}: {why}", icon.name()))
}

/// The pixel at `(x, y)`.
fn at(picture: &Picture, x: u32, y: u32) -> [u8; 4] {
    let index = usize::try_from((y * picture.size + x) * 4).unwrap_or(usize::MAX);
    picture
        .pixels
        .get(index..index + 4)
        .and_then(|four| <[u8; 4]>::try_from(four).ok())
        .unwrap_or_default()
}

/// **Every shape draws, in both arrows, at every scale a display has**, into
/// exactly the square asked for, with its hotspot inside it.
#[test]
fn every_shape_draws_at_every_scale() {
    for icon in EVERY_SHAPE {
        for silhouette in Silhouette::ALL {
            for size in SIZES {
                let picture = drawn(icon, silhouette, size);
                assert_eq!(picture.size, size);
                assert_eq!(
                    picture.pixels.len(),
                    usize::try_from(size * size * 4).unwrap_or(0)
                );
                assert!(picture.hotspot.0 < size && picture.hotspot.1 < size);
                assert!(
                    picture
                        .pixels
                        .chunks(4)
                        .any(|pixel| pixel == [0x10, 0x2A, 0x43, 255]),
                    "{} at {size} has no solid navy in it",
                    icon.name()
                );
            }
        }
    }
}

/// **Nothing is cut off at the square's edge**: the outermost ring of pixels
/// carries none of a shape or its keyline, so the artwork is inside its box.
#[test]
fn every_shape_is_inside_its_square() {
    let size = 64;
    for icon in EVERY_SHAPE {
        for silhouette in Silhouette::ALL {
            let art = artwork(crate::drawing_for(icon), silhouette);
            let covered = coverage(&art, size, Layer::Keyline)
                .unwrap_or_else(|why| unreachable!("{}: {why}", icon.name()));
            for i in 0..size {
                for (x, y) in [(i, 0), (i, size - 1), (0, i), (size - 1, i)] {
                    let index = usize::try_from(y * size + x).unwrap_or(usize::MAX);
                    assert_eq!(
                        covered.get(index).copied(),
                        Some(0),
                        "{} reaches the edge of its square at ({x}, {y})",
                        icon.name()
                    );
                }
            }
        }
    }
}

/// **Premultiplied**, because both renderers and an XCursor file blend it as
/// such: no channel is ever more than the pixel's own opacity.
#[test]
fn every_pixel_is_premultiplied() {
    for icon in EVERY_SHAPE {
        let picture = drawn(icon, Silhouette::Kite, 48);
        for pixel in picture.pixels.chunks(4) {
            if let [red, green, blue, alpha] = pixel {
                assert!(red <= alpha && green <= alpha && blue <= alpha, "{pixel:?}");
            }
        }
    }
}

/// **Navy inside, white round it, a shadow under it**: the fill is
/// `text/primary`, the keyline `bg/surface`, and the lowest pixel with anything
/// in it is below the lowest white one.
#[test]
fn the_fill_is_navy_the_keyline_white_and_the_shadow_below() {
    for silhouette in Silhouette::ALL {
        let picture = drawn(CursorIcon::Default, silhouette, 48);
        // (13, 13) units is inside both arrows; 1.5 pixels to a unit here.
        assert_eq!(
            at(&picture, 19, 19),
            [0x10, 0x2A, 0x43, 255],
            "{silhouette:?}"
        );
        assert!(
            picture
                .pixels
                .chunks(4)
                .any(|pixel| pixel == [255, 255, 255, 255]),
            "{silhouette:?} has no white keyline"
        );
        let lowest = |test: &dyn Fn(&[u8]) -> bool| {
            picture
                .pixels
                .chunks(4)
                .enumerate()
                .filter(|(_, pixel)| test(pixel))
                .map(|(index, _)| index / 48)
                .max()
                .unwrap_or(0)
        };
        let lowest_white = lowest(&|pixel| pixel == [255, 255, 255, 255]);
        let lowest_anything = lowest(&|pixel| pixel.get(3).is_some_and(|&alpha| alpha > 0));
        assert!(
            lowest_anything > lowest_white,
            "{silhouette:?}: nothing falls below the shape, so there is no shadow"
        );
    }
}

/// **Deep teal is the agent's, never the person's pointer** (ADR 0067). No
/// solid pixel of any shape is anywhere near `accent/default`.
#[test]
fn the_persons_pointer_is_never_the_agents_colour() {
    let teal = Role::AccentDefault.colour();
    let teal = [teal.red(), teal.green(), teal.blue()].map(i32::from);
    for icon in EVERY_SHAPE {
        for silhouette in Silhouette::ALL {
            let picture = drawn(icon, silhouette, 48);
            for pixel in picture.pixels.chunks(4) {
                if let [red, green, blue, 255] = pixel {
                    let distance: i32 = [*red, *green, *blue]
                        .map(i32::from)
                        .iter()
                        .zip(teal)
                        .map(|(one, other)| (one - other).abs())
                        .sum();
                    assert!(distance > 60, "{} has a teal pixel {pixel:?}", icon.name());
                }
            }
        }
    }
}

/// **The hotspot lands on the tip at every scale**: the kite's tip is at (6, 5)
/// units, which is pixel (4, 3) at 24 and (9, 7) at 48; the resize arrows click
/// in the middle.
#[test]
fn the_hotspot_follows_the_scale() {
    assert_eq!(
        drawn(CursorIcon::Default, Silhouette::Kite, 24).hotspot,
        (4, 3)
    );
    assert_eq!(
        drawn(CursorIcon::Default, Silhouette::Kite, 48).hotspot,
        (9, 7)
    );
    assert_eq!(
        drawn(CursorIcon::Default, Silhouette::Classic, 48).hotspot,
        (9, 6)
    );
    assert_eq!(
        drawn(CursorIcon::EwResize, Silhouette::Kite, 48).hotspot,
        (24, 24)
    );
}

/// **A double arrow is the same turned half a turn**, so it never says one end
/// moves more than the other.
#[test]
fn every_double_arrow_points_both_ways() {
    let size = 32;
    for drawing in [
        Drawing::Across,
        Drawing::Down,
        Drawing::Falling,
        Drawing::Rising,
        Drawing::Move,
    ] {
        let covered = coverage(&artwork(drawing, Silhouette::Kite), size, Layer::Fill)
            .unwrap_or_else(|why| unreachable!("{drawing:?}: {why}"));
        let turned: Vec<u8> = covered.iter().rev().copied().collect();
        for (index, (one, other)) in covered.iter().zip(&turned).enumerate() {
            assert!(
                one.abs_diff(*other) <= 16,
                "{drawing:?} is heavier at one end, at pixel {index}: {one} against {other}"
            );
        }
    }
}

/// **The logical size passes through the display's scale**: 24 at 100 per cent,
/// twice that at 200, and a display with no answer drawn at one-to-one.
#[test]
fn the_size_follows_the_display() {
    assert_eq!(pixels_at(100), LOGICAL_SIZE);
    assert_eq!(pixels_at(125), 30);
    assert_eq!(pixels_at(175), 42);
    assert_eq!(pixels_at(200), 48);
    assert_eq!(pixels_at(300), 72);
    assert_eq!(pixels_at(0), LOGICAL_SIZE);
}

/// **A square that cannot be drawn is refused by name**, not drawn empty: no
/// pixels, and one larger than the compositor will allocate mid-frame.
#[test]
fn a_size_that_cannot_be_drawn_is_refused() {
    assert_eq!(
        draw(CursorIcon::Default, Silhouette::Kite, 0),
        Err(NotDrawn::NoPixels)
    );
    assert_eq!(
        draw(CursorIcon::Default, Silhouette::Kite, LARGEST + 1),
        Err(NotDrawn::TooLarge(LARGEST + 1))
    );
    assert_eq!(
        draw(CursorIcon::Default, Silhouette::Kite, u32::MAX),
        Err(NotDrawn::TooLarge(u32::MAX))
    );
    assert!(draw(CursorIcon::Default, Silhouette::Kite, 1).is_ok());
}
