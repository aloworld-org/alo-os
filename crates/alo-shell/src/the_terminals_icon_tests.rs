#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::{across_at, the_terminals_icon};

/// One pixel of a picture, red green blue alpha.
fn at(picture: &alo_dock::artwork::APicture, x: u32, y: u32) -> [u8; 4] {
    let i = usize::try_from((y * picture.across() + x) * 4).expect("a place in the picture");
    let mut out = [0; 4];
    out.copy_from_slice(picture.pixels().get(i..i + 4).expect("a pixel"));
    out
}

/// **The tile is the design's 16 at 100 per cent, and 32 at 200**: drawn
/// sharper on a dense display, never smaller.
#[test]
fn the_tile_follows_the_displays_scale() {
    assert_eq!(across_at(100), 16);
    assert_eq!(across_at(200), 32);
    assert_eq!(across_at(150), 24);
    let icon = the_terminals_icon(across_at(200)).expect("a tile is drawn");
    assert_eq!((icon.across(), icon.down()), (32, 32));
}

/// **Navy, with a white mark on it, and a rounded corner**, as the render of
/// `557:148` shows: the tile's left middle is `text/primary`, the mark's
/// stroke is white, and the corner pixel is not fully covered.
#[test]
fn it_is_a_navy_tile_with_a_white_mark() {
    let icon = the_terminals_icon(64).expect("a tile is drawn");
    assert_eq!(
        at(&icon, 2, 32),
        [0x10, 0x2A, 0x43, 0xFF],
        "the tile is navy"
    );
    assert!(at(&icon, 0, 0)[3] < 0xFF, "the corner is rounded");
    let white = (0..64)
        .flat_map(|y| (0..64).map(move |x| (x, y)))
        .filter(|(x, y)| {
            let [r, g, b, _] = at(&icon, *x, *y);
            r > 0xF0 && g > 0xF0 && b > 0xF0
        })
        .count();
    assert!(white > 40, "the mark is drawn in white: {white} pixels");
    // **And it stays inside the tile's middle**: the mark is 14 of 16, so
    // the outer two pixels of 64 on every side carry none of it.
    for n in 0..64 {
        for (x, y) in [(n, 0), (n, 63), (0, n), (63, n)] {
            let [r, g, b, _] = at(&icon, x, y);
            assert!(
                r < 0x80 && g < 0x80 && b < 0x80,
                "the mark reaches the edge at ({x}, {y})"
            );
        }
    }
}
