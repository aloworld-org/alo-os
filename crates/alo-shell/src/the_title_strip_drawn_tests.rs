//! The strip: rounded at the top, square at the bottom, in the look's colours.
#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or a missing pixel is the failure being reported"
)]

use super::*;

/// The strip of `557:212`'s edge, at its place in the 1440 frame.
fn strip() -> Rectangle<i32, Physical> {
    Rectangle::new((672, 132).into(), (720, 32).into())
}

/// The straight-alpha pixel at `(x, y)` of a raster.
fn pixel(raster: &Washed, x: i32, y: i32) -> [u8; 4] {
    let area = raster.area;
    let at = usize::try_from(((y - area.loc.y) * area.size.w + (x - area.loc.x)) * 4)
        .expect("inside the raster");
    let mut it = [0_u8; 4];
    it.copy_from_slice(&raster.pixels[at..at + 4]);
    it
}

/// **Rounded at the top, square at the bottom** — `554:25`'s `[10, 10, 0, 0]`.
#[test]
fn the_top_corners_are_rounded_and_the_bottom_ones_are_not() {
    let drawn = the_strip(strip(), 1.0, WhichLook::Focused, (1440, 960)).expect("on the display");
    let (l, t) = (drawn.area.loc.x, drawn.area.loc.y);
    let (r, b) = (l + drawn.area.size.w - 1, t + drawn.area.size.h - 1);
    assert_eq!(pixel(&drawn, l, t)[3], 0, "the top-left corner is cut");
    assert_eq!(pixel(&drawn, r, t)[3], 0, "the top-right corner is cut");
    assert_eq!(
        pixel(&drawn, l, b)[3],
        255,
        "the bottom-left corner is square"
    );
    assert_eq!(
        pixel(&drawn, r, b)[3],
        255,
        "the bottom-right corner is square"
    );
}

/// **The ground and the line are the look's.**
#[test]
fn the_focused_strip_is_white_and_every_other_is_cream() {
    for (look, ground, line) in [
        (WhichLook::Focused, [0xFF, 0xFF, 0xFF], [0xCB, 0xD5, 0xE1]),
        (
            WhichLook::NotFocused,
            [0xFA, 0xF7, 0xF2],
            [0xE7, 0xEB, 0xEF],
        ),
    ] {
        let drawn = the_strip(strip(), 1.0, look, (1440, 960)).expect("on the display");
        let middle = pixel(&drawn, 672 + 360, 132 + 16);
        assert_eq!(middle, [ground[0], ground[1], ground[2], 255], "{look:?}");
        let bottom_line = pixel(&drawn, 672 + 360, 132 + 31);
        assert_eq!(bottom_line, [line[0], line[1], line[2], 255], "{look:?}");
    }
}

/// **One unit wider each side**, so its sides meet the line round the window.
#[test]
fn the_strip_reaches_the_line_round_the_window() {
    for (scale, wider) in [(1.0, 1), (2.0, 2)] {
        let drawn =
            the_strip(strip(), scale, WhichLook::NotFocused, (2880, 1920)).expect("on the display");
        assert_eq!(drawn.area.loc.x, 672 - wider);
        assert_eq!(drawn.area.size.w, 720 + wider * 2);
        assert_eq!(drawn.area.size.h, 32);
    }
}

/// **Off the display nothing is drawn**, and a strip wholly off it is none.
#[test]
fn a_strip_off_the_display_is_not_drawn() {
    let off = Rectangle::new((2000, 10).into(), (300, 32).into());
    assert!(the_strip(off, 1.0, WhichLook::Focused, (1440, 960)).is_none());
}
