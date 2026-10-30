//! That a shortcut or a click which sends the keyboard to a window takes it
//! off the Dock, and that one which goes nowhere leaves it there.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_models::InferenceSource;
use alo_shortcuts::{Action, Chord, Key, Modifiers, Shortcuts};
use alo_tasks::ATask;

use super::sends_the_keyboard_to_a_window;
use crate::Server;
use crate::a_chord_reaches_its_action::WhatTheChordCameTo;
use crate::which_area_has_the_keyboard::AnArea;

/// A server on a socket of its own, with the shortcuts this release ships, and
/// the keyboard put on the Dock by F6 — where the twelfth walk had it.
fn on_the_dock(named: &str) -> (tempfile::TempDir, Server) {
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind(directory.path(), named).expect("a bound socket");
    server.the_shortcuts_are(Shortcuts::shipped());
    let f6 = Chord::checked(Modifiers::none(), Key::F6).expect("a function key may stand alone");
    assert_eq!(
        server.carry_out_a_chord(f6),
        WhatTheChordCameTo::Carried(Action::TheNextArea)
    );
    assert_eq!(
        server.which_area_has_the_keyboard,
        Some(AnArea::TheDock),
        "the premise: F6 put the keyboard on the Dock"
    );
    assert!(server.the_dock_is_taking_the_keys());
    (directory, server)
}

/// The chord this release ships for an action.
fn the_chord_for(action: Action) -> Chord {
    Shortcuts::shipped()
        .chord_for(action)
        .expect("this release ships a chord for it")
}

/// **The walk's own sequence**: F6 onto the Dock, then `Ctrl`+`Alt`+`T`. The
/// terminal opens and the Dock lets go, so Enter goes where the letters go
/// rather than pressing the Dock's button behind the terminal.
#[test]
fn opening_the_terminal_from_the_dock_takes_the_keyboard_off_it() {
    let (_directory, mut server) = on_the_dock("terminal-from-the-dock");

    let came_to = server.carry_out_a_chord(the_chord_for(Action::Terminal));

    assert_eq!(came_to, WhatTheChordCameTo::Carried(Action::Terminal));
    assert_eq!(
        server.which_area_has_the_keyboard, None,
        "the terminal opened and the Dock still held the keyboard"
    );
    assert_eq!(
        server.which_slot_on_the_bar, None,
        "a focus ring stayed on the Dock"
    );
    assert!(
        !server.the_dock_is_taking_the_keys(),
        "Enter would still press the Dock's focused button"
    );
}

/// **A shortcut that went nowhere leaves the keyboard where it was.** Moving
/// to the next window with no window open is refused, and a person on the
/// Dock is still on the Dock.
#[test]
fn a_shortcut_that_went_nowhere_leaves_the_keyboard_on_the_dock() {
    let (_directory, mut server) = on_the_dock("nowhere-from-the-dock");

    let came_to = server.carry_out_a_chord(the_chord_for(Action::NextWindow));

    assert_ne!(
        came_to,
        WhatTheChordCameTo::Carried(Action::NextWindow),
        "the premise: with no window open, there was nowhere to go"
    );
    assert_eq!(server.which_area_has_the_keyboard, Some(AnArea::TheDock));
    assert!(server.the_dock_is_taking_the_keys());
}

/// **A task window that takes the keyboard takes it off the Dock** — the
/// press on a task window and the Bar opening one both arrive here.
#[test]
fn a_task_window_taking_the_keyboard_takes_it_off_the_dock() {
    let (_directory, mut server) = on_the_dock("task-from-the-dock");
    let which = server.tasks_mut().and_now_this_one(ATask::asked_for(
        "Tidy my invoices folder",
        InferenceSource::ThisMachine,
    ));

    server.this_task_took_the_keyboard(which);

    assert!(server.a_task_is_taking_the_keys());
    assert_eq!(server.which_area_has_the_keyboard, None);
    assert!(!server.the_dock_is_taking_the_keys());
}

/// **F6 is not a window**, so it is not on this list: it moves the keyboard
/// among the areas itself, and a second F6 from the Dock goes back to the work
/// by its own rule rather than by this one.
#[test]
fn the_area_moves_are_not_moves_to_a_window() {
    assert!(!sends_the_keyboard_to_a_window(Action::TheNextArea));
    assert!(!sends_the_keyboard_to_a_window(Action::ThePreviousArea));
    assert!(sends_the_keyboard_to_a_window(Action::Terminal));
}
