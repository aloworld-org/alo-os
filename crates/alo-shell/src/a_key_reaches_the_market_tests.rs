//! That a key pressed at the market reaches it — through the seat, on a real
//! XKB state, and not through a helper that calls the window itself.
//!
//! `crate::a_key_reaches_settings_tests` records what the shortcut costs: a
//! test that called the window directly stayed green after the line carrying a
//! press out had been deleted. So every press below goes in as an evdev code.

use smithay::backend::input::KeyState;
use smithay::input::keyboard::XkbConfig;
use std::os::unix::fs::PermissionsExt as _;

use crate::{InputError, MarketPressed, Server};

/// Escape, as evdev numbers it.
const KEY_ESC: u32 = 1;
/// Down.
const KEY_DOWN: u32 = 108;
/// Right.
const KEY_RIGHT: u32 = 106;
/// `k`, which this window has no use for.
const KEY_K: u32 = 37;

/// A session with a keyboard on a US layout, in a private directory of its own.
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

/// Open the market the way a person does: by the action their Dock entry asks
/// for.
fn with_the_market_open(what: &str) -> (tempfile::TempDir, Server) {
    let (runtime, mut server) = a_session(what);
    server.do_this_native_action(alo_dock::ANativeAction::Applications);
    assert!(server.market.is_open(), "the market did not open");
    (runtime, server)
}

/// **The seat takes the keys while the window is open, and stops the moment it
/// closes.**
///
/// Asked per input batch by the lane, which is why it is asserted on both sides
/// of the press that closes: a question answered once per lifetime would leave
/// a shell swallowing every key for ever after the window went away.
#[test]
fn the_seat_takes_the_keys_only_while_the_window_is_open() {
    let (_runtime, mut server) = a_session("market-taking");
    assert!(!server.the_market_is_taking_the_keys());

    server.do_this_native_action(alo_dock::ANativeAction::Applications);
    assert!(server.the_market_is_taking_the_keys());

    assert_eq!(
        server
            .the_market_takes_this_key(KEY_ESC, KeyState::Pressed, 1)
            .expect("a seat with a keyboard"),
        Some(MarketPressed::Closed)
    );
    assert!(
        !server.the_market_is_taking_the_keys(),
        "the window closed and the seat is still swallowing every key"
    );
}

/// **Escape closes it, which is the fault this file exists to prevent.**
///
/// A person who pressed the Applications entry and could not dismiss what
/// opened is the state Settings was in until `a_key_reaches_settings` was
/// written. Every link below this one was built and tested before this change
/// and nothing on a running machine called any of them.
#[test]
fn escape_closes_the_window() {
    let (_runtime, mut server) = with_the_market_open("market-escape");

    let did = server
        .the_market_takes_this_key(KEY_ESC, KeyState::Pressed, 1)
        .expect("a seat with a keyboard");

    assert_eq!(did, Some(MarketPressed::Closed));
    assert!(!server.market.is_open(), "Escape did not close the window");
}

/// **The arrows move through the list and narrow it**, which is the rest of
/// what a person can do here.
#[test]
fn the_arrows_move_and_narrow() {
    let (_runtime, mut server) = with_the_market_open("market-arrows");

    assert_eq!(
        server
            .the_market_takes_this_key(KEY_DOWN, KeyState::Pressed, 1)
            .expect("a seat with a keyboard"),
        Some(MarketPressed::Moved)
    );
    server
        .the_market_takes_this_key(KEY_DOWN, KeyState::Released, 2)
        .expect("a seat with a keyboard");
    assert_eq!(
        server
            .the_market_takes_this_key(KEY_RIGHT, KeyState::Pressed, 3)
            .expect("a seat with a keyboard"),
        Some(MarketPressed::Narrowed)
    );
}

/// **A held key means something once**, and a release means nothing.
///
/// The seat's own rule, asserted on this road because a window that acted on
/// every repeat would walk the whole list while a person leant on an arrow.
#[test]
fn a_held_key_means_something_once() {
    let (_runtime, mut server) = with_the_market_open("market-held");

    assert_eq!(
        server
            .the_market_takes_this_key(KEY_DOWN, KeyState::Pressed, 1)
            .expect("a seat with a keyboard"),
        Some(MarketPressed::Moved)
    );
    assert_eq!(
        server
            .the_market_takes_this_key(KEY_DOWN, KeyState::Pressed, 2)
            .expect("a seat with a keyboard"),
        None,
        "a key already down was read as a second press"
    );
    assert_eq!(
        server
            .the_market_takes_this_key(KEY_DOWN, KeyState::Released, 3)
            .expect("a seat with a keyboard"),
        None,
        "a release moved the list"
    );
}

/// **A key this window has no use for does nothing to it**, and is still read
/// off the seat — which is what keeps the window's own keys and a letter on one
/// road rather than two.
#[test]
fn a_key_this_window_has_no_use_for_does_nothing() {
    let (_runtime, mut server) = with_the_market_open("market-nothing");

    assert_eq!(
        server
            .the_market_takes_this_key(KEY_K, KeyState::Pressed, 1)
            .expect("a seat with a keyboard"),
        Some(MarketPressed::Nothing)
    );
    assert!(server.market.is_open(), "a letter closed the window");
}

/// **A key pressed at a shut window does nothing at all**, rather than opening
/// one or acting on a catalogue that is not there.
#[test]
fn a_key_at_a_shut_window_does_nothing() {
    let (_runtime, mut server) = a_session("market-shut");

    assert_eq!(
        server
            .the_market_takes_this_key(KEY_DOWN, KeyState::Pressed, 1)
            .expect("a seat with a keyboard"),
        Some(MarketPressed::Nothing)
    );
    assert!(!server.market.is_open(), "a key opened the window");
}

/// **A strange code and a missing keyboard are told apart.**
///
/// A code outside evdev's range is not an error on this road — a keyboard with
/// a code evdev does not have is not a reason to take a person's window away —
/// and a display bound without a keyboard is, because then nothing can be read
/// at all and the lane needs to know.
#[test]
fn a_strange_code_is_not_a_failure_and_a_missing_keyboard_is() {
    let (_runtime, mut server) = with_the_market_open("market-strange");
    for code in [0, 0x300, u32::MAX - 8] {
        assert_eq!(
            server
                .the_market_takes_this_key(code, KeyState::Pressed, 1)
                .expect("a code evdev does not have is not a failure"),
            None
        );
    }
    assert!(server.market.is_open());

    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut bare = Server::bind(runtime.path(), "market-no-keyboard").expect("a display");
    bare.do_this_native_action(alo_dock::ANativeAction::Applications);
    assert!(matches!(
        bare.the_market_takes_this_key(KEY_ESC, KeyState::Pressed, 1),
        Err(InputError::Unavailable)
    ));
}
