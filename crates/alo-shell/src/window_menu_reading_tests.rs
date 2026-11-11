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
    let met = what_a_reader_meets(&a_panel(), None, None);
    assert_eq!(met.panel.name.says(), "Window");
    assert_eq!(met.panel.role, Role::List);
    assert!(!met.panel.can_be_used(), "the panel itself is not pressed");
}

/// **The choices are the ones drawn, and the one in hand is selected**;
/// nothing is selected before the keyboard is on one.
#[test]
fn the_choice_the_keyboard_is_on_is_told_as_selected() {
    let panel = a_panel();
    let nothing = what_a_reader_meets(&panel, None, None);
    assert!(nothing.rows.iter().all(|(_, selected)| !selected));

    let mut keyboard = TheKeyboardInTheMenu::opened_from(OnTheEdge::WindowActions);
    let _ = keyboard.told(crate::window_menu_keys::AMenuKey::Next, &panel);
    let _ = keyboard.told(crate::window_menu_keys::AMenuKey::Next, &panel);
    let met = what_a_reader_meets(&panel, Some(keyboard), None);
    let told: Vec<(&str, bool)> = met
        .rows
        .iter()
        .map(|(control, selected)| (control.name.says(), *selected))
        .collect();
    assert_eq!(
        told,
        vec![
            ("Left", false),
            ("Maximise", true),
            ("Right", false),
            ("Full screen", false)
        ]
    );
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
            "access.left",
            "access.right",
            "access.maximise",
            "access.restore",
            "access.full-screen",
            "access.exit-full-screen",
            "access.ask-about-this-window",
            "access.left-half-preview",
            "access.right-half-preview",
            "access.no-room-in-half"
        ]
    );
}

/// **A placement preview has a spoken equivalent**: with the keyboard on
/// Left, a reader hears where the window would go — or that it cannot fit —
/// and no other choice is renamed.
#[test]
fn a_reader_hears_the_preview_of_the_side_in_hand() {
    let panel = a_panel();
    let mut keyboard = TheKeyboardInTheMenu::opened_from(OnTheEdge::WindowActions);
    let _ = keyboard.told(crate::window_menu_keys::AMenuKey::Next, &panel);
    for says in [
        alo_access::words::LEFT_HALF_PREVIEW,
        alo_access::words::NO_ROOM_IN_HALF,
    ] {
        let met = what_a_reader_meets(&panel, Some(keyboard), Some(says));
        let told: Vec<&str> = met.rows.iter().map(|(it, _)| it.name.says()).collect();
        assert_eq!(told, vec![says.says(), "Maximise", "Right", "Full screen"]);
    }
}

/// **The alo row is told as a place to type**, named by its prompt — and
/// every other choice is still a button.
#[test]
fn the_alo_row_is_an_entry_and_the_rest_are_buttons() {
    let window = Rectangle::new(Point::from((112, 314)), Size::from((1056, 466)));
    let roomy = Size::from((4000, 3000));
    let edge = edge_of(window, Decorations::TheShellDraws, true, roomy);
    let how = HowTheWindowIs {
        alo_may_help: true,
        ..HowTheWindowIs::default()
    };
    let panel = the_menu_of(&edge, how, roomy).expect("a panel");
    let keyboard = TheKeyboardInTheMenu::in_the_field(OnTheEdge::WindowActions);
    let met = what_a_reader_meets(&panel, Some(keyboard), None);
    let (field, selected) = met
        .rows
        .iter()
        .find(|(control, _)| control.role == Role::Entry)
        .expect("the alo row is an entry");
    assert_eq!(field.name.says(), "Ask about this window…");
    assert!(selected, "the keyboard in the field is not told");
    assert_eq!(
        met.rows
            .iter()
            .filter(|(control, _)| control.role == Role::Button)
            .count(),
        met.rows.len() - 1
    );
}
