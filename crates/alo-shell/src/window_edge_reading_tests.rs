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

/// **An application with its own header is not announced twice.**
///
/// It keeps its own buttons, so alo's edge carries only a menu — and until the
/// menu has an action it has no word, so a reader is told about nothing of
/// alo's rather than about something alo cannot name.
#[test]
fn an_application_with_its_own_header_gets_no_duplicate_controls() {
    let edge = edge_of(window(), Decorations::TheApplicationDraws, true);
    assert_eq!(edge.controls.len(), 1);
    assert_eq!(edge.controls[0].does, OnTheEdge::Menu);
    assert!(
        what_a_reader_is_told(Decorations::TheApplicationDraws).is_empty(),
        "alo announced a control over an application that has its own"
    );
}

/// **The window menu has no action, and that is recorded rather than faked.**
///
/// A reader announcing a menu it could only name in English would be worse
/// than one that does not announce it. The fix is an `Action` with a word like
/// every other control's, and this test is what will fail when somebody adds
/// one — at which point the two assertions below swap round.
#[test]
fn the_window_menu_is_the_one_control_with_no_action_yet() {
    assert!(what_it_does(OnTheEdge::Menu).is_none());
    for named in [OnTheEdge::Minimise, OnTheEdge::Maximise, OnTheEdge::Close] {
        assert!(what_it_does(named).is_some(), "{named:?} lost its action");
    }
}
