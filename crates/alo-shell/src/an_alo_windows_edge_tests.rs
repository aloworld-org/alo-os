//! The edge over one of alo's own windows: Close alone, a title and an icon's
//! place, sitting on the window at every scale, and pressed where it is drawn.

use smithay::utils::{Logical, Physical, Point, Rectangle, Size};

use super::{OnItsEdge, the_edge_over, what_a_press_is_on};
use crate::window_edge::OnTheEdge;

/// A room no edge is clamped by.
fn anywhere() -> Size<i32, Logical> {
    Size::from((10_000, 10_000))
}

/// A window drawn where the terminal's design frame puts one.
fn a_window() -> Rectangle<i32, Physical> {
    Rectangle::new((200, 124).into(), (1040, 800).into())
}

/// **Close and nothing else, with a title and an icon**, and the edge's band
/// ending where the window begins.
#[test]
fn close_alone_with_a_title_and_an_icon_on_the_window() {
    let edge = the_edge_over(a_window(), anywhere(), 100);
    let controls: Vec<OnTheEdge> = edge.controls.iter().map(|control| control.does).collect();
    assert_eq!(controls, vec![OnTheEdge::Close]);
    assert!(edge.title.is_some());
    assert!(edge.icon.is_some());
    assert_eq!(edge.region.loc.y + edge.region.size.h, 124);
    assert_eq!((edge.region.loc.x, edge.region.size.w), (200, 1040));
}

/// **At 200 per cent it is laid out in the edge's own units**, half the
/// display's pixels, so drawn back at twice it meets the window.
#[test]
fn at_two_hundred_it_still_meets_the_window() {
    let edge = the_edge_over(a_window(), anywhere(), 200);
    assert_eq!(edge.region.loc.y + edge.region.size.h, 62);
    assert_eq!((edge.region.loc.x, edge.region.size.w), (100, 520));
}

/// **A press finds Close where it is drawn**, the rest of the edge is the
/// rest, and a press on the window itself is not the edge's.
#[test]
fn a_press_finds_close_where_it_is_drawn() {
    let edge = the_edge_over(a_window(), anywhere(), 100);
    let close = edge
        .controls
        .first()
        .map(|control| control.target)
        .unwrap_or_default();
    let middle = Point::from((
        close.loc.x + close.size.w / 2,
        close.loc.y + close.size.h / 2,
    ));
    assert_eq!(
        what_a_press_is_on(a_window(), middle, (anywhere(), 100)),
        Some(OnItsEdge::Close)
    );
    let title = edge.title.unwrap_or_default();
    assert_eq!(
        what_a_press_is_on(
            a_window(),
            Point::from((title.x + 4, title.y + 4)),
            (anywhere(), 100)
        ),
        Some(OnItsEdge::TheRest)
    );
    assert_eq!(
        what_a_press_is_on(a_window(), Point::from((600, 500)), (anywhere(), 100)),
        None
    );
}
