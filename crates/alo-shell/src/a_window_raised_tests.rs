//! The line and the shadow round a window: round it, never on it.
#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or a missing pixel is the failure being reported"
)]

use super::*;

/// The window `557:212` draws focused, at its place in the 1440 frame, less
/// its 44-tall edge.
fn dolphin() -> Rectangle<i32, Physical> {
    Rectangle::new((672, 164).into(), (720, 556).into())
}

/// The straight-alpha pixel at `(x, y)` among these rasters, if one has it.
fn pixel(rasters: &[Washed], x: i32, y: i32) -> Option<[u8; 4]> {
    rasters.iter().find_map(|raster| {
        let area = raster.area;
        let inside = x >= area.loc.x
            && x < area.loc.x + area.size.w
            && y >= area.loc.y
            && y < area.loc.y + area.size.h;
        inside.then(|| {
            let at = usize::try_from(((y - area.loc.y) * area.size.w + (x - area.loc.x)) * 4)
                .expect("inside");
            let mut it = [0_u8; 4];
            it.copy_from_slice(&raster.pixels[at..at + 4]);
            it
        })
    })
}

/// **Nothing is drawn on the application's own pixels**, at any scale — the
/// board's *never dimmed, cropped or recoloured*.
#[test]
fn not_one_pixel_lands_on_the_window() {
    for scale in [1.0, 1.25, 2.0] {
        for look in [WhichLook::Focused, WhichLook::NotFocused] {
            let bands = round_the_window(dolphin(), scale, look, (2880, 1920));
            assert!(!bands.is_empty());
            for band in &bands {
                assert!(
                    band.area.intersection(dolphin()).is_none(),
                    "{look:?} at {scale}: {:?} is on the window",
                    band.area
                );
            }
        }
    }
}

/// **The line is one unit, just outside, in the look's colour**: `border/strong`
/// round the focused window, `border/default` round every other.
#[test]
fn the_line_is_the_pixel_next_to_the_window_in_the_looks_colour() {
    let window = dolphin();
    for (look, colour) in [
        (WhichLook::Focused, [0xCB, 0xD5, 0xE1]),
        (WhichLook::NotFocused, [0xE7, 0xEB, 0xEF]),
    ] {
        let bands = round_the_window(window, 1.0, look, (1440, 960));
        let mid_y = window.loc.y + 200;
        let left = pixel(&bands, window.loc.x - 1, mid_y).expect("left of the window");
        let right =
            pixel(&bands, window.loc.x + window.size.w, mid_y).expect("right of the window");
        let below = pixel(&bands, window.loc.x + 100, window.loc.y + window.size.h)
            .expect("below the window");
        for it in [left, right, below] {
            assert_eq!(it, [colour[0], colour[1], colour[2], 255], "{look:?}");
        }
        // Two out is the shadow, not the line.
        let beyond = pixel(&bands, window.loc.x - 2, mid_y).expect("further left");
        assert_ne!(
            beyond[3], 255,
            "{look:?}: the line is one pixel and no more"
        );
    }
}

/// **At twice the density the line is two pixels**, not one: the same line,
/// sharper, which is what *nothing is built to one screen size* asks.
#[test]
fn a_dense_display_draws_the_same_line_twice_as_many_pixels_thick() {
    let window = Rectangle::new((200, 200).into(), (800, 600).into());
    let bands = round_the_window(window, 2.0, WhichLook::NotFocused, (2880, 1920));
    let mid_y = 500;
    for x in [199, 198] {
        assert_eq!(
            pixel(&bands, x, mid_y).expect("left")[3],
            255,
            "column {x} is the line"
        );
    }
    assert_ne!(pixel(&bands, 197, mid_y).expect("left")[3], 255);
}

/// **The focused window stands further off the canvas**: below it, at the
/// same distance, its shadow is darker than an unfocused window's.
#[test]
fn the_focused_windows_shadow_is_deeper_below_it() {
    let window = dolphin();
    let below = (window.loc.x + 360, window.loc.y + window.size.h + 12);
    let alpha = |look| {
        pixel(
            &round_the_window(window, 1.0, look, (1440, 960)),
            below.0,
            below.1,
        )
        .expect("below the window")[3]
    };
    let (high, low) = (alpha(WhichLook::Focused), alpha(WhichLook::NotFocused));
    assert!(high > 0 && low > 0, "{high} {low}");
    assert!(
        high > low,
        "focused {high} is not deeper than unfocused {low}"
    );
}

/// **Nothing is drawn off the display**, so a window at its edge loses its
/// shadow there rather than the frame.
#[test]
fn a_window_at_the_displays_edge_keeps_its_rasters_on_it() {
    let at_the_corner = Rectangle::new((0, 0).into(), (300, 200).into());
    for band in round_the_window(at_the_corner, 1.0, WhichLook::Focused, (1280, 800)) {
        assert!(
            band.area.loc.x >= 0 && band.area.loc.y >= 0,
            "{:?}",
            band.area
        );
        assert!(band.area.loc.x + band.area.size.w <= 1280);
        assert!(band.area.loc.y + band.area.size.h <= 800);
    }
}

/// **A window with no area has nothing round it.**
#[test]
fn a_window_with_no_area_is_left_alone() {
    let nothing = Rectangle::new((10, 10).into(), (0, 40).into());
    assert!(round_the_window(nothing, 1.0, WhichLook::Focused, (1280, 800)).is_empty());
}
