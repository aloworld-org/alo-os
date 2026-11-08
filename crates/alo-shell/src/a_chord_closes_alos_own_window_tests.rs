//! **Alt+F4 at one of alo's own windows does what its Close does**, pressed
//! by evdev code down the road each window's keys take on a running machine —
//! and, as carefully, **what it must not close**: a running command without
//! asking, a window behind an application, or anything when the chord is not
//! Close window.
//!
//! The terminal is a real shell on a real pseudoterminal, because *a command
//! is running* is the kernel's answer and a mocked one would test the mock.
//!
//! That an application **behind** one of these windows is not asked to close
//! needs a real client, and is held in `tests/window_close`.

use std::os::unix::fs::PermissionsExt as _;
use std::time::Duration;

use alo_shortcuts::{Action, Shortcuts};
use alo_terminal::IsACommandRunning;
use alo_terminal::what_a_person_typed::{ANamedKey, Held, WhatWasTyped};
use smithay::backend::input::KeyState;
use smithay::input::keyboard::XkbConfig;

use crate::Server;
use crate::a_chord_reaches_its_action::WhatTheChordCameTo;
use crate::closing_the_terminal::OnTheQuestion;
use crate::the_terminal_a_person_opened::A_SENSIBLE_FIRST_GRID;
use crate::which_window_is_in_front::AKindOfWindow;

/// evdev's codes.
const LEFT_ALT: u32 = 56;
const F4: u32 = 62;
const F5: u32 = 63;
const ESCAPE: u32 = 1;

/// A session with a keyboard on a US layout and the shipped shortcuts.
fn a_session(what: &str) -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind_keyboard(
        runtime.path(),
        what,
        XkbConfig {
            layout: "us",
            ..XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    server.the_shortcuts_are(Shortcuts::shipped());
    (runtime, server)
}

/// Alt and `key`, held and released the way a person presses them, down the
/// road `deliver` is.
fn alt_and(server: &mut Server, key: u32, mut deliver: impl FnMut(&mut Server, u32, KeyState)) {
    deliver(server, LEFT_ALT, KeyState::Pressed);
    deliver(server, key, KeyState::Pressed);
    deliver(server, key, KeyState::Released);
    deliver(server, LEFT_ALT, KeyState::Released);
}

/// A key down the seat's own road, the one alo's terminal takes keys on.
fn at_the_seat(server: &mut Server, code: u32, state: KeyState) {
    let _ = server
        .keyboard_key(code, state, 1)
        .expect("the seat took the key");
}

/// A key down Settings' road, as `crate::direct_desktop` hands it over.
fn at_settings(server: &mut Server, code: u32, state: KeyState) {
    crate::a_key_reaches_settings::the_key_settings_took(
        server,
        crate::DirectKeyEvent {
            code,
            state,
            time: 1,
        },
        &crate::settings_testing::settings_words(),
    )
    .expect("Settings took the key");
}

/// A key down the record's road, as `crate::direct_desktop` hands it over.
fn at_the_record(server: &mut Server, code: u32, state: KeyState) {
    crate::a_key_reaches_the_record::the_key_the_record_took(
        server,
        crate::DirectKeyEvent {
            code,
            state,
            time: 1,
        },
    )
    .expect("the record took the key");
}

/// Pump the terminal until it reads `wanted`, or ten seconds pass.
fn until_it_reads(server: &mut Server, wanted: IsACommandRunning) -> bool {
    for _ in 0..400 {
        let _ = server.the_terminal_was_written_to();
        if server
            .alos_own_terminal()
            .is_some_and(|terminal| terminal.is_a_command_running() == wanted)
        {
            return true;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    false
}

/// A session with alo's terminal open, in front and taking the keys, its
/// shell at the prompt.
fn with_a_terminal(what: &str) -> (tempfile::TempDir, Server) {
    let (runtime, mut server) = a_session(what);
    assert!(server.open_alos_own_terminal(A_SENSIBLE_FIRST_GRID));
    server.the_person_went_to(AKindOfWindow::TheTerminal);
    assert!(
        until_it_reads(&mut server, IsACommandRunning::No),
        "the shell never reached its prompt"
    );
    assert!(server.the_terminal_is_taking_the_keys(), "the control");
    (runtime, server)
}

/// The same, with `cat` holding the terminal.
fn with_a_command_running(what: &str) -> (tempfile::TempDir, Server) {
    let (runtime, mut server) = with_a_terminal(what);
    for character in "cat".chars() {
        let _ = server
            .a_key_reached_alos_own_terminal(WhatWasTyped::Character(character), Held::default());
    }
    let _ = server
        .a_key_reached_alos_own_terminal(WhatWasTyped::Named(ANamedKey::Enter), Held::default());
    assert!(
        until_it_reads(&mut server, IsACommandRunning::Yes),
        "`cat` never held the terminal, so this proves nothing"
    );
    (runtime, server)
}

/// **Idle, Alt+F4 closes alo's terminal at once**, as its Close does.
#[test]
fn alt_f4_closes_an_idle_terminal() {
    let (_runtime, mut server) = with_a_terminal("alt-f4-idle-terminal");

    alt_and(&mut server, F4, at_the_seat);

    assert!(server.alos_own_terminal().is_none(), "the terminal stayed");
    assert!(server.the_question_closing_asks().is_none(), "it asked");
}

/// **With a command running, Alt+F4 asks first** — the question its Close
/// asks, with the keyboard on Cancel — and the terminal and the command are
/// both still there while it asks. This is the walk of 2026-11-08.
#[test]
fn alt_f4_asks_while_a_command_runs() {
    let (_runtime, mut server) = with_a_command_running("alt-f4-running-terminal");

    alt_and(&mut server, F4, at_the_seat);

    assert_eq!(
        server.the_question_closing_asks(),
        Some(OnTheQuestion::Cancel),
        "Alt+F4 did not ask, which is what the walk found"
    );
    assert!(server.alos_own_terminal().is_some());
    assert_eq!(
        server
            .alos_own_terminal()
            .map(crate::the_terminal_a_person_opened::TheTerminal::is_a_command_running),
        Some(IsACommandRunning::Yes)
    );
}

/// **The refusal path: Alt+F4 twice is not *Close anyway*, and Escape is
/// Cancel.** A second press while the question is open asks again and ends
/// nothing; Escape closes only the question, and the command still runs.
#[test]
fn alt_f4_again_and_escape_end_nothing() {
    let (_runtime, mut server) = with_a_command_running("alt-f4-twice");
    let shell = server
        .alos_own_terminal()
        .expect("a terminal")
        .the_shells_process();

    alt_and(&mut server, F4, at_the_seat);
    alt_and(&mut server, F4, at_the_seat);
    assert_eq!(
        server.the_question_closing_asks(),
        Some(OnTheQuestion::Cancel),
        "a second Alt+F4 answered the question"
    );
    assert!(
        server.alos_own_terminal().is_some(),
        "two presses closed it"
    );

    at_the_seat(&mut server, ESCAPE, KeyState::Pressed);
    at_the_seat(&mut server, ESCAPE, KeyState::Released);

    assert!(server.the_question_closing_asks().is_none());
    let terminal = server.alos_own_terminal().expect("Escape closed it");
    assert_eq!(terminal.the_shells_process(), shell);
    assert_eq!(terminal.is_a_command_running(), IsACommandRunning::Yes);
}

/// **Alt+F4 closes Settings**, down the road Settings' keys take.
#[test]
fn alt_f4_closes_settings() {
    let machine = crate::settings_testing::a_persons_machine("alt-f4-settings");
    let (_runtime, mut server) = a_session("alt-f4-settings");
    server.the_settings_places_are(machine.places.clone());
    server.settings = machine.opened();
    assert!(server.settings_is_taking_the_keys(), "the control");

    alt_and(&mut server, F4, at_settings);

    assert!(
        !server.the_settings_window().is_open(),
        "Alt+F4 left Settings open"
    );
}

/// **With a provider form open, Alt+F4 gives it up and closes**, as Close on
/// the edge does. The form takes no chord, so this is the one press the form
/// must not swallow.
#[test]
fn alt_f4_closes_settings_with_a_form_open() {
    let machine = crate::settings_testing::a_persons_machine("alt-f4-form");
    let (_runtime, mut server) = a_session("alt-f4-form");
    server.the_settings_places_are(machine.places.clone());
    server.settings = machine.opened();
    server.a_provider_is_added();
    assert!(
        server.a_provider_is_being_filled_in().is_some(),
        "the control"
    );

    alt_and(&mut server, F4, at_settings);

    assert!(!server.the_settings_window().is_open());
    assert!(
        server.a_provider_is_being_filled_in().is_none(),
        "the form outlived the window it was in"
    );
}

/// **Alt+F4 closes the record**, down the road the record's keys take.
#[test]
fn alt_f4_closes_the_record() {
    let (_runtime, mut server) = a_session("alt-f4-record");
    // Bound, not a temporary: the record's folder goes when this does.
    let kept = crate::record_testing::an_afternoon_kept();
    server.the_record_is_kept(Ok(kept.recounting()));
    let _ = server.the_record_is_opened();
    assert!(server.the_record_is_taking_the_keys(), "the control");

    alt_and(&mut server, F4, at_the_record);

    assert!(
        !server.the_record_window().is_open(),
        "Alt+F4 left the record open"
    );
}

/// **The refusal path: a window that does not have the keyboard is not
/// closed.** Settings open behind an application the person went to stays
/// open; Alt+F4 is the application's.
#[test]
fn a_window_behind_an_application_is_not_closed() {
    let machine = crate::settings_testing::a_persons_machine("alt-f4-behind");
    let (_runtime, mut server) = a_session("alt-f4-behind");
    server.the_settings_places_are(machine.places.clone());
    server.settings = machine.opened();
    server.the_person_went_to(AKindOfWindow::Applications);
    assert!(!server.settings_is_taking_the_keys(), "the control");

    let close = Shortcuts::shipped()
        .chord_for(Action::CloseWindow)
        .expect("the release ships Alt+F4");
    assert!(
        !server.a_chord_closes_alos_own_window(&Shortcuts::shipped(), close),
        "a window without the keyboard took Alt+F4"
    );
    assert!(server.the_settings_window().is_open(), "it was closed");
}

/// **The refusal path: any other chord closes nothing**, even with one of
/// alo's windows taking the keys — Alt+F5 is not Alt+F4.
#[test]
fn another_chord_closes_nothing() {
    let (_runtime, mut server) = with_a_terminal("alt-f5-closes-nothing");

    let settings_chord = Shortcuts::shipped()
        .chord_for(Action::Settings)
        .expect("the release ships a chord for Settings");
    assert!(!server.a_chord_closes_alos_own_window(&Shortcuts::shipped(), settings_chord));
    alt_and(&mut server, F5, at_the_seat);

    assert!(server.alos_own_terminal().is_some(), "it was closed");
    assert!(server.the_question_closing_asks().is_none());
}

/// **With none of alo's windows open, the chord goes on to the applications'
/// road** — here, with nothing focused, a refusal and not a close.
#[test]
fn with_none_open_the_chord_is_the_applications() {
    let (_runtime, mut server) = a_session("alt-f4-nothing-open");
    let close = Shortcuts::shipped()
        .chord_for(Action::CloseWindow)
        .expect("the release ships Alt+F4");

    assert!(!server.a_chord_closes_alos_own_window(&Shortcuts::shipped(), close));
    assert_eq!(
        server.the_chord_does_what_it_names(close),
        WhatTheChordCameTo::Refused,
        "with no window focused the applications' road refuses"
    );
}
