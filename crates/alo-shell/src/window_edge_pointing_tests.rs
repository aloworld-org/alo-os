//! What a pointer reaches on an edge, and the three places it reaches nothing.
#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "in a test, a panic on an unexpected shape is the failure being reported"
)]

use smithay::utils::{Logical, Point, Rectangle, Size};

use super::{UnderThePointer, pointing_at, what_is_under};
use crate::window_edge::{
    A_HIGHLIGHT_IS, A_TARGET_IS, BETWEEN_TARGETS, Decorations, OnTheEdge, THE_PADDING_IS,
    WindowEdge, edge_of,
};

/// A window somewhere that is not the origin, so a position confused with an
/// offset fails rather than passing by accident.
fn a_window() -> Rectangle<i32, Logical> {
    Rectangle::new(Point::from((300, 200)), Size::from((800, 600)))
}

fn revealed() -> WindowEdge {
    edge_of(
        a_window(),
        Decorations::TheShellDraws,
        true,
        smithay::utils::Size::from((1920, 1080)),
    )
}

fn middle_of(area: Rectangle<i32, Logical>) -> Point<i32, Logical> {
    Point::from((area.loc.x + area.size.w / 2, area.loc.y + area.size.h / 2))
}

/// Every control a revealed window has can be pressed, and gives back itself.
///
/// The list is read from the edge's own controls rather than written here, so
/// a control added to the layout is covered the day it is added.
#[test]
fn every_control_answers_at_its_middle() {
    let edge = revealed();
    assert!(
        !edge.controls.is_empty(),
        "a revealed shell edge has controls"
    );
    for control in &edge.controls {
        assert_eq!(
            what_is_under(&edge, middle_of(control.target)),
            Some(UnderThePointer::Control(control.does)),
            "{:?} did not answer at the middle of its own target",
            control.does
        );
    }
}

/// **The target answers, not the pill that is drawn.**
///
/// The highlight is 32 x 28 inside a 44 x 44 target, so there is a margin
/// around it that is still the control. A hit test written against what is
/// drawn would ignore a press there, which feels like a bad mouse and not
/// like a bug - so each of the four sides of that margin is checked.
#[test]
fn the_margin_outside_the_highlight_is_still_the_control() {
    let edge = revealed();
    for control in &edge.controls {
        let pill = control.highlight;
        assert_eq!(
            (pill.size.w, pill.size.h),
            A_HIGHLIGHT_IS,
            "this test is only meaningful while the pill is smaller than the target"
        );
        let outside = [
            Point::from((control.target.loc.x, middle_of(pill).y)),
            Point::from((control.target.loc.x + A_TARGET_IS - 1, middle_of(pill).y)),
            Point::from((middle_of(pill).x, control.target.loc.y)),
            Point::from((middle_of(pill).x, control.target.loc.y + A_TARGET_IS - 1)),
        ];
        for at in outside {
            assert!(
                !pill.contains(at),
                "{at:?} was meant to be outside the drawn pill"
            );
            assert_eq!(
                what_is_under(&edge, at),
                Some(UnderThePointer::Control(control.does)),
                "{:?} ignored a press at {at:?}, inside its target and outside its pill",
                control.does
            );
        }
    }
}

/// The four units between one target and the next belong to no control.
#[test]
fn the_gap_between_two_targets_belongs_to_nobody() {
    let edge = revealed();
    assert!(
        edge.controls.len() >= 2,
        "a gap needs two controls either side of it"
    );
    // **The actions, not every control.** Move sits at the leading padding and
    // the gap between it and the first action is the drag area, which is as
    // wide as the window allows rather than one `BETWEEN_TARGETS`.
    let actions: Vec<_> = edge
        .controls
        .iter()
        .filter(|it| it.does != OnTheEdge::Move)
        .copied()
        .collect();
    for pair in actions.windows(2) {
        let (left, right) = (pair[0], pair[1]);
        let gap = left.target.loc.x + A_TARGET_IS;
        assert_eq!(
            right.target.loc.x - gap,
            BETWEEN_TARGETS,
            "the layout stopped leaving a gap, so this test stopped measuring one"
        );
        for x in gap..right.target.loc.x {
            let at = Point::from((x, middle_of(left.target).y));
            assert!(
                !matches!(what_is_under(&edge, at), Some(UnderThePointer::Control(_))),
                "{at:?} is in the gap between {:?} and {:?} and answered a control",
                left.does,
                right.does
            );
        }
    }
}

/// A target's last unit is inside it and the next one is not.
///
/// `Rectangle::contains` is half-open, and this is what keeps two rectangles
/// from both claiming the unit where they meet.
#[test]
fn a_target_ends_where_it_says_it_ends() {
    let edge = revealed();
    let last = *edge
        .controls
        .last()
        .expect("a revealed shell edge has controls");
    let y = middle_of(last.target).y;
    assert_eq!(
        what_is_under(&edge, Point::from((last.target.loc.x + A_TARGET_IS - 1, y))),
        Some(UnderThePointer::Control(last.does)),
        "the last unit of a target is inside it"
    );
    assert!(
        !matches!(
            what_is_under(&edge, Point::from((last.target.loc.x + A_TARGET_IS, y))),
            Some(UnderThePointer::Control(does)) if does == last.does
        ),
        "one unit past a target is not that target"
    );
}

/// No **action** sits inside the drag region — and Move deliberately does.
///
/// `WindowEdge::drag` carried the sentence *never under a control* and that
/// stopped being the whole truth on 2026-10-11: the compact design makes Move
/// a 44 × 44 control at the leading end of the drag area, so one control is
/// now inside it on purpose. Pressing Move *is* beginning a drag.
///
/// What still must hold, and is what this guards, is that no control a press
/// would **act** on — Minimise, Maximise, Close, Menu — is reachable by a
/// pointer that thinks it is dragging. That is what makes the order inside
/// `what_is_under` a guard rather than a tiebreak.
#[test]
fn no_action_is_ever_also_the_drag() {
    let edge = revealed();
    for control in edge.controls.iter().filter(|it| it.does != OnTheEdge::Move) {
        assert!(
            edge.drag.intersection(control.target).is_none(),
            "the target of {:?} overlaps the drag region",
            control.does
        );
    }
    // The control on this test: Move is inside it, so the filter above is
    // excluding something real rather than excluding nothing.
    let moves = edge
        .controls
        .iter()
        .find(|it| it.does == OnTheEdge::Move)
        .expect("a revealed edge lays out Move");
    assert_eq!(
        edge.drag.intersection(moves.target),
        Some(moves.target),
        "Move is meant to be wholly inside the drag region"
    );
    assert_eq!(
        what_is_under(&edge, middle_of(edge.drag)),
        Some(UnderThePointer::Drag)
    );
}

/// The leading units before the drag region answer nothing, and are still on
/// the edge.
#[test]
fn the_leading_edge_is_on_the_edge_and_presses_nothing() {
    let edge = revealed();
    // **Four units, not eight.** The compact design puts the leading padding
    // at 4 and a Move target immediately after it, where the older capture had
    // an 8-unit inset with nothing in it. So the band that presses nothing is
    // narrower than it was, and it is still there.
    let y = middle_of(edge.drag).y;
    for x in edge.region.loc.x..edge.region.loc.x + THE_PADDING_IS {
        assert_eq!(
            what_is_under(&edge, Point::from((x, y))),
            Some(UnderThePointer::TheEdgeAndNothingOnIt),
            "x = {x} is before the first target and should press nothing"
        );
    }
    // And the unit after it is Move, so the loop above stops in the right
    // place rather than merely stopping.
    assert_eq!(
        what_is_under(&edge, Point::from((edge.region.loc.x + THE_PADDING_IS, y))),
        Some(UnderThePointer::Control(OnTheEdge::Move))
    );
}

/// Off the region is [`None`], which is what a host conceals on.
#[test]
fn away_from_the_edge_is_not_on_the_edge() {
    let edge = revealed();
    let away = [
        Point::from((edge.region.loc.x - 1, middle_of(edge.region).y)),
        Point::from((
            edge.region.loc.x + edge.region.size.w,
            middle_of(edge.region).y,
        )),
        Point::from((middle_of(edge.region).x, edge.region.loc.y - 1)),
        // Inside the window itself, one unit below the region.
        Point::from((
            middle_of(edge.region).x,
            edge.region.loc.y + edge.region.size.h,
        )),
    ];
    for at in away {
        assert_eq!(what_is_under(&edge, at), None, "{at:?} is not on the edge");
    }
}

/// At rest nothing is drawn to press, so nothing answers a control - and the
/// region still answers, because that is what reveals the edge.
#[test]
fn an_edge_at_rest_has_nothing_to_press_and_is_still_there() {
    let edge = edge_of(
        a_window(),
        Decorations::TheShellDraws,
        false,
        smithay::utils::Size::from((1920, 1080)),
    );
    assert!(
        edge.controls.is_empty(),
        "a concealed edge draws no control"
    );
    assert!(
        what_is_under(&edge, middle_of(edge.region)).is_some(),
        "a concealed edge is still an edge a pointer is on"
    );
    for x in edge.region.loc.x..edge.region.loc.x + edge.region.size.w {
        for y in edge.region.loc.y..edge.region.loc.y + edge.region.size.h {
            assert!(
                !matches!(
                    what_is_under(&edge, Point::from((x, y))),
                    Some(UnderThePointer::Control(_))
                ),
                "({x}, {y}) pressed a control on an edge that draws none"
            );
        }
    }
}

/// An application that draws its own header gets the menu and nothing else,
/// and the menu is reachable.
#[test]
fn the_menu_is_reachable_on_an_application_that_draws_its_own_header() {
    let edge = edge_of(
        a_window(),
        Decorations::TheApplicationDraws,
        true,
        smithay::utils::Size::from((1920, 1080)),
    );
    assert_eq!(
        edge.controls.iter().map(|one| one.does).collect::<Vec<_>>(),
        vec![OnTheEdge::Move, OnTheEdge::Menu],
        "the owner's ruling: no duplicate window buttons, so movement and the menu and no Close"
    );
    let menu = *edge
        .controls
        .iter()
        .find(|it| it.does == OnTheEdge::Menu)
        .expect("an app-drawn edge lays out the menu");
    assert_eq!(
        what_is_under(&edge, middle_of(menu.target)),
        Some(UnderThePointer::Control(OnTheEdge::Menu))
    );
}

/// `pointing_at` highlights a control and nothing else.
#[test]
fn only_a_control_is_hovered() {
    let edge = revealed();
    let close = the_close_of(&edge);
    assert_eq!(
        pointing_at(&edge, Some(middle_of(close.target)), None).hovered,
        Some(OnTheEdge::Close)
    );
    for at in [middle_of(edge.drag), middle_of(edge.region)] {
        if matches!(what_is_under(&edge, at), Some(UnderThePointer::Control(_))) {
            continue;
        }
        assert_eq!(
            pointing_at(&edge, Some(at), None).hovered,
            None,
            "{at:?} is not a control and highlighted one"
        );
    }
    assert_eq!(pointing_at(&edge, None, None).hovered, None);
}

/// Keyboard focus is passed through and never guessed from the pointer.
///
/// A focus outline that followed the mouse would be the clearest possible
/// sign that two different questions had been answered by one.
#[test]
fn focus_is_not_a_thing_the_pointer_decides() {
    let edge = revealed();
    let close = the_close_of(&edge);
    let pointing = pointing_at(
        &edge,
        Some(middle_of(close.target)),
        Some(OnTheEdge::Minimise),
    );
    assert_eq!(pointing.hovered, Some(OnTheEdge::Close));
    assert_eq!(pointing.focused, Some(OnTheEdge::Minimise));

    // And with no pointer at all, the focus survives.
    let only_focus = pointing_at(&edge, None, Some(OnTheEdge::Minimise));
    assert_eq!(only_focus.hovered, None);
    assert_eq!(only_focus.focused, Some(OnTheEdge::Minimise));
}

/// What a press would do, end to end: a position becomes an action.
///
/// The two halves existed separately - a target that answers, and an action
/// each control maps to - and nothing joined them. This is the join, and it
/// is the whole reason this file exists.
#[test]
fn a_position_becomes_an_action() {
    let edge = revealed();
    let under = what_is_under(&edge, middle_of(the_close_of(&edge).target));
    let Some(UnderThePointer::Control(does)) = under else {
        panic!("a press on Close reached {under:?}");
    };
    assert_eq!(
        crate::window_edge_reading::what_it_does(does),
        Some(alo_shortcuts::Action::CloseWindow)
    );
}

fn the_close_of(edge: &WindowEdge) -> crate::window_edge::EdgeControl {
    *edge
        .controls
        .iter()
        .find(|one| one.does == OnTheEdge::Close)
        .expect("a shell-drawn edge has Close")
}
