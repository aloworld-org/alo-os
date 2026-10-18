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
/// Enter, the one above the right Shift — which is Install here.
const KEY_ENTER: u32 = 28;

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

/// **Pressing Install records the ask for the lane, once.**
///
/// The press is the event and the window's state is not: `MarketWindow` goes on
/// showing what was asked for until the host answers, so a lane that read the
/// window instead would start the same download on every frame.
/// `Server::asked_to_install` carries the whole argument; this is the test of
/// it.
#[test]
fn pressing_install_records_one_ask_for_the_lane() {
    let (_runtime, mut server) = with_the_market_open("market-asks");

    assert!(
        server.take_what_was_asked_to_install().is_none(),
        "a window nobody has pressed Install on has an ask waiting"
    );

    // Enter is Install, through the seat like every other key here.
    assert_eq!(
        server
            .the_market_takes_this_key(KEY_ENTER, KeyState::Pressed, 1)
            .expect("a seat with a keyboard"),
        Some(MarketPressed::AskedToInstall)
    );

    let asked = server
        .take_what_was_asked_to_install()
        .expect("the press was recorded for the lane");
    assert!(
        !asked.application().identifier().is_empty(),
        "the ask names no application"
    );
    assert_eq!(
        server.take_what_was_asked_to_install(),
        None,
        "one press was taken twice, so the lane would install it twice"
    );

    // **And the window still shows it**, which is why the ask is taken from the
    // server rather than from the window: the row says *installing it* until
    // the host answers.
    assert_eq!(
        server
            .market
            .what_they_asked_for()
            .map(|one| one.application().identifier().to_owned()),
        Some(asked.application().identifier().to_owned()),
        "the window forgot what it is showing when the lane took the ask"
    );
}

/// **A key that is not Install records nothing**, so a person moving through
/// the list does not start downloads.
#[test]
fn moving_through_the_list_records_no_ask() {
    let (_runtime, mut server) = with_the_market_open("market-no-asks");

    for code in [KEY_DOWN, KEY_RIGHT, KEY_K] {
        server
            .the_market_takes_this_key(code, KeyState::Pressed, 1)
            .expect("a seat with a keyboard");
        server
            .the_market_takes_this_key(code, KeyState::Released, 2)
            .expect("a seat with a keyboard");
        assert!(
            server.take_what_was_asked_to_install().is_none(),
            "moving through the list asked for an installation"
        );
    }
}

/// **An event this machine cannot make sense of does not end the session, and
/// is counted.**
///
/// The fault this holds shut was measured on a booted machine on 2026-10-17: a
/// desktop drew its Dock, was sent its first pointer event, and **ended** —
/// because the lane turned every `InputError` into an `io::Error` and an
/// `io::Error` out of its closure is the end of the display loop.
///
/// Both halves are asserted, and the second is the one that matters: a refusal
/// nobody can see is **worse** than a loud end, because it is what hid the
/// missing `enable_pointer` call for as long as nobody booted a machine. So the
/// count is part of the behaviour, not a diagnostic beside it.
#[test]
fn an_event_this_machine_cannot_read_is_counted_rather_than_fatal() {
    let (_runtime, mut server) = with_the_market_open("market-refused");

    assert_eq!(server.how_much_input_was_refused(), 0, "the premise");

    // A code outside evdev's range: `the_market_takes_this_key` answers
    // `InvalidKey`, which is about this one event.
    for code in [0, 0x300, u32::MAX - 8] {
        let why = server
            .market_key(code, KeyState::Pressed, 1)
            .expect_err("a code evdev does not have is refused");
        assert!(
            why.is_about_one_event(),
            "{why:?} would end the display loop, so one impossible code takes the desktop away"
        );
        server.an_input_event_was_refused();
    }

    assert_eq!(
        server.how_much_input_was_refused(),
        3,
        "a refusal was survived without being counted, which is how a pointer that does nothing \
         for ever goes unnoticed"
    );
    assert!(
        server.market.is_open(),
        "the window closed because the machine met a key it could not read"
    );
}

/// **A seat that cannot be made is not one event**, so the two are told apart.
///
/// `InputError::Socket` and `Keymap` happen while a display is stood up and
/// mean it cannot be used at all. A lane that treated those as survivable would
/// keep drawing against a display it does not have.
#[test]
fn a_display_that_could_not_be_made_is_not_one_event() {
    use crate::InputError;

    for about_the_seat in [
        InputError::Keymap("no such layout".to_owned()),
        InputError::Socket(crate::SocketError::InvalidName),
    ] {
        assert!(
            !about_the_seat.is_about_one_event(),
            "{about_the_seat:?} would be survived, so a lane would keep drawing against a \
             display it could not make"
        );
    }

    // And the five that are, named rather than counted, so adding a variant is
    // a decision somebody writes down here.
    for about_one_event in [
        InputError::InvalidKey,
        InputError::InvalidPointer,
        InputError::Unmapped,
        InputError::PointerUnavailable,
        InputError::Unavailable,
    ] {
        assert!(about_one_event.is_about_one_event(), "{about_one_event:?}");
    }
}
