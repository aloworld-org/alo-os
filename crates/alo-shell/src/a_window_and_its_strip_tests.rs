//! What a window takes with its strip: its size on the plane at each zoom and
//! scale, the two ways between a window and what it takes, and the refusals.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::ItsStrip;
use smithay::utils::{Logical, Point, Rectangle, Size};

/// The strip at this zoom and scale, which every case here can have.
fn at(zoom: f64, scale: u32) -> ItsStrip {
    ItsStrip::at(zoom, scale).expect("a zoom above nought")
}

/// **32 tall at life size**, the design's strip; twice that zoomed out to half
/// or on a display at 200 %; and rounded up between, never down.
#[test]
fn the_strip_is_the_designs_height_through_the_scale_and_the_zoom() {
    let window = Rectangle::<i32, Logical>::new((500, 400).into(), (600, 300).into());
    for (zoom, scale, tall) in [
        (1.0, 100, 32),
        (0.5, 100, 64),
        (1.0, 200, 64),
        (2.0, 100, 16),
        (0.5, 150, 96),
        // 32 ÷ 0.3 is 106.7: a strip is given 107, never 106.
        (0.3, 100, 107),
    ] {
        let taken = at(zoom, scale).round(window);
        assert_eq!(
            window.loc.y - taken.loc.y,
            tall,
            "at zoom {zoom} and {scale} %"
        );
        assert_eq!(
            taken.loc.y + taken.size.h,
            window.loc.y + window.size.h,
            "the bottom moved at zoom {zoom} and {scale} %"
        );
    }
}

/// **A scale of nought is the ordinary one**, as the edge reads it, not a
/// strip of nothing.
#[test]
fn a_scale_of_nought_is_the_ordinary_scale() {
    assert_eq!(at(1.0, 0), at(1.0, 100));
}

/// **A zoom that cannot be divided by is refused**, not reasoned about.
#[test]
fn a_zoom_that_cannot_be_divided_by_is_refused() {
    for zoom in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(ItsStrip::at(zoom, 100), None, "a zoom of {zoom}");
    }
    // So small that the strip would not fit an i32 on the plane.
    assert_eq!(ItsStrip::at(1e-12, 100), None, "a strip past an i32");
}

/// **A wide window takes its own width**: its strip is as wide as it is.
#[test]
fn a_wide_window_takes_its_own_width() {
    let window = Rectangle::<i32, Logical>::new((10, 100).into(), (600, 300).into());
    let taken = at(1.0, 100).round(window);
    assert_eq!((taken.loc.x, taken.size.w), (10, 600));
    assert_eq!((taken.loc.y, taken.size.h), (68, 332));
}

/// **A narrow window takes its strip's width**, centred over it with the odd
/// unit on the right, as `window_edge::edge_of` draws it.
#[test]
fn a_narrow_window_takes_the_narrowest_strips_width_centred_over_it() {
    let narrow = Rectangle::<i32, Logical>::new((200, 100).into(), (15, 15).into());
    let taken = at(1.0, 100).round(narrow);
    let drawn = crate::window_edge::edge_of(
        narrow,
        crate::window_edge::Decorations::TheShellDraws,
        true,
        Size::from((2000, 2000)),
    )
    .strip
    .expect("a revealed edge has a strip");
    assert_eq!(taken.size.w, crate::window_edge::THE_COMPACT_EDGE_IS);
    assert_eq!(
        (taken.loc.x, taken.size.w),
        (drawn.loc.x, drawn.size.w),
        "taken across is not where the strip is drawn"
    );
    assert_eq!(
        taken.loc.y, drawn.loc.y,
        "taken is not from the strip's top"
    );
}

/// **The two ways meet**: a window put in what it takes is back where it was,
/// wide or narrow, odd or even, at any zoom.
#[test]
fn a_window_put_back_in_what_it_takes_is_where_it_was() {
    for zoom in [1.0, 0.5, 0.3, 2.0] {
        let strip = at(zoom, 125);
        for size in [(600, 300), (15, 15), (16, 40), (1, 1)] {
            let window = Rectangle::<i32, Logical>::new((-37, 91).into(), size.into());
            let taken = strip.round(window);
            assert_eq!(
                strip.the_window_in(taken.loc, window.size),
                window.loc,
                "{size:?} at zoom {zoom}"
            );
            assert_eq!(strip.round_a_size(window.size), taken.size);
        }
    }
}

/// **Far out on the plane nothing wraps**: the arithmetic saturates.
#[test]
fn a_window_at_the_end_of_the_plane_does_not_wrap() {
    let window = Rectangle::<i32, Logical>::new((i32::MIN, i32::MIN).into(), (10, 10).into());
    let taken = at(1.0, 100).round(window);
    assert_eq!(taken.loc, Point::from((i32::MIN, i32::MIN)));
}

/// **Rule 3 beside the active window keeps the strips level**: the new window's
/// strip starts where the active one's does, one gap to its right, and the two
/// windows' tops are level.
#[test]
fn beside_the_active_window_the_two_strips_are_level_and_a_gap_apart() {
    let strip = at(1.0, 100);
    let terminal = Rectangle::<i32, Logical>::new((48, 116).into(), (560, 476).into());
    let view = Rectangle::new((0, 0).into(), (1440, 960).into());
    let at = strip.where_a_window_opens(
        view,
        &[terminal],
        Some(terminal),
        Size::from((560, 476)),
        alo_strings::Direction::LeftToRight,
        &[],
    );
    assert_eq!(
        (at.x, at.y),
        (48 + 560 + crate::where_a_window_opens::A_SMALL_GAP, 116)
    );
}
