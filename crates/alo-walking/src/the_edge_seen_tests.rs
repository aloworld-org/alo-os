//! Finding alo's teal edge round a window in a picture of the screen — and not
//! finding one in the other teal things a screen holds.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on a ring not found is the failure being reported"
)]

use super::*;
use crate::frame::Frame;

/// The canvas a window sits on, in the light theme: warm white.
const CANVAS: [u8; 3] = [250, 247, 242];

/// A window's own dark ground, as VSCodium's is.
const DARK: [u8; 3] = [24, 24, 27];

/// The band's soft teal, which is not the edge.
const SOFT: [u8; 3] = [222, 238, 238];

/// A screen the walk's size, with a dark window on it.
fn a_screen() -> Frame {
    let mut frame = Frame::filled(1280, 800, CANVAS);
    for y in 30..660 {
        for x in 16..1152 {
            frame = frame.with_pixel(x, y, DARK);
        }
    }
    frame
}

/// Paint a rectangle.
fn filled(mut frame: Frame, (x0, y0, x1, y1): (usize, usize, usize, usize), rgb: [u8; 3]) -> Frame {
    for y in y0..y1 {
        for x in x0..x1 {
            frame = frame.with_pixel(x, y, rgb);
        }
    }
    frame
}

/// A ring `wide` pixels thick round `(x0, y0)`–`(x1, y1)`, with corners cut
/// away `corner` pixels each way, as a rounded ring's straight sides are.
fn a_ring(
    frame: Frame,
    (x0, y0, x1, y1): (usize, usize, usize, usize),
    wide: usize,
    corner: usize,
    rgb: [u8; 3],
) -> Frame {
    let frame = filled(frame, (x0 + corner, y0, x1 - corner, y0 + wide), rgb);
    let frame = filled(frame, (x0 + corner, y1 - wide, x1 - corner, y1), rgb);
    let frame = filled(frame, (x0, y0 + corner, x0 + wide, y1 - corner), rgb);
    filled(frame, (x1 - wide, y0 + corner, x1, y1 - corner), rgb)
}

/// The edge round the window in the screenshot of 2026-11-15, as the shell
/// draws it at scale 1: two pixels, the card's corner of 16.
fn the_edge_round_vscodium(frame: Frame) -> Frame {
    a_ring(frame, (14, 28, 1154, 662), 2, 16, TEAL)
}

/// **The edge round a window is found, and where it is.**
#[test]
fn the_edge_round_a_window_is_found_where_it_is() {
    let frame = the_edge_round_vscodium(a_screen());
    let Ok(ring) = a_ring_in(&frame) else {
        panic!("the edge was not found");
    };
    assert!(ring.left.abs_diff(14) <= 2, "{ring:?}");
    assert!(ring.right.abs_diff(1153) <= 2, "{ring:?}");
    assert!(ring.top.abs_diff(28) <= 2, "{ring:?}");
    assert!(ring.bottom.abs_diff(661) <= 2, "{ring:?}");
}

/// **The egress indicator over the top-left of the edge does not hide it**:
/// law 1 keeps the indicator over everything, and the screenshot this work
/// began from had it half over VSCodium's top.
#[test]
fn the_edge_is_found_with_the_indicator_over_its_top() {
    let frame = the_edge_round_vscodium(a_screen());
    let frame = filled(frame, (20, 10, 540, 48), [255, 255, 255]);
    assert!(a_ring_in(&frame).is_ok(), "the indicator hid the edge");
}

/// **A bare screen has no edge**, and says the longest teal it did find.
#[test]
fn a_bare_screen_has_no_edge() {
    let frame = a_screen();
    let Err(nearest) = a_ring_in(&frame) else {
        panic!("an edge was found on a bare screen");
    };
    assert_eq!(nearest.across, 0);
    assert_eq!(nearest.down, 0);
}

/// **Other teal on a screen is not an edge**: the indicator's teal circle, alo's
/// mark, one long teal line, two sides of a window, three sides of one.
#[test]
fn other_teal_on_a_screen_is_not_an_edge() {
    let circle = filled(a_screen(), (28, 18, 52, 42), TEAL);
    assert!(a_ring_in(&circle).is_err(), "a teal circle read as an edge");

    let line = filled(a_screen(), (100, 300, 900, 302), TEAL);
    let Err(nearest) = a_ring_in(&line) else {
        panic!("one line read as an edge");
    };
    assert_eq!(nearest.across, 800, "{nearest:?}");

    // An L: the top and the left, and nothing else.
    let two = filled(a_screen(), (30, 28, 1138, 30), TEAL);
    let two = filled(two, (14, 44, 16, 646), TEAL);
    assert!(a_ring_in(&two).is_err(), "two sides read as an edge");

    // Three sides: no foot.
    let three = filled(two, (1152, 44, 1154, 646), TEAL);
    assert!(a_ring_in(&three).is_err(), "three sides read as an edge");
}

/// **A ring in the band's soft teal is not alo's edge**, nor is a ring in any
/// colour more than a little way from deep teal.
#[test]
fn a_ring_in_another_colour_is_not_alos_edge() {
    let soft = a_ring(a_screen(), (14, 28, 1154, 662), 2, 16, SOFT);
    assert!(a_ring_in(&soft).is_err(), "the soft teal read as the edge");
    let navy = a_ring(a_screen(), (14, 28, 1154, 662), 2, 16, [0x10, 0x2A, 0x43]);
    assert!(a_ring_in(&navy).is_err(), "a navy ring read as alo's edge");
}

/// **A small teal ring is not a window's edge**: the shortest side a window
/// alo works in can have is far longer than any control's outline.
#[test]
fn a_small_teal_ring_is_not_a_windows_edge() {
    let small = a_ring(a_screen(), (400, 400, 480, 440), 2, 6, TEAL);
    assert!(
        a_ring_in(&small).is_err(),
        "a control's outline read as an edge"
    );
}

/// **The colour is the one the shell draws the edge in**: `accent/default`,
/// the token `alo_shell::alos_band_in_a_window` paints the edge with, and
/// deep teal `#0F6B72`, the brand anchor ADR 0067 says never moves.
#[test]
fn the_colour_is_the_shells_accent_and_deep_teal() {
    let accent = alo_appearance::Role::AccentDefault.colour();
    assert_eq!(TEAL, [accent.red(), accent.green(), accent.blue()]);
    assert_eq!(TEAL, [0x0F, 0x6B, 0x72]);
}

/// **A teal block is not an edge**: the edge is thin — the focus ring's
/// weight — so a filled teal rectangle, whose sides are teal too, is not a
/// window wearing alo.
#[test]
fn a_teal_block_is_not_an_edge() {
    let block = filled(a_screen(), (200, 200, 600, 500), TEAL);
    assert!(a_ring_in(&block).is_err(), "a teal block read as an edge");
    let thick = a_ring(a_screen(), (14, 28, 1154, 662), 24, 16, TEAL);
    assert!(
        a_ring_in(&thick).is_err(),
        "a thick frame read as the thin edge"
    );
}

/// **The edge at scale 2 is four pixels and its corner 32**, and is found.
#[test]
fn the_edge_at_twice_the_density_is_found() {
    let frame = a_ring(a_screen(), (14, 28, 1154, 662), 4, 32, TEAL);
    assert!(
        a_ring_in(&frame).is_ok(),
        "the edge at scale 2 was not found"
    );
}

/// **A frame thicker than the edge is not the edge, whatever its corners**: a
/// 24-pixel teal frame with corners of 48 closes round a rectangle just as the
/// edge does, and only its thickness tells it apart.
#[test]
fn a_thick_frame_with_wide_corners_is_not_the_edge() {
    let thick = a_ring(a_screen(), (14, 28, 1154, 662), 24, 48, TEAL);
    assert!(
        a_ring_in(&thick).is_err(),
        "a 24-pixel frame read as the thin edge"
    );
}

/// **The border at rest and while sending are both found**: 3 and 6 at a
/// scale of 1, and 6 while sending at a scale of 3 is 18.
#[test]
fn the_border_at_rest_and_while_sending_is_found() {
    for (wide, corner) in [(3, 16), (6, 16), (18, 48)] {
        let frame = a_ring(a_screen(), (14, 28, 1154, 662), wide, corner, TEAL);
        assert!(
            a_ring_in(&frame).is_ok(),
            "a border {wide} wide was not found"
        );
    }
}

/// A picture the walk kept, read from `tests/fixtures`.
fn a_kept_picture(name: &str) -> Frame {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    let Ok(picture) = image::open(&path) else {
        panic!("{} could not be read", path.display());
    };
    let rgb = picture.to_rgb8();
    let mut ppm = format!("P6\n{} {}\n255\n", rgb.width(), rgb.height()).into_bytes();
    ppm.extend_from_slice(rgb.as_raw());
    let Ok(frame) = Frame::from_ppm(&ppm) else {
        panic!("{} is not a picture", path.display());
    };
    frame
}

/// **VSCodium still there after its edge has gone is seen to be there** —
/// the walk of 2026-11-16 on `main` at ab4021d6 (run 43872632801): its edge was
/// found round VSCodium in picture 57, taken before VSCodium had drawn
/// anything in its window, and picture 59, with the edge gone and the editor
/// drawn, is VSCodium exactly where it was. Every pixel inside had changed, so
/// a check that asked them to be as they were said the window was gone.
#[test]
fn vscodium_after_its_edge_has_gone_is_still_there() {
    let wearing = a_kept_picture("walk-ab4021d6-57-vscodium-wearing-alo.png");
    let after = a_kept_picture("walk-ab4021d6-59-vscodium-after-alo.png");
    let Ok(ring) = a_ring_in(&wearing) else {
        panic!("the edge in picture 57 was not found");
    };
    assert!(a_ring_in(&after).is_err(), "picture 59 still has an edge");
    assert!(
        a_window_is_where_the_edge_was(ring, &after),
        "VSCodium in picture 59 was not seen where its edge was, {ring:?}"
    );
}

/// **A screen with no window where the edge was is not a window still
/// there**: a black screen, or the canvas alone.
#[test]
fn no_window_where_the_edge_was_is_not_a_window_still_there() {
    let wearing = a_kept_picture("walk-ab4021d6-57-vscodium-wearing-alo.png");
    let Ok(ring) = a_ring_in(&wearing) else {
        panic!("the edge in picture 57 was not found");
    };
    let (wide, high) = wearing.size();
    assert!(!a_window_is_where_the_edge_was(
        ring,
        &Frame::filled(wide, high, [0, 0, 0])
    ));
    assert!(!a_window_is_where_the_edge_was(
        ring,
        &Frame::filled(wide, high, CANVAS)
    ));
}

/// **An edge with a side past the screen's edge is still alo's edge** — the
/// walk of 2026-11-16 on `main` at 4f3a92d4 (run 43889781022) placed VSCodium
/// at x −24, so its border's left side was off the display; the top and foot
/// run to the screen's left edge and the right side closes it.
#[test]
fn an_edge_with_its_left_side_off_the_screen_is_found() {
    // The border round (−24, 70)–(1112, 700): top and foot from the screen's
    // left edge, no left side, the right side at 1109.
    let frame = filled(a_screen(), (0, 70, 1096, 73), TEAL);
    let frame = filled(frame, (0, 697, 1096, 700), TEAL);
    let frame = filled(frame, (1109, 86, 1112, 684), TEAL);
    let Ok(ring) = a_ring_in(&frame) else {
        panic!("an edge cut by the screen's left was not found");
    };
    assert_eq!(ring.left, 0, "{ring:?}");
    assert!(ring.right.abs_diff(1111) <= 2, "{ring:?}");
}

/// **But a side missing in the middle of the screen is still not an edge**:
/// a side may be absent only where it would lie past the screen.
#[test]
fn an_edge_missing_a_side_on_the_screen_is_not_found() {
    let frame = filled(a_screen(), (40, 70, 1096, 73), TEAL);
    let frame = filled(frame, (40, 697, 1096, 700), TEAL);
    let frame = filled(frame, (1109, 86, 1112, 684), TEAL);
    assert!(
        a_ring_in(&frame).is_err(),
        "three sides in the middle read as an edge"
    );
}
