//! The folded light behind the sign-in screen: five curves, each filled with
//! its own gradient.
//!
//! Read from `docs/design/figma-snapshot/176-10686-wallpaper.svg`, which was
//! exported from the live file on 2026-10-12. Every number below appears
//! there. The screen was drawn on flat `#FAF7F2` until this existed, and
//! `crate::sign_in_surface`'s header said so in as many words: *not drawn: the
//! wallpaper … it needs a path rasteriser or a render baked at build time,
//! which is a decision rather than a line of code.* This is that decision made
//! the first way.
//!
//! # Why the artwork is transcribed and not parsed
//!
//! The shell ships no SVG parser and is not getting one for five shapes. The
//! paths and gradients are constants here with the file they came from named
//! beside them, which is the arrangement `crate::sign_in_layout` already uses
//! for the geometry it read from `176-10686.xml`: a test can hold a number
//! against the committed export without the shell carrying a parser for the
//! format that export happens to be in.
//!
//! # Cover, not stretch
//!
//! The design is 1440 × 960 and a screen is whatever the person has. **The
//! paths already run past their own viewBox** — out to x = 1600 and y = 1100,
//! and in from x = -180 — so they are drawn to be clipped. The fit is
//! therefore *cover*: scale by whichever of width and height needs the most,
//! centre, and let the rest fall off the edge. Stretching to fit would flatten
//! the folds on a wide screen into something the design does not contain.
//!
//! # What it does not do
//!
//! No personal content reaches this, which is the owner's own direction:
//! *keep personal application content out of the wallpaper.* Nothing here is
//! given a window, a name or a file — it takes a size and returns pixels.

/// One step of an outline, in the design's own coordinates.
///
/// Only the four the export uses. `H` and `V` are already lines by the time
/// they are written down here, because a horizontal line is a line.
#[derive(Debug, Clone, Copy)]
enum Step {
    /// Begin the outline here.
    Begin(f64, f64),
    /// A cubic: two control points, then the end.
    Curve(f64, f64, f64, f64, f64, f64),
    /// A straight run to here.
    Line(f64, f64),
}

/// A linear gradient in the design's coordinates, as `userSpaceOnUse` means.
#[derive(Debug, Clone, Copy)]
struct Gradient {
    /// Where the gradient begins.
    from: (f64, f64),
    /// Where it ends. Equal `x` with differing `y` is a vertical gradient,
    /// which two of the five are.
    to: (f64, f64),
    /// Offset and colour, in order, the first at 0 and the last at 1.
    stops: &'static [(f64, [u8; 3])],
}

/// One fold: an outline and the gradient that fills it.
#[derive(Debug, Clone, Copy)]
struct Fold {
    /// Its outline, closed implicitly: the last point joins the first.
    outline: &'static [Step],
    /// What fills it.
    gradient: Gradient,
}

/// The size the artwork is drawn at.
const THE_DESIGN_IS: (f64, f64) = (1440.0, 960.0);

/// How many straight pieces a cubic becomes.
///
/// The longest curve here runs about a thousand units, so 128 pieces leaves
/// segments of roughly eight. A chord that long across a curve of this radius
/// departs from it by under a fiftieth of a unit — below what one pixel can
/// show, which is why this is a constant and not an adaptive subdivision.
const PIECES_PER_CURVE: usize = 128;

/// How many sub-scanlines a pixel row is sampled with.
///
/// Four, vertically, with exact coverage horizontally. A fold's edge is a
/// shallow curve across most of the screen, where the horizontal term carries
/// the quality; four sub-rows is enough for the steep parts.
const SUB_ROWS: usize = 4;

/// The five folds, back to front, exactly as the export draws them.
///
/// 1 to 3 are the folds down the right-hand side, 4 and 5 the pair in the
/// bottom-left corner. Drawn in this order because they overlap, and the
/// export's order is what makes the lighter ones read as being in front.
const THE_FOLDS: [Fold; 5] = [
    // `Vector`: M1270 -180 C955 18 980 231 1160 421 C1350 623 1310 787
    //           1016 1060 H1600 V-180 H1270 Z
    Fold {
        outline: &[
            Step::Begin(1270.0, -180.0),
            Step::Curve(955.0, 18.0, 980.0, 231.0, 1160.0, 421.0),
            Step::Curve(1350.0, 623.0, 1310.0, 787.0, 1016.0, 1060.0),
            Step::Line(1600.0, 1060.0),
            Step::Line(1600.0, -180.0),
            Step::Line(1270.0, -180.0),
        ],
        gradient: Gradient {
            from: (1016.0, -180.0),
            to: (1775.56, 52.5228),
            stops: &[
                (0.0, [0x8F, 0xA5, 0xB6]),
                (0.38, [0xD5, 0xDF, 0xE3]),
                (0.72, [0xF7, 0xF5, 0xEF]),
                (1.0, [0xB7, 0xC6, 0xCE]),
            ],
        },
    },
    // `Vector_2`: M1422 -180 C1090 25 1095 220 1260 415 C1431 616 1410 844
    //             1140 1060 H1600 V-180 H1422 Z
    Fold {
        outline: &[
            Step::Begin(1422.0, -180.0),
            Step::Curve(1090.0, 25.0, 1095.0, 220.0, 1260.0, 415.0),
            Step::Curve(1431.0, 616.0, 1410.0, 844.0, 1140.0, 1060.0),
            Step::Line(1600.0, 1060.0),
            Step::Line(1600.0, -180.0),
            Step::Line(1422.0, -180.0),
        ],
        gradient: Gradient {
            from: (1140.0, -180.0),
            to: (1736.07, -47.3265),
            stops: &[
                (0.0, [0x53, 0x6F, 0x83]),
                (0.32, [0xC5, 0xD2, 0xD9]),
                (0.65, [0xFD, 0xFB, 0xF6]),
                (1.0, [0xD5, 0xDD, 0xE0]),
            ],
        },
    },
    // `Vector_3`: M1548 -120 C1278 62 1250 242 1370 406 C1503 587 1501 806
    //             1356 990 L1600 1000 V-120 H1548 Z
    Fold {
        outline: &[
            Step::Begin(1548.0, -120.0),
            Step::Curve(1278.0, 62.0, 1250.0, 242.0, 1370.0, 406.0),
            Step::Curve(1503.0, 587.0, 1501.0, 806.0, 1356.0, 990.0),
            Step::Line(1600.0, 1000.0),
            Step::Line(1600.0, -120.0),
            Step::Line(1548.0, -120.0),
        ],
        gradient: Gradient {
            from: (1302.32, -120.0),
            to: (1302.32, 1000.0),
            stops: &[
                (0.0, [0xFF, 0xFF, 0xFF]),
                (0.55, [0xE8, 0xE7, 0xDF]),
                (1.0, [0xB7, 0xBE, 0xC0]),
            ],
        },
    },
    // `Vector_4`: M-180 765 C75 615 182 702 302 841 C405 960 584 975 812 1080
    //             L-180 1100 V765 Z
    Fold {
        outline: &[
            Step::Begin(-180.0, 765.0),
            Step::Curve(75.0, 615.0, 182.0, 702.0, 302.0, 841.0),
            Step::Curve(405.0, 960.0, 584.0, 975.0, 812.0, 1080.0),
            Step::Line(-180.0, 1100.0),
            Step::Line(-180.0, 765.0),
        ],
        gradient: Gradient {
            from: (-180.0, 686.465),
            to: (113.735, 1391.09),
            stops: &[
                (0.0, [0xE7, 0xE9, 0xE4]),
                (0.45, [0xFA, 0xF7, 0xF2]),
                (1.0, [0x9F, 0xB1, 0xBC]),
            ],
        },
    },
    // `Vector_5`: M-180 881.998 C54 720.998 161 785.998 273 896.998
    //             C382 1004 576 1052 715 1080 L-180 1100 V881.998 Z
    Fold {
        outline: &[
            Step::Begin(-180.0, 881.998),
            Step::Curve(54.0, 720.998, 161.0, 785.998, 273.0, 896.998),
            Step::Curve(382.0, 1004.0, 576.0, 1052.0, 715.0, 1080.0),
            Step::Line(-180.0, 1100.0),
            Step::Line(-180.0, 881.998),
        ],
        gradient: Gradient {
            from: (-180.0, 785.359),
            to: (-180.0, 1100.0),
            stops: &[
                (0.0, [0xFF, 0xFF, 0xFF]),
                (0.55, [0xE8, 0xE7, 0xDF]),
                (1.0, [0xB7, 0xBE, 0xC0]),
            ],
        },
    },
];

/// One straight piece of an outline, in screen pixels.
#[derive(Debug, Clone, Copy)]
struct Edge {
    /// The end with the smaller `y`.
    top: (f64, f64),
    /// The end with the larger `y`.
    bottom: (f64, f64),
    /// Which way the outline was going: +1 downwards, -1 upwards. The winding
    /// rule needs the direction the edge was written in, which sorting the two
    /// ends by `y` would otherwise throw away.
    going: i32,
}

/// Draw the wallpaper for a screen of this many pixels.
///
/// The ground with the five folds on it, one `[r, g, b]` per pixel, row by
/// row. The caller owns what it does with them; this does not know what a
/// frame is.
///
/// An empty vector for a screen with no area, which is the one case with
/// nothing to draw rather than a refusal worth reporting.
#[must_use]
pub(crate) fn the_wallpaper(width: i32, height: i32, ground: [u8; 3]) -> Vec<[u8; 3]> {
    if width <= 0 || height <= 0 {
        return Vec::new();
    }
    let (w, h) = (width as usize, height as usize);
    let mut pixels = vec![ground; w * h];

    // **Cover**: whichever axis needs the larger scale sets it, and the
    // artwork is centred, so the overflow falls off both sides equally.
    let scale = (f64::from(width) / THE_DESIGN_IS.0).max(f64::from(height) / THE_DESIGN_IS.1);
    let left = (f64::from(width) - THE_DESIGN_IS.0 * scale) / 2.0;
    let top = (f64::from(height) - THE_DESIGN_IS.1 * scale) / 2.0;
    let onto = |(x, y): (f64, f64)| (x * scale + left, y * scale + top);

    for fold in THE_FOLDS {
        let edges = edges_of(fold.outline, &onto);
        fill(&mut pixels, w, h, &edges, fold.gradient, &onto);
    }
    pixels
}

/// An outline, flattened and placed, as the edges a scanline crosses.
///
/// Horizontal edges are dropped: a scanline cannot cross one, and keeping them
/// would mean a division by zero at the crossing.
fn edges_of(outline: &[Step], onto: &impl Fn((f64, f64)) -> (f64, f64)) -> Vec<Edge> {
    let mut points: Vec<(f64, f64)> = Vec::new();
    let mut at = (0.0, 0.0);
    for step in outline {
        match *step {
            Step::Begin(x, y) => {
                at = (x, y);
                points.push(onto(at));
            }
            Step::Line(x, y) => {
                at = (x, y);
                points.push(onto(at));
            }
            Step::Curve(c1x, c1y, c2x, c2y, x, y) => {
                let end = (x, y);
                for piece in 1..=PIECES_PER_CURVE {
                    #[expect(
                        clippy::cast_precision_loss,
                        reason = "the count is 128 and every value of it is exact in an f64"
                    )]
                    let t = piece as f64 / PIECES_PER_CURVE as f64;
                    points.push(onto(cubic(at, (c1x, c1y), (c2x, c2y), end, t)));
                }
                at = end;
            }
        }
    }

    let mut edges = Vec::with_capacity(points.len());
    for (a, b) in points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
    {
        if (a.1 - b.1).abs() < f64::EPSILON {
            continue;
        }
        edges.push(if a.1 < b.1 {
            Edge {
                top: *a,
                bottom: *b,
                going: 1,
            }
        } else {
            Edge {
                top: *b,
                bottom: *a,
                going: -1,
            }
        });
    }
    edges
}

/// One point on a cubic, by de Casteljau's weights written out.
fn cubic(p0: (f64, f64), p1: (f64, f64), p2: (f64, f64), p3: (f64, f64), t: f64) -> (f64, f64) {
    let u = 1.0 - t;
    let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    (
        a * p0.0 + b * p1.0 + c * p2.0 + d * p3.0,
        a * p0.1 + b * p1.1 + c * p2.1 + d * p3.1,
    )
}

/// Fill one closed outline, sampling each pixel row at [`SUB_ROWS`] heights.
///
/// **Coverage, not a hard edge.** Each sub-scanline contributes a fraction of
/// a row, and a span's two ends contribute the fraction of the pixel they
/// actually cover, so a curve crossing a pixel diagonally comes out part-lit.
/// The same judgement `crate::sign_in_surface` makes about the entry's
/// corners, by a different method because this shape has no distance field.
fn fill(
    pixels: &mut [[u8; 3]],
    w: usize,
    h: usize,
    edges: &[Edge],
    gradient: Gradient,
    onto: &impl Fn((f64, f64)) -> (f64, f64),
) {
    if edges.is_empty() {
        return;
    }
    let from = onto(gradient.from);
    let to = onto(gradient.to);
    let along = (to.0 - from.0, to.1 - from.1);
    let length = along.0.mul_add(along.0, along.1 * along.1);

    let mut coverage = vec![0.0_f64; w];
    let mut crossings: Vec<(f64, i32)> = Vec::with_capacity(edges.len());
    let share = 1.0 / SUB_ROWS as f64;

    for (y, row) in pixels.chunks_mut(w).enumerate().take(h) {
        coverage.iter_mut().for_each(|at| *at = 0.0);
        let mut any = false;

        for sub in 0..SUB_ROWS {
            #[expect(
                clippy::cast_precision_loss,
                reason = "a screen is millions of pixels, not quadrillions, and every row \
                          number is exact in an f64"
            )]
            let line = y as f64 + (sub as f64 + 0.5) * share;
            crossings.clear();
            for edge in edges {
                // Half-open in `y`, so a vertex shared by two edges is counted
                // once rather than twice or not at all.
                if line < edge.top.1 || line >= edge.bottom.1 {
                    continue;
                }
                let how_far = (line - edge.top.1) / (edge.bottom.1 - edge.top.1);
                crossings.push((
                    edge.top.0 + (edge.bottom.0 - edge.top.0) * how_far,
                    edge.going,
                ));
            }
            if crossings.len() < 2 {
                continue;
            }
            crossings.sort_by(|a, b| a.0.total_cmp(&b.0));

            // Nonzero winding: inside wherever the running total is not zero.
            let mut winding = 0;
            let mut began = 0.0;
            for (x, going) in &crossings {
                let was = winding;
                winding += going;
                if was == 0 && winding != 0 {
                    began = *x;
                } else if was != 0 && winding == 0 {
                    any |= add_span(&mut coverage, began, *x, share, w);
                }
            }
        }

        if !any {
            continue;
        }
        for (x, (under, how_much)) in row.iter_mut().zip(coverage.iter()).enumerate() {
            if *how_much <= 0.0 {
                continue;
            }
            #[expect(
                clippy::cast_precision_loss,
                reason = "as above: a pixel's own coordinate is exact in an f64"
            )]
            let point = (x as f64 + 0.5, y as f64 + 0.5);
            let t = if length <= f64::EPSILON {
                0.0
            } else {
                ((point.0 - from.0) * along.0 + (point.1 - from.1) * along.1) / length
            };
            *under = mixed(*under, colour_at(gradient.stops, t), how_much.min(1.0));
        }
    }
}

/// Add a horizontal span's coverage, with its two ends part-covering a pixel.
///
/// Answers whether it covered anything, so a row no fold reaches costs one
/// boolean rather than a walk along its width.
fn add_span(coverage: &mut [f64], from: f64, to: f64, share: f64, w: usize) -> bool {
    #[expect(
        clippy::cast_precision_loss,
        reason = "a screen's width is thousands of pixels and exact in an f64"
    )]
    let right_edge = w as f64;
    let (from, to) = (from.max(0.0), to.min(right_edge));
    if to <= from {
        return false;
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "both ends were clamped into 0..=w just above"
    )]
    let (first, last) = (from.floor() as usize, (to.ceil() as usize).min(w));
    let mut covered = false;
    for (x, at) in coverage.iter_mut().enumerate().take(last).skip(first) {
        #[expect(
            clippy::cast_precision_loss,
            reason = "as above: a pixel's own coordinate is exact in an f64"
        )]
        let (left, right) = (x as f64, x as f64 + 1.0);
        let piece = to.min(right) - from.max(left);
        if piece > 0.0 {
            *at += piece * share;
            covered = true;
        }
    }
    covered
}

/// The colour at this distance along a gradient.
///
/// Clamped at both ends, which is what `userSpaceOnUse` with no `spreadMethod`
/// means: a point before the first stop is the first stop's colour.
fn colour_at(stops: &[(f64, [u8; 3])], t: f64) -> [u8; 3] {
    let Some((first_at, first)) = stops.first().copied() else {
        return [0, 0, 0];
    };
    if t <= first_at {
        return first;
    }
    for (pair, next) in stops.iter().zip(stops.iter().skip(1)) {
        let ((at, from), (next_at, to)) = (*pair, *next);
        if t <= next_at {
            let span = next_at - at;
            let how_far = if span <= f64::EPSILON {
                0.0
            } else {
                (t - at) / span
            };
            return mixed(from, to, how_far);
        }
    }
    stops.last().map_or(first, |(_, last)| *last)
}

/// `over` laid on `under` at this much coverage.
fn mixed(under: [u8; 3], over: [u8; 3], how_much: f64) -> [u8; 3] {
    let how_much = how_much.clamp(0.0, 1.0);
    let one = |u: u8, o: u8| {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "both ends are bytes and the mix is clamped between them"
        )]
        {
            (f64::from(u) + (f64::from(o) - f64::from(u)) * how_much).round() as u8
        }
    };
    [
        one(under[0], over[0]),
        one(under[1], over[1]),
        one(under[2], over[2]),
    ]
}

#[cfg(test)]
#[path = "sign_in_wallpaper_tests.rs"]
mod tests;
