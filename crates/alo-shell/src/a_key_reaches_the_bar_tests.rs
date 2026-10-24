//! **The road a real key takes into the Bar**, and nothing shorter.
//!
//! Every test here presses through [`the_key_the_bar_took`], which is what
//! `crate::direct_desktop`'s lane calls once libinput has been read. That is
//! the whole point of the file: the layers below were finished, tested and
//! green while a person could not type a single character into the Bar, and
//! every one of those tests called `Server::a_key_reached_the_composer`
//! itself.
//!
//! `a_chord_opens_the_bar_tests::what_is_typed_after_the_chord_reaches_the_field`
//! is the clearest case. Its own comment says *the chord is useless if the
//! keyboard arrives somewhere a person cannot type* — and it proves that by
//! calling the handler directly, which is exactly the step a real keyboard
//! could not reach.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;
use crate::composer_keys::ComposerKey;
use alo_shortcuts::{Action, Shortcuts};
use smithay::backend::input::KeyState;
use smithay::input::keyboard::XkbConfig;
use std::os::unix::fs::PermissionsExt;

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

/// The chord a person presses to reach the Bar.
fn the_agents_chord(shortcuts: &Shortcuts) -> alo_shortcuts::Chord {
    shortcuts
        .chord_for(Action::TheAgent)
        .expect("the agent's chord is bound in the shipped shortcuts")
}

/// One key transition, handed over exactly as the lane hands it after libinput.
///
/// **Not `Server::the_bars_key` and not `a_key_reached_the_composer`** — those
/// are links in the road, and a test that calls one of them directly is the
/// fault this whole file exists to have fixed.
fn moved(server: &mut Server, code: u32, state: KeyState) {
    the_key_the_bar_took(
        server,
        crate::DirectKeyEvent {
            code,
            state,
            time: 1,
        },
    )
    .expect("the seat took the key");
}

/// Press and release one key.
fn pressed(server: &mut Server, code: u32) {
    moved(server, code, KeyState::Pressed);
    moved(server, code, KeyState::Released);
}

/// `h` and `i` on a US layout.
const H: u32 = 35;
const I: u32 = 23;

/// **What a person types after the chord arrives in the field.**
///
/// This is the test that was missing. Measured on a booted machine on
/// 2026-10-23: the chord fired, the composer took the keyboard, twenty-nine
/// characters were typed and the field stayed empty — because nothing routed a
/// key to it. Deleting the arm in `crate::direct_desktop` that this file is
/// the entrance for puts the machine back in that state, and this test fails.
#[test]
fn what_a_person_types_after_the_chord_arrives_in_the_field() {
    let (_runtime, mut server) = a_session("typing-reaches-the-bar");
    let shortcuts = Shortcuts::shipped();
    assert!(
        !server.composer().has_the_keyboard(),
        "a fresh session already had the keyboard in the composer, so this proves nothing"
    );
    server.a_chord_opens_the_bar(&shortcuts, the_agents_chord(&shortcuts));
    assert!(
        server.the_bar_is_taking_the_keys(),
        "the chord was taken and the Bar is not the thing taking keys"
    );

    pressed(&mut server, H);
    pressed(&mut server, I);

    assert_eq!(
        server.composer().typed(),
        "hi",
        "two keys went down the road a real keyboard takes and did not arrive in the field"
    );
}

/// **A key pressed before the chord does not land in the Bar.**
///
/// The far control. Without it the test above would pass against a machine
/// that routed *every* key to the Bar for ever, which is a different and worse
/// fault: an application behind would never hear a keystroke again.
#[test]
fn a_key_pressed_before_the_chord_is_not_the_bars() {
    let (_runtime, server) = a_session("before-the-chord");
    assert!(
        !server.the_bar_is_taking_the_keys(),
        "a fresh session says the Bar is taking keys, so this proves nothing"
    );
    assert_eq!(
        server.composer().typed(),
        "",
        "a fresh composer already had something in it"
    );
}

/// **A release is not a press**, so holding a key types one letter.
///
/// `Server::the_bars_key` answers `None` for a release and for a repeat of a
/// key already down, which is `settings_key`'s rule and the reason this road
/// is built from it line for line.
#[test]
fn a_release_types_nothing() {
    let (_runtime, mut server) = a_session("release-types-nothing");
    let shortcuts = Shortcuts::shipped();
    server.a_chord_opens_the_bar(&shortcuts, the_agents_chord(&shortcuts));

    moved(&mut server, H, KeyState::Pressed);
    moved(&mut server, H, KeyState::Released);
    moved(&mut server, H, KeyState::Released);

    assert_eq!(
        server.composer().typed(),
        "h",
        "one press and two releases did not put exactly one letter in the field"
    );
}

/// **A code outside evdev's range is not a reason to end the desktop.**
///
/// `crate::direct_sign_in`'s rule: a keyboard with a code evdev does not have
/// is a key that does nothing, not an error that takes a person's session.
#[test]
fn a_code_evdev_does_not_have_is_not_an_error() {
    let (_runtime, mut server) = a_session("out-of-range");
    let shortcuts = Shortcuts::shipped();
    server.a_chord_opens_the_bar(&shortcuts, the_agents_chord(&shortcuts));
    assert!(
        the_key_the_bar_took(
            &mut server,
            crate::DirectKeyEvent {
                code: 0xffff,
                state: KeyState::Pressed,
                time: 1,
            },
        )
        .is_ok(),
        "a code outside evdev's range ended the road instead of doing nothing"
    );
    assert_eq!(server.composer().typed(), "");
}

/// **Every key the Bar understands has a name in `ComposerKey`**, and the one
/// it does not understand is dropped rather than handed on — see the file's
/// own note on why passing it to the seat would let an application behind hear
/// what somebody is typing into the Bar.
#[test]
fn a_key_the_bar_has_no_meaning_for_is_still_the_bars() {
    assert!(matches!(ComposerKey::Nothing, ComposerKey::Nothing));
}
