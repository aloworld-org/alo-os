//! What a reader meets in an open window panel.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::*;
use crate::window_edge::{Decorations, OnTheEdge, edge_of};
use crate::window_menu::{HowTheWindowIs, the_menu_of};
use smithay::utils::{Point, Rectangle, Size};

/// The Notes window's panel, open.
fn a_panel() -> WindowMenu {
    let window = Rectangle::new(Point::from((112, 314)), Size::from((1056, 466)));
    let roomy = Size::from((4000, 3000));
    let edge = edge_of(window, Decorations::TheShellDraws, true, roomy);
    the_menu_of(&edge, HowTheWindowIs::default(), roomy).expect("a panel")
}

/// **The panel is named *Window*, its heading**, and is a list a reader walks.
#[test]
fn the_panel_is_named_by_its_heading() {
    let met = what_a_reader_meets(&a_panel(), None);
    assert_eq!(met.panel.name.says(), "Window");
    assert_eq!(met.panel.role, Role::List);
    assert!(!met.panel.can_be_used(), "the panel itself is not pressed");
}

/// **The choices are the ones drawn, and the one in hand is selected**;
/// nothing is selected before the keyboard is on one.
#[test]
fn the_choice_the_keyboard_is_on_is_told_as_selected() {
    let panel = a_panel();
    let nothing = what_a_reader_meets(&panel, None);
    assert!(nothing.rows.iter().all(|(_, selected)| !selected));

    let mut keyboard = TheKeyboardInTheMenu::opened_from(OnTheEdge::WindowActions);
    let _ = keyboard.told(crate::window_menu_keys::AMenuKey::Next, &panel);
    let _ = keyboard.told(crate::window_menu_keys::AMenuKey::Next, &panel);
    let met = what_a_reader_meets(&panel, Some(keyboard));
    let told: Vec<(&str, bool)> = met
        .rows
        .iter()
        .map(|(control, selected)| (control.name.says(), *selected))
        .collect();
    assert_eq!(told, vec![("Maximise", false), ("Full screen", true)]);
    assert!(
        met.rows
            .iter()
            .all(|(control, _)| control.state == State::OneOfAFew)
    );
}

/// **Every word the panel can say is counted**, so `alo-access` can hold its
/// words to being said.
#[test]
fn every_word_the_panel_says_is_listed() {
    let keys: Vec<&str> = every_word_this_surface_says()
        .iter()
        .map(Word::named)
        .collect();
    assert_eq!(
        keys,
        vec![
            "access.the-window-panel",
            "access.maximise",
            "access.restore",
            "access.full-screen",
            "access.exit-full-screen"
        ]
    );
}
