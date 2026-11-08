//! That a window and the pointer reach an edge's units through the display's
//! scale: the identity at 100, exactly half at 200, and a client window's edge
//! laid out from them meeting its window when it is drawn back.

use smithay::utils::{Logical, Point, Rectangle, Size};

use super::{a_point_in_the_edges_units, a_window_in_the_edges_units};
use crate::window_edge::{Decorations, edge_of};

/// **At 100 nothing moves**: the window and the pointer are their own units.
#[test]
fn at_one_hundred_the_conversion_is_the_identity() {
    let window = Rectangle::<i32, Logical>::new((137, 91).into(), (803, 611).into());
    assert_eq!(a_window_in_the_edges_units(window, 100), window);
    let at = Point::<i32, Logical>::from((999, 7));
    assert_eq!(a_point_in_the_edges_units(at, 100), at);
}

/// **At 200 both are exactly half**, and a nought scale is read as one rather
/// than divided by.
#[test]
fn at_two_hundred_the_window_and_the_pointer_are_halved() {
    let window = Rectangle::<i32, Logical>::new((400, 248).into(), (2080, 1600).into());
    assert_eq!(
        a_window_in_the_edges_units(window, 200),
        Rectangle::new((200, 124).into(), (1040, 800).into())
    );
    assert_eq!(
        a_point_in_the_edges_units(Point::from((1001, 51)), 200),
        Point::from((500, 25)),
        "a pixel belongs to the unit it is drawn from, rounding down"
    );
    assert_eq!(a_window_in_the_edges_units(window, 0), window);
}

/// **A client window's edge, laid out from its halved rectangle and drawn back
/// at 200, meets the window** — where the draw used to put it twice the
/// window's distance from the corner.
#[test]
fn at_two_hundred_a_client_windows_edge_meets_its_window_when_drawn() {
    let window = Rectangle::<i32, Logical>::new((400, 248).into(), (2080, 1600).into());
    let edge = edge_of(
        a_window_in_the_edges_units(window, 200),
        Decorations::TheShellDraws,
        true,
        Size::from((10_000, 10_000)),
    );
    assert_eq!(
        (edge.region.loc.y + edge.region.size.h) * 2,
        window.loc.y,
        "drawn back at 200, the edge's foot is not the window's top"
    );
}

/// **The edge's scale is the display's times the zoom** — the owner's
/// *"Title bars shrink with zoom"* of 2026-11-08 — at three zooms and three
/// scales, and refuses nothing it cannot multiply by.
#[test]
fn an_edge_is_scaled_by_the_display_and_the_zoom() {
    use super::a_scale_through_the_zoom;
    for (display, zoom, scaled) in [
        (100, 0.5, 50),
        (100, 1.0, 100),
        (100, 2.0, 200),
        (150, 0.5, 75),
        (150, 1.0, 150),
        (150, 2.0, 300),
        (200, 0.5, 100),
        (200, 1.0, 200),
        (200, 2.0, 400),
    ] {
        assert_eq!(
            a_scale_through_the_zoom(display, zoom),
            scaled,
            "{display} % at {zoom}"
        );
    }
    // A zoom nothing can multiply by is life size; a display of nought is the
    // ordinary one; and a far zoom out is never a scale of nothing.
    for zoom in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(a_scale_through_the_zoom(150, zoom), 150, "a zoom of {zoom}");
    }
    assert_eq!(a_scale_through_the_zoom(0, 1.0), 100);
    assert_eq!(a_scale_through_the_zoom(100, 0.001), 1);
    assert_eq!(a_scale_through_the_zoom(u32::MAX, 10.0), u32::MAX);
}
