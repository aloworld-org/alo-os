//! **History on a Dock down either side: the design's line drawing, not the
//! word.**
//!
//! The owner's decision of 2026-10-28. A side Dock's slots are 48 across, and
//! the word *History* does not fit in one in every language this machine
//! speaks — in English it already runs to the slot's edges. The design draws
//! an icon there (`Dock edges / Button / History`, `348:23633`): a clock face
//! with an arrow turning back, one stroke, one colour, 28 × 28 at 8, 8 inside
//! the button. The word becomes its tooltip (`crate::dock_tooltip`) and stays
//! its accessible name (`crate::the_dock_as_a_reader_meets_it`), so nobody
//! loses it.
//!
//! **A Dock across the top or the bottom keeps the word**, which is the
//! owner's instruction rather than a size rule: there the slot is laid out for
//! it.
//!
//! # Drawn here, with nothing new
//!
//! The drawing is three strokes — the arrow's two sides, the clock's rim, and
//! its hands — so it is held as those strokes rather than as an image file,
//! and drawn the way every round edge in this shell is drawn: by how far each
//! pixel is from the line. A stroke with round ends and round joins is exactly
//! the set of points within half its width of the path, so that distance is
//! the whole of the rasteriser, and no image decoder or vector library is
//! needed. The path is copied from the design's own export, read on 2026-10-29
//! and compared with the copy taken a day earlier: the same numbers.

use alo_dock::Edge;

/// Whether History is the drawing rather than the word on a Dock along `edge`.
#[must_use]
pub(crate) const fn history_is_a_drawing_on(edge: Edge) -> bool {
    matches!(edge, Edge::Left | Edge::Right)
}

/// The side of the square the design draws it in, in its own units.
const ITS_SQUARE: f32 = 28.0;

/// How wide its one stroke is, in the same units: the design's `1.98333`.
const ITS_STROKE: f32 = 1.983_33;

/// One step of the path, in the design's 28-unit square.
#[derive(Debug, Clone, Copy)]
enum Step {
    /// Lift the pen and put it here.
    Move(f32, f32),
    /// A straight line to here.
    Line(f32, f32),
    /// A cubic curve through two controls to here.
    Curve(f32, f32, f32, f32, f32, f32),
}

/// The design's path, step for step.
const THE_PATH: [Step; 15] = [
    // The arrow's two sides.
    Step::Move(10.5, 8.166_67),
    Step::Line(4.666_67, 8.166_67),
    Step::Line(4.666_67, 2.333_33),
    // The rim, from the arrow round to just short of where it began.
    Step::Move(4.666_67, 8.166_67),
    Step::Curve(5.979_85, 6.200_18, 7.918_36, 4.734_53, 10.168_3, 4.007_03),
    Step::Curve(12.418_3, 3.279_53, 14.847_9, 3.332_8, 17.063_8, 4.158_19),
    Step::Curve(19.279_7, 4.983_58, 21.152_1, 6.532_78, 22.377_9, 8.554_92),
    Step::Curve(23.603_6, 10.577_1, 24.110_9, 12.953_7, 23.817_6, 15.300_1),
    Step::Curve(23.524_3, 17.646_5, 22.447_6, 19.825_2, 20.761_8, 21.483_4),
    Step::Curve(19.076, 23.141_6, 16.879_9, 24.182_2, 14.529, 24.436_7),
    Step::Curve(12.178_1, 24.691_3, 9.810_09, 24.144_8, 7.808_45, 22.885_9),
    Step::Curve(5.806_81, 21.627, 4.288_72, 19.729_2, 3.5, 17.5),
    // The hands.
    Step::Move(14.0, 8.166_67),
    Step::Line(14.0, 14.0),
    Step::Line(17.5, 16.333_3),
];

/// How many straight pieces each curve is drawn as. Sixteen keeps the largest
/// gap between a piece and its curve far under a tenth of a pixel at any
/// scale a display is drawn at.
const PIECES_A_CURVE: u16 = 16;

/// The path as straight segments, in pixels of a square `side` across.
fn segments(side: f32) -> Vec<((f32, f32), (f32, f32))> {
    let at = |x: f32, y: f32| (x * side / ITS_SQUARE, y * side / ITS_SQUARE);
    let mut pen = (0.0, 0.0);
    let mut out = Vec::new();
    for step in THE_PATH {
        match step {
            Step::Move(x, y) => pen = at(x, y),
            Step::Line(x, y) => {
                let to = at(x, y);
                out.push((pen, to));
                pen = to;
            }
            Step::Curve(ax, ay, bx, by, x, y) => {
                let (a, b, to) = (at(ax, ay), at(bx, by), at(x, y));
                let from = pen;
                for piece in 1..=PIECES_A_CURVE {
                    let t = f32::from(piece) / f32::from(PIECES_A_CURVE);
                    let u = 1.0 - t;
                    let along = |p: f32, q: f32, r: f32, s: f32| {
                        u * u * u * p + 3.0 * u * u * t * q + 3.0 * u * t * t * r + t * t * t * s
                    };
                    let next = (along(from.0, a.0, b.0, to.0), along(from.1, a.1, b.1, to.1));
                    out.push((pen, next));
                    pen = next;
                }
            }
        }
    }
    out
}

/// How far `p` is from the segment `a`–`b`.
fn from_a_segment(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let length = dx * dx + dy * dy;
    let t = if length > 0.0 {
        (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / length).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (x, y) = (a.0 + t * dx - p.0, a.1 + t * dy - p.1);
    (x * x + y * y).sqrt()
}

/// The drawing in a square `side` pixels across, `ink` over `ground`, row by
/// row from the top — opaque, as every inked box in this shell is.
///
/// Empty for a square with no area.
#[must_use]
pub(crate) fn the_history_mark(side: i32, ground: [u8; 3], ink: [u8; 3]) -> Vec<[u8; 3]> {
    let Ok(across) = usize::try_from(side) else {
        return Vec::new();
    };
    #[expect(
        clippy::cast_precision_loss,
        reason = "a glyph's side in pixels is a few dozen, well inside an f32"
    )]
    let side = side as f32;
    let pieces = segments(side);
    let half = ITS_STROKE * side / ITS_SQUARE / 2.0;
    let mut pixels = Vec::with_capacity(across * across);
    for row in 0..across {
        for column in 0..across {
            #[expect(
                clippy::cast_precision_loss,
                reason = "a pixel's row and column inside a glyph are a few dozen"
            )]
            let p = (column as f32 + 0.5, row as f32 + 0.5);
            let nearest = pieces
                .iter()
                .map(|(a, b)| from_a_segment(p, *a, *b))
                .fold(f32::INFINITY, f32::min);
            let covered = (half + 0.5 - nearest).clamp(0.0, 1.0);
            pixels.push(mixed(ground, ink, covered));
        }
    }
    pixels
}

/// `ink` over `ground`, `covered` of the way.
fn mixed(ground: [u8; 3], ink: [u8; 3], covered: f32) -> [u8; 3] {
    let one = |g: u8, i: u8| {
        let value = f32::from(g) + (f32::from(i) - f32::from(g)) * covered;
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a value between two bytes, rounded and clamped, is a byte"
        )]
        let byte = value.round().clamp(0.0, 255.0) as u8;
        byte
    };
    [
        one(ground[0], ink[0]),
        one(ground[1], ink[1]),
        one(ground[2], ink[2]),
    ]
}

#[cfg(test)]
#[path = "dock_history_mark_tests.rs"]
mod tests;
