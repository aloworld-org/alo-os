//! That the window a person goes to comes in front, and that **the keys and
//! the window drawn on top agree** after every kind of switch: Ctrl+Alt+T,
//! Super+Tab, Alt+Tab and a click.
//!
//! The terminal here is a real shell on a real pseudoterminal, and every key
//! and press goes the road a person's does: through the task window's seat
//! while a task window holds the keyboard, through the ordinary seat
//! otherwise — the choice `crate::direct_desktop` makes.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_models::InferenceSource;
use alo_tasks::ATask;
use smithay::backend::input::{ButtonState, KeyState};
use smithay::input::keyboard::XkbConfig;
use smithay::utils::{Logical, Physical, Point, Rectangle};

use super::{AKindOfWindow, WhichIsInFront};
use crate::Server;

/// evdev's codes for the keys these tests press.
const LEFT_CTRL: u32 = 29;
const LEFT_ALT: u32 = 56;
const LEFT_SUPER: u32 = 125;
const TAB: u32 = 15;
const T: u32 = 20;

#[test]
fn as_it_was_the_task_windows_are_in_front() {
    let order = WhichIsInFront::default();
    assert_eq!(
        order.bottom_to_top(),
        [
            AKindOfWindow::Applications,
            AKindOfWindow::TheTerminal,
            AKindOfWindow::TaskWindows,
            AKindOfWindow::TheRecord,
            AKindOfWindow::Settings
        ]
    );
    assert!(order.the_task_windows_are_in_front());
}

#[test]
fn the_kind_gone_to_comes_in_front_and_the_others_keep_their_order() {
    let mut order = WhichIsInFront::default();
    order.comes_to_the_front(AKindOfWindow::Applications);
    assert_eq!(
        order.bottom_to_top(),
        [
            AKindOfWindow::TheTerminal,
            AKindOfWindow::TaskWindows,
            AKindOfWindow::TheRecord,
            AKindOfWindow::Settings,
            AKindOfWindow::Applications
        ]
    );
    assert!(!order.the_task_windows_are_in_front());
    order.comes_to_the_front(AKindOfWindow::Applications);
    assert_eq!(
        order.top_to_bottom(),
        [
            AKindOfWindow::Applications,
            AKindOfWindow::Settings,
            AKindOfWindow::TheRecord,
            AKindOfWindow::TaskWindows,
            AKindOfWindow::TheTerminal
        ],
        "going to what is already in front moved something"
    );
}

/// A session with a keyboard, a pointer and the shipped shortcuts.
fn a_session() -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind_keyboard(
        runtime.path(),
        "the-window-you-go-to",
        XkbConfig {
            layout: "us",
            ..XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    server.enable_pointer().expect("a pointer on this seat");
    server.the_shortcuts_are(alo_shortcuts::Shortcuts::shipped());
    (runtime, server)
}

/// Where things are, on the screen.
struct ThePlaces {
    /// Inside the task window and the terminal both.
    both: Point<f64, Logical>,
    /// Inside the terminal only.
    the_terminal_alone: Point<f64, Logical>,
    /// Inside the task window only.
    the_task_alone: Point<f64, Logical>,
}

/// A task window the person asked for, holding the keyboard, and alo's own
/// terminal open and drawn so that it overlaps the task window's left side.
fn a_task_and_a_terminal(server: &mut Server) -> ThePlaces {
    assert!(
        server.open_alos_own_terminal(crate::the_terminal_a_person_opened::A_SENSIBLE_FIRST_GRID),
        "the control: a terminal opened"
    );
    let which = server.tasks_mut().and_now_this_one(ATask::asked_for(
        "tidy my invoices",
        InferenceSource::ThisMachine,
    ));
    let view = Rectangle::new((0, 0).into(), (1920, 1080).into());
    server.tasks_mut().put_whatever_has_nowhere_to_be(
        view,
        alo_strings::Direction::LeftToRight,
        &[],
        100,
    );
    server.this_task_took_the_keyboard(which);
    let task = server
        .tasks()
        .where_each_went()
        .get(which.at())
        .copied()
        .flatten()
        .expect("the task window was placed");
    assert!(
        task.loc.x >= 300,
        "the premise: room for the terminal left of it"
    );
    // **Drawn where a frame would draw it**, set here so the geometry is the
    // test's: two hundred over the task window's left side.
    server.where_the_terminal_is = Some(Rectangle::<i32, Physical>::new(
        (task.loc.x - 200, task.loc.y).into(),
        (400, task.size.h).into(),
    ));
    let middle = f64::from(task.loc.y + task.size.h / 2);
    ThePlaces {
        both: (f64::from(task.loc.x + 100), middle).into(),
        the_terminal_alone: (f64::from(task.loc.x - 100), middle).into(),
        the_task_alone: (f64::from(task.loc.x + 600), middle).into(),
    }
}

/// One key event, down the road `crate::direct_desktop` sends it.
fn a_key(server: &mut Server, code: u32, state: KeyState) {
    if server.a_task_is_taking_the_keys() {
        let strings = alo_strings::Strings::of(alo_strings::Vocabulary::empty());
        crate::a_key_reaches_the_task_in_front::the_key_the_task_took(
            server,
            crate::DirectKeyEvent {
                code,
                state,
                time: 1,
            },
            None,
            &strings,
        )
        .expect("the task's seat took the key");
    } else {
        let _ = server
            .keyboard_key(code, state, 1)
            .expect("the seat took the key");
    }
}

/// A shortcut, held and released the way a person presses one.
fn a_shortcut(server: &mut Server, held: &[u32], key: u32) {
    for &down in held {
        a_key(server, down, KeyState::Pressed);
    }
    a_key(server, key, KeyState::Pressed);
    a_key(server, key, KeyState::Released);
    for &up in held.iter().rev() {
        a_key(server, up, KeyState::Released);
    }
}

/// A click at this point.
fn a_click(server: &mut Server, at: Point<f64, Logical>) {
    server
        .pointer_motion(at.x, at.y, 1)
        .expect("the pointer moved");
    for state in [ButtonState::Pressed, ButtonState::Released] {
        let _ = server
            .pointer_button(0x110, state, 2)
            .expect("the press was handled");
    }
}

/// Which kind of window the keys go to now.
fn the_keys_go_to(server: &Server) -> AKindOfWindow {
    if server.a_task_is_taking_the_keys() {
        AKindOfWindow::TaskWindows
    } else if server.the_terminal_is_taking_the_keys() {
        AKindOfWindow::TheTerminal
    } else {
        AKindOfWindow::Applications
    }
}

/// **The keys go to the window drawn on top where both are drawn**, and the
/// painter's front is the same kind.
fn they_agree(server: &Server, places: &ThePlaces, expected: AKindOfWindow, after: &str) {
    assert_eq!(the_keys_go_to(server), expected, "after {after}, the keys");
    assert_eq!(
        server.the_kind_under(places.both),
        Some(expected),
        "after {after}, the window on top where both are drawn"
    );
    assert_eq!(
        server.which_is_in_front().top_to_bottom().first(),
        Some(&expected),
        "after {after}, the window painted last"
    );
}

#[test]
fn a_task_asked_for_is_in_front_and_has_the_keys() {
    let (_runtime, mut server) = a_session();
    let places = a_task_and_a_terminal(&mut server);
    they_agree(&server, &places, AKindOfWindow::TaskWindows, "asking alo");
}

/// **Ctrl+Alt+T from a task window brings the terminal in front of it.**
/// Until 2026-10-31 the terminal took the keys from behind the task window.
#[test]
fn the_terminal_shortcut_brings_the_terminal_in_front() {
    let (_runtime, mut server) = a_session();
    let places = a_task_and_a_terminal(&mut server);

    a_shortcut(&mut server, &[LEFT_CTRL, LEFT_ALT], T);

    they_agree(&server, &places, AKindOfWindow::TheTerminal, "Ctrl+Alt+T");
}

/// **Super+Tab goes round every window**: from the terminal to the task
/// window left behind it, and back — the task window is never more than one
/// Super+Tab away.
#[test]
fn super_tab_reaches_the_task_window_behind_and_back_again() {
    let (_runtime, mut server) = a_session();
    let places = a_task_and_a_terminal(&mut server);
    a_shortcut(&mut server, &[LEFT_CTRL, LEFT_ALT], T);
    they_agree(&server, &places, AKindOfWindow::TheTerminal, "the premise");

    a_shortcut(&mut server, &[LEFT_SUPER], TAB);
    they_agree(&server, &places, AKindOfWindow::TaskWindows, "Super+Tab");

    a_shortcut(&mut server, &[LEFT_SUPER], TAB);
    they_agree(
        &server,
        &places,
        AKindOfWindow::TheTerminal,
        "a second Super+Tab",
    );
}

/// **Alt+Tab goes round the same way.**
#[test]
fn alt_tab_reaches_the_task_window_behind() {
    let (_runtime, mut server) = a_session();
    let places = a_task_and_a_terminal(&mut server);
    a_shortcut(&mut server, &[LEFT_CTRL, LEFT_ALT], T);

    a_shortcut(&mut server, &[LEFT_ALT], TAB);

    they_agree(&server, &places, AKindOfWindow::TaskWindows, "Alt+Tab");
}

/// **A click goes to what is drawn on top where it lands**, and brings what
/// it lands on in front: on the terminal alone, on the task window alone, and
/// where both are drawn.
#[test]
fn a_click_brings_what_it_lands_on_in_front() {
    let (_runtime, mut server) = a_session();
    let places = a_task_and_a_terminal(&mut server);

    a_click(&mut server, places.the_terminal_alone);
    they_agree(
        &server,
        &places,
        AKindOfWindow::TheTerminal,
        "a click on the terminal",
    );

    // Where both are drawn the terminal is on top now, so the click is its.
    a_click(&mut server, places.both);
    they_agree(
        &server,
        &places,
        AKindOfWindow::TheTerminal,
        "a click where the terminal is on top",
    );

    a_click(&mut server, places.the_task_alone);
    they_agree(
        &server,
        &places,
        AKindOfWindow::TaskWindows,
        "a click on the task window",
    );

    // And now the task window is on top where both are drawn.
    a_click(&mut server, places.both);
    they_agree(
        &server,
        &places,
        AKindOfWindow::TaskWindows,
        "a click where the task window is on top",
    );
}

/// **Nothing can type into a window hidden behind another**: with the task
/// window in front, the terminal behind it takes no key.
#[test]
fn a_terminal_behind_a_task_window_takes_no_key() {
    let (_runtime, mut server) = a_session();
    let _places = a_task_and_a_terminal(&mut server);
    a_shortcut(&mut server, &[LEFT_CTRL, LEFT_ALT], T);
    a_shortcut(&mut server, &[LEFT_SUPER], TAB);

    assert!(server.a_task_is_taking_the_keys(), "the premise");
    // **The task window lets go while it is still in front**, as it does
    // when the keyboard goes to the Dock and back.
    server.the_task_gave_the_keyboard_up();
    assert!(
        !server.the_terminal_is_taking_the_keys(),
        "the terminal took keys from behind a task window"
    );
}

/// **Closing the last task window leaves the terminal in front**, and it
/// takes the keys again: a kind with nothing open is in front of nothing.
#[test]
fn with_no_task_window_open_the_terminal_is_in_front_again() {
    let (_runtime, mut server) = a_session();
    let _places = a_task_and_a_terminal(&mut server);
    let which = server
        .tasks()
        .the_one_in_front()
        .expect("the premise: one task");
    server
        .tasks_mut()
        .the_mut(which)
        .expect("the task is open")
        .alo_says(alo_tasks::conversation::Turn::from_alo("Done."));

    assert_eq!(
        server.the_person_closes_a_task(which),
        crate::closing_a_task_window::WhatClosingCameTo::Closed
    );

    assert_eq!(server.the_kind_in_front(), AKindOfWindow::TheTerminal);
    assert!(server.the_terminal_is_taking_the_keys());
}
