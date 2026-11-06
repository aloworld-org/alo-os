//! What the double-headed arrows have to be, rather than what they look like.

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
            the_cursor_over(one),
            the_cursor_over(other),
            "{one:?} and {other:?} are opposite ends of one arrow and do not share it"
        );
    }
    for (one, other) in [
        (FrameEdge::Left, FrameEdge::Top),
        (FrameEdge::Left, FrameEdge::TopLeft),
        (FrameEdge::Top, FrameEdge::TopRight),
        (FrameEdge::TopLeft, FrameEdge::TopRight),
    ] {
        assert_ne!(
            the_cursor_over(one),
            the_cursor_over(other),
            "{one:?} and {other:?} are different directions and share one arrow"
        );
    }
}

/// **The hotspot is the middle, not the tip** — ADR 0071's own words — at every
/// size the pointer is drawn.
#[test]
fn the_hotspot_is_the_middle_rather_than_the_tip() {
    for edge in FrameEdge::ALL {
        for size in [24, 48] {
            let picture = alo_cursors::draw(
                the_cursor_over(edge),
                alo_cursors::Silhouette::default(),
                size,
            );
            assert_eq!(
                picture.map(|drawn| drawn.hotspot),
                Ok((size / 2, size / 2)),
                "{edge:?} at {size}"
            );
        }
    }
}

/// **Every edge's arrow is a double arrow**: none of them fell through to the
/// plain arrow, which over an edge would say *this moves* rather than *this
/// resizes*.
#[test]
fn no_edge_shows_the_plain_arrow() {
    for edge in FrameEdge::ALL {
        assert_ne!(
            alo_cursors::drawing_for(the_cursor_over(edge)),
            alo_cursors::Drawing::Arrow,
            "{edge:?}"
        );
    }
}
