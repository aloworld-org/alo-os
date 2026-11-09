//! What a pointer is on, on a band: each control at its own target, the band
//! between them, and nothing a hidden control would have been.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::*;
use crate::window_edge::{A_TARGET_IS, BETWEEN_TARGETS, Decorations, edge_of};
use smithay::utils::{Rectangle, Size};

/// The Notes window, its band at the origin.
fn a_window() -> Rectangle<i32, Logical> {
    Rectangle::new(Point::from((0, 54)), Size::from((1056, 466)))
}

/// Its band, revealed.
fn revealed() -> WindowEdge {
    edge_of(
        a_window(),
        Decorations::TheShellDraws,
        true,
        Size::from((4000, 3000)),
    )
}

/// The middle of a rectangle.
fn middle_of(area: Rectangle<i32, Logical>) -> Point<i32, Logical> {
    Point::from((area.loc.x + area.size.w / 2, area.loc.y + area.size.h / 2))
}

/// **Every control answers at its middle and at its corners** — the whole
/// 44 × 44, never a smaller drawn shape inside it.
#[test]
fn every_control_answers_across_its_whole_target() {
    let edge = revealed();
    for control in &edge.controls {
        let t = control.target;
        for at in [
            middle_of(t),
            t.loc,
            Point::from((t.loc.x + A_TARGET_IS - 1, t.loc.y + A_TARGET_IS - 1)),
        ] {
            assert_eq!(
                what_is_under(&edge, at),
                Some(UnderThePointer::Control(control.does)),
                "{:?} at {at:?}",
                control.does
            );
        }
    }
}

/// **The gap between two targets is the band's**, and a press there starts
/// no control.
#[test]
fn the_gap_between_two_targets_presses_nothing() {
    let edge = revealed();
    let first = edge.controls.first().expect("a control").target;
    let in_the_gap = Point::from((first.loc.x + A_TARGET_IS + BETWEEN_TARGETS / 2, 27));
    assert!(!matches!(
        what_is_under(&edge, in_the_gap),
        Some(UnderThePointer::Control(_))
    ));
}

/// **The title is the drag**, the band's only free-move handle.
#[test]
fn the_title_is_the_drag() {
    assert_eq!(
        what_is_under(&revealed(), Point::from((40, 27))),
        Some(UnderThePointer::Drag)
    );
}

/// **At rest a press where a control would be is on the band**, because a
/// control nobody can see is never pressed.
#[test]
fn at_rest_where_a_control_would_be_is_the_band() {
    let at_rest = edge_of(
        a_window(),
        Decorations::TheShellDraws,
        false,
        Size::from((4000, 3000)),
    );
    let where_close_is = revealed()
        .the_control(OnTheEdge::Close)
        .map(|it| middle_of(it.target))
        .expect("Close");
    assert_eq!(
        what_is_under(&at_rest, where_close_is),
        Some(UnderThePointer::Drag)
    );
}

/// **Off the band is not on it**, above it and in the content below.
#[test]
fn off_the_band_is_nothing() {
    let edge = revealed();
    assert_eq!(what_is_under(&edge, Point::from((500, -1))), None);
    assert_eq!(what_is_under(&edge, Point::from((500, 54))), None);
}

/// **Only a control is hovered**; the band around it is not.
#[test]
fn only_a_control_is_hovered() {
    let edge = revealed();
    let close = edge.the_control(OnTheEdge::Close).expect("Close").target;
    assert_eq!(
        pointing_at(&edge, Some(middle_of(close)), None).hovered,
        Some(OnTheEdge::Close)
    );
    assert_eq!(
        pointing_at(&edge, Some(Point::from((40, 27))), None).hovered,
        None
    );
    assert_eq!(pointing_at(&edge, None, None).hovered, None);
}

/// **Focus is the keyboard's, not the pointer's**: both are carried, apart.
#[test]
fn focus_is_not_a_thing_the_pointer_decides() {
    let edge = revealed();
    let close = edge.the_control(OnTheEdge::Close).expect("Close").target;
    let pointing = pointing_at(&edge, Some(middle_of(close)), Some(OnTheEdge::Minimise));
    assert_eq!(pointing.hovered, Some(OnTheEdge::Close));
    assert_eq!(pointing.focused, Some(OnTheEdge::Minimise));
}
