//! What a reader is told, and the clause a mouse-shaped build fails quietly.
#![expect(
    clippy::indexing_slicing,
    clippy::expect_used,
    reason = "in a test, a panic on a control the edge said was there is the failure being reported"
)]

use super::*;
use crate::window_edge::{Decorations, edge_of};
use smithay::utils::{Logical, Point, Rectangle, Size};

/// A window with room above it for its edge.
fn window() -> Rectangle<i32, Logical> {
    Rectangle::new(Point::from((40, 100)), Size::from((600, 400)))
}

/// **Reachable without hovering.**
///
/// The owner: *screen-reader and keyboard users must reach the new edge
/// without first hovering.* So a concealed edge announces exactly what a
/// revealed one does. An implementation that gated the tree on `revealed`
/// would pass every drawing test and leave a keyboard user unable to reach a
/// control they can hear.
#[test]
fn a_concealed_edge_is_reached_just_as_a_revealed_one_is() {
    // The edge draws nothing at rest, correctly - and a reader is told the
    // same either way, because what a window has does not change with where a
    // pointer is.
    let concealed = edge_of(window(), Decorations::TheShellDraws, false);
    assert!(
        concealed.controls.is_empty(),
        "a resting edge draws no control"
    );
    let told = what_a_reader_is_told(Decorations::TheShellDraws);
    assert_eq!(told.len(), 3, "{told:?}");
    assert!(
        told.iter().all(|control| control.can_be_used()),
        "a person could hear a control they could not reach"
    );
}

/// **The three window actions are the ones the machine already has.**
///
/// Not new actions and not new words: the edge moved where a control is, not
/// what pressing it does.
#[test]
fn the_edge_reuses_the_window_actions_that_already_exist() {
    let edge = edge_of(window(), Decorations::TheShellDraws, true);
    let does: Vec<_> = edge
        .controls
        .iter()
        .filter_map(|control| what_it_does(control.does))
        .collect();
    assert_eq!(
        does,
        vec![
            Action::MinimiseWindow,
            Action::MaximiseWindow,
            Action::CloseWindow,
        ]
    );
}

/// **Every control a reader is told about can be acted on**, and is named by
/// the action's own word rather than by a second string that has to agree.
#[test]
fn every_control_is_a_button_a_person_can_use() {
    let told = what_a_reader_is_told(Decorations::TheShellDraws);
    assert_eq!(told.len(), 3, "{told:?}");
    for control in &told {
        assert!(
            control.can_be_used(),
            "a reader was told about something nobody can press: {control:?}"
        );
        assert_eq!(control.role, alo_access::tree::Role::Button);
        let does = control.does.expect("a window control does something");
        assert_eq!(
            control.name,
            does.word(),
            "the name is not the action's word"
        );
    }
}

/// **An application with its own header is not announced twice — and now its
/// menu is.**
///
/// It keeps its own buttons, so alo's edge carries only a menu. Until
/// 2026-10-10 that menu had no action and therefore no word, so a reader on
/// such a window was told about nothing of alo's at all. It has one now, which
/// is the whole point: the menu is the road to the full title, and an
/// application-decorated window is exactly where alo has no title of its own
/// to show.
#[test]
fn an_application_with_its_own_header_gets_no_duplicate_controls() {
    let edge = edge_of(window(), Decorations::TheApplicationDraws, true);
    assert_eq!(edge.controls.len(), 1);
    assert_eq!(edge.controls[0].does, OnTheEdge::Menu);
    let told = what_a_reader_is_told(Decorations::TheApplicationDraws);
    assert_eq!(
        told.len(),
        1,
        "an application-decorated window offers alo's menu and nothing else: {told:?}"
    );
    assert_eq!(
        told[0].does,
        Some(alo_shortcuts::Action::WindowOptions),
        "the one thing announced is not the window's options"
    );
}

/// **Every control on the edge has an action, including the menu.**
///
/// This test was `the_window_menu_is_the_one_control_with_no_action_yet` and
/// held the opposite, with a note saying it would fail the day somebody gave
/// the menu one — *at which point the two assertions below swap round*. They
/// have. The owner's ruling of 2026-10-10 made the menu the road to a long
/// title, so a control with no action was a title with no road.
#[test]
fn every_control_on_the_edge_has_an_action() {
    for control in [
        OnTheEdge::Minimise,
        OnTheEdge::Maximise,
        OnTheEdge::Close,
        OnTheEdge::Menu,
    ] {
        assert!(
            what_it_does(control).is_some(),
            "{control:?} has no action, so a reader has no word for it"
        );
    }
    assert_eq!(
        what_it_does(OnTheEdge::Menu),
        Some(alo_shortcuts::Action::WindowOptions),
        "the menu's action is not the one named for what it does"
    );
}
