//! What the double-headed arrows have to be, rather than what they look like.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::*;

/// **Eight directions are answered by four shapes, and the pairs are the right
/// pairs.**
///
/// A double-headed arrow means the same thing at both ends, so opposite edges
/// share one. Asserted as *these two are the same and these two are not* rather
/// than by counting, because four distinct shapes paired wrongly would pass a
/// count and point a person the wrong way.
#[test]
fn opposite_edges_share_one_arrow_and_the_pairs_are_right() {
    for (one, other) in [
        (FrameEdge::Left, FrameEdge::Right),
        (FrameEdge::Top, FrameEdge::Bottom),
        (FrameEdge::TopLeft, FrameEdge::BottomRight),
        (FrameEdge::TopRight, FrameEdge::BottomLeft),
    ] {
        assert_eq!(
            the_cursor_over(one).as_ptr(),
            the_cursor_over(other).as_ptr(),
            "{one:?} and {other:?} are opposite ends of one arrow and do not share it"
        );
    }
    // And the four are actually four: an edge pair and a corner pair that
    // resolved to the same shape would say a corner resizes in one axis.
    for (one, other) in [
        (FrameEdge::Left, FrameEdge::Top),
        (FrameEdge::Left, FrameEdge::TopLeft),
        (FrameEdge::Top, FrameEdge::TopRight),
        (FrameEdge::TopLeft, FrameEdge::TopRight),
    ] {
        assert_ne!(
            the_cursor_over(one).as_ptr(),
            the_cursor_over(other).as_ptr(),
            "{one:?} and {other:?} are different directions and share one arrow"
        );
    }
}

/// **Every arrow points both ways**, which is the whole claim it makes.
///
/// A shape that was heavier at one end would tell somebody the edge moves in one
/// direction, which is the opposite of what a resize handle means. Checked as
/// symmetry about the centre rather than by eye: `ACROSS` and `DOWN` mirror
/// through their middle, and the two diagonals mirror through theirs when turned
/// half a turn.
#[test]
fn every_arrow_points_both_ways() {
    for edge in FrameEdge::ALL {
        let shape = the_cursor_over(edge);
        let rows: Vec<Vec<char>> = shape
            .iter()
            .map(|row| {
                let mut row: Vec<char> = row.chars().collect();
                // Rows are written without trailing spaces; a mask is rectangular.
                let width = shape.iter().map(|it| it.chars().count()).max().unwrap_or(0);
                row.resize(width, ' ');
                row
            })
            .collect();
        let turned: Vec<Vec<char>> = rows
            .iter()
            .rev()
            .map(|row| row.iter().rev().copied().collect())
            .collect();
        assert_eq!(
            rows, turned,
            "{edge:?}'s arrow is not the same turned half a turn, so it points one way"
        );
    }
}

/// **The hotspot is the middle, not the tip** — ADR 0071's own words, and the
/// difference from `crate::default_cursor`, whose arrow has its tip at (0, 0).
#[test]
fn the_hotspot_is_the_middle_rather_than_the_tip() {
    for edge in FrameEdge::ALL {
        let shape = the_cursor_over(edge);
        let (x, y) = the_hotspot_of(shape).expect("every arrow has rows");
        let width = shape
            .iter()
            .map(|row| i32::try_from(row.chars().count()).unwrap_or(0))
            .max()
            .expect("every arrow has rows");
        let height = i32::try_from(shape.len()).expect("an arrow is not that tall");
        assert_eq!((x, y), (width / 2, height / 2), "{edge:?}");
        assert!(
            x > 0 && y > 0,
            "{edge:?}'s hotspot is at an edge of its own shape, which is a tip"
        );
    }
}

/// **Nothing but outline, interior and nothing.**
///
/// The same three characters `crate::default_cursor` uses. A stray character
/// would be drawn as whichever branch the painter happened to take, which is a
/// cursor that looks different for a reason nobody wrote down.
#[test]
fn an_arrow_is_made_of_the_three_things_a_cursor_is_made_of() {
    for edge in FrameEdge::ALL {
        for row in the_cursor_over(edge) {
            for glyph in row.chars() {
                assert!(
                    matches!(glyph, 'B' | 'W' | ' '),
                    "{edge:?}'s arrow holds {glyph:?}, which is not outline, interior or nothing"
                );
            }
        }
    }
}
