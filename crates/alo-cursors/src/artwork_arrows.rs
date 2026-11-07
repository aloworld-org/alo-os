//! **The arrows that say which way something moves**: one double-headed arrow
//! turned to each direction, and the four-way arrow.
//!
//! One arrow turned, rather than four drawn, so the four directions cannot
//! differ in weight: a resize arrow heavier at one end would tell somebody the
//! edge moves only one way. The hotspot is the middle, as ADR 0071 says, because
//! an arrow pointing both ways has no tip to put it at.

use crate::artwork::{Artwork, Part, polygon};

/// The middle of the box, which is every one of these arrows' hotspot.
const MIDDLE: f32 = 16.0;
/// From the middle to the point of a head, in units.
const REACH: f32 = 11.0;
/// How long a head is, from its point to its base.
const HEAD: f32 = 5.0;
/// Half a head's width at its base.
const HEAD_HALF: f32 = 4.0;
/// Half the shaft's width.
const SHAFT_HALF: f32 = 1.25;

/// A double-headed arrow, left and right, turned by `degrees` clockwise.
pub(crate) fn both_ways(degrees: f32) -> Artwork {
    let base = REACH - HEAD;
    let across = [
        (-REACH, 0.0),
        (-base, -HEAD_HALF),
        (-base, -SHAFT_HALF),
        (base, -SHAFT_HALF),
        (base, -HEAD_HALF),
        (REACH, 0.0),
        (base, HEAD_HALF),
        (base, SHAFT_HALF),
        (-base, SHAFT_HALF),
        (-base, HEAD_HALF),
    ];
    let turned: Vec<_> = across.iter().map(|&at| placed(turn(at, degrees))).collect();
    Artwork {
        parts: vec![Part::filled(polygon(&turned))],
        hotspot: (MIDDLE, MIDDLE),
    }
}

/// Four heads on a cross: up, right, down and left.
pub(crate) fn four_ways() -> Artwork {
    let base = REACH - HEAD;
    // One arm, from the corner before it to just before the corner after it;
    // the next arm is this one turned a quarter, so the four join exactly.
    let arm = [
        (-SHAFT_HALF, -SHAFT_HALF),
        (-SHAFT_HALF, -base),
        (-HEAD_HALF, -base),
        (0.0, -REACH),
        (HEAD_HALF, -base),
        (SHAFT_HALF, -base),
    ];
    let points: Vec<_> = [0.0, 90.0, 180.0, 270.0]
        .iter()
        .flat_map(|&degrees| arm.iter().map(move |&at| placed(turn(at, degrees))))
        .collect();
    Artwork {
        parts: vec![Part::filled(polygon(&points))],
        hotspot: (MIDDLE, MIDDLE),
    }
}

/// `at` turned clockwise on the screen by `degrees` about the middle.
fn turn((x, y): (f32, f32), degrees: f32) -> (f32, f32) {
    let (sin, cos) = degrees.to_radians().sin_cos();
    (x * cos - y * sin, x * sin + y * cos)
}

/// A point relative to the middle, put on the box.
fn placed((x, y): (f32, f32)) -> (f32, f32) {
    (MIDDLE + x, MIDDLE + y)
}
