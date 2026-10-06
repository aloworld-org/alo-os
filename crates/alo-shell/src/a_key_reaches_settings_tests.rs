//! **The road this file is named for, driven on a real seat.**
//!
//! The one thing these tests may not do is assert that some source file
//! mentions `a_key_reaches_settings`. Eight faults on 2026-10-05 came from
//! trusting a name — `docs/misreadings/a-name-is-not-a-fact-about-the-thing.md`
//! — and the eighth was inside the check written to catch the family. So every
//! test below presses a key on a seat with a real XKB keymap and asks what
//! became of the window and of the session's chords.
//!
//! The libinput half is not tested here and cannot be: a libinput `Event` needs
//! a login seat and a device. What is tested is everything from the evdev code
//! inward, which is where all the new behaviour is;
//! `crate::libinput_routing::translate` is the one reading of what libinput
//! said and has its own tests.

use super::*;
use crate::SettingsRow;
use crate::settings_testing::{a_persons_machine, words};
use alo_shortcuts::{Action, Shortcuts};
use smithay::backend::input::KeyState;
use smithay::input::keyboard::XkbConfig;
use std::os::unix::fs::PermissionsExt;

/// Escape.
const KEY_ESC: u32 = 1;
/// Down.
const KEY_DOWN: u32 = 108;
/// Enter.
const KEY_ENTER: u32 = 28;
/// Left Control.
const KEY_LEFTCTRL: u32 = 29;
/// Left Alt.
const KEY_LEFTALT: u32 = 56;
/// T.
const KEY_T: u32 = 20;

/// A session on this machine's seat, with a keyboard on a US layout.
fn a_session(what: &str) -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind_keyboard(
        runtime.path(),
        what,
        XkbConfig {
            layout: "us",
            ..XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    (runtime, server)
}

/// One press and its release, down the road the lane drives.
fn tapped(server: &mut Server, code: u32, strings: &alo_strings::Strings) {
    pressed(server, code, strings);
    moved(server, code, KeyState::Released, strings);
}

/// One press, down the road the lane drives — and nothing else.
fn pressed(server: &mut Server, code: u32, strings: &alo_strings::Strings) {
    moved(server, code, KeyState::Pressed, strings);
}

/// One key transition, handed to [`the_key_settings_took`] exactly as
/// `crate::direct_desktop`'s lane hands it over once libinput has been read.
///
/// **Not `Server::settings_key` and not [`the_press_is_carried_out`]** — those
/// are links in the road, and a test that called one of them directly was the
/// fault this helper exists to have fixed: the line that turns a key into a
/// press could be deleted with every test still green.
fn moved(server: &mut Server, code: u32, state: KeyState, strings: &alo_strings::Strings) {
    the_key_settings_took(
        server,
        crate::DirectKeyEvent {
            code,
            state,
            time: 1,
        },
        strings,
    )
    .expect("the seat took the key");
}

/// **Escape closes the window**, which is the whole reason this road could not
/// wait for another change.
///
/// `SettingsWindow::close` had no production caller when the chord that opens
/// the window landed, so a person who pressed `⊞`+`I` had a window they could
/// not dismiss. This is the test that would have caught that.
#[test]
fn escape_closes_a_window_a_person_opened() {
    let machine = a_persons_machine("escape");
    let strings = words();
    let (_runtime, mut server) = a_session("escape");
    server.the_settings_places_are(machine.places.clone());
    server.settings = machine.opened();

    assert!(
        server.settings_is_taking_the_keys(),
        "an open window does not claim the seat's keys"
    );

    tapped(&mut server, KEY_ESC, &strings);

    assert!(
        !server.the_settings_window().is_open(),
        "Escape left the window open, so a person cannot dismiss it"
    );
    assert!(
        !server.settings_is_taking_the_keys(),
        "a closed window still claims the seat's keys, so the session is deaf"
    );
}

/// **A chord bound in Settings is in force without signing out.**
///
/// The window writes the person's file; `Server::the_shortcuts_are` is what
/// makes a chord *mean* it. Without the second call this test's last assertion
/// fails while the file on disk is perfectly correct — which is the shape of
/// fault this whole change exists to end.
#[test]
fn a_chord_bound_in_settings_is_in_force_at_once() {
    let machine = a_persons_machine("bound");
    let strings = words();
    let (_runtime, mut server) = a_session("bound");
    server.the_settings_places_are(machine.places.clone());
    server.settings = machine.opened();
    server.the_shortcuts_are(Shortcuts::shipped());

    // The rows are asked at the moment the presses will be carried out, not at
    // noon: a grant that has run out is not a row, and indexing a list drawn
    // at a different moment would move the focus somewhere else.
    let now = std::time::SystemTime::now();
    let rows = server.the_settings_window().rows(now);
    let (at, action) = rows
        .iter()
        .enumerate()
        .find_map(|(at, row)| match row {
            SettingsRow::Shortcut(action) => Some((at, *action)),
            _ => None,
        })
        .expect("this machine's Settings draws the shortcuts it keeps");
    let was = Shortcuts::shipped()
        .chord_for(action)
        .expect("a drawn binding has a chord");

    for _ in 0..at {
        tapped(&mut server, KEY_DOWN, &strings);
    }
    assert_eq!(
        server.the_settings_window().focused(now),
        Some(SettingsRow::Shortcut(action)),
        "the focus is not on the binding this test is about to change"
    );

    tapped(&mut server, KEY_ENTER, &strings);
    assert!(
        server.the_settings_window().is_waiting_for_a_chord(),
        "choosing a binding did not start waiting for a chord"
    );

    // Control+Alt+T, held as a person holds it: the modifiers down, the key
    // tapped, the modifiers up.
    pressed(&mut server, KEY_LEFTCTRL, &strings);
    pressed(&mut server, KEY_LEFTALT, &strings);
    tapped(&mut server, KEY_T, &strings);

    let bound = server
        .the_settings_window()
        .shortcuts()
        .and_then(|shortcuts| shortcuts.chord_for(action))
        .expect("the window still holds a chord for the action");
    assert_ne!(bound, was, "the chord Settings holds was not changed");

    let in_force = server
        .shortcuts
        .as_ref()
        .and_then(|shortcuts| shortcuts.chord_for(action))
        .expect("the session still has a chord for the action");
    assert_eq!(
        in_force, bound,
        "the session is still bound by the old chord, so Settings wrote the \
         file and nothing a person presses has changed until they sign out"
    );
}

/// **A press on a session nobody told where settings are kept does nothing**,
/// rather than being carried out against places this crate invented.
///
/// The sign-in screen, the nested lane and every existing test are that
/// session. None of them can have an open window — the chord that opens one
/// needs the places — so this is the state, not a path a person reaches.
#[test]
fn a_session_with_nowhere_to_keep_anything_carries_nothing_out() {
    let machine = a_persons_machine("untold");
    let strings = words();
    let (_runtime, mut server) = a_session("untold");
    server.settings = machine.opened();

    tapped(&mut server, KEY_ESC, &strings);

    assert!(
        server.the_settings_window().is_open(),
        "a press was carried out against places nobody told this session"
    );
}

/// **A key the keyboard has and evdev does not is not a refusal.**
///
/// `crate::direct_sign_in` makes the same argument for the screen a machine
/// boots to: a strange code is not a reason to take a person's window away.
/// Here it is checked through the seat rather than through the lane, because a
/// libinput event cannot be made in a test.
#[test]
fn a_code_outside_evdev_is_refused_without_touching_the_window() {
    let machine = a_persons_machine("strange");
    let (_runtime, mut server) = a_session("strange");
    server.the_settings_places_are(machine.places.clone());
    server.settings = machine.opened();

    for code in [0, 0x300] {
        // Swallowed by the road rather than returned, which is the contract:
        // the lane turns a refusal here into the end of the display lifetime.
        assert!(
            the_key_settings_took(
                &mut server,
                crate::DirectKeyEvent {
                    code,
                    state: KeyState::Pressed,
                    time: 1,
                },
                &words(),
            )
            .is_ok(),
            "a code outside evdev's range ended the session: {code}"
        );
        // And the seat refuses it, which is where the swallowing happens.
        assert!(
            matches!(
                server.settings_key(code, KeyState::Pressed, 1),
                Err(InputError::InvalidKey)
            ),
            "a code outside evdev's range was accepted by the seat: {code}"
        );
    }
    assert!(
        server.the_settings_window().is_open(),
        "a strange code closed the window"
    );
}

/// The shipped bindings have a chord for Settings itself, which is what
/// `crate::a_chord_reaches_its_action` routes. Named here so that a release
/// which dropped it fails beside the road a person leaves the window by.
#[test]
fn the_chord_that_opens_settings_is_still_shipped() {
    assert!(Shortcuts::shipped().chord_for(Action::Settings).is_some());
}
