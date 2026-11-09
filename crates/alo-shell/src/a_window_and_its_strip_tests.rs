//! What a window takes with its strip: its size on the plane at each display
//! scale, the two ways between a window and what it takes, and the refusals.

use super::ItsStrip;
use smithay::utils::{Logical, Point, Rectangle, Size};

/// The strip on a display at this scale.
fn at(scale: u32) -> ItsStrip {
    ItsStrip::at(scale)
}

/// **54 tall at 100 %**, the design’s band (`11:31`, since 2026-11-09); through the display’s scale
/// and rounded up, never down; and **the same at every zoom**, because since
/// the owner’s *"Title bars shrink with zoom"* of 2026-11-08 the strip zooms
/// with its window, so on the plane it does not change.
#[test]
fn the_strip_is_the_designs_height_through_the_displays_scale() {
    let window = Rectangle::<i32, Logical>::new((500, 400).into(), (600, 300).into());
    for (scale, tall) in [(100, 54), (150, 81), (200, 108), (125, 68), (133, 72)] {
        let taken = at(scale).round(window);
        assert_eq!(window.loc.y - taken.loc.y, tall, "at {scale} %");
        assert_eq!(
            taken.loc.y + taken.size.h,
            window.loc.y + window.size.h,
            "the bottom moved at {scale} %"
        );
    }
}

/// **A scale of nought is the ordinary one**, as the edge reads it, not a
/// strip of nothing.
#[test]
fn a_scale_of_nought_is_the_ordinary_scale() {
    assert_eq!(at(0), at(100));
}

/// **A scale past any display does not wrap**: it saturates.
#[test]
fn a_scale_past_any_display_does_not_wrap() {
    let window = Rectangle::<i32, Logical>::new((0, 0).into(), (10, 10).into());
    assert!(
        at(u32::MAX).round(window).size.h > 0,
        "a strip wrapped negative"
    );
}

/// **A wide window takes its own width**: its strip is as wide as it is.
#[test]
fn a_wide_window_takes_its_own_width() {
    let window = Rectangle::<i32, Logical>::new((10, 100).into(), (600, 300).into());
    let taken = at(100).round(window);
    assert_eq!((taken.loc.x, taken.size.w), (10, 600));
    assert_eq!((taken.loc.y, taken.size.h), (46, 354));
}

/// **A narrow window takes its strip's width**, centred over it with the odd
/// unit on the right, as `window_edge::edge_of` draws it.
#[test]
fn a_narrow_window_takes_the_narrowest_strips_width_centred_over_it() {
    let narrow = Rectangle::<i32, Logical>::new((200, 100).into(), (15, 15).into());
    let taken = at(100).round(narrow);
    let drawn = crate::window_edge::edge_of(
        narrow,
        crate::window_edge::Decorations::TheShellDraws,
        true,
        Size::from((2000, 2000)),
    )
    .strip;
    assert_eq!(taken.size.w, crate::window_edge::THE_NARROWEST_EDGE_IS);
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
    for scale in [100, 125, 150, 200] {
        let strip = at(scale);
        for size in [(600, 300), (15, 15), (16, 40), (1, 1)] {
            let window = Rectangle::<i32, Logical>::new((-37, 91).into(), size.into());
            let taken = strip.round(window);
            assert_eq!(
                strip.the_window_in(taken.loc, window.size),
                window.loc,
                "{size:?} at {scale} %"
            );
            assert_eq!(strip.round_a_size(window.size), taken.size);
        }
    }
}

/// **Far out on the plane nothing wraps**: the arithmetic saturates.
#[test]
fn a_window_at_the_end_of_the_plane_does_not_wrap() {
    let window = Rectangle::<i32, Logical>::new((i32::MIN, i32::MIN).into(), (10, 10).into());
    let taken = at(100).round(window);
    assert_eq!(taken.loc, Point::from((i32::MIN, i32::MIN)));
}

/// **Rule 3 beside the active window keeps the strips level**: the new window's
/// strip starts where the active one's does, one gap to its right, and the two
/// windows' tops are level.
#[test]
fn beside_the_active_window_the_two_strips_are_level_and_a_gap_apart() {
    let strip = at(100);
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
