//! The wallpaper, asked of the pixels it produced and of the three mechanisms
//! that produce them.
//!
//! **The mechanisms are tested directly and the result is read back.** A test
//! that only read pixels could not tell a half-covered edge from a lucky
//! gradient value, and a test that only checked the span arithmetic would pass
//! on a rasteriser that drew nothing at all. So: `add_span`, `colour_at` and
//! `cubic` are asked their own questions, and `the_wallpaper` is asked what it
//! actually drew.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a raster that is not there is the failure being reported"
)]

use super::{
    Gradient, PIECES_PER_CURVE, SUB_ROWS, THE_DESIGN_IS, THE_FOLDS, add_span, colour_at, cubic,
    the_wallpaper,
};

/// The ground the sign-in screen is drawn on, which is what the folds sit upon.
const GROUND: [u8; 3] = [0xFA, 0xF7, 0xF2];

/// The size the design is drawn at, as whole pixels.
const REFERENCE: (i32, i32) = (1440, 960);

/// The pixel at a point of a raster of this width.
fn at(pixels: &[[u8; 3]], w: i32, x: i32, y: i32) -> [u8; 3] {
    let i = (y * w + x) as usize;
    *pixels.get(i).expect("that point is inside the raster")
}

/// **The raster is the screen, exactly one colour per pixel.**
#[test]
fn the_raster_is_the_whole_screen() {
    let drawn = the_wallpaper(REFERENCE.0, REFERENCE.1, GROUND);
    assert_eq!(drawn.len(), (REFERENCE.0 * REFERENCE.1) as usize);
}

/// **A screen with no area draws nothing**, rather than a panic or a row of
/// one pixel.
#[test]
fn a_screen_with_no_area_draws_nothing() {
    for (w, h) in [(0, 960), (1440, 0), (0, 0), (-1, 10), (10, -1)] {
        assert!(
            the_wallpaper(w, h, GROUND).is_empty(),
            "{w}x{h} produced pixels"
        );
    }
    // The control: a screen that does have area produces some, so the refusals
    // above are about the size and not a function that always returns nothing.
    assert!(!the_wallpaper(8, 8, GROUND).is_empty());
}

/// **The folds are drawn where the design puts them, and nowhere else.**
///
/// The right-hand side and the bottom-left corner carry artwork; the middle
/// left of the screen is bare ground. The third assertion is the control: if
/// the bare point were also covered, the other two would prove only that the
/// function paints everything.
#[test]
fn the_folds_are_on_the_right_and_in_the_bottom_left_corner_and_nowhere_else() {
    let drawn = the_wallpaper(REFERENCE.0, REFERENCE.1, GROUND);
    let pixel = |x, y| at(&drawn, REFERENCE.0, x, y);

    assert_ne!(
        pixel(1400, 300),
        GROUND,
        "the folds down the right-hand side were not drawn"
    );
    assert_ne!(
        pixel(100, 920),
        GROUND,
        "the folds in the bottom-left corner were not drawn"
    );
    assert_eq!(
        pixel(200, 300),
        GROUND,
        "the middle left of the screen is bare in the design and something was drawn on it"
    );
    assert_eq!(
        pixel(600, 120),
        GROUND,
        "the top middle is bare in the design and something was drawn on it"
    );
}

/// **They are gradients, not flat shapes.**
///
/// Five flat fills on a ground would give six colours. Reading the actual
/// count is what tells a gradient from a fill that happens to sit in the right
/// place, and it is the whole reason this file carries gradient stops at all.
#[test]
fn the_folds_are_filled_with_gradients_rather_than_flat_colour() {
    let drawn = the_wallpaper(REFERENCE.0, REFERENCE.1, GROUND);
    let mut seen: Vec<[u8; 3]> = Vec::new();
    for colour in &drawn {
        if !seen.contains(colour) {
            seen.push(*colour);
            if seen.len() > 400 {
                break;
            }
        }
    }
    assert!(
        seen.len() > 400,
        "only {} distinct colours: five flat fills on a ground would give six, so this is \
         not drawing gradients",
        seen.len()
    );
}

/// **The same size twice is the same picture**, so a frame does not shimmer
/// between redraws.
#[test]
fn it_draws_the_same_picture_twice() {
    assert_eq!(
        the_wallpaper(640, 480, GROUND),
        the_wallpaper(640, 480, GROUND)
    );
}

/// **It covers rather than stretches.**
///
/// On a screen wider than the design's own ratio the folds must still reach
/// the right edge, and on a taller one they must still reach the bottom. A fit
/// that letterboxed would leave bare ground at an edge the design fills.
#[test]
fn the_artwork_covers_a_screen_of_any_shape() {
    for (w, h) in [
        (1920, 960),
        (1280, 800),
        (1366, 768),
        (2560, 1440),
        (800, 600),
    ] {
        let drawn = the_wallpaper(w, h, GROUND);
        assert_eq!(drawn.len(), (w * h) as usize, "{w}x{h} is the wrong length");
        assert_ne!(
            at(&drawn, w, w - 2, h / 3),
            GROUND,
            "the right-hand folds do not reach the right edge of {w}x{h}"
        );
        assert_ne!(
            at(&drawn, w, w / 8, h - 2),
            GROUND,
            "the corner folds do not reach the bottom of {w}x{h}"
        );
    }
}

/// **A fold's edge is part-covered, not a staircase.**
///
/// Walking in from the bare middle towards the right-hand folds, the first
/// pixel that is not the ground must be *closer* to the ground than the pixel
/// three further in. A hard edge would jump to the fold's colour in one step
/// and the two distances would match.
#[test]
fn the_edge_of_a_fold_is_part_covered() {
    let drawn = the_wallpaper(REFERENCE.0, REFERENCE.1, GROUND);
    let away_from_ground = |c: [u8; 3]| {
        i32::from(c[0]).abs_diff(i32::from(GROUND[0]))
            + i32::from(c[1]).abs_diff(i32::from(GROUND[1]))
            + i32::from(c[2]).abs_diff(i32::from(GROUND[2]))
    };
    let y = 300;
    let first = (600..REFERENCE.0)
        .find(|x| at(&drawn, REFERENCE.0, *x, y) != GROUND)
        .expect("the row crosses a fold somewhere");
    assert!(
        first + 3 < REFERENCE.0,
        "the crossing is at the very edge of the screen, so this proves nothing"
    );
    let edge = away_from_ground(at(&drawn, REFERENCE.0, first, y));
    let inside = away_from_ground(at(&drawn, REFERENCE.0, first + 3, y));
    assert!(
        edge < inside,
        "the first covered pixel at x={first} is already as dark as the inside \
         ({edge} against {inside}): the edge is a staircase, not a coverage"
    );
}

/// **A span covers the pixels it crosses, and the fraction at each end.**
///
/// The antialiasing itself, asked directly. A whole pixel takes the full
/// share; a pixel the span enters halfway takes half of it.
#[test]
fn a_span_covers_whole_pixels_fully_and_part_pixels_in_part() {
    let mut coverage = vec![0.0_f64; 8];
    assert!(add_span(&mut coverage, 2.0, 5.0, 1.0, 8));
    assert_eq!(
        coverage,
        vec![0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0],
        "a span on whole-pixel boundaries did not cover exactly three pixels"
    );

    let mut coverage = vec![0.0_f64; 8];
    assert!(add_span(&mut coverage, 2.5, 4.25, 1.0, 8));
    assert!((coverage.get(2).copied().unwrap_or_default() - 0.5).abs() < 1e-9);
    assert!((coverage.get(3).copied().unwrap_or_default() - 1.0).abs() < 1e-9);
    assert!((coverage.get(4).copied().unwrap_or_default() - 0.25).abs() < 1e-9);

    // The share is how much of a pixel row one sub-scanline is worth, and four
    // of them must come to exactly one.
    let mut coverage = vec![0.0_f64; 4];
    let share = 1.0 / SUB_ROWS as f64;
    for _ in 0..SUB_ROWS {
        add_span(&mut coverage, 1.0, 3.0, share, 4);
    }
    assert!((coverage.get(1).copied().unwrap_or_default() - 1.0).abs() < 1e-9);
}

/// **A span outside the row, or inside out, covers nothing.**
#[test]
fn a_span_that_covers_nothing_says_so() {
    let mut coverage = vec![0.0_f64; 8];
    for (from, to) in [(-5.0, -1.0), (9.0, 12.0), (4.0, 4.0), (5.0, 2.0)] {
        assert!(
            !add_span(&mut coverage, from, to, 1.0, 8),
            "{from}..{to} claimed to cover something"
        );
    }
    assert!(
        coverage.iter().all(|at| *at == 0.0),
        "a span that covers nothing wrote coverage anyway"
    );
}

/// **A gradient is its stops, interpolated, and clamped at both ends.**
#[test]
fn a_gradient_reads_its_stops_and_clamps_outside_them() {
    let stops: &[(f64, [u8; 3])] = &[
        (0.0, [0, 0, 0]),
        (0.5, [100, 100, 100]),
        (1.0, [200, 200, 200]),
    ];
    assert_eq!(colour_at(stops, 0.0), [0, 0, 0]);
    assert_eq!(colour_at(stops, 0.5), [100, 100, 100]);
    assert_eq!(colour_at(stops, 1.0), [200, 200, 200]);
    assert_eq!(colour_at(stops, 0.25), [50, 50, 50]);
    assert_eq!(colour_at(stops, 0.75), [150, 150, 150]);
    // Clamped: no `spreadMethod` in the export means the ends simply hold.
    assert_eq!(colour_at(stops, -3.0), [0, 0, 0]);
    assert_eq!(colour_at(stops, 4.0), [200, 200, 200]);
}

/// **A cubic begins and ends where it is told**, which is what keeps a
/// flattened outline closed.
#[test]
fn a_cubic_begins_and_ends_at_its_own_points() {
    let (p0, p1, p2, p3) = ((0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (20.0, 10.0));
    assert_eq!(cubic(p0, p1, p2, p3, 0.0), p0);
    assert_eq!(cubic(p0, p1, p2, p3, 1.0), p3);
    let middle = cubic(p0, p1, p2, p3, 0.5);
    assert!(
        middle.0 > p0.0 && middle.0 < p3.0 && middle.1 > p0.1 && middle.1 < p3.1,
        "the middle of this curve left the box its ends make: {middle:?}"
    );
}

/// **Every fold the export draws is here, with a gradient that is in order.**
///
/// The transcription's own check: five folds, each outline beginning with a
/// move, and every gradient's stops running from 0 to 1 without going
/// backwards. A stop out of order would interpolate into the wrong colour and
/// nothing else would catch it.
#[test]
fn the_transcription_is_five_folds_with_gradients_in_order() {
    assert_eq!(THE_FOLDS.len(), 5, "the export draws five paths");
    assert_eq!(THE_DESIGN_IS, (1440.0, 960.0));
    const {
        assert!(
            PIECES_PER_CURVE >= 32,
            "a curve this long needs more pieces"
        )
    };

    for (which, fold) in THE_FOLDS.iter().enumerate() {
        assert!(
            matches!(fold.outline.first(), Some(super::Step::Begin(_, _))),
            "fold {which} does not begin with a move"
        );
        assert!(
            fold.outline.len() >= 4,
            "fold {which} has too few steps to close"
        );
        let Gradient { stops, from, to } = fold.gradient;
        assert!(stops.len() >= 2, "fold {which} has a gradient of one stop");
        assert_eq!(
            stops.first().map(|(at, _)| *at),
            Some(0.0),
            "fold {which}'s gradient does not begin at 0"
        );
        assert_eq!(
            stops.last().map(|(at, _)| *at),
            Some(1.0),
            "fold {which}'s gradient does not end at 1"
        );
        for (a, b) in stops.iter().zip(stops.iter().skip(1)) {
            assert!(
                a.0 < b.0,
                "fold {which}'s stops go backwards at {} then {}",
                a.0,
                b.0
            );
        }
        assert!(
            from != to,
            "fold {which}'s gradient begins and ends at the same point"
        );
    }
}
