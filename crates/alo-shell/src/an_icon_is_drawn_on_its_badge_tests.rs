//! Pixels read back off a surface after an icon was drawn onto it.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_dock::artwork::APicture;
use smithay::utils::{Physical, Rectangle};

use super::pictures_onto;
use crate::painted::Washed;

/// A surface of one flat colour, at the origin unless told otherwise.
fn a_surface(wide: i32, tall: i32, colour: [u8; 4]) -> Washed {
    let mut pixels = Vec::new();
    for _ in 0..wide * tall {
        pixels.extend_from_slice(&colour);
    }
    Washed {
        area: Rectangle::new((0, 0).into(), (wide, tall).into()),
        pixels,
    }
}

/// A picture of one flat colour.
fn a_picture(across: u32, down: u32, colour: [u8; 4]) -> APicture {
    let mut pixels = Vec::new();
    for _ in 0..across * down {
        pixels.extend_from_slice(&colour);
    }
    APicture::of(across, down, pixels).unwrap()
}

/// The pixel at this position, as four bytes.
fn pixel_at(washed: &Washed, x: i32, y: i32) -> [u8; 4] {
    let at = (y as usize * washed.area.size.w as usize + x as usize) * 4;
    washed.pixels.get(at..at + 4).unwrap().try_into().unwrap()
}

fn a_box(x: i32, y: i32, wide: i32, tall: i32) -> Rectangle<i32, Physical> {
    Rectangle::new((x, y).into(), (wide, tall).into())
}

#[test]
fn an_opaque_icon_covers_the_badge_it_is_drawn_on() {
    let mut surface = a_surface(8, 8, [10, 10, 10, 255]);
    let icon = a_picture(4, 4, [200, 50, 25, 255]);
    pictures_onto(&mut surface, &[(a_box(2, 2, 4, 4), &icon)]);

    assert_eq!(
        pixel_at(&surface, 3, 3),
        [200, 50, 25, 255],
        "inside the box"
    );
    // The controls at both ends: outside the box on each side is untouched, so
    // the assertion above is the icon landing and not the whole surface being
    // overwritten.
    assert_eq!(
        pixel_at(&surface, 0, 0),
        [10, 10, 10, 255],
        "above and left"
    );
    assert_eq!(
        pixel_at(&surface, 7, 7),
        [10, 10, 10, 255],
        "below and right"
    );
}

#[test]
fn it_lands_exactly_where_it_is_put() {
    let mut surface = a_surface(8, 8, [0, 0, 0, 255]);
    let icon = a_picture(2, 2, [255, 255, 255, 255]);
    pictures_onto(&mut surface, &[(a_box(1, 5, 2, 2), &icon)]);

    assert_eq!(pixel_at(&surface, 1, 5), [255, 255, 255, 255]);
    assert_eq!(pixel_at(&surface, 2, 6), [255, 255, 255, 255]);
    // One pixel outside each edge of the box.
    assert_eq!(pixel_at(&surface, 0, 5), [0, 0, 0, 255]);
    assert_eq!(pixel_at(&surface, 3, 5), [0, 0, 0, 255]);
    assert_eq!(pixel_at(&surface, 1, 4), [0, 0, 0, 255]);
    assert_eq!(pixel_at(&surface, 1, 7), [0, 0, 0, 255]);
}

#[test]
fn a_fully_transparent_icon_leaves_the_badge_showing() {
    // This is what keeps the circle visible around a rounded icon: the badge is
    // not erased first, so the transparent border of an icon shows it.
    let mut surface = a_surface(4, 4, [70, 80, 90, 255]);
    let icon = a_picture(4, 4, [0, 0, 0, 0]);
    pictures_onto(&mut surface, &[(a_box(0, 0, 4, 4), &icon)]);
    for y in 0..4 {
        for x in 0..4 {
            assert_eq!(
                pixel_at(&surface, x, y),
                [70, 80, 90, 255],
                "nothing opaque was drawn, so nothing changed"
            );
        }
    }
}

#[test]
fn a_half_transparent_icon_mixes_with_what_is_under_it() {
    let mut surface = a_surface(2, 2, [0, 0, 0, 255]);
    let icon = a_picture(2, 2, [255, 255, 255, 128]);
    pictures_onto(&mut surface, &[(a_box(0, 0, 2, 2), &icon)]);
    let drawn = pixel_at(&surface, 0, 0);
    assert!(
        (100..=160).contains(&drawn[0]),
        "white at half opacity over black is about halfway between them, and was {}",
        drawn[0]
    );
    assert_eq!(drawn[3], 255, "over an opaque surface the result is opaque");
}

#[test]
fn a_picture_is_scaled_into_the_box_it_is_given() {
    // The box is the Dock's, and the file on disk is whatever size the theme
    // had. A picture drawn at its own size would overflow its slot or sit in a
    // corner of it.
    let mut surface = a_surface(8, 8, [0, 0, 0, 255]);
    let icon = a_picture(64, 64, [111, 122, 133, 255]);
    pictures_onto(&mut surface, &[(a_box(2, 2, 4, 4), &icon)]);

    assert_eq!(pixel_at(&surface, 2, 2), [111, 122, 133, 255]);
    assert_eq!(pixel_at(&surface, 5, 5), [111, 122, 133, 255]);
    assert_eq!(
        pixel_at(&surface, 6, 6),
        [0, 0, 0, 255],
        "and it stopped at the edge of its box rather than running on at its own size"
    );
}

#[test]
fn a_box_that_runs_off_the_surface_is_clipped_rather_than_refused() {
    let mut surface = a_surface(4, 4, [0, 0, 0, 255]);
    let icon = a_picture(4, 4, [255, 0, 0, 255]);
    pictures_onto(&mut surface, &[(a_box(2, 2, 4, 4), &icon)]);
    assert_eq!(
        pixel_at(&surface, 3, 3),
        [255, 0, 0, 255],
        "the part that fits is drawn"
    );
    assert_eq!(
        pixel_at(&surface, 0, 0),
        [0, 0, 0, 255],
        "and nothing else is"
    );
}

#[test]
fn a_box_entirely_off_the_surface_draws_nothing_and_does_not_panic() {
    let mut surface = a_surface(4, 4, [5, 5, 5, 255]);
    let icon = a_picture(2, 2, [255, 0, 0, 255]);
    pictures_onto(&mut surface, &[(a_box(-10, -10, 2, 2), &icon)]);
    pictures_onto(&mut surface, &[(a_box(100, 100, 2, 2), &icon)]);
    for y in 0..4 {
        for x in 0..4 {
            assert_eq!(pixel_at(&surface, x, y), [5, 5, 5, 255]);
        }
    }
}

#[test]
fn the_surfaces_own_position_is_subtracted_rather_than_ignored() {
    // **The fault this is here for.** A Dock's surface is a buffer whose
    // `area.loc` is where it sits on the display, and the rectangles handed in
    // are in the display's pixels too. Writing at the rectangle's raw position
    // would put every icon at the wrong place by exactly the Dock's offset —
    // which on a bottom Dock is most of the way up the screen, and off the
    // buffer entirely, so nothing would be drawn at all.
    let mut surface = Washed {
        area: Rectangle::new((100, 200).into(), (4, 4).into()),
        pixels: vec![0; 4 * 4 * 4],
    };
    let icon = a_picture(2, 2, [255, 255, 255, 255]);
    pictures_onto(&mut surface, &[(a_box(101, 201, 2, 2), &icon)]);
    assert_eq!(
        pixel_at(&surface, 1, 1),
        [255, 255, 255, 255],
        "one pixel into the buffer, which is where (101, 201) is on a surface at (100, 200)"
    );
    assert_eq!(
        pixel_at(&surface, 0, 0),
        [0, 0, 0, 0],
        "and not at its origin"
    );
}

#[test]
fn several_applications_are_each_drawn_in_their_own_box() {
    let mut surface = a_surface(8, 4, [0, 0, 0, 255]);
    let first = a_picture(2, 2, [255, 0, 0, 255]);
    let second = a_picture(2, 2, [0, 255, 0, 255]);
    pictures_onto(
        &mut surface,
        &[(a_box(0, 0, 2, 2), &first), (a_box(4, 0, 2, 2), &second)],
    );
    assert_eq!(pixel_at(&surface, 0, 0), [255, 0, 0, 255]);
    assert_eq!(pixel_at(&surface, 4, 0), [0, 255, 0, 255]);
    assert_eq!(
        pixel_at(&surface, 2, 0),
        [0, 0, 0, 255],
        "the gap between them"
    );
}

#[test]
fn a_surface_with_no_area_is_left_alone() {
    let mut surface = Washed {
        area: Rectangle::new((0, 0).into(), (0, 0).into()),
        pixels: Vec::new(),
    };
    let icon = a_picture(2, 2, [255, 0, 0, 255]);
    pictures_onto(&mut surface, &[(a_box(0, 0, 2, 2), &icon)]);
    assert!(surface.pixels.is_empty());
}
