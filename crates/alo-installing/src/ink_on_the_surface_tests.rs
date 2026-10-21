//! That the bytes a row becomes are the right bytes — asked on a machine with
//! no screen, which is the half of drawing that can be.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use super::a_row_of_ink;
use crate::a_line_as_coverage::a_line_as_coverage;
use crate::a_surface_to_draw_on::{Grey, TheSurface};
use crate::alos_own_face::AlosOwnFace;

/// The background the environment draws on, and the ink on it.
const BENEATH: Grey = Grey::of(0x18);
const INK: Grey = Grey::of(0xE8);

/// A surface the shape a real framebuffer is: four bytes a pixel, and a stride
/// wider than the visible width, because that padding is where a fault hides.
fn a_surface() -> TheSurface {
    // Built the way the real one is -- from the three things the kernel
    // publishes -- rather than by reaching into the struct, so these tests
    // exercise the road a boot takes. Stride 5184 is 1280 pixels of four
    // bytes plus 64 of padding, and the padding is where a fault hides.
    let size = concat!("1280,800", "\n");
    let stride = concat!("5184", "\n");
    let depth = concat!("32", "\n");
    TheSurface::from_what_the_kernel_says(size, stride, depth)
        .expect("a surface the shape a framebuffer is")
}

fn face() -> AlosOwnFace<'static> {
    AlosOwnFace::carried().expect("the typeface compiled into this program")
}

#[test]
fn a_row_is_the_stride_long_so_no_padding_is_left_unwritten() {
    // The fault this guards: a row written only as far as the visible width
    // leaves the padding holding whatever the firmware put there, which shows
    // as a strip of noise down the right-hand edge.
    let surface = a_surface();
    let drawn = a_line_as_coverage(face(), "alo OS", 16, 25);
    let row = a_row_of_ink(surface, BENEATH, INK, &drawn, 100, 10);
    assert_eq!(row.len(), 5184);
}

#[test]
fn a_row_with_no_coverage_is_exactly_the_background() {
    let surface = a_surface();
    let drawn = a_line_as_coverage(face(), "alo OS", 16, 25);
    // Row nought of a line of lower-case letters is above every letter, so
    // nothing covers it.
    let blank = a_row_of_ink(surface, BENEATH, INK, &drawn, 100, 0);
    assert!(
        blank.iter().all(|&byte| byte == 0x18),
        "a row with nothing on it is not the background"
    );
}

#[test]
fn the_ink_actually_reaches_the_row() {
    // **The check that this is not a quiet nothing.** Everything else could
    // pass against a function that returns a background-coloured row, and a
    // screen drawn from those rows is a blank screen with no error anywhere.
    let surface = a_surface();
    let drawn = a_line_as_coverage(face(), "alo OS", 16, 25);
    let any_ink = (0..drawn.down()).any(|row_of_the_line| {
        a_row_of_ink(surface, BENEATH, INK, &drawn, 100, row_of_the_line)
            .iter()
            .any(|&byte| byte != 0x18)
    });
    assert!(any_ink, "no row of the line had any ink in it at all");
}

#[test]
fn full_coverage_gives_the_ink_and_none_gives_the_background() {
    // The two ends of the blend, which is where an off-by-one in the
    // arithmetic would show as letters that are never quite the right weight.
    let surface = a_surface();
    let solid = a_line_as_coverage(face(), "█", 16, 25);
    if !solid.anything_at_all() {
        // Manrope may not carry a full block, and that is not this test's
        // subject — the blend is checked directly below instead.
        return;
    }
    let row = (0..solid.down())
        .map(|row_of_the_line| a_row_of_ink(surface, BENEATH, INK, &solid, 0, row_of_the_line))
        .find(|row| row.contains(&0xE8));
    assert!(
        row.is_some(),
        "a fully covered pixel did not come out as the ink"
    );
}

#[test]
fn the_blend_is_monotonic_and_hits_both_ends_exactly() {
    // Asserted on the arithmetic itself, because the two ends are what a
    // letter's body and the paper around it are, and everything between is its
    // edge. A blend that never reaches either end draws grey text on grey.
    let mut last = 0_u8;
    for covered in 0..=255_u8 {
        let mixed = super::mixed(Grey::of(0), Grey::of(255), covered);
        assert!(
            mixed >= last,
            "coverage {covered} mixed to {mixed}, less than {last} before it"
        );
        last = mixed;
    }
    assert_eq!(super::mixed(Grey::of(0x18), Grey::of(0xE8), 0), 0x18);
    assert_eq!(super::mixed(Grey::of(0x18), Grey::of(0xE8), 255), 0xE8);
    // And half coverage is about half way, rather than biased to either side.
    let half = super::mixed(Grey::of(0), Grey::of(255), 128);
    assert!(
        (126..=130).contains(&half),
        "half coverage mixed to {half}, which is not half way"
    );
}

#[test]
fn every_channel_of_a_pixel_gets_the_same_value() {
    // Which is what makes a grey a grey whatever order the channels are in —
    // the property `a_surface_to_draw_on` chose a grey for, and one this file
    // must not quietly lose.
    let surface = a_surface();
    let drawn = a_line_as_coverage(face(), "H", 36, 56);
    for row_of_the_line in 0..drawn.down() {
        let row = a_row_of_ink(surface, BENEATH, INK, &drawn, 0, row_of_the_line);
        for pixel in 0..40_usize {
            let at = pixel * 4;
            let four = &row[at..at + 4];
            assert!(
                four.iter().all(|&byte| byte == four[0]),
                "pixel {pixel} of row {row_of_the_line} is {four:?}, which is not a grey"
            );
        }
    }
}

#[test]
fn a_line_is_clipped_at_the_right_edge_rather_than_wrapping() {
    // A letter continued on the next row down reads as a corrupted screen, so
    // the part past the edge is dropped. Placed so that the line starts eight
    // pixels before the edge and is far wider than that.
    let surface = a_surface();
    let drawn = a_line_as_coverage(face(), "alo OS alo OS alo OS", 16, 25);
    assert!(drawn.across() > 8, "the line is too short to clip");

    for row_of_the_line in 0..drawn.down() {
        let row = a_row_of_ink(surface, BENEATH, INK, &drawn, 1280 - 8, row_of_the_line);
        // Nothing is written at or past the visible width, so the first pixel
        // of the row — the far left — is untouched background.
        assert!(
            row[..4].iter().all(|&byte| byte == 0x18),
            "the clipped part of the line wrapped round to the left"
        );
        // And the padding past the visible width is the background too.
        let padding = 1280 * 4;
        assert!(
            row[padding..].iter().all(|&byte| byte == 0x18),
            "ink was written into the stride's padding"
        );
    }
}

#[test]
fn a_line_placed_entirely_off_the_right_edge_draws_nothing() {
    let surface = a_surface();
    let drawn = a_line_as_coverage(face(), "alo OS", 16, 25);
    let row = a_row_of_ink(surface, BENEATH, INK, &drawn, 1280, 10);
    assert!(
        row.iter().all(|&byte| byte == 0x18),
        "a line starting at the right edge still drew something"
    );
    // And one placed so far across that the arithmetic would overflow.
    let row = a_row_of_ink(surface, BENEATH, INK, &drawn, u32::MAX, 10);
    assert!(
        row.iter().all(|&byte| byte == 0x18),
        "a line placed at the far end of the number line drew something"
    );
}

#[test]
fn a_row_past_the_end_of_the_line_is_the_background() {
    let surface = a_surface();
    let drawn = a_line_as_coverage(face(), "alo OS", 16, 25);
    let row = a_row_of_ink(surface, BENEATH, INK, &drawn, 100, drawn.down() + 5);
    assert!(row.iter().all(|&byte| byte == 0x18));
}
