//! The edge over one of alo's own windows: Minimise and Close, a title and an icon's
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

/// **Minimise and Close, with a title** — no band carries an icon since
/// 2026-11-09, and Minimise came with the panel's place for alo's windows
/// (design `27:2`) — and the band ending where the window begins.
#[test]
fn minimise_and_close_with_a_title_on_the_window() {
    let edge = the_edge_over(a_window(), anywhere(), 100);
    let controls: Vec<OnTheEdge> = edge.controls.iter().map(|control| control.does).collect();
    assert_eq!(controls, vec![OnTheEdge::Minimise, OnTheEdge::Close]);
    assert!(edge.title.is_some());
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
    let middle_of = |does: OnTheEdge| {
        let target = edge
            .controls
            .iter()
            .find(|control| control.does == does)
            .map(|control| control.target)
            .unwrap_or_default();
        Point::from((
            target.loc.x + target.size.w / 2,
            target.loc.y + target.size.h / 2,
        ))
    };
    assert_eq!(
        what_a_press_is_on(a_window(), middle_of(OnTheEdge::Close), (anywhere(), 100)),
        Some(OnItsEdge::Close)
    );
    assert_eq!(
        what_a_press_is_on(
            a_window(),
            middle_of(OnTheEdge::Minimise),
            (anywhere(), 100)
        ),
        Some(OnItsEdge::Minimise)
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
