//! Where a press lands while a window panel is open — and that nothing passes
//! through it.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::*;
use crate::window_edge::{Decorations, edge_of};
use smithay::utils::{Rectangle, Size};

/// The Notes window's band and its open panel.
fn notes() -> (crate::window_edge::WindowEdge, WindowMenu) {
    let window = Rectangle::new(Point::from((112, 314)), Size::from((1056, 466)));
    let roomy = Size::from((4000, 3000));
    let edge = edge_of(window, Decorations::TheShellDraws, true, roomy);
    let menu = the_menu_of(&edge, HowTheWindowIs::default(), roomy).expect("a panel");
    (edge, menu)
}

/// Window actions' target on that band.
fn window_actions(edge: &crate::window_edge::WindowEdge) -> Option<Rectangle<i32, Logical>> {
    edge.the_control(OnTheEdge::WindowActions)
        .map(|it| it.target)
}

/// **A press on a choice chooses it.**
#[test]
fn a_press_on_a_choice_chooses_it() {
    let (edge, menu) = notes();
    assert_eq!(
        where_it_landed(&menu, window_actions(&edge), Point::from((900, 400))),
        ThePress::Chose(WhatARowDoes::Maximise)
    );
    assert_eq!(
        where_it_landed(&menu, window_actions(&edge), Point::from((1000, 510))),
        ThePress::Chose(WhatARowDoes::FullScreen)
    );
}

/// **A press inside the panel on nothing is the panel's**, and never reaches
/// the application under it — which is what the caller does with this answer.
#[test]
fn a_press_inside_on_nothing_is_the_panels_and_goes_no_further() {
    let (edge, menu) = notes();
    assert_eq!(
        where_it_landed(&menu, window_actions(&edge), Point::from((1100, 330))),
        ThePress::InsideOnNothing
    );
}

/// **A press outside closes it and applies nothing.**
#[test]
fn a_press_outside_dismisses_without_choosing() {
    let (edge, menu) = notes();
    assert_eq!(
        where_it_landed(&menu, window_actions(&edge), Point::from((200, 600))),
        ThePress::Dismissed
    );
}

/// **A press on Window actions is left to it**, so pressing it again closes
/// the panel rather than closing and reopening it.
#[test]
fn a_press_on_window_actions_is_left_to_the_band() {
    let (edge, menu) = notes();
    let at = window_actions(&edge).expect("shown").loc;
    assert_eq!(
        where_it_landed(&menu, window_actions(&edge), at),
        ThePress::OnWindowActions
    );
}
